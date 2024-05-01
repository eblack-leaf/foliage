mod ash;
mod color;
mod coordinate;
mod ginkgo;
mod willow;

use crate::ash::Ash;
use crate::coordinate::DeviceContext;
use crate::ginkgo::Ginkgo;
use crate::willow::WindowHandle;
pub use ash::Render;
pub use coordinate::{Area, CoordinateUnit, Coordinates};
use std::sync::Arc;
use willow::Willow;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

pub struct Foliage {
    willow: Willow,
    ash: Ash,
    ginkgo: Ginkgo,
}
impl Foliage {
    pub fn new() -> Self {
        Self {
            willow: Willow::default(),
            ash: Ash::default(),
            ginkgo: Ginkgo::default(),
        }
    }
    pub fn set_window_size<A: Into<Area<DeviceContext>>>(&mut self, a: A) {}
    pub fn set_worker_path<S: AsRef<str>>(&mut self, s: S) {}
    pub fn set_window_title<S: AsRef<str>>(&mut self, s: S) {}
    pub fn set_android_connection(&mut self, ac: AndroidConnection) {}
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
        event_loop.run_app(&mut self).expect("event-loop-run-app");
    }
}

impl ApplicationHandler for Foliage {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let requested_area = self.willow.requested_area();
        let attributes = WindowAttributes::default()
            .with_title(self.willow.title.clone().unwrap_or_default())
            .with_resizable(self.willow.resizable.unwrap_or(true))
            .with_min_inner_size(self.willow.min_size.unwrap_or(Area::device((320, 320))));
        #[cfg(all(
            not(target_family = "wasm"),
            not(target_os = "android"),
            not(target_os = "ios")
        ))]
        let attributes = attributes.with_inner_size(requested_area);
        let window = event_loop.create_window(attributes).unwrap();
        self.willow.handle = WindowHandle(Some(Arc::new(window)));
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::ActivationTokenDone { .. } => {}
            WindowEvent::Resized(_) => {}
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
            WindowEvent::ScaleFactorChanged { .. } => {}
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
