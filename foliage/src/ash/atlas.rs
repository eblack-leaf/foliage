//! The atlas: every glyph the backend has rasterised, on one texture.
//!
//! # A cut is a glyph at a density
//!
//! What is rasterised is a character of a face at a size *for a display*, because the bitmap is made
//! in device pixels and the display says how many of those one logical pixel is. So the density is
//! part of the key rather than something applied afterwards: the same character at the same size on
//! a different screen is a different cut, and asking for it makes one.
//!
//! Everything the atlas hands back about *where* a glyph goes is in logical pixels. The device
//! pixels stop at the texture, which is the same line [`Ginkgo`](crate::ginkgo) draws.

use std::collections::HashMap;

use tracing::{debug, error};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Extent3d,
    FilterMode, Origin3d, SamplerBindingType, SamplerDescriptor, ShaderStages,
    TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor,
    TextureViewDimension,
};

use crate::coordinate::{Area, Position};
use crate::ginkgo::Ginkgo;
use crate::text::font::{Font, Fonts};

/// How wide and tall the one texture is, in device pixels.
///
/// A fixed sheet rather than one that grows: growing means re-cutting every glyph on it, and a
/// monospaced app addresses a small alphabet at a handful of sizes. Four megabytes of coverage holds
/// thousands of cuts, and running out is reported rather than absorbed -- see [`Atlas::pack`].
const SIDE: u32 = 2048;

/// A pixel of clearance around every cut, so that filtering a glyph at its edge cannot reach into
/// the one packed beside it.
const GUTTER: u32 = 1;

/// One rasterised character: a face, at a size, for a display of a given density.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Cut {
    pub(crate) font: Font,
    pub(crate) size: u32,
    /// Device pixels per logical one, in thousandths. An integer because a key has to compare
    /// exactly, and a float that came from a window manager does not.
    pub(crate) density: u32,
    pub(crate) character: char,
}

impl Cut {
    pub(crate) fn new(font: Font, size: u32, scale: f32, character: char) -> Self {
        Self {
            font,
            size,
            density: (scale * 1000.0).round().max(1.0) as u32,
            character,
        }
    }

    /// The density as the rasteriser takes it.
    fn scale(self) -> f32 {
        self.density as f32 / 1000.0
    }
}

/// Where a cut ended up: the ink's box inside a character cell, and its rect on the sheet.
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct Placed {
    /// Where the ink's top-left corner sits inside its cell, in logical pixels.
    pub(crate) offset: Position,
    /// How large the ink is, in logical pixels.
    pub(crate) area: Area,
    /// The ink's rect on the sheet, normalised: origin then size.
    pub(crate) uv: [f32; 4],
}

/// The one sheet, what is on it, and where the next cut goes.
pub(crate) struct Atlas {
    texture: Texture,
    layout: BindGroupLayout,
    binding: BindGroup,
    held: HashMap<Cut, Placed>,
    /// The shelf being filled: where the next cut goes on it, and how tall it is.
    x: u32,
    y: u32,
    shelf: u32,
    /// Whether the sheet has run out, so the report is made once rather than per glyph per frame.
    full: bool,
}

impl Atlas {
    pub(crate) fn new(ginkgo: &Ginkgo) -> Self {
        let device = ginkgo.device();
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("atlas"),
            size: Extent3d {
                width: SIDE,
                height: SIDE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            // Coverage, one channel. A glyph has an alpha and no colour of its own -- what it is
            // filled with is the element's, and is carried on the instance.
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("atlas"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..SamplerDescriptor::default()
        });
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("atlas-layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let binding = device.create_bind_group(&BindGroupDescriptor {
            label: Some("atlas-binding"),
            layout: &layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });
        Self {
            texture,
            layout,
            binding,
            held: HashMap::new(),
            x: 0,
            y: 0,
            shelf: 0,
            full: false,
        }
    }

    /// Where `cut` is on the sheet, cutting it if this is the first time it has been asked for.
    pub(crate) fn place(&mut self, fonts: &Fonts, ginkgo: &Ginkgo, cut: Cut) -> Placed {
        if let Some(placed) = self.held.get(&cut) {
            return *placed;
        }
        let ink = fonts.rasterize(cut.font, cut.size, cut.scale(), cut.character);
        // A character with no outline -- and there are more of them than a space -- is held as
        // nothing rather than not held: asking again must not rasterise again.
        let placed = match ink.width == 0 || ink.height == 0 {
            true => Placed {
                offset: ink.offset,
                ..Placed::default()
            },
            false => match self.pack(ginkgo, ink.width, ink.height) {
                Some((x, y)) => {
                    ginkgo.queue().write_texture(
                        TexelCopyTextureInfo {
                            texture: &self.texture,
                            mip_level: 0,
                            origin: Origin3d { x, y, z: 0 },
                            aspect: TextureAspect::All,
                        },
                        &ink.coverage,
                        TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(ink.width),
                            rows_per_image: Some(ink.height),
                        },
                        Extent3d {
                            width: ink.width,
                            height: ink.height,
                            depth_or_array_layers: 1,
                        },
                    );
                    let side = SIDE as f32;
                    let scale = cut.scale();
                    Placed {
                        offset: ink.offset,
                        area: Area::new(ink.width as f32 / scale, ink.height as f32 / scale),
                        uv: [
                            x as f32 / side,
                            y as f32 / side,
                            ink.width as f32 / side,
                            ink.height as f32 / side,
                        ],
                    }
                }
                // Held as nothing, so the character costs one failed pack rather than one per frame.
                None => Placed::default(),
            },
        };
        debug!(
            character = ?cut.character,
            size = cut.size,
            width = ink.width,
            height = ink.height,
            "glyph cut"
        );
        self.held.insert(cut, placed);
        placed
    }

    /// Where a bitmap of this size goes on the sheet, or `None` if there is no room left.
    ///
    /// Shelves: a row is filled left to right and the next one starts below the tallest cut in it.
    /// Glyphs of one size are all much of a height, so the waste is a line of it per size rather
    /// than per glyph, and a shelf packer needs no bookkeeping to be undone when nothing is ever
    /// removed.
    fn pack(&mut self, ginkgo: &Ginkgo, width: u32, height: u32) -> Option<(u32, u32)> {
        if width > SIDE || height > SIDE {
            return None;
        }
        if self.x + width > SIDE {
            self.y += self.shelf + GUTTER;
            self.x = 0;
            self.shelf = 0;
        }
        if self.y + height > SIDE {
            if !self.full {
                self.full = true;
                error!(
                    side = SIDE,
                    held = self.held.len(),
                    "the glyph atlas is full: further characters will not draw"
                );
            }
            return None;
        }
        let _ = ginkgo;
        let at = (self.x, self.y);
        self.x += width + GUTTER;
        self.shelf = self.shelf.max(height);
        Some(at)
    }

    /// The layout a pipeline sampling the sheet declares at group 1.
    pub(crate) fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }

    /// The sheet, as a pass binds it.
    pub(crate) fn binding(&self) -> &BindGroup {
        &self.binding
    }
}
