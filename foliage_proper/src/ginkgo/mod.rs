use bevy_ecs::prelude::Resource;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use wgpu::{
    BackendOptions, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BlendState, Buffer, BufferAddress, BufferUsages, ColorTargetState,
    CompareFunction, CompositeAlphaMode, DepthStencilState, DeviceDescriptor, Extent3d, Features,
    FilterMode, FragmentState, InstanceDescriptor, Limits, LoadOp, MultisampleState, Operations,
    Origin3d, PipelineLayout, PipelineLayoutDescriptor, PowerPreference, PresentMode,
    PrimitiveState, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, Sampler, SamplerDescriptor, ShaderModule,
    ShaderModuleDescriptor, StoreOp, SurfaceConfiguration, SurfaceError, TexelCopyBufferLayout,
    TexelCopyTextureInfo, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView, TextureViewDescriptor, VertexAttribute, VertexBufferLayout,
    VertexStepMode,
};

use binding::BindingBuilder;
use depth::Depth;
use msaa::Msaa;
use viewport::Viewport;

use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, Coordinates, Physical};
use crate::willow::Willow;

pub(crate) mod binding;
pub(crate) mod depth;
pub(crate) mod msaa;
pub mod viewport;

#[derive(Default)]
pub(crate) struct Ginkgo {
    context: Option<GraphicContext>,
    configuration: Option<ViewConfiguration>,
    viewport: Option<Viewport>,
}

