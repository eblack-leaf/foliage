pub mod depth_texture;
pub mod msaa;
pub mod uniform;
pub mod viewport;

use crate::color::Color;
use crate::coordinate::{Area, CoordinateUnit, DeviceContext, Section};
use crate::window::{WindowDescriptor, WindowHandle};
use depth_texture::DepthTexture;
use msaa::Msaa;
use serde::{Deserialize, Serialize};
use viewport::{Viewport, ViewportHandle};
use wgpu::{
    BindGroupLayoutEntry, InstanceDescriptor, LoadOp, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, StoreOp, TextureFormat, TextureView,
};
use winit::event_loop::EventLoopWindowTarget;

#[derive(Copy, Clone)]
pub struct ClearColor(pub Color);

pub struct Ginkgo {
    pub instance: Option<wgpu::Instance>,
    pub surface: Option<wgpu::Surface>,
    pub adapter: Option<wgpu::Adapter>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,
    pub configuration: Option<wgpu::SurfaceConfiguration>,
    pub viewport: Option<Viewport>,
    pub(crate) depth_texture: Option<DepthTexture>,
    pub msaa: Option<Msaa>,
    pub clear_color: ClearColor,
    pub(crate) initialized: bool,
}

impl Ginkgo {
    pub(crate) fn new() -> Self {
        Self {
            instance: None,
            surface: None,
            adapter: None,
            device: None,
            queue: None,
            configuration: None,
            viewport: None,
            depth_texture: None,
            msaa: None,
            clear_color: ClearColor(Color::OFF_BLACK.into()),
            initialized: false,
        }
    }
    pub fn vertex_uniform_bind_group_layout_entry(binding: u32) -> BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }
    pub fn alpha_color_target_state(&self) -> [Option<wgpu::ColorTargetState>; 1] {
        [Some(wgpu::ColorTargetState {
            format: self.configuration.as_ref().unwrap().format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: Default::default(),
        })]
    }
    pub(crate) fn color_attachment<'a>(
        &'a self,
        surface_texture_view: &'a wgpu::TextureView,
    ) -> [Option<RenderPassColorAttachment>; 1] {
        let (view, resolve_target) = match self
            .msaa
            .as_ref()
            .unwrap()
            .multisampled_texture_view
            .as_ref()
        {
            None => (surface_texture_view, None),
            Some(v) => (v, Some(surface_texture_view)),
        };
        [Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target,
            ops: wgpu::Operations {
                load: LoadOp::Clear(self.clear_color.0.into()),
                store: self.msaa.as_ref().unwrap().color_attachment_store_op(),
            },
        })]
    }
    pub(crate) fn depth_stencil_attachment(&self) -> Option<RenderPassDepthStencilAttachment> {
        Some(RenderPassDepthStencilAttachment {
            view: self.depth_texture.as_ref().unwrap().view(),
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(self.viewport.as_ref().unwrap().far_layer().z),
                store: StoreOp::Store,
            }),
            stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0u32),
                store: StoreOp::Store,
            }),
        })
    }
    pub(crate) fn color_attachment_format(&self) -> [Option<TextureFormat>; 1] {
        [Some(self.configuration.as_ref().unwrap().format)]
    }
    pub(crate) fn msaa_samples(&self) -> u32 {
        self.msaa.as_ref().unwrap().samples()
    }
    pub(crate) fn render_bundle_depth_stencil(&self) -> Option<wgpu::RenderBundleDepthStencil> {
        Some(wgpu::RenderBundleDepthStencil {
            format: self.depth_texture.as_ref().unwrap().format,
            depth_read_only: false,
            stencil_read_only: false,
        })
    }
    pub(crate) fn get_instance(&mut self) {
        self.instance
            .replace(wgpu::Instance::new(InstanceDescriptor {
                backends: wgpu::Backends::all(),
                flags: wgpu::InstanceFlags::default(),
                dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
                gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
            }));
    }
    pub(crate) fn suspend(&mut self) {
        #[cfg(target_os = "android")]
        {
            self.surface.take();
            self.depth_texture.take();
        }
    }
    pub(crate) fn create_depth_texture(&mut self, area: Area<DeviceContext>) {
        self.depth_texture.replace(DepthTexture::new(
            self.device.as_ref().unwrap(),
            area,
            TextureFormat::Depth24PlusStencil8,
            self.msaa.as_ref().unwrap(),
        ));
    }
    pub(crate) fn create_msaa(&mut self, requested: u32) {
        let msaa_flags = self
            .adapter
            .as_ref()
            .unwrap()
            .get_texture_format_features(self.configuration.as_ref().unwrap().format)
            .flags;
        let max_sample_count = {
            // TODO add 16 if possible
            if msaa_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X8) {
                8
            } else if msaa_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4) {
                4
            } else if msaa_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X2) {
                2
            } else {
                1
            }
        };
        self.msaa.replace(Msaa::new(
            self.device.as_ref().unwrap(),
            self.configuration.as_ref().unwrap(),
            max_sample_count as u32,
            requested,
        ));
    }
    pub(crate) fn create_viewport(
        &mut self,
        section: Section<DeviceContext>,
        scale_factor: CoordinateUnit,
    ) -> ViewportHandle {
        let viewport = Viewport::new(
            self.device.as_ref().unwrap(),
            section,
            (0.into(), 100.into()),
        );
        self.viewport.replace(viewport);
        ViewportHandle::new(section.to_interface(scale_factor))
    }
    pub(crate) async fn initialize(&mut self, window_handle: &WindowHandle) {
        self.get_instance();
        self.create_surface(window_handle);
        self.get_adapter().await;
        self.get_device_and_queue().await;
    }
    pub(crate) fn post_window_initialization(
        &mut self,
        window_handle: &WindowHandle,
    ) -> ViewportHandle {
        let area = window_handle.area();
        let scale_factor = window_handle.scale_factor();
        self.create_surface_configuration(area);
        self.create_msaa(1);
        self.resize(area, scale_factor)
    }
    pub(crate) fn adjust_viewport(
        &mut self,
        section: Section<DeviceContext>,
        scale_factor: CoordinateUnit,
    ) -> ViewportHandle {
        self.viewport
            .as_mut()
            .unwrap()
            .adjust(self.queue.as_ref().unwrap(), section);
        ViewportHandle::new(section.to_interface(scale_factor))
    }
    pub(crate) fn resize(
        &mut self,
        area: Area<DeviceContext>,
        scale_factor: CoordinateUnit,
    ) -> ViewportHandle {
        self.create_surface_configuration(area);
        self.configure_surface();
        self.create_depth_texture(area);
        let viewport_handle = if self.viewport.is_none() {
            self.create_viewport(Section::new((0, 0), (area)), scale_factor)
        } else {
            let section = self.viewport.as_ref().unwrap().section();
            self.adjust_viewport(section.with_area(area), scale_factor)
        };
        viewport_handle
    }
    pub(crate) fn resume(
        &mut self,
        event_loop_window_target: &EventLoopWindowTarget<()>,
        window: &mut WindowHandle,
        desc: &WindowDescriptor,
    ) -> Option<ViewportHandle> {
        return if !self.initialized {
            #[cfg(not(target_family = "wasm"))]
            {
                *window = WindowHandle::some(event_loop_window_target, desc);
                futures::executor::block_on(self.initialize(window));
            }
            let viewport_handle = self.post_window_initialization(window);
            self.initialized = true;
            Some(viewport_handle)
        } else {
            #[cfg(target_os = "android")]
            {
                self.create_surface(window);
                return self.resize(window.area());
            }
            None
        };
    }
    pub(crate) fn create_surface(&mut self, window: &WindowHandle) {
        if let Some(instance) = self.instance.as_ref() {
            self.surface
                .replace(unsafe { instance.create_surface(window.value()).expect("surface") });
        }
    }
    pub(crate) fn get_surface_format(&self) -> TextureFormat {
        *self
            .surface
            .as_ref()
            .expect("surface")
            .get_capabilities(self.adapter.as_ref().expect("adapter"))
            .formats
            .first()
            .expect("surface format unsupported")
    }
    pub(crate) async fn get_adapter(&mut self) {
        let adapter = self
            .instance
            .as_ref()
            .unwrap()
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: self.surface.as_ref(),
            })
            .await
            .expect("adapter request failed");
        self.adapter.replace(adapter);
    }
    pub(crate) async fn get_device_and_queue(&mut self) {
        let features =
            wgpu::Features::default() | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
        let limits = wgpu::Limits::default();
        #[cfg(any(target_os = "android", target_family = "wasm"))]
        let limits = wgpu::Limits::downlevel_webgl2_defaults();
        let limits = limits.using_resolution(self.adapter.as_ref().expect("adapter").limits());
        let (device, queue) = self
            .adapter
            .as_ref()
            .unwrap()
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("device/queue"),
                    features,
                    limits,
                },
                None,
            )
            .await
            .expect("device/queue request failed");
        self.device.replace(device);
        self.queue.replace(queue);
    }
    pub(crate) fn create_surface_configuration(&mut self, area: Area<DeviceContext>) {
        let surface_format = self.get_surface_format();
        self.configuration.replace(wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: area.width.max(1f32) as u32,
            height: area.height.max(1f32) as u32,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![surface_format],
        });
    }
    pub(crate) fn configure_surface(&mut self) {
        self.surface.as_ref().unwrap().configure(
            self.device.as_ref().unwrap(),
            self.configuration.as_ref().unwrap(),
        );
    }
    pub(crate) fn surface_texture(&mut self) -> wgpu::SurfaceTexture {
        if let Ok(frame) = self.surface.as_ref().unwrap().get_current_texture() {
            frame
        } else {
            self.configure_surface();
            self.surface
                .as_ref()
                .unwrap()
                .get_current_texture()
                .expect("swapchain")
        }
    }
}
