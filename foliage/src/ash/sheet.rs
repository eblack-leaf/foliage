//! The sheet: every registered mark's distance field, on one texture.
//!
//! # Why marks share a sheet and pictures do not
//!
//! A field is small, square, and one of a set an app names at boot -- so they pack once, never move,
//! and every mark on the surface draws in a single pass under one binding. A picture is none of
//! those things: arbitrary size, arbitrary count, and arriving whenever a fetch finishes. Packing
//! those onto a shared sheet would need eviction, and eviction of something a run of instances is
//! already pointing at.
//!
//! They also cannot share this one whatever their sizes: the glyph atlas holds coverage in one
//! channel and this holds a distance in three, and one texture cannot mean both.

use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};
use tracing::{debug, error};
use wgpu::{
    BindGroup, BindGroupLayout, Extent3d, Origin3d, TexelCopyBufferLayout, TexelCopyTextureInfo,
    Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureViewDescriptor,
};

use crate::ash::quad::{bind, sampler, sampling};
use crate::color::Color;
use crate::coordinate::Section;
use crate::ginkgo::Ginkgo;
use crate::icon::{Field, Fields, IconInstance};

/// How wide and tall the one sheet is, in texels.
///
/// A field is typically 32 or 64 a side, so this holds hundreds of them -- an app has a set of
/// marks, not an unbounded stream, and running out is reported rather than absorbed.
const SIDE: u32 = 1024;

/// A texel of clearance around every field, so that filtering one at its edge cannot reach into the
/// one packed beside it.
///
/// It matters more here than for coverage. A glyph's neighbouring texels are alpha, and bleeding a
/// little alpha softens an edge; a field's are a *distance*, and the median of three channels taken
/// from the wrong field is not a softer edge but a wrong one.
const GUTTER: u32 = 2;

/// Where a field ended up on the sheet, and what its distances mean there.
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct Packed {
    /// The field's rect on the sheet, normalised: origin then size.
    pub(crate) uv: [f32; 4],
    /// How many texels of the field its baked distance range spans.
    pub(crate) range: f32,
    /// How many texels the field is across, which is what turns its range into a screen-space one
    /// at whatever size the mark is drawn.
    pub(crate) side: f32,
}

/// One mark, in the form the GPU takes it.
///
/// `#[repr(C)]` over the types the coordinate module guarantees the layout of. Two of its four parts
/// are the backend's own -- where the field sits on the sheet, and how many screen pixels its
/// distance range covers -- which is why extraction states a box and a name and this is derived from
/// them.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct MarkQuad {
    pub(crate) section: Section,
    pub(crate) color: Color,
    /// The field's rect on the sheet, normalised: origin then size.
    pub(crate) uv: [f32; 4],
    /// How many screen pixels the field's baked range covers at this instance's size, which is what
    /// sets the width of the edge the shader smooths over.
    pub(crate) range: f32,
}

impl MarkQuad {
    pub(crate) fn new(instance: IconInstance, packed: Packed, scale: f32) -> Self {
        // The mark's box is square, so either side does. In device pixels, because the edge being
        // measured is one of those.
        let drawn = (instance.section.width() * scale).max(1.0);
        Self {
            section: instance.section,
            color: instance.color,
            uv: packed.uv,
            range: match packed.side > 0.0 {
                true => drawn / packed.side * packed.range,
                // Nothing packed under this name. It draws blank whatever the range says.
                false => 1.0,
            },
        }
    }
}

/// The one sheet, what is on it, and where the next field goes.
pub(crate) struct Sheet {
    texture: Texture,
    layout: BindGroupLayout,
    binding: BindGroup,
    held: HashMap<Field, Packed>,
    /// The shelf being filled: where the next field goes on it, and how tall it is.
    x: u32,
    y: u32,
    shelf: u32,
    /// Whether the sheet has run out, so the report is made once rather than per mark per frame.
    full: bool,
}

impl Sheet {
    pub(crate) fn new(ginkgo: &Ginkgo) -> Self {
        let device = ginkgo.device();
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("sheet"),
            size: Extent3d {
                width: SIDE,
                height: SIDE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            // Three channels of distance and a fourth the bake may carry. Not coverage: what a mark
            // is filled with is the element's, and what is here is only where its edge runs.
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&TextureViewDescriptor::default());
        let layout = sampling(device, "sheet");
        let binding = bind(
            device,
            &layout,
            &view,
            &sampler(device, "sheet"),
            "sheet-binding",
        );
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

    /// Where `field` is on the sheet, packing it if this is the first time it has been asked for.
    ///
    /// A mark that was never registered, or that will not fit, is held as nothing: it draws blank
    /// and costs one failed lookup rather than one per frame. That degrades rather than corrupts,
    /// and the trace says so once.
    pub(crate) fn place(&mut self, fields: &Fields, ginkgo: &Ginkgo, field: Field) -> Packed {
        if let Some(packed) = self.held.get(&field) {
            return *packed;
        }
        let packed = match fields.mark(field) {
            None => {
                error!(field = field.0, "no such icon is registered");
                Packed::default()
            }
            Some(mark) => match self.pack(mark.side) {
                None => Packed::default(),
                Some((x, y)) => {
                    ginkgo.queue().write_texture(
                        TexelCopyTextureInfo {
                            texture: &self.texture,
                            mip_level: 0,
                            origin: Origin3d { x, y, z: 0 },
                            aspect: TextureAspect::All,
                        },
                        &mark.field,
                        TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(mark.side * 4),
                            rows_per_image: Some(mark.side),
                        },
                        Extent3d {
                            width: mark.side,
                            height: mark.side,
                            depth_or_array_layers: 1,
                        },
                    );
                    let sheet = SIDE as f32;
                    debug!(field = field.0, side = mark.side, "icon packed");
                    Packed {
                        uv: [
                            x as f32 / sheet,
                            y as f32 / sheet,
                            mark.side as f32 / sheet,
                            mark.side as f32 / sheet,
                        ],
                        range: mark.range,
                        side: mark.side as f32,
                    }
                }
            },
        };
        self.held.insert(field, packed);
        packed
    }

    /// Where a field of this size goes on the sheet, or `None` if there is no room left.
    ///
    /// Shelves, as the glyph atlas packs: a row is filled left to right and the next starts below
    /// the tallest field in it. Fields of one size are all the same height, so the waste is a line
    /// of it per size rather than per field.
    fn pack(&mut self, side: u32) -> Option<(u32, u32)> {
        if side > SIDE {
            return None;
        }
        if self.x + side > SIDE {
            self.y += self.shelf + GUTTER;
            self.x = 0;
            self.shelf = 0;
        }
        if self.y + side > SIDE {
            if !self.full {
                self.full = true;
                error!(
                    side = SIDE,
                    held = self.held.len(),
                    "the icon sheet is full: further marks will not draw"
                );
            }
            return None;
        }
        let at = (self.x, self.y);
        self.x += side + GUTTER;
        self.shelf = self.shelf.max(side);
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
