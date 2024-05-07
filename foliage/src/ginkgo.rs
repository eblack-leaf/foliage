use bevy_ecs::prelude::Resource;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupEntry, BlendState, ColorTargetState, CompareFunction, CompositeAlphaMode,
    DepthStencilState, DeviceDescriptor, Extent3d, Features, InstanceDescriptor, Limits, LoadOp,
    MultisampleState, Operations, PowerPreference, PresentMode, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RequestAdapterOptions, StoreOp, SurfaceConfiguration,
    TextureDescriptor, TextureDimension, TextureFormat, TextureFormatFeatureFlags, TextureUsages,
    TextureView, TextureViewDescriptor,
};

use crate::color::Color;
use crate::coordinate::{DeviceContext, NumericalContext, Position};
use crate::willow::{NearFarDescriptor, Willow};
use crate::{Area, CoordinateUnit, Section};

#[derive(Default)]
pub(crate) struct Ginkgo {
    context: Option<GraphicContext>,
    configuration: Option<ViewConfiguration>,
    viewport: Option<Viewport>,
}
impl Ginkgo {
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
    ) -> [Option<RenderPassColorAttachment>; 1] {
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
            view: &self.configuration().depth.view,
            depth_ops: Some(Operations {
                load: LoadOp::Clear(self.viewport().near_far.far.0),
                store: StoreOp::Store,
            }),
            stencil_ops: Some(Operations {
                load: LoadOp::Clear(0u32),
                store: StoreOp::Store,
            }),
        })
    }
    pub(crate) fn surface_texture(&self) -> wgpu::SurfaceTexture {
        let context = self.context();
        return if let Ok(frame) = context.surface.get_current_texture() {
            frame
        } else {
            context
                .surface
                .configure(&context.device, &self.configuration().config);
            context
                .surface
                .get_current_texture()
                .expect("swapchain-configure")
        };
    }
    pub(crate) fn viewport(&self) -> &Viewport {
        self.viewport.as_ref().unwrap()
    }
    pub(crate) fn position_viewport(&mut self, position: Position<NumericalContext>) {
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
        self.context.as_mut().unwrap().surface = self
            .context()
            .instance
            .create_surface(willow.window())
            .expect("surface");
    }
    pub(crate) fn acquired(&self) -> bool {
        self.context.is_some()
    }
    pub(crate) fn configured(&self) -> bool {
        self.configuration.is_some()
    }
    pub(crate) async fn acquire_context(&mut self, willow: &Willow) {
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
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
                },
                None,
            )
            .await
            .expect("device/queue");
        self.context.replace(GraphicContext {
            surface,
            instance,
            adapter,
            device,
            queue,
            surface_format,
        });
    }
    pub(crate) fn configure_view(&mut self, willow: &Willow) {
        let scale_factor = ScaleFactor::new(willow.window().scale_factor() as f32);
        let area = willow.actual_area().max(Area::device((1, 1)));
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
            .configure(&self.context().device, &config);
        self.configuration.replace(ViewConfiguration {
            msaa,
            depth,
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
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) surface_format: TextureFormat,
}
pub(crate) struct ViewConfiguration {
    pub(crate) msaa: Msaa,
    pub(crate) depth: Depth,
    pub(crate) scale_factor: ScaleFactor,
    pub(crate) config: SurfaceConfiguration,
}
pub struct ScaleFactor(f32);
impl Default for ScaleFactor {
    fn default() -> Self {
        Self(1.0)
    }
}
impl ScaleFactor {
    pub fn value(&self) -> f32 {
        self.0
    }
    pub fn new(f: f32) -> Self {
        Self(f.round())
    }
}
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
pub struct Msaa {
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
    pub fn samples(&self) -> u32 {
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
pub struct Uniform<Data: Pod + Zeroable> {
    pub data: Data,
    pub buffer: wgpu::Buffer,
}
impl<Data: Pod + Zeroable + PartialEq> Uniform<Data> {
    pub fn write(&mut self, context: &GraphicContext, data: Data) {
        if self.data != data {
            context
                .queue
                .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
        }
    }
    pub fn new(context: &GraphicContext, data: Data) -> Self {
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
pub struct AggregateUniform<Repr: Pod + Zeroable + PartialEq> {
    pub uniform: Uniform<[Repr; 4]>,
}
impl<Repr: Pod + Zeroable + PartialEq> AggregateUniform<Repr> {
    pub fn new(context: &GraphicContext, d: [Repr; 4]) -> Self {
        Self {
            uniform: Uniform::new(context, d),
        }
    }
    pub fn write(&mut self, context: &GraphicContext) {
        self.uniform.write(context, self.uniform.data);
    }
    pub fn set(&mut self, i: usize, r: Repr) {
        self.uniform.data[i] = r;
    }
}
type ViewportRepresentation = [[CoordinateUnit; 4]; 4];
pub struct Viewport {
    translation: Position<NumericalContext>,
    area: Area<NumericalContext>,
    near_far: NearFarDescriptor,
    matrix: ViewportRepresentation,
    uniform: Uniform<ViewportRepresentation>,
}
impl Viewport {
    pub(crate) fn set_position(
        &mut self,
        position: Position<NumericalContext>,
        context: &GraphicContext,
    ) {
        self.translation = position.to_numerical();
        self.matrix = self.remake();
        self.uniform.write(context, self.matrix);
    }
    pub(crate) fn set_size(&mut self, area: Area<NumericalContext>, context: &GraphicContext) {
        self.area = area;
        self.matrix = self.remake();
        self.uniform.write(context, self.matrix);
    }

    fn remake(&mut self) -> ViewportRepresentation {
        Self::generate(
            Section::new(self.translation.coordinates, self.area.coordinates),
            self.near_far,
        )
    }
    pub(crate) fn new(
        context: &GraphicContext,
        section: Section<NumericalContext>,
        near_far: NearFarDescriptor,
    ) -> Self {
        let matrix = Self::generate(section, near_far);
        Self {
            translation: section.position.to_numerical(),
            area: section.area,
            near_far,
            matrix,
            uniform: Uniform::new(context, matrix),
        }
    }
    fn generate(
        section: Section<NumericalContext>,
        near_far: NearFarDescriptor,
    ) -> ViewportRepresentation {
        let right_left = 2f32 / (section.right() - section.x());
        let top_bottom = 2f32 / (section.y() - section.bottom());
        let nf = 1f32 / (near_far.far.0 - near_far.near.0);
        [
            [right_left, 0f32, 0f32, right_left * -section.x() - 1f32],
            [0f32, top_bottom, 0f32, top_bottom * -section.y() + 1f32],
            [0f32, 0f32, nf, nf * near_far.near.0],
            [0f32, 0f32, 0f32, 1f32],
        ]
    }
}
#[derive(Default, Resource)]
pub struct ViewportHandle {
    translation: Position<NumericalContext>,
    area: Area<NumericalContext>,
    changes: bool,
}

impl ViewportHandle {
    pub(crate) fn new(area: Area<NumericalContext>) -> Self {
        Self {
            translation: Position::default(),
            area,
            changes: false,
        }
    }
    pub fn translate(&mut self, position: Position<NumericalContext>) {
        self.translation += position;
        self.changes = true;
    }
    pub(crate) fn changes(&mut self) -> Option<Position<NumericalContext>> {
        if self.changes {
            self.changes = false;
            return Some(self.translation);
        }
        None
    }
    pub(crate) fn resize(&mut self, area: Area<NumericalContext>) {
        self.area = area;
    }
    pub fn section(&self) -> Section<NumericalContext> {
        Section::new(self.translation.coordinates, self.area.coordinates)
    }
}
