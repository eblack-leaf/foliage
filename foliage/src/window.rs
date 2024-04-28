use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};
#[derive(Default)]
pub struct WindowHandle {
    handle: Option<Window>,
}
impl ApplicationHandler for WindowHandle {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        todo!()
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.handle.replace(
            event_loop
                .create_window(WindowAttributes::default())
                .unwrap(),
        );
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {
        todo!()
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        todo!()
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        todo!()
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        todo!()
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        todo!()
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        todo!()
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        todo!()
    }
}