impl Ginkgo {
    pub(crate) fn write_texture<TexelData: Default + Sized + Clone + Pod + Zeroable>(
        &self,
        texture: &Texture,
        position: Coordinates,
        extent: Coordinates,
        data: Vec<TexelData>,
    ) {
        self.context().queue.write_texture(
            TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: Origin3d {
                    x: position.a() as u32,
                    y: position.b() as u32,
                    z: 0,
                },
                aspect: Default::default(),
            },
            bytemuck::cast_slice(&data),
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((extent.a() * size_of::<TexelData>() as CoordinateUnit) as u32),
                rows_per_image: Some(
                    (extent.b() * size_of::<TexelData>() as CoordinateUnit) as u32,
                ),
            },
            Extent3d {
                width: extent.a() as u32,
                height: extent.b() as u32,
                depth_or_array_layers: 1,
            },
        );
    }
    #[cfg(not(target_family = "wasm"))]
    #[allow(unused)]
    pub(crate) fn png_to_cov<P: AsRef<std::path::Path>>(png: P, cov: P) {
        let data = Ginkgo::png_to_r8unorm_d2(png);
        let content = rmp_serde::to_vec(data.as_slice()).unwrap();
        std::fs::write(cov, content).unwrap();
    }
    #[cfg(not(target_family = "wasm"))]
    pub(crate) fn png_to_r8unorm_d2<P: AsRef<std::path::Path>>(path: P) -> Vec<u8> {
        let image = image::load_from_memory(std::fs::read(path).unwrap().as_slice())
            .expect("png-to-r8unorm-d2");
        let texture_data = image
            .to_rgba8()
            .enumerate_pixels()
            .map(|p| -> u8 { p.2 .0[3] })
            .collect::<Vec<u8>>();
        texture_data
    }
    pub(crate) fn vertex_buffer_layout<A: Pod + Zeroable>(
        step: VertexStepMode,
        attrs: &[VertexAttribute],
    ) -> VertexBufferLayout {
        VertexBufferLayout {
            array_stride: Ginkgo::memory_size::<A>(1),
            step_mode: step,
            attributes: attrs,
        }
    }
    pub(crate) fn create_sampler(&self, filter: bool) -> Sampler {
        let descriptor = if filter {
            SamplerDescriptor {
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                ..SamplerDescriptor::default()
            }
        } else {
            SamplerDescriptor::default()
        };
        self.context().device.create_sampler(&descriptor)
    }
    pub(crate) fn create_texture(
        &self,
        format: TextureFormat,
        coordinates: Coordinates,
        mips: u32,
        data: &[u8],
    ) -> (Texture, TextureView) {
        let texture = self.context().device.create_texture_with_data(
            &self.context().queue,
            &TextureDescriptor {
                label: Some("ginkgo-texture"),
                size: Extent3d {
                    width: coordinates.a() as u32,
                    height: coordinates.b() as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: mips,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[format],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            data,
        );
        let view = texture.create_view(&TextureViewDescriptor::default());
        (texture, view)
    }
    pub(crate) fn memory_size<B>(n: u32) -> BufferAddress {
        (std::mem::size_of::<B>() * n as usize) as BufferAddress
    }
    pub(crate) fn fragment_state<'a>(
        module: &'a ShaderModule,
        entry_point: &'a str,
        targets: &'a [Option<ColorTargetState>],
    ) -> Option<FragmentState<'a>> {
        Some(FragmentState {
            module,
            entry_point: Some(entry_point),
            compilation_options: Default::default(),
            targets,
        })
    }
    pub(crate) fn texture_bind_group_entry(view: &TextureView, binding: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(view),
        }
    }
    pub(crate) fn sampler_bind_group_entry(
        sampler: &wgpu::Sampler,
        binding: u32,
    ) -> BindGroupEntry {
        BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Sampler(sampler),
        }
    }
    pub(crate) fn triangle_list_primitive() -> PrimitiveState {
        PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        }
    }
    pub(crate) fn bind_group_layout_entry(binding: u32) -> BindingBuilder {
        BindingBuilder::new(binding)
    }
    pub(crate) fn create_bind_group_layout(
        &self,
        desc: &BindGroupLayoutDescriptor,
    ) -> BindGroupLayout {
        let bind_group_layout = self.context().device.create_bind_group_layout(desc);
        bind_group_layout
    }
    pub(crate) fn create_bind_group(&self, desc: &BindGroupDescriptor) -> BindGroup {
        let bind_group = self.context().device.create_bind_group(desc);
        bind_group
    }
    pub(crate) fn create_pipeline_layout(&self, desc: &PipelineLayoutDescriptor) -> PipelineLayout {
        let layout = self.context().device.create_pipeline_layout(desc);
        layout
    }
    pub(crate) fn create_shader(&self, shader_source: ShaderModuleDescriptor) -> ShaderModule {
        let shader = self.context().device.create_shader_module(shader_source);
        shader
    }
    pub(crate) fn create_vertex_buffer<R: Pod + Zeroable, VB: AsRef<[R]>>(
        &self,
        vb_data: VB,
    ) -> Buffer {
        let vertex_buffer =
            self.context()
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("vertex-buffer"),
                    contents: bytemuck::cast_slice(vb_data.as_ref()),
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                });
        vertex_buffer
    }
    pub(crate) fn create_pipeline(&self, desc: &RenderPipelineDescriptor) -> RenderPipeline {
        let pipeline = self.context().device.create_render_pipeline(desc);
        pipeline
    }
    pub(crate) fn uniform_bind_group_entry<U: Pod + Zeroable>(
        uniform: &Uniform<U>,
        binding: u32,
    ) -> BindGroupEntry {
        BindGroupEntry {
            binding,
            resource: uniform.buffer.as_entire_binding(),
        }
    }
    pub(crate) fn alpha_color_target_state(&self) -> [Option<ColorTargetState>; 1] {
        [Some(ColorTargetState {
            format: self.configuration().config.format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: Default::default(),
        })]
    }
    pub(crate) fn msaa_state(&self) -> MultisampleState {
        MultisampleState {
            count: self.configuration().msaa.samples(),
            ..MultisampleState::default()
        }
    }
    pub(crate) fn depth_stencil_state(&self) -> Option<DepthStencilState> {
        Some(DepthStencilState {
            format: Depth::FORMAT,
            depth_write_enabled: true,
            depth_compare: CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        })
    }
    pub(crate) fn viewport_bind_group_entry(&self, binding: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding,
            resource: self.viewport().uniform.buffer.as_entire_binding(),
        }
    }
    pub(crate) fn color_attachment<'a>(
        &'a self,
        surface_view: &'a TextureView,
        clear_color: Color,
    ) -> [Option<RenderPassColorAttachment<'a>>; 1] {
        let (view, resolve_target) = match self.configuration().msaa.view.as_ref() {
            None => (surface_view, None),
            Some(v) => (v, Some(surface_view)),
        };
        [Some(RenderPassColorAttachment {
            view,
            resolve_target,
            ops: Operations {
                load: LoadOp::Clear(clear_color.into()),
                store: self.configuration().msaa.color_attachment_store_op(),
            },
        })]
    }
    pub(crate) fn depth_stencil_attachment(&self) -> Option<RenderPassDepthStencilAttachment> {
        Some(RenderPassDepthStencilAttachment {
            view: &self.configuration().depth.as_ref().unwrap().view,
            depth_ops: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: StoreOp::Store,
            }),
            stencil_ops: Some(Operations {
                load: LoadOp::Clear(0u32),
                store: StoreOp::Store,
            }),
        })
    }
    pub(crate) fn suspend(&mut self) {
        if let Some(context) = self.context.as_mut() {
            context.surface.take();
        }
        if let Some(config) = self.configuration.as_mut() {
            config.depth.take();
        }
    }
    pub(crate) fn surface_texture(&self) -> Option<wgpu::SurfaceTexture> {
        let context = self.context();
        match context.surface.as_ref().unwrap().get_current_texture() {
            Ok(surface) => Some(surface),
            Err(err) => match err {
                SurfaceError::Timeout => None,
                SurfaceError::Outdated | SurfaceError::Lost => {
                    context
                        .surface
                        .as_ref()
                        .unwrap()
                        .configure(&context.device, &self.configuration().config);
                    context.surface.as_ref().unwrap().get_current_texture().ok()
                }
                SurfaceError::OutOfMemory | SurfaceError::Other => panic!("out-of-memory"),
            },
        }
    }
    pub(crate) fn viewport(&self) -> &Viewport {
        self.viewport.as_ref().unwrap()
    }
    pub(crate) fn position_viewport(&mut self, position: Position<Physical>) {
        self.viewport
            .as_mut()
            .unwrap()
            .set_position(position, self.context.as_ref().unwrap());
    }
    pub(crate) fn create_viewport(&mut self, willow: &Willow) {
        self.viewport.replace(Viewport::new(
            self.context(),
            Section::new(
                willow.starting_position.unwrap_or_default().coordinates,
                willow.actual_area().coordinates,
            ),
            willow.near_far.unwrap_or_default(),
        ));
    }
    pub(crate) fn size_viewport(&mut self, willow: &Willow) {
        self.viewport.as_mut().unwrap().set_size(
            willow.actual_area().to_numerical(),
            self.context.as_ref().unwrap(),
        );
    }
    pub(crate) fn recreate_surface(&mut self, willow: &Willow) {
        let surface = self
            .context()
            .instance
            .create_surface(willow.window())
            .expect("surface");
        self.context.as_mut().unwrap().surface.replace(surface);
    }
    pub(crate) fn acquired(&self) -> bool {
        self.context.is_some()
    }
    #[allow(unused)]
    pub(crate) fn configured(&self) -> bool {
        self.configuration.is_some()
    }
    pub(crate) async fn acquire_context(&mut self, willow: &Willow) {
        let instance = wgpu::Instance::new(&InstanceDescriptor {
            backends: wgpu::Backends::VULKAN
                | wgpu::Backends::METAL
                | wgpu::Backends::DX12
                | wgpu::Backends::GL,
            flags: wgpu::InstanceFlags::default(),
            backend_options: BackendOptions::default(),
        });
        let surface = instance.create_surface(willow.window()).expect("window");
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("adapter");
        let surface_format = surface
            .get_capabilities(&adapter)
            .formats
            .first()
            .expect("surface-format")
            .remove_srgb_suffix();
        let features = Features::default() | Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
        cfg_if::cfg_if! {
            if #[cfg(any(target_os = "android", target_family = "wasm"))] {
                let limits = Limits::downlevel_webgl2_defaults();
            } else {
                let limits = Limits::default();
            }
        }
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("device/queue"),
                    required_features: features,
                    required_limits: limits.using_resolution(adapter.limits()),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .expect("device/queue");
        self.context.replace(GraphicContext {
            surface: Some(surface),
            instance,
            adapter,
            device,
            queue,
            surface_format,
        });
    }
    pub(crate) fn configure_view(&mut self, willow: &Willow) {
        let scale_factor = ScaleFactor::new(willow.window().scale_factor() as f32);
        let area = willow.actual_area().max(Area::physical((1, 1)));
        let msaa = Msaa::new(self.context(), 1, area);
        let depth = Depth::new(self.context(), &msaa, area);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self.context().surface_format,
            width: area.width() as u32,
            height: area.height() as u32,
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![self.context().surface_format],
        };
        self.context()
            .surface
            .as_ref()
            .unwrap()
            .configure(&self.context().device, &config);
        self.configuration.replace(ViewConfiguration {
            msaa,
            depth: Some(depth),
            scale_factor,
            config,
        });
    }
    pub(crate) fn context(&self) -> &GraphicContext {
        self.context.as_ref().unwrap()
    }
    pub(crate) fn configuration(&self) -> &ViewConfiguration {
        self.configuration.as_ref().unwrap()
    }
}

