pub struct Msaa {
    #[allow(unused)]
    pub(crate) max_samples: u32,
    pub(crate) actual: u32,
    pub(crate) multisampled_texture_view: Option<wgpu::TextureView>,
}

impl Msaa {
    pub(crate) fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        max_samples: u32,
        requested: u32,
    ) -> Self {
        let actual = requested.min(max_samples);
        let multisampled_texture_view = if actual > 1u32 {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("msaa"),
                size: wgpu::Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: actual,
                dimension: wgpu::TextureDimension::D2,
                format: config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            Some(texture.create_view(&wgpu::TextureViewDescriptor::default()))
        } else {
            None
        };
        Self {
            max_samples,
            actual,
            multisampled_texture_view,
        }
    }
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
    #[allow(unused)]
    pub(crate) fn multisampled_texture_view(&self) -> Option<&wgpu::TextureView> {
        self.multisampled_texture_view.as_ref()
    }
    pub(crate) fn multisample_state(&self) -> wgpu::MultisampleState {
        wgpu::MultisampleState {
            count: self.samples(),
            ..wgpu::MultisampleState::default()
        }
    }
}
