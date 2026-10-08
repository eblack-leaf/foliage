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
//!
//! # What is on it is what the frame draws
//!
//! A cut is packed the first time a run asks for it and stays, because the ordinary frame draws
//! what the last one did and should find every cut it needs already there. Nothing is taken off one
//! at a time. When a cut finds no room, the sheet is cut again from the runs the frame draws and
//! nothing else -- so whatever the frame stopped drawing gives its room back at once: words that
//! were rewritten, a size no longer in use. Only when the frame's own cuts do not fit is the sheet
//! full, and that is reported. A density that changed takes every cut off at once, since none of
//! them is drawn at the density that is left.
//!
//! [`Cuts`] is the bookkeeping that decides all of it, apart from the texture it is about, so what
//! happens to a cut is decided the same way with no device at all.

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
use crate::text::font::{Font, Fonts, Ink};

/// How wide and tall the one texture is, in device pixels.
///
/// A fixed sheet rather than one that grows: growing means re-cutting every glyph on it, and a
/// monospaced app addresses a small alphabet at a handful of sizes. Four megabytes of coverage holds
/// thousands of cuts. Running out cuts it again from what the frame draws, and only what the frame
/// draws not fitting is reported -- see [`Cuts`].
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
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Placed {
    /// Where the ink's top-left corner sits inside its cell, in logical pixels.
    pub(crate) offset: Position,
    /// How large the ink is, in logical pixels.
    pub(crate) area: Area,
    /// The ink's rect on the sheet, normalised: origin then size.
    pub(crate) uv: [f32; 4],
}

/// The one sheet, and what is on it.
pub(crate) struct Atlas {
    texture: Texture,
    layout: BindGroupLayout,
    binding: BindGroup,
    cuts: Cuts,
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
            cuts: Cuts::new(SIDE),
        }
    }

    /// Where `cut` is on the sheet, cutting it if this is the first time it has been asked for.
    ///
    /// A cut that found no room is held as nothing for now, and [`overflowed`](Atlas::overflowed)
    /// says so: cutting the sheet again is the caller's, because the caller is what knows which runs
    /// the frame draws.
    pub(crate) fn place(&mut self, fonts: &Fonts, ginkgo: &Ginkgo, cut: Cut) -> Placed {
        let (placed, packed) = self.cuts.place(fonts, cut);
        if let Some(Packed { x, y, ink }) = packed {
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
        }
        placed
    }

    /// Whether a cut has found no room since the sheet was last cut again.
    pub(crate) fn overflowed(&self) -> bool {
        self.cuts.overflowed()
    }

    /// Takes every cut off, for the sheet to be cut again from what is asked for next.
    ///
    /// The texture keeps what was written to it until a new cut is written over it, and nothing
    /// points at what is left in between.
    pub(crate) fn clear(&mut self) {
        self.cuts.clear();
    }

    /// Says the sheet has been cut again from everything the frame draws. See [`Cuts::settle`].
    pub(crate) fn settle(&mut self) {
        self.cuts.settle();
    }

    /// Takes every cut off and forgets that the sheet was ever full, for one cut at a new density.
    pub(crate) fn forget(&mut self) {
        self.cuts.forget();
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

/// What is on the sheet and where, where the next cut goes, and whether one has run out of room:
/// the atlas, less the texture.
pub(crate) struct Cuts {
    held: HashMap<Cut, Placed>,
    shelves: Shelves,
    /// Whether a cut has found no room since the sheet was last cut again.
    overflowed: bool,
    /// Whether, the last time the sheet was cut again from what the frame draws, that did not fit
    /// either. Kept so the report is made once rather than every time it is found again.
    full: bool,
}

/// A cut just packed: where on the sheet, and the ink to write there.
pub(crate) struct Packed {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) ink: Ink,
}

impl Cuts {
    /// An empty sheet `side` texels across and down.
    pub(crate) fn new(side: u32) -> Self {
        Self {
            held: HashMap::new(),
            shelves: Shelves::new(side),
            overflowed: false,
            full: false,
        }
    }

