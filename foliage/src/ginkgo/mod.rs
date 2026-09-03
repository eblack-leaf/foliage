//! Ginkgo -- the GPU.
//!
//! The device, the surface, and everything sized against it. Every renderer in [`Ash`](crate::ash)
//! asks this for what it needs to build a pipeline and hands it back a draw, so there is one place
//! that knows what a device is and one place that talks to a surface.
//!
//! # The scale factor stops here
//!
//! The engine is written in logical pixels throughout. The surface is configured in physical ones,
//! and the display's ratio between them is applied at exactly two places, both of them below this
//! line: the surface and its depth attachment are sized in physical pixels, and the projection is
//! built from the logical area. Between them the rasteriser does the conversion, so no instance,
//! no radius and no section is ever carried in device pixels.

use std::sync::Arc;

use tracing::{debug, info};
use wgpu::{
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType,
    ColorTargetState, CommandEncoderDescriptor, CompareFunction, CompositeAlphaMode,
    CurrentSurfaceTexture, DepthStencilState, Device, DeviceDescriptor, Features,
    InstanceDescriptor, Limits, LoadOp, MultisampleState, Operations, PowerPreference, PresentMode,
    PrimitiveState, PrimitiveTopology, Queue, RenderPass, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptions, ShaderStages, StoreOp, Surface, SurfaceColorSpace,
    SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor,
};
use winit::window::Window;

use crate::color::Color;
use crate::coordinate::{Area, Section};
use crate::ginkgo::depth::Depth;
use crate::ginkgo::viewport::Viewport;

pub(crate) mod depth;
pub(crate) mod viewport;

/// The graphics context and the surface it draws to.
pub(crate) struct Ginkgo {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    configuration: SurfaceConfiguration,
    depth: Depth,
    viewport: Viewport,
    /// What every renderer binds at group 0, so a pipeline's own layout starts at one.
    layout: BindGroupLayout,
    binding: BindGroup,
    /// How many device pixels one logical pixel is. Read only where the two have to be converted,
    /// which is the surface configuration and the scissor.
    scale: f32,
}

