mod color;
mod coordinate;
mod gfx;
mod job;
pub mod window;

use crate::coordinate::CoordinateUnit;
use crate::job::Job;
use gfx::GfxContext;
use serde::{Deserialize, Serialize};
use window::{WindowDescriptor, WindowHandle};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopBuilder, EventLoopWindowTarget};

pub struct Foliage {
    window_descriptor: Option<WindowDescriptor>,
    leaf_queue: (),
    job: Job,
}
impl Foliage {
    pub fn new() -> Self {
        Self {
            window_descriptor: None,
            leaf_queue: (),
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
                let rt = tokio::runtime::Runtime::new().expect("tokio");
                rt.block_on(self.internal_run());
            }
        }
    }
    async fn internal_run(mut self) {
        let event_loop = EventLoopBuilder::<()>::with_user_event()
            .build()
            .expect("event-loop");
        let mut window_handle = WindowHandle::none();
        let window_desc = self.window_descriptor.unwrap_or_default();
        let mut gfx_context = GfxContext::new();
        #[cfg(target_family = "wasm")]
        {
            window_handle =
                WindowHandle::some(&event_loop, self.window_descriptor.unwrap_or_default());
            gfx_context.initialize(window_handle).await;
        }
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
            |event, event_loop_window_target: &EventLoopWindowTarget<()>| {
                match event {
                    Event::NewEvents(_) => {}
                    Event::WindowEvent {
                        window_id: _window_id,
                        event: w_event,
                    } => match w_event {
                        WindowEvent::ActivationTokenDone { .. } => {}
                        WindowEvent::Resized(size) => {
                            let new_handle = gfx_context.resize(
                                (size.width, size.height).into(),
                                window_handle.scale_factor(),
                            );
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
                            window_handle.value().request_redraw();
                        }
                    },
                    Event::DeviceEvent { .. } => {}
                    Event::UserEvent(_e) => {}
                    Event::Suspended => {
                        gfx_context.suspend();
                    }
                    Event::Resumed => {
                        if let Some(vh) = gfx_context.resume(
                            event_loop_window_target,
                            &mut window_handle,
                            &window_desc,
                        ) {
                            // adjust viewport handle here
                        }
                        // logical init here for leafs one-shot
                    }
                    Event::AboutToWait => {
                        if self.job.resumed() {
                            self.job.exec_main();
                            window_handle.value().request_redraw();
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
