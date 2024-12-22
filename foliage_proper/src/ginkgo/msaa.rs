use wgpu::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormatFeatureFlags, TextureUsages,
    TextureViewDescriptor,
};

use crate::coordinate::area::Area;
use crate::coordinate::DeviceContext;
use crate::ginkgo::GraphicContext;

pub(crate) struct Msaa {
    pub(crate) max_samples: u32,
    pub(crate) actual: u32,
    pub(crate) view: Option<wgpu::TextureView>,
}

impl Msaa {
    pub(crate) fn color_attachment_store_op(&self) -> wgpu::StoreOp {
        if self.samples() == 1u32 {
            wgpu::StoreOp::Store
        } else {
            wgpu::StoreOp::Discard
        }
    }
    pub(crate) fn samples(&self) -> u32 {
        self.actual
    }
    pub(crate) fn new(context: &GraphicContext, requested: u32, area: Area<DeviceContext>) -> Self {
        let flags = context
            .adapter
            .get_texture_format_features(context.surface_format)
            .flags;
        let max_samples = {
            if flags.contains(TextureFormatFeatureFlags::MULTISAMPLE_X16) {
                16
            } else if flags.contains(TextureFormatFeatureFlags::MULTISAMPLE_X8) {
                8
            } else if flags.contains(TextureFormatFeatureFlags::MULTISAMPLE_X4) {
                4
            } else if flags.contains(TextureFormatFeatureFlags::MULTISAMPLE_X2) {
                2
            } else {
                1
            }
        };
        let actual = requested.min(max_samples);
        Self {
            max_samples,
            actual,
            view: if actual > 1 {
                Some(
                    context
                        .device
                        .create_texture(&TextureDescriptor {
                            label: Some("msaa"),
                            size: Extent3d {
                                width: area.width() as u32,
                                height: area.height() as u32,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: actual,
                            dimension: TextureDimension::D2,
                            format: context.surface_format,
                            usage: TextureUsages::RENDER_ATTACHMENT,
                            view_formats: &[],
                        })
                        .create_view(&TextureViewDescriptor::default()),
                )
            } else {
                None
            },
        }
    }
}