pub(crate) struct GraphicContext {
    pub(crate) surface: Option<wgpu::Surface<'static>>,
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) surface_format: TextureFormat,
}

pub(crate) struct ViewConfiguration {
    pub(crate) msaa: Msaa,
    pub(crate) depth: Option<Depth>,
    pub(crate) scale_factor: ScaleFactor,
    pub(crate) config: SurfaceConfiguration,
}
#[derive(Copy, Clone, PartialEq, Resource)]
pub struct ScaleFactor(f32);

impl Default for ScaleFactor {
    fn default() -> Self {
        Self(1.0)
    }
}

impl ScaleFactor {
    pub fn value(&self) -> f32 {
        self.0.round()
    }
    pub(crate) fn new(f: f32) -> Self {
        Self(f)
    }
}

pub(crate) struct Uniform<Data: Pod + Zeroable> {
    pub(crate) data: Data,
    pub(crate) buffer: wgpu::Buffer,
}

impl<Data: Pod + Zeroable + PartialEq> Uniform<Data> {
    pub(crate) fn write(&mut self, context: &GraphicContext, data: Data) {
        context
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
    }
    pub(crate) fn new(context: &GraphicContext, data: Data) -> Self {
        Self {
            data,
            buffer: context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("uniform"),
                    contents: bytemuck::cast_slice(&[data]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }),
        }
    }
}

pub(crate) struct VectorUniform<Repr: Pod + Zeroable + PartialEq> {
    pub(crate) uniform: Uniform<[Repr; 4]>,
}

impl<Repr: Pod + Zeroable + PartialEq> VectorUniform<Repr> {
    pub(crate) fn new(context: &GraphicContext, d: [Repr; 4]) -> Self {
        Self {
            uniform: Uniform::new(context, d),
        }
    }
    pub(crate) fn write(&mut self, context: &GraphicContext) {
        self.uniform.write(context, self.uniform.data);
    }
    pub(crate) fn set(&mut self, i: usize, r: Repr) {
        self.uniform.data[i] = r;
    }
}
