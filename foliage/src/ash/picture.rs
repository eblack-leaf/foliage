//! The pictures the backend is holding: one texture each.
//!
//! Not a sheet, and that is the decision rather than an omission. A picture is arbitrarily large,
//! arrives whenever a decode finishes, and there may be any number of them -- so packing them onto
//! a shared texture would need eviction, and evicting one orphans every instance already pointing at
//! it. A texture each costs a binding change between spans that draw different pictures, which is
//! what the stack's spans already cut on.

use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};
use tracing::debug;
use wgpu::{
    BindGroup, BindGroupLayout, Extent3d, Origin3d, Sampler, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureViewDescriptor,
};

use crate::ash::quad::{bind, sampler, sampling};
use crate::coordinate::Section;
use crate::ginkgo::Ginkgo;
use crate::image::Plate;

/// One picture, in the form the GPU takes it.
///
/// `#[repr(C)]` over the types the coordinate module guarantees the layout of. Which texture to bind
/// is not here: it is the span's, because a binding is not per-instance state.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct PictureQuad {
    pub(crate) section: Section,
    pub(crate) crop: [f32; 4],
    pub(crate) radii: [f32; 4],
    pub(crate) opacity: f32,
}

/// Every picture the backend has uploaded.
pub(crate) struct Pictures {
    layout: BindGroupLayout,
    sampler: Sampler,
    held: HashMap<Plate, BindGroup>,
}

impl Pictures {
    pub(crate) fn new(ginkgo: &Ginkgo) -> Self {
        let device = ginkgo.device();
        Self {
            layout: sampling(device, "picture"),
            sampler: sampler(device, "picture"),
            held: HashMap::new(),
        }
    }

    /// Uploads one picture, replacing whatever was held under that name.
    ///
    /// Called for every plate the drain loaded, whether or not it was held before, which is what
    /// makes writing the same name again -- a re-fetch at a higher resolution -- reach every element
    /// drawing it.
    pub(crate) fn upload(&mut self, ginkgo: &Ginkgo, plate: Plate, pixels: &[u8], size: (u32, u32)) {
        let device = ginkgo.device();
        let (width, height) = size;
        if width == 0 || height == 0 {
            return;
        }
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("picture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        ginkgo.queue().write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            pixels,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&TextureViewDescriptor::default());
        self.held.insert(
            plate,
            bind(device, &self.layout, &view, &self.sampler, "picture-binding"),
        );
        debug!(plate = plate.0, width, height, "picture uploaded");
    }

    /// The binding for one picture, or `None` if its pixels have not been uploaded.
    pub(crate) fn binding(&self, plate: Plate) -> Option<&BindGroup> {
        self.held.get(&plate)
    }

    /// The layout a pipeline sampling a picture declares at group 1.
    pub(crate) fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }
}
