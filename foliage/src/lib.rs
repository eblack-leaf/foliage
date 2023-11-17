#![allow(clippy::type_complexity)]
// pub mod ash;
pub mod ash;
pub mod color;
pub mod coordinate;
pub mod differential;
pub mod elm;
pub mod ginkgo;
pub mod job;
pub mod window;

use crate::coordinate::CoordinateUnit;
use crate::elm::Elm;
use crate::ginkgo::Ginkgo;

use crate::ash::render::Render;
use crate::ash::{Ash, RenderLeafletStorage};
use ash::identification::RenderIdentification;
use ash::leaflet::RenderLeaflet;
use window::{WindowDescriptor, WindowHandle};
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopWindowTarget};

pub trait Leaf {
    fn attach(elm: &mut Elm);
}
pub(crate) struct Leaflet(pub(crate) Box<fn(&mut Elm)>);
impl Leaflet {
    pub(crate) fn leaf_fn<T: Leaf>() -> Self {
        Self(Box::new(T::attach))
    }
}
pub struct Foliage {
    window_descriptor: Option<WindowDescriptor>,
    leaf_queue: Option<Vec<Leaflet>>,
    render_queue: Option<RenderLeafletStorage>,
}
impl Default for Foliage {
    fn default() -> Self {
        Foliage::new()
    }
}
impl Foliage {
    pub fn new() -> Self {
        Self {
            window_descriptor: None,
            leaf_queue: Some(vec![]),
            render_queue: Some(RenderLeafletStorage::new()),
        }
    }
    pub fn with_window_descriptor(mut self, desc: WindowDescriptor) -> Self {
        self.window_descriptor.replace(desc);
        self
    }
    pub fn with_leaf<T: Leaf>(mut self) -> Self {
        self.leaf_queue
            .as_mut()
            .unwrap()
            .push(Leaflet::leaf_fn::<T>());
        self
    }
    pub fn with_renderer<T: Render + 'static>(mut self) -> Self {
        self.render_queue
            .as_mut()
            .unwrap()
            .insert(T::id(), RenderLeaflet::leaf_fn::<T>());
        self
    }
    pub fn run(self) {
        cfg_if::cfg_if! {
            if #[cfg(target_family = "wasm")] {
                wasm_bindgen_futures::spawn_local(self.internal_run());
            } else {
                pollster::block_on(self.internal_run());
            }
        }
    }
    async fn internal_run(mut self) {
        let event_loop = EventLoopBuilder::<()>::with_user_event()
            .build()
            .expect("event-loop");
        let mut window_handle = WindowHandle::none();
        let window_desc = self.window_descriptor.unwrap_or_default();
        let mut ginkgo = Ginkgo::new();
        #[cfg(target_family = "wasm")]
        {
            window_handle =
                WindowHandle::some(&event_loop, self.window_descriptor.unwrap_or_default());
            ginkgo.initialize(window_handle).await;
        }
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                use winit::platform::web::EventLoopExtWebSys;
                let event_loop_function = EventLoop::spawn;
            } else {
                let event_loop_function = EventLoop::run;
            }
        }
        let mut elm = Elm::new();
        let mut ash = Ash::new();
        let _ = (event_loop_function)(
            event_loop,
            |event, event_loop_window_target: &EventLoopWindowTarget<()>| {
                if elm.job.can_idle() {
                    event_loop_window_target.set_control_flow(ControlFlow::Wait);
                } else {
                    event_loop_window_target.set_control_flow(ControlFlow::Poll);
                }
                match event {
                    Event::NewEvents(cause) => match cause {
                        StartCause::ResumeTimeReached { .. } => {}
                        StartCause::WaitCancelled { .. } => {}
                        StartCause::Poll => {}
                        StartCause::Init => {}
                    },
                    Event::WindowEvent {
                        window_id: _window_id,
                        event: w_event,
                    } => match w_event {
                        WindowEvent::ActivationTokenDone { .. } => {}
                        WindowEvent::Resized(size) => {
                            let _new_handle = ginkgo.resize(
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
                            let render_packet_package = elm.render_packet_package();
                            ash.extract(render_packet_package);
                            ash.prepare(&ginkgo);
                            ash.record(&ginkgo);
                            ash.render(&mut ginkgo);
                            window_handle.value().request_redraw();
                        }
                    },
                    Event::DeviceEvent { .. } => {}
                    Event::UserEvent(_e) => {}
                    Event::Suspended => {
                        ginkgo.suspend();
                    }
                    Event::Resumed => {
                        if let Some(_vh) = ginkgo.resume(
                            event_loop_window_target,
                            &mut window_handle,
                            &window_desc,
                        ) {
                            // adjust viewport handle here
                        }
                        if !elm.initialized() {
                            elm.attach_leafs(self.leaf_queue.take().unwrap());
                            ash.establish(&ginkgo, self.render_queue.take().unwrap());
                            elm.finish_initialization();
                        }
                    }
                    Event::AboutToWait => {
                        if elm.job.resumed() {
                            elm.job.exec_main();
                            window_handle.value().request_redraw();
                        }
                    }
                    Event::LoopExiting => {
                        elm.job.exec_teardown();
                    }
                    Event::MemoryWarning => {}
                }
            },
        );
    }
}
