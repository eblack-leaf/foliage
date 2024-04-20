use wgpu::{DepthStencilState, TextureFormat};

use crate::coordinate::area::Area;
use crate::coordinate::DeviceContext;
use crate::ginkgo::msaa::Msaa;

pub(crate) struct DepthTexture {
    pub(crate) format: wgpu::TextureFormat,
    #[allow(unused)]
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
}

impl DepthTexture {
    pub(crate) fn new(
        device: &wgpu::Device,
        area: Area<DeviceContext>,
        format: TextureFormat,
        msaa: &Msaa,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth-texture"),
            size: wgpu::Extent3d {
                width: area.width.max(1f32) as u32,
                height: area.height.max(1f32) as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: msaa.samples(),
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[format],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            format,
            texture,
            view,
        }
    }
    #[allow(unused)]
    pub fn depth_format(&self) -> TextureFormat {
        self.format
    }
    pub(crate) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
    #[allow(unused)]
    pub fn depth_stencil_state(&self) -> DepthStencilState {
        wgpu::DepthStencilState {
            format: self.depth_format(),
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        }
    }
}
