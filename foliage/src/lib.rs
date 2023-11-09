mod color;
mod coordinate;
mod job;

use crate::color::Color;
use crate::coordinate::{CoordinateUnit, DeviceContext, InterfaceContext, Section};
use crate::job::Job;
use bevy_ecs::prelude::{IntoSystemConfigs, Resource};
use coordinate::Area;
use gloo_worker::{HandlerId, Worker, WorkerScope};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use wgpu::{InstanceDescriptor, TextureFormat};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopBuilder, EventLoopWindowTarget};
use winit::window::WindowBuilder;

pub struct Foliage {
    window_descriptor: Option<WindowDescriptor>,
    leaf_queue: (),
    render_request: RenderRequest,
    job: Job,
}
impl Foliage {
    pub fn new() -> Self {
        Self {
            window_descriptor: None,
            leaf_queue: (),
            render_request: RenderRequest::default(),
            job: Job::new(),
        }
    }
    pub fn with_window_descriptor(mut self, desc: WindowDescriptor) -> Self {
        self.window_descriptor.replace(desc);
        self
    }
    pub fn with_leaf(mut self) -> Self {
        todo!()
    }
    pub fn run(mut self) {
        cfg_if::cfg_if! {
            if #[cfg(target_family = "wasm")] {
                wasm_bindgen_futures::spawn_local(self.internal_run());
            } else {
                futures::executor::block_on(self.internal_run());
            }
        }
    }
    async fn internal_run(mut self) {
        let event_loop = EventLoopBuilder::<RenderResponse>::with_user_event()
            .build()
            .expect("event-loop");
        let window = Rc::new(Window::new(
            &event_loop,
            self.window_descriptor.unwrap_or_default(),
        ));
        let mut gfx_context = GfxContext::new(window.clone());
        #[cfg(target_family = "wasm")]
        gfx_context.acquire_surface(&window);
        gfx_context.get_adapter().await;
        gfx_context.get_device_and_queue().await;
        // TODO bridge code here
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                use winit::platform::web::EventLoopExtWebSys;
                let event_loop_function = EventLoop::spawn;
            } else {
                let event_loop_function = EventLoop::run;
            }
        }
        let _ = (event_loop_function)(
            event_loop,
            |event, event_loop_window_target: &EventLoopWindowTarget<RenderResponse>| {
                match event {
                    Event::NewEvents(_) => {}
                    Event::WindowEvent {
                        window_id: _window_id,
                        event: w_event,
                    } => match w_event {
                        WindowEvent::ActivationTokenDone { .. } => {}
                        WindowEvent::Resized(size) => {
                            // adjust viewport handle
                            // self.render_request.viewport_handle.replace(...);
                            gfx_context.configure_surface((size.width, size.height).into());
                        }
                        WindowEvent::Moved(_) => {}
                        WindowEvent::CloseRequested => {
                            event_loop_window_target.exit();
                        }
                        WindowEvent::Destroyed => {}
                        WindowEvent::DroppedFile(_) => {}
                        WindowEvent::HoveredFile(_) => {}
                        WindowEvent::HoveredFileCancelled => {}
                        WindowEvent::Focused(_) => {}
                        WindowEvent::KeyboardInput { .. } => {}
                        WindowEvent::ModifiersChanged(_) => {}
                        WindowEvent::Ime(_) => {}
                        WindowEvent::CursorMoved { .. } => {}
                        WindowEvent::CursorEntered { .. } => {}
                        WindowEvent::CursorLeft { .. } => {}
                        WindowEvent::MouseWheel { .. } => {}
                        WindowEvent::MouseInput { .. } => {}
                        WindowEvent::TouchpadMagnify { .. } => {}
                        WindowEvent::SmartMagnify { .. } => {}
                        WindowEvent::TouchpadRotate { .. } => {}
                        WindowEvent::TouchpadPressure { .. } => {}
                        WindowEvent::AxisMotion { .. } => {}
                        WindowEvent::Touch(_) => {}
                        WindowEvent::ScaleFactorChanged { .. } => {}
                        WindowEvent::ThemeChanged(_) => {}
                        WindowEvent::Occluded(_) => {}
                        WindowEvent::RedrawRequested => {
                            // render here for now
                            window.as_ref().0.as_ref().expect("window").request_redraw();
                        }
                    },
                    Event::DeviceEvent { .. } => {}
                    Event::UserEvent(_e) => {
                        // read _e as renderer event and handle responses
                        window.as_ref().0.as_ref().expect("window").request_redraw();
                    }
                    Event::Suspended => {
                        // TODO run this from signal in other thread
                        gfx_context.suspend();
                    }
                    Event::Resumed => {
                        // adjust viewport handle here
                        // TODO run this from signal in other thread
                        gfx_context.resume(&window);
                    }
                    Event::AboutToWait => {
                        if self.job.resumed() {
                            self.job.exec_main();
                            window.as_ref().0.as_ref().expect("window").request_redraw();
                        }
                    }
                    Event::LoopExiting => {
                        self.job.exec_teardown();
                    }
                    Event::MemoryWarning => {}
                }
            },
        );
    }
}
#[derive(Copy, Clone)]
pub struct ClearColor(pub Color);
#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct ViewportHandle {
    pub section: Section<InterfaceContext>,
}
pub struct Viewport {}
pub(crate) struct DepthTexture {}
impl DepthTexture {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
pub struct Msaa {}
impl Msaa {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
pub struct GfxContext {
    pub instance: Option<wgpu::Instance>,
    pub surface: Option<wgpu::Surface>,
    pub adapter: Option<wgpu::Adapter>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,
    pub configuration: Option<wgpu::SurfaceConfiguration>,
    pub viewport: Option<Viewport>,
    pub(crate) depth_texture: Option<DepthTexture>,
    pub msaa: Option<Msaa>,
    pub(crate) window: Option<Rc<Window>>,
    pub clear_color: ClearColor,
    pub(crate) initialized: bool,
}
impl GfxContext {
    pub(crate) fn new(window: Rc<Window>) -> Self {
        Self {
            instance: Some(wgpu::Instance::new(InstanceDescriptor {
                backends: wgpu::Backends::all(),
                flags: wgpu::InstanceFlags::default(),
                dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
                gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
            })),
            surface: None,
            adapter: None,
            device: None,
            queue: None,
            configuration: None,
            viewport: None,
            depth_texture: None,
            msaa: None,
            window: Some(window),
            clear_color: ClearColor(Color::GREY_DARK.into()),
            initialized: false,
        }
    }
    pub(crate) fn suspend(&mut self) {
        #[cfg(target_os = "android")]
        {
            self.surface.take();
            self.depth_texture.take();
        }
    }
    pub(crate) fn create_depth_texture(&mut self, area: Area<DeviceContext>) {
        self.depth_texture.replace(DepthTexture::new());
    }
    pub(crate) fn create_msaa(&mut self) {
        self.msaa.replace(Msaa::new());
    }
    pub(crate) fn create_viewport(&mut self, section: ()) {
        todo!()
    }
    pub(crate) fn resume(&mut self, window: &Window) {
        if !self.initialized {
            #[cfg(not(target_family = "wasm"))]
            self.create_surface(window);
            let area = window.area();
            self.configure_surface(area);
            self.create_depth_texture(area);
            self.initialized = true;
        } else {
            #[cfg(target_os = "android")]
            {
                self.create_surface(window);
                self.configure_surface(window.area());
            }
        }
    }
    pub(crate) fn create_surface(&mut self, window: &Window) {
        if let Some(instance) = self.instance.as_ref() {
            self.surface.replace(unsafe {
                instance
                    .create_surface(window.0.as_ref().expect("window"))
                    .expect("surface")
            });
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
        let limits = wgpu::Limits::default()
            .using_resolution(self.adapter.as_ref().expect("adapter").limits());
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
    pub(crate) fn configure_surface(&mut self, area: Area<DeviceContext>) {
        self.create_surface_configuration(area);
        self.surface.as_ref().unwrap().configure(
            self.device.as_ref().unwrap(),
            self.configuration.as_ref().unwrap(),
        );
    }
}
impl Worker for GfxContext {
    type Message = ();
    type Input = ();
    type Output = ();

    fn create(scope: &WorkerScope<Self>) -> Self {
        todo!()
    }

    fn update(&mut self, scope: &WorkerScope<Self>, msg: Self::Message) {
        todo!()
    }

    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, id: HandlerId) {
        todo!()
    }
}
#[derive(Default)]
pub struct WindowDescriptor {
    desktop_dimensions: Option<Area<DeviceContext>>,
    title: Option<&'static str>,
}
impl WindowDescriptor {
    pub fn new() -> Self {
        Self {
            desktop_dimensions: None,
            title: None,
        }
    }
    pub fn with_desktop_dimensions<A: Into<Area<DeviceContext>>>(mut self, dims: A) -> Self {
        self.desktop_dimensions.replace(dims.into());
        self
    }
    pub fn with_title(mut self, title: &'static str) -> Self {
        self.title.replace(title);
        self
    }
}
pub(crate) struct Window(pub(crate) Option<winit::window::Window>);
impl Window {
    pub(crate) fn new<Hook>(
        event_loop: &EventLoop<Hook>,
        window_descriptor: WindowDescriptor,
    ) -> Self {
        let mut builder = WindowBuilder::new()
            .with_resizable(false)
            .with_title(window_descriptor.title.unwrap_or_default());
        #[cfg(all(
            not(target_family = "wasm"),
            not(target_os = "android"),
            not(target_os = "ios")
        ))]
        if let Some(dims) = window_descriptor.desktop_dimensions {
            builder =
                builder.with_inner_size(PhysicalSize::new(dims.width as i32, dims.height as i32));
        }
        let window = builder.build(event_loop).expect("window creation");
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            let canvas = window.canvas().expect("Couldn't get canvas");
            canvas.style().set_css_text("height: 100%; width: 100%;");
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| body.append_child(&canvas).ok())
                .expect("couldn't append canvas to document body");
        }
        Self(Some(window))
    }
    pub(crate) fn area(&self) -> Area<DeviceContext> {
        let width = self.0.as_ref().unwrap().inner_size().width as CoordinateUnit;
        let height = self.0.as_ref().unwrap().inner_size().height as CoordinateUnit;
        Area::new(width, height)
    }
}
#[derive(Serialize, Deserialize)]
pub(crate) enum RenderLifecycleMarker {
    Suspend,
    Resume,
}
#[derive(Serialize, Deserialize, Default)]
pub(crate) struct RenderRequest {
    pub(crate) render_packets: Option<()>,
    pub(crate) lifecycle_marker: Option<RenderLifecycleMarker>,
    pub(crate) viewport_handle: Option<ViewportHandle>,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct RenderResponse {}