    /// Where `cut` is on the sheet, rasterising and packing it if this is the first time it has been
    /// asked for -- and then, beside it, the ink and where to write it.
    ///
    /// A character with no outline -- and there are more of them than a space -- is held as nothing
    /// rather than not held: asking again must not rasterise again. So is one that found no room,
    /// for as long as the sheet stands; that one also marks the sheet
    /// [`overflowed`](Cuts::overflowed), and is asked for again once the sheet is cut again.
    pub(crate) fn place(&mut self, fonts: &Fonts, cut: Cut) -> (Placed, Option<Packed>) {
        if let Some(placed) = self.held.get(&cut) {
            return (*placed, None);
        }
        let ink = fonts.rasterize(cut.font, cut.size, cut.scale(), cut.character);
        debug!(
            character = ?cut.character,
            size = cut.size,
            width = ink.width,
            height = ink.height,
            "glyph cut"
        );
        let (placed, packed) = match ink.width == 0 || ink.height == 0 {
            true => (
                Placed {
                    offset: ink.offset,
                    ..Placed::default()
                },
                None,
            ),
            false => match self.shelves.pack(ink.width, ink.height) {
                Some((x, y)) => {
                    let side = self.shelves.side as f32;
                    let scale = cut.scale();
                    let placed = Placed {
                        offset: ink.offset,
                        area: Area::new(ink.width as f32 / scale, ink.height as f32 / scale),
                        uv: [
                            x as f32 / side,
                            y as f32 / side,
                            ink.width as f32 / side,
                            ink.height as f32 / side,
                        ],
                    };
                    (placed, Some(Packed { x, y, ink }))
                }
                None => {
                    self.overflowed = true;
                    (Placed::default(), None)
                }
            },
        };
        self.held.insert(cut, placed);
        (placed, packed)
    }

    /// Whether a cut has found no room since the sheet was last cut again.
    pub(crate) fn overflowed(&self) -> bool {
        self.overflowed
    }

    /// Whether the sheet was found full: what the frame draws, cut again on its own, did not fit.
    #[cfg(test)]
    pub(crate) fn full(&self) -> bool {
        self.full
    }

    /// Takes every cut off, for the sheet to be cut again from what is asked for next.
    pub(crate) fn clear(&mut self) {
        self.held.clear();
        self.shelves.clear();
        self.overflowed = false;
    }

    /// Says the sheet has been cut again from everything the frame draws, and nothing else.
    ///
    /// A cut that found no room then is one the frame's own cuts leave none for, and no cutting
    /// again could make room for it. So that is the sheet full, and it is reported -- once, until a
    /// time the frame's cuts do fit. What found no room stays held as nothing, and is asked for
    /// again when something new runs out of room and the sheet is cut again for it.
    pub(crate) fn settle(&mut self) {
        match (self.overflowed, self.full) {
            (true, false) => {
                self.full = true;
                error!(
                    side = self.shelves.side,
                    held = self.held.len(),
                    "the glyph atlas is full: what this frame draws does not fit, and the \
                     characters that found no room will not draw"
                );
            }
            (true, true) => {}
            (false, _) => self.full = false,
        }
        self.overflowed = false;
    }

    /// Takes every cut off and forgets the sheet was ever full: a sheet cut at a new density is a new
    /// sheet.
    pub(crate) fn forget(&mut self) {
        self.clear();
        self.full = false;
    }
}

/// Where the next bitmap goes on a square sheet: shelves.
///
/// A row is filled left to right and the next one starts below the tallest bitmap in it. Glyphs of
/// one size are all much of a height, so the waste is a line of it per size rather than per glyph,
/// and a shelf packer needs no bookkeeping to be undone when what is on it is only ever taken off
/// all at once.
pub(crate) struct Shelves {
    /// How wide and tall the sheet is.
    side: u32,
    /// The shelf being filled: where the next bitmap goes on it, and how tall it is.
    x: u32,
    y: u32,
    shelf: u32,
}

impl Shelves {
    pub(crate) fn new(side: u32) -> Self {
        Self {
            side,
            x: 0,
            y: 0,
            shelf: 0,
        }
    }

    /// Where a bitmap of this size goes, or `None` if there is no room left for it.
    pub(crate) fn pack(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        if width > self.side || height > self.side {
            return None;
        }
        if self.x + width > self.side {
            self.y += self.shelf + GUTTER;
            self.x = 0;
            self.shelf = 0;
        }
        if self.y + height > self.side {
            return None;
        }
        let at = (self.x, self.y);
        self.x += width + GUTTER;
        self.shelf = self.shelf.max(height);
        Some(at)
    }

    /// Empties the sheet, so the next bitmap goes at its top-left corner again.
    pub(crate) fn clear(&mut self) {
        self.x = 0;
        self.y = 0;
        self.shelf = 0;
    }
}
