mod coordinate;
mod interop;
mod window;

pub use coordinate::{Area, CoordinateUnit, Coordinates};
pub use interop::AndroidConnection;
use window::WindowHandle;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{WindowAttributes, WindowId};

pub struct Foliage {}
impl Foliage {
    pub fn new() -> Self {
        Self {}
    }
    pub fn set_window_size<A: Into<Area>>(&mut self, a: A) {}
    pub fn set_worker_path<S: AsRef<str>>(&mut self, s: S) {}
    pub fn set_window_title<S: AsRef<str>>(&mut self, s: S) {}
    pub fn set_android_connection(&mut self, ac: AndroidConnection) {}
    pub fn run(self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Wait);
        let mut engen = Engen::default();
        event_loop.run_app(&mut engen).expect("event-loop-run-app");
    }
}

#[derive(Default)]
pub struct Engen {
    window_handle: WindowHandle,
}

impl ApplicationHandler for Engen {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window_handle.handle.replace(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title(self.window_handle.title.clone().unwrap_or_default())
                        .with_resizable(self.window_handle.resizable.unwrap_or(true))
                        .with_min_inner_size(
                            self.window_handle
                                .min_size
                                .unwrap_or(Area::device((320, 320))),
                        ),
                )
                .unwrap(),
        );
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
