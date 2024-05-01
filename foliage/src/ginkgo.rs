use crate::coordinate::{DeviceContext, LogicalContext, Position};
use crate::willow::Willow;
use crate::{Area, Section};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use wgpu::{
    CompositeAlphaMode, DeviceDescriptor, Extent3d, Features, InstanceDescriptor, Limits,
    PowerPreference, PresentMode, RequestAdapterOptions, SurfaceConfiguration, TextureDescriptor,
    TextureDimension, TextureFormat, TextureFormatFeatureFlags, TextureUsages,
    TextureViewDescriptor,
};

#[derive(Default)]
pub(crate) struct Ginkgo {
    context: Option<GraphicContext>,
    configuration: Option<ViewConfiguration>,
    viewport: Option<Viewport>,
}
impl Ginkgo {
    pub(crate) fn viewport(&self) -> &Viewport {
        self.viewport.as_ref().unwrap()
    }
    pub(crate) fn position_viewport(&mut self, position: Position<LogicalContext>) {
        todo!()
    }
    pub(crate) fn create_viewport(&mut self, willow: &Willow) {
        todo!()
    }
    pub(crate) fn resize_viewport(&mut self, willow: &Willow) {
        self.viewport.as_mut().unwrap().resize(willow.actual_area());
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
    pub(crate) view: wgpu::TextureView,
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
                        depth_or_array_layers: 0,
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
    pub fn write(&mut self, ginkgo: &Ginkgo, data: Data) {
        if self.data != data {
            ginkgo
                .context()
                .queue
                .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
        }
    }
    pub fn new(ginkgo: &Ginkgo, data: Data) -> Self {
        Self {
            data,
            buffer: ginkgo
                .context()
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
    pub fn new(ginkgo: &Ginkgo, d: [Repr; 4]) -> Self {
        Self {
            uniform: Uniform::new(ginkgo, d),
        }
    }
    pub fn write(&mut self, ginkgo: &Ginkgo) {
        self.uniform.write(ginkgo, self.uniform.data);
    }
    pub fn set(&mut self, i: usize, r: Repr) {
        self.uniform.data[i] = r;
    }
}
pub struct Viewport {}
impl Viewport {
    pub(crate) fn translate(&mut self, position: Position<LogicalContext>) {
        todo!()
    }
    pub(crate) fn resize(&mut self, area: Area<DeviceContext>) {
        todo!()
    }
    pub(crate) fn new() -> Self {
        Self {}
    }
}
#[derive(Default)]
pub struct ViewportHandle {
    translation: Position<LogicalContext>,
    area: Area<LogicalContext>,
}
impl ViewportHandle {
    pub fn translate(&mut self, position: Position<LogicalContext>) {
        todo!()
    }
    pub(crate) fn read_size_change(&mut self, area: Area<DeviceContext>) {
        todo!()
    }
    pub fn section(&self) -> Section<LogicalContext> {
        todo!()
    }
}