use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

pub use ash::Render;
pub use coordinate::{
    Area, CoordinateUnit, Coordinates, DeviceContext, Layer, LogicalContext, NumericalContext,
    Position, Section,
};
pub use elm::Elm;
use willow::Willow;

use crate::ash::Ash;
use crate::ginkgo::Ginkgo;

mod ash;
mod color;
mod coordinate;
mod elm;
mod ginkgo;
mod willow;

pub struct Foliage {
    willow: Willow,
    ash: Ash,
    ginkgo: Ginkgo,
    worker_path: String,
    android_connection: AndroidConnection,
}
impl Foliage {
    pub fn new() -> Self {
        Self {
            willow: Willow::default(),
            ash: Ash::default(),
            ginkgo: Ginkgo::default(),
            worker_path: "".to_string(),
            android_connection: AndroidConnection::default(),
        }
    }
    pub fn set_window_size<A: Into<Area<DeviceContext>>>(&mut self, a: A) {
        self.willow.requested_size.replace(a.into());
    }
    pub fn set_worker_path<S: AsRef<str>>(&mut self, s: S) {
        self.worker_path = s.as_ref().to_string();
    }
    pub fn set_window_title<S: AsRef<str>>(&mut self, s: S) {
        self.willow.title.replace(s.as_ref().to_string());
    }
    pub fn set_android_connection(&mut self, ac: AndroidConnection) {
        self.android_connection = ac;
    }
    pub fn add_renderer<R: Render>(&mut self) {
        // queue to render engen a call to Render::create
    }
    pub fn run(mut self) {
        cfg_if::cfg_if! {
            if #[cfg(target_family = "wasm")] {
                wasm_bindgen_futures::spawn_local(self.internal_run());
            } else {
                pollster::block_on(self.internal_run());
            }
        }
    }
    async fn internal_run(mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Wait);
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                use winit::platform::web::EventLoopExtWebSys;
                let event_loop_function = EventLoop::spawn_app;
            } else {
                let event_loop_function = EventLoop::run_app;
            }
        }
        #[cfg(target_family = "wasm")]
        if !self.ginkgo.acquired() {
            self.willow.connect(&event_loop);
            self.ginkgo.acquire_context(&self.willow).await;
        }
        let proxy = event_loop.create_proxy();
        // bridge
        // insert bridge into ecs
        (event_loop_function)(event_loop, &mut self).expect("event-loop-run-app");
    }
}

impl ApplicationHandler for Foliage {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(not(target_family = "wasm"))]
        if !self.ginkgo.acquired() {
            self.willow.connect(event_loop);
            pollster::block_on(self.ginkgo.acquire_context(&self.willow));
            self.ginkgo.configure_view(&self.willow);
            self.ginkgo.create_viewport(&self.willow);
        } else {
            #[cfg(target_os = "android")]
            {
                self.ginkgo.recreate_surface(&self.willow);
                self.ginkgo.configure_view(&self.willow);
                self.ginkgo.resize_viewport(&self.willow);
            }
        }
        #[cfg(target_family = "wasm")]
        if !self.ginkgo.configured() {
            self.ginkgo.configure_view(&self.willow);
            self.ginkgo.create_viewport(&self.willow);
        }
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::ActivationTokenDone { .. } => {}
            WindowEvent::Resized(_) => {
                // elm.resize_viewport_handle(&self.willow);
                self.ginkgo.configure_view(&self.willow);
                self.ginkgo.size_viewport(&self.willow);
            }
            WindowEvent::Moved(_) => {}
            WindowEvent::CloseRequested => {
                event_loop.exit();
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
            WindowEvent::PinchGesture { .. } => {}
            WindowEvent::PanGesture { .. } => {}
            WindowEvent::DoubleTapGesture { .. } => {}
            WindowEvent::RotationGesture { .. } => {}
            WindowEvent::TouchpadPressure { .. } => {}
            WindowEvent::AxisMotion { .. } => {}
            WindowEvent::Touch(_) => {}
            WindowEvent::ScaleFactorChanged {
                scale_factor: _scale_factor,
                ..
            } => {
                // elm.resize_viewport_handle(&self.willow);
                self.ginkgo.configure_view(&self.willow);
                self.ginkgo.size_viewport(&self.willow);
            }
            WindowEvent::ThemeChanged(_) => {}
            WindowEvent::Occluded(_) => {}
            WindowEvent::RedrawRequested => {
                // if !drawn => draw
                // which implies that WindowHandle needs to store that somehow
            }
        }
    }
}

#[cfg(not(target_os = "android"))]
#[derive(Default, Copy, Clone)]
pub struct AndroidConnection();

#[cfg(target_os = "android")]
pub struct AndroidConnection();
