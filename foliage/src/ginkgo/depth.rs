use crate::coordinate::area::Area;
use crate::coordinate::DeviceContext;
use crate::ginkgo::msaa::Msaa;
use crate::ginkgo::GraphicContext;
use wgpu::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

pub(crate) struct Depth {
    pub(crate) view: TextureView,
}

impl Depth {
    pub(crate) fn new(context: &GraphicContext, msaa: &Msaa, area: Area<DeviceContext>) -> Self {
        Self {
            view: context
                .device
                .create_texture(&TextureDescriptor {
                    label: Some("depth"),
                    size: Extent3d {
                        width: area.width().max(1.0) as u32,
                        height: area.height().max(1.0) as u32,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: msaa.samples(),
                    dimension: TextureDimension::D2,
                    format: Depth::FORMAT,
                    usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                    view_formats: &[Depth::FORMAT],
                })
                .create_view(&TextureViewDescriptor::default()),
        }
    }
    pub(crate) const FORMAT: TextureFormat = TextureFormat::Depth24PlusStencil8;
}
