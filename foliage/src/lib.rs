mod coordinate;
mod interop;
mod window;

use crate::window::WindowHandle;
pub use coordinate::{Area, CoordinateUnit, Coordinates};
pub use interop::AndroidConnection;
use winit::event_loop::{ControlFlow, EventLoop};

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
        let mut window_handle = WindowHandle::default();
        event_loop
            .run_app(&mut window_handle)
            .expect("event-loop-run-app");
    }
}