impl Ginkgo {
    /// Acquires a device for `window` and configures a surface on it.
    ///
    /// Asynchronous because the web's is: a browser resolves an adapter and a device through
    /// promises, and there is no thread there to block. Native awaits it to completion at the
    /// callsite instead.
    pub(crate) async fn acquire(window: Arc<Window>, area: Area, scale: f32) -> Self {
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN | Backends::METAL | Backends::DX12 | Backends::GL,
            ..InstanceDescriptor::new_without_display_handle()
        });
        let surface = instance.create_surface(window).expect("surface");
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                apply_limit_buckets: false,
            })
            .await
            .expect("adapter");
        info!(adapter = adapter.get_info().name, backend = ?adapter.get_info().backend, "adapter");
        // Web and Android run on downlevel devices, and asking for more than one of those has is a
        // failure at device request rather than at the draw that needed it.
        let limits = if cfg!(any(target_family = "wasm", target_os = "android")) {
            Limits::downlevel_webgl2_defaults()
        } else {
            Limits::default()
        };
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("foliage"),
                required_features: Features::default(),
                required_limits: limits.using_resolution(adapter.limits()),
                ..DeviceDescriptor::default()
            })
            .await
            .expect("device");
        // A `Color` holds sRGB channels, which is the form a scheme is stated in and the form a
        // shader reads. Taking the surface's non-sRGB view writes those channels through unchanged;
        // the sRGB view would encode them a second time.
        let format = surface
            .get_capabilities(&adapter)
            .formats
            .first()
            .expect("surface format")
            .remove_srgb_suffix();
        info!(?format, "surface");
        let configuration = configuration(format, area, scale);
        surface.configure(&device, &configuration);
        let depth = Depth::new(&device, &configuration);
        let viewport = Viewport::new(&device, area);
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("viewport-layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let binding = device.create_bind_group(&BindGroupDescriptor {
            label: Some("viewport-binding"),
            layout: &layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: viewport.binding(),
            }],
        });
        Self {
            device,
            queue,
            surface,
            configuration,
            depth,
            viewport,
            layout,
            binding,
            scale,
        }
    }

    /// Resizes the surface and everything measured against it.
    ///
    /// The projection is rebuilt from the logical area and the attachments from the physical one,
    /// which is the whole of what the scale factor is for.
    pub(crate) fn resize(&mut self, area: Area, scale: f32) {
        self.scale = scale;
        self.configuration = configuration(self.configuration.format, area, scale);
        self.surface.configure(&self.device, &self.configuration);
        self.depth = Depth::new(&self.device, &self.configuration);
        self.viewport.resize(&self.queue, area);
        debug!(
            width = self.configuration.width,
            height = self.configuration.height,
            scale,
            "surface configured"
        );
    }

    /// The scissor rect a logical clip becomes, in the surface's own pixels.
    ///
    /// Expanded outward to whole pixels rather than truncated: a fractional scale factor makes these
    /// rects fractional, and taking the floor of both edges would shave the far side off every
    /// clipped region. Clamped to the surface, because a scissor outside the attachment is not a
    /// smaller region -- it is invalid.
    pub(crate) fn scissor(&self, clip: Section) -> (u32, u32, u32, u32) {
        let width = self.configuration.width as f32;
        let height = self.configuration.height as f32;
        let left = (clip.left() * self.scale).floor().clamp(0.0, width);
        let top = (clip.top() * self.scale).floor().clamp(0.0, height);
        let right = (clip.right() * self.scale).ceil().clamp(left, width);
        let bottom = (clip.bottom() * self.scale).ceil().clamp(top, height);
        (
            left as u32,
            top as u32,
            (right - left) as u32,
            (bottom - top) as u32,
        )
    }

    pub(crate) fn device(&self) -> &Device {
        &self.device
    }

    pub(crate) fn queue(&self) -> &Queue {
        &self.queue
    }

    /// The layout every pipeline declares at group 0, and the projection bound into it.
    pub(crate) fn viewport_layout(&self) -> &BindGroupLayout {
        &self.layout
    }

    /// One quad's worth of triangles, wound counter-clockwise and never culled.
    pub(crate) fn triangles() -> PrimitiveState {
        PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            cull_mode: None,
            ..PrimitiveState::default()
        }
    }

    /// Straight alpha blending onto the surface's own format.
    pub(crate) fn blending(&self) -> [Option<ColorTargetState>; 1] {
        [Some(ColorTargetState {
            format: self.configuration.format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: Default::default(),
        })]
    }

    /// One sample. The rounded-rectangle field antialiases its own edge from the screen-space
    /// derivative, so there is nothing left for multisampling to do to it.
    pub(crate) fn samples() -> MultisampleState {
        MultisampleState::default()
    }

    /// The depth state every pipeline shares: nearer wins, and every draw writes.
    ///
    /// `LessEqual` rather than `Less` so that a redraw of the same content leaves the same result.
    pub(crate) fn depth_state() -> Option<DepthStencilState> {
        Some(DepthStencilState {
            format: Depth::FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(CompareFunction::LessEqual),
            stencil: Default::default(),
            bias: Default::default(),
        })
    }

    /// Draws one frame, clearing to `clear` and handing the pass to `draw`.
    ///
    /// A surface that cannot be acquired is skipped rather than retried in place: it means the
    /// window is gone, occluded, or being resized, and the next paint has everything it needs to
    /// produce the same image again. Nothing is lost by not painting, because what is drawn is
    /// held by the renderers rather than consumed by the draw.
    pub(crate) fn draw(&self, clear: Color, draw: impl FnOnce(&mut RenderPass<'_>)) {
        let Some(frame) = self.acquire_frame() else {
            return;
        };
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("draw"),
            });
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("draw"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: clear.red as f64,
                            g: clear.green as f64,
                            b: clear.blue as f64,
                            a: clear.alpha as f64,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(self.depth.attachment()),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.binding, &[]);
            draw(&mut pass);
        }
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
    }

    /// The surface's next texture, reconfiguring once if the surface went stale under us.
    fn acquire_frame(&self) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) | CurrentSurfaceTexture::Suboptimal(frame) => {
                Some(frame)
            }
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                debug!("surface reconfigured");
                self.surface.configure(&self.device, &self.configuration);
                match self.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(frame)
                    | CurrentSurfaceTexture::Suboptimal(frame) => Some(frame),
                    _ => None,
                }
            }
            CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => None,
            CurrentSurfaceTexture::Validation => panic!("surface validation"),
        }
    }
}

/// The surface as the platform holds it: physical pixels, and at least one of each.
fn configuration(format: TextureFormat, area: Area, scale: f32) -> SurfaceConfiguration {
    SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format,
        width: (area.width * scale).round().max(1.0) as u32,
        height: (area.height * scale).round().max(1.0) as u32,
        present_mode: PresentMode::Fifo,
        desired_maximum_frame_latency: 2,
        alpha_mode: CompositeAlphaMode::Auto,
        color_space: SurfaceColorSpace::Auto,
        view_formats: vec![format],
    }
}
