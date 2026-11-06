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
//!
//! # A surface, or a picture
//!
//! What a frame is drawn into is a window's surface while an app runs, and a texture with no window
//! behind it under the [`Greenhouse`](crate::Greenhouse) -- the same renderers, the same pass, and a
//! picture read back at the end of it instead of one presented. Everything above the draw is the
//! one path either way, which is what makes a picture of a frame a picture of what an app shows.

use std::sync::Arc;

use tracing::{debug, info};
use wgpu::{
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType,
    ColorTargetState, CommandEncoderDescriptor, CompareFunction, CompositeAlphaMode,
    CurrentSurfaceTexture, DepthStencilState, Device, DeviceDescriptor, Features,
    InstanceDescriptor, InstanceFlags, Limits, LoadOp, MultisampleState, Operations,
    PowerPreference, PresentMode, PrimitiveState, PrimitiveTopology, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, ShaderStages, StoreOp,
    Surface, SurfaceColorSpace, SurfaceConfiguration, Texture, TextureFormat, TextureUsages,
    TextureView, TextureViewDescriptor,
};
use winit::window::Window;

use crate::color::Color;
use crate::coordinate::{Area, Section};
use crate::ginkgo::depth::Depth;
use crate::ginkgo::viewport::Viewport;

pub(crate) mod depth;
pub(crate) mod viewport;

/// What a frame is drawn into.
enum Target {
    /// A window's, presented at the end of every draw.
    Surface(Surface<'static>),
    /// A texture nothing presents, read back by whoever asked for the picture. Never made on the
    /// web, where a test cannot wait on the device to read one.
    #[cfg_attr(target_family = "wasm", allow(dead_code))]
    Offscreen(Texture),
}

/// The graphics context and the surface it draws to.
pub(crate) struct Ginkgo {
    device: Device,
    queue: Queue,
    target: Target,
    /// The target's format and size in device pixels, whichever target it is: a texture is
    /// described the way a surface is configured, so nothing that reads one has to ask which.
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
            // A debug build otherwise asks wgpu to name every object it makes, and naming a Vulkan
            // one goes through `vkSetDebugUtilsObjectNameEXT`. The Android emulator's driver
            // segfaults inside that call, so a debug build of an app that runs everywhere else dies
            // before its first frame on the one device most likely to be developed against.
            //
            // The names are a convenience for reading a capture. Giving them up on Android costs
            // that and nothing else.
            flags: match cfg!(target_os = "android") {
                true => InstanceFlags::empty(),
                false => InstanceFlags::from_build_config(),
            },
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
        Self::assemble(
            device,
            queue,
            Target::Surface(surface),
            configuration,
            area,
            scale,
        )
    }

