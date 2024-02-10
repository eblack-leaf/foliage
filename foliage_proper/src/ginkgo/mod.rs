#[cfg(not(target_family = "wasm"))]
use std::path::Path;

use bytemuck::{Pod, Zeroable};
use depth_texture::DepthTexture;
use msaa::Msaa;
use viewport::Viewport;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupEntry, BindGroupLayoutEntry, Buffer, BufferAddress, ColorTargetState,
    DepthStencilState, Extent3d, FragmentState, InstanceDescriptor, LoadOp, MultisampleState,
    PrimitiveState, RenderPassColorAttachment, RenderPassDepthStencilAttachment, ShaderModule,
    StoreOp, TextureDimension, TextureFormat, TextureUsages, TextureView,
};
use winit::event_loop::EventLoopWindowTarget;

use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, DeviceContext, InterfaceContext};
use crate::window::{ScaleFactor, WindowDescriptor, WindowHandle};

pub mod depth_texture;
pub mod msaa;
pub mod uniform;
pub mod viewport;

#[derive(Copy, Clone)]
pub struct ClearColor(pub Color);

pub struct Ginkgo {
    pub instance: Option<wgpu::Instance>,
    pub surface: Option<wgpu::Surface<'static>>,
    pub adapter: Option<wgpu::Adapter>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,
    pub configuration: Option<wgpu::SurfaceConfiguration>,
    pub viewport: Option<Viewport>,
    pub(crate) depth_texture: Option<DepthTexture>,
    pub msaa: Option<Msaa>,
    pub clear_color: ClearColor,
    pub(crate) initialized: bool,
    scale_factor: ScaleFactor,
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
            clear_color: ClearColor(Color::BLACK),
            initialized: false,
            scale_factor: ScaleFactor::new(1.0),
        }
    }
    pub fn scale_factor(&self) -> CoordinateUnit {
        self.scale_factor.factor()
    }
    pub(crate) fn set_scale_factor(&mut self, factor: CoordinateUnit) {
        self.scale_factor = ScaleFactor::new(factor);
    }
    pub fn viewport_bind_group_entry(&self, binding: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding,
            resource: self
                .viewport
                .as_ref()
                .unwrap()
                .gpu_repr
                .buffer
                .as_entire_binding(),
        }
    }
    pub fn fragment_state<'a>(
        &'a self,
        module: &'a ShaderModule,
        entry_point: &'a str,
        targets: &'a [Option<ColorTargetState>],
    ) -> Option<FragmentState<'a>> {
        Some(FragmentState {
            module,
            entry_point,
            targets,
        })
    }
    pub fn buffer_address<T>(count: u32) -> BufferAddress {
        (std::mem::size_of::<T>() * count as usize) as BufferAddress
    }
    pub fn vertex_buffer_with_data<T: Pod + Zeroable>(&self, t: &[T], label: &str) -> Buffer {
        self.device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(t),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
    }
    pub fn index_buffer_with_data<T: Pod + Zeroable>(&self, t: &[T], label: &str) -> Buffer {
        self.device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(t),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            })
    }
    pub fn texture_bind_group_entry(view: &TextureView, binding: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(view),
        }
    }
    pub fn sampler_bind_group_entry(sampler: &wgpu::Sampler, binding: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Sampler(sampler),
        }
    }
    #[cfg(not(target_family = "wasm"))]
    pub fn png_to_cov<P: AsRef<Path>>(png: P, cov: P) {
        let data = Ginkgo::png_to_r8unorm_d2(png);
        let content = rmp_serde::to_vec(data.as_slice()).unwrap();
        std::fs::write(cov, content).unwrap();
    }
    #[cfg(not(target_family = "wasm"))]
    pub fn png_to_r8unorm_d2<P: AsRef<Path>>(path: P) -> Vec<u8> {
        let image = image::load_from_memory(std::fs::read(path).unwrap().as_slice())
            .expect("png-to-r8unorm-d2");
        let texture_data = image
            .to_rgba8()
            .enumerate_pixels()
            .map(|p| -> u8 { p.2 .0[3] })
            .collect::<Vec<u8>>();
        texture_data
    }
    pub fn texture_r8unorm_d2(
        &self,
        width: u32,
        height: u32,
        mips: u32,
        data: &[u8],
    ) -> (wgpu::Texture, TextureView) {
        let texture = self.device().create_texture_with_data(
            self.queue(),
            &wgpu::TextureDescriptor {
                label: Some("ginkgo-r8unorm-d2"),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: mips,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[TextureFormat::R8Unorm],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            data,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }
    pub fn texture_rgba8unorm_srgb_d2(
        &self,
        width: u32,
        height: u32,
        mips: u32,
        data: &[u8],
    ) -> (wgpu::Texture, TextureView) {
        let texture = self.device().create_texture_with_data(
            self.queue(),
            &wgpu::TextureDescriptor {
                label: Some("ginkgo-rgba8unorm-srgb-d2"),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: mips,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[TextureFormat::Rgba8UnormSrgb],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            data,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }
    pub fn texture_rgba8unorm_d2(
        &self,
        width: u32,
        height: u32,
        mips: u32,
        data: &[u8],
    ) -> (wgpu::Texture, TextureView) {
        let texture = self.device().create_texture_with_data(
            self.queue(),
            &wgpu::TextureDescriptor {
                label: Some("ginkgo-rgba8unorm-d2"),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: mips,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[TextureFormat::Rgba8Unorm],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            data,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }
    pub fn triangle_list_primitive() -> PrimitiveState {
        PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        }
    }
    pub fn sampler_bind_group_layout_entry(binding: u32) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        }
    }
    pub fn texture_d2_bind_group_entry(binding: u32) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
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
    pub fn alpha_color_target_state(&self) -> [Option<ColorTargetState>; 1] {
        [Some(ColorTargetState {
            format: self.configuration.as_ref().unwrap().format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: Default::default(),
        })]
    }
    pub fn msaa_multisample_state(&self) -> MultisampleState {
        self.msaa.as_ref().unwrap().multisample_state()
    }
    pub fn device(&self) -> &wgpu::Device {
        self.device.as_ref().unwrap()
    }
    pub fn queue(&self) -> &wgpu::Queue {
        self.queue.as_ref().unwrap()
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
    pub fn depth_stencil_state(&self) -> Option<DepthStencilState> {
        Some(self.depth_texture.as_ref().unwrap().depth_stencil_state())
    }
    pub(crate) fn depth_stencil_attachment(&self) -> Option<RenderPassDepthStencilAttachment> {
        Some(RenderPassDepthStencilAttachment {
            view: self.depth_texture.as_ref().unwrap().view(),
            depth_ops: Some(wgpu::Operations {
                load: LoadOp::Clear(self.viewport.as_ref().unwrap().far_layer().z),
                store: StoreOp::Store,
            }),
            stencil_ops: Some(wgpu::Operations {
                load: LoadOp::Clear(0u32),
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
    pub(crate) fn create_viewport(&mut self, section: Section<DeviceContext>) {
        let viewport = Viewport::new(
            self.device.as_ref().unwrap(),
            section,
            (0.into(), 100.into()),
        );
        self.viewport.replace(viewport);
    }
    pub(crate) async fn initialize(&mut self, window_handle: WindowHandle) {
        self.get_instance();
        self.create_surface(window_handle);
        self.get_adapter().await;
        self.get_device_and_queue().await;
    }
    pub(crate) fn post_window_initialization(
        &mut self,
        window_handle: &WindowHandle,
    ) -> Area<InterfaceContext> {
        let area = window_handle.area();
        let scale_factor = window_handle.scale_factor();
        self.create_surface_configuration(area);
        self.create_msaa(1);
        self.resize(area, scale_factor)
    }
    pub(crate) fn adjust_viewport(&mut self, section: Section<DeviceContext>) {
        self.viewport
            .as_mut()
            .unwrap()
            .adjust(self.queue.as_ref().unwrap(), section);
    }
    pub(crate) fn adjust_viewport_pos(&mut self, position: Option<Position<InterfaceContext>>) {
        if let Some(position) = position {
            self.viewport.as_mut().unwrap().adjust_pos(
                self.queue.as_ref().unwrap(),
                position.to_device(self.scale_factor.factor()),
            );
        }
    }
    pub(crate) fn resize(
        &mut self,
        area: Area<DeviceContext>,
        scale_factor: CoordinateUnit,
    ) -> Area<InterfaceContext> {
        self.scale_factor = ScaleFactor::new(scale_factor);
        self.create_surface_configuration(area);
        self.configure_surface();
        self.create_depth_texture(area);
        if self.viewport.is_none() {
            self.create_viewport(Section::new((0, 0), area));
        } else {
            let section = self.viewport.as_ref().unwrap().section();
            self.adjust_viewport(section.with_area(area));
        };
        area.to_interface(self.scale_factor.factor())
    }
    pub(crate) fn resume(
        &mut self,
        _event_loop_window_target: &EventLoopWindowTarget,
        window: &mut WindowHandle,
        _desc: &WindowDescriptor,
    ) -> Option<Area<InterfaceContext>> {
        return if !self.initialized {
            #[cfg(not(target_family = "wasm"))]
            {
                *window = WindowHandle::some(_event_loop_window_target, _desc);
                self.scale_factor = ScaleFactor::new(window.scale_factor());
                pollster::block_on(self.initialize(window.clone()));
            }
            let viewport_area = self.post_window_initialization(window);
            self.initialized = true;
            Some(viewport_area)
        } else {
            #[cfg(target_os = "android")]
            {
                self.create_surface(window.clone());
                self.resize(window.area(), window.scale_factor());
            }
            None
        };
    }
    pub(crate) fn create_surface(&mut self, window: WindowHandle) {
        if let Some(instance) = self.instance.as_ref() {
            self.scale_factor = ScaleFactor::new(window.scale_factor());
            self.surface
                .replace(instance.create_surface(window.0.unwrap()).expect("surface"));
        }
    }
    pub(crate) fn get_surface_format(&self) -> TextureFormat {
        let formats = self
            .surface
            .as_ref()
            .expect("surface")
            .get_capabilities(self.adapter.as_ref().expect("adapter"))
            .formats;
        *formats.first().expect("surface format unsupported")
    }
    pub(crate) async fn get_adapter(&mut self) {
        tracing::trace!("ginkgo:adapter-begin");
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
        tracing::trace!("ginkgo:adapter-end");
    }
    pub(crate) async fn get_device_and_queue(&mut self) {
        tracing::trace!("ginkgo:device-begin");
        let features =
            wgpu::Features::default() | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
        cfg_if::cfg_if! {
            if #[cfg(any(target_os = "android", target_family = "wasm"))] {
                let limits = wgpu::Limits::downlevel_webgl2_defaults();
            } else {
                let limits = wgpu::Limits::default();
            }
        }
        let limits = limits.using_resolution(self.adapter.as_ref().expect("adapter").limits());
        let (device, queue) = self
            .adapter
            .as_ref()
            .unwrap()
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("device/queue"),
                    required_features: features,
                    required_limits: limits,
                },
                None,
            )
            .await
            .expect("device/queue request failed");
        self.device.replace(device);
        self.queue.replace(queue);
        tracing::trace!("ginkgo:device-end");
    }
    pub(crate) fn create_surface_configuration(&mut self, area: Area<DeviceContext>) {
        let surface_format = self.get_surface_format().remove_srgb_suffix();
        self.configuration.replace(wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: area.width.max(1f32) as u32,
            height: area.height.max(1f32) as u32,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![surface_format],
        });
    }
    pub(crate) fn configure_surface(&self) {
        self.surface.as_ref().unwrap().configure(
            self.device.as_ref().unwrap(),
            self.configuration.as_ref().unwrap(),
        );
    }
    pub(crate) fn surface_texture(&mut self) -> Option<wgpu::SurfaceTexture> {
        if let Some(surface) = self.surface.as_ref() {
            return if let Ok(frame) = surface.get_current_texture() {
                Some(frame)
            } else {
                self.configure_surface();
                Some(surface.get_current_texture().expect("swapchain"))
            };
        }
        None
    }
}