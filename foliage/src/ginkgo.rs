use crate::willow::Willow;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use wgpu::{
    DeviceDescriptor, Features, InstanceDescriptor, Limits, PowerPreference, RequestAdapterOptions,
};

#[derive(Default)]
pub(crate) struct Ginkgo {
    context: Option<GraphicContext>,
    configuration: Option<ViewConfiguration>,
}
impl Ginkgo {
    pub(crate) async fn acquire_context(&mut self, willow: &Willow) {
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });
        let scale_factor = ScaleFactor::new(willow.window().scale_factor() as f32);
        let surface = instance.create_surface(willow.window()).expect("window");
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("adapter");
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
}
pub(crate) struct ViewConfiguration {
    pub(crate) viewport: Viewport,
    pub(crate) msaa: Msaa,
    pub(crate) depth: Depth,
    pub(crate) scale_factor: ScaleFactor,
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
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
}
pub(crate) struct Msaa {
    pub(crate) max_samples: u32,
    pub(crate) actual: u32,
    pub(crate) view: Option<wgpu::TextureView>,
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
    uniform: Uniform<[Repr; 4]>,
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
#[derive(Default)]
pub struct Viewport {}