    /// Acquires a device with no window, drawing into a texture that can be read back.
    ///
    /// `None` where the machine has no adapter at all -- a container with no GPU and no software
    /// rasteriser installed -- because a test asking for a picture there has nothing to compare and
    /// should say so rather than fail. A software adapter is taken where it is the only one.
    #[cfg(not(target_family = "wasm"))]
    pub(crate) async fn offscreen(area: Area, scale: f32) -> Option<Self> {
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN | Backends::METAL | Backends::DX12 | Backends::GL,
            flags: InstanceFlags::from_build_config(),
            ..InstanceDescriptor::new_without_display_handle()
        });
        let mut adapter = None;
        for fallback in [false, true] {
            adapter = instance
                .request_adapter(&RequestAdapterOptions {
                    power_preference: PowerPreference::default(),
                    force_fallback_adapter: fallback,
                    compatible_surface: None,
                    apply_limit_buckets: false,
                })
                .await
                .ok();
            if adapter.is_some() {
                break;
            }
        }
        let adapter = adapter?;
        info!(adapter = adapter.get_info().name, backend = ?adapter.get_info().backend, "offscreen adapter");
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("foliage-offscreen"),
                required_features: Features::default(),
                // The least any target has, so a picture drawn here is drawn under the limits the
                // most constrained platform holds the renderers to.
                required_limits: Limits::downlevel_defaults().using_resolution(adapter.limits()),
                ..DeviceDescriptor::default()
            })
            .await
            .ok()?;
        // Not sRGB, for the reason a surface's view is not: a `Color` is already sRGB channels, and
        // the bytes read back are the ones a surface would have shown.
        let configuration = configuration(TextureFormat::Rgba8Unorm, area, scale);
        let canvas = canvas(&device, &configuration);
        Some(Self::assemble(
            device,
            queue,
            Target::Offscreen(canvas),
            configuration,
            area,
            scale,
        ))
    }

    /// Everything sized against the target, once there is a device and a target to size against.
    fn assemble(
        device: Device,
        queue: Queue,
        target: Target,
        configuration: SurfaceConfiguration,
        area: Area,
        scale: f32,
    ) -> Self {
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
            target,
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
        match &mut self.target {
            Target::Surface(surface) => surface.configure(&self.device, &self.configuration),
            Target::Offscreen(texture) => *texture = canvas(&self.device, &self.configuration),
        }
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

    /// How many device pixels one logical pixel is. Read where the two have to be converted, which
    /// is the surface, the scissor, and the density a glyph is cut at.
    pub(crate) fn scale(&self) -> f32 {
        self.scale
    }

    pub(crate) fn queue(&self) -> &Queue {
        &self.queue
    }

    /// The format everything is drawn in: the window's, and every picture drawn apart from it.
    pub(crate) fn format(&self) -> TextureFormat {
        self.configuration.format
    }

    /// How large the target is, in device pixels.
    pub(crate) fn size(&self) -> (u32, u32) {
        (self.configuration.width, self.configuration.height)
    }

    /// A depth attachment of the target's size, for a picture drawn apart from it.
    pub(crate) fn depth(&self) -> Depth {
        Depth::new(&self.device, &self.configuration)
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
    /// Each of `layers` is drawn first, in a pass of its own handed to `layer` with its index:
    /// cleared to nothing, so what is drawn into it is all that is in it. The window's pass comes
    /// after them all, so a picture is finished before anything lays it down.
    ///
    /// A surface that cannot be acquired is skipped rather than retried in place: it means the
    /// window is gone, occluded, or being resized, and the next paint has everything it needs to
    /// produce the same image again. Nothing is lost by not painting, because what is drawn is
    /// held by the renderers rather than consumed by the draw.
    pub(crate) fn draw(
        &self,
        clear: Color,
        layers: &[(&TextureView, &Depth)],
        mut layer: impl FnMut(usize, &mut RenderPass<'_>),
        draw: impl FnOnce(&mut RenderPass<'_>),
    ) {
        let (frame, view) = match &self.target {
            Target::Surface(surface) => {
                let Some(frame) = self.acquire_frame(surface) else {
                    return;
                };
                let view = frame.texture.create_view(&TextureViewDescriptor::default());
                (Some(frame), view)
            }
            Target::Offscreen(texture) => {
                (None, texture.create_view(&TextureViewDescriptor::default()))
            }
        };
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("draw"),
            });
        for (index, (target, depth)) in layers.iter().enumerate() {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("layer"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(depth.attachment()),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.binding, &[]);
            layer(index, &mut pass);
        }
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
        if let Some(frame) = frame {
            self.queue.present(frame);
        }
    }

    /// What the last draw left in an offscreen target: its width and height in device pixels, and
    /// its texels as RGBA, row-major, with no padding between rows.
    ///
    /// `None` on a surface, which is presented rather than kept, or if the copy could not be
    /// mapped. Waits for the device, so it is a test's call and never a frame's.
    #[cfg(not(target_family = "wasm"))]
    pub(crate) fn read(&self) -> Option<(u32, u32, Vec<u8>)> {
        let Target::Offscreen(texture) = &self.target else {
            return None;
        };
        let width = self.configuration.width;
        let height = self.configuration.height;
        // A copy's rows are aligned to what the device requires, and a picture's are not.
        let row = width * 4;
        let aligned =
            row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (aligned * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("readback"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(aligned),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));
        let slice = buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |mapped| {
            sender.send(mapped).ok();
        });
        self.device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        receiver.recv().ok()?.ok()?;
        let mut pixels = Vec::with_capacity((row * height) as usize);
        {
            let mapped = slice.get_mapped_range().ok()?;
            for line in mapped.chunks(aligned as usize).take(height as usize) {
                pixels.extend_from_slice(&line[..row as usize]);
            }
        }
        buffer.unmap();
        Some((width, height, pixels))
    }

    /// The surface's next texture, reconfiguring once if the surface went stale under us.
    fn acquire_frame(&self, surface: &Surface<'static>) -> Option<wgpu::SurfaceTexture> {
        match surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) | CurrentSurfaceTexture::Suboptimal(frame) => {
                Some(frame)
            }
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                debug!("surface reconfigured");
                surface.configure(&self.device, &self.configuration);
                match surface.get_current_texture() {
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

/// The texture an offscreen target draws into, at the size and format it is described as.
fn canvas(device: &Device, configuration: &SurfaceConfiguration) -> Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("offscreen"),
        size: wgpu::Extent3d {
            width: configuration.width,
            height: configuration.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: configuration.format,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}
