#![allow(clippy::type_complexity)]

pub use bevy_ecs;
pub use wgpu;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopWindowTarget};

use ash::identification::RenderIdentification;
use ash::leaflet::RenderLeaflet;
use elm::{Leaf, Leaflet};
use window::{WindowDescriptor, WindowHandle};

use crate::ash::render::Render;
use crate::ash::Ash;
use crate::button::Button;
use crate::circle::Circle;
use crate::coordinate::{CoordinateLeaf, CoordinateUnit};
use crate::elm::Elm;
use crate::ginkgo::Ginkgo;
use crate::icon::Icon;
use crate::panel::Panel;
use crate::rectangle::Rectangle;
use crate::text::Text;

use self::ash::leaflet::RenderLeafletStorage;

pub mod ash;
pub mod button;
pub mod circle;
pub mod color;
pub mod coordinate;
pub mod differential;
pub mod elm;
pub mod ginkgo;
pub mod icon;
pub mod instance;
pub mod job;
pub mod panel;
pub mod rectangle;
pub mod scene;
pub mod text;
pub mod texture;
pub mod window;
mod r_scene;

#[cfg(not(target_os = "android"))]
pub type AndroidInterface = ();

#[cfg(target_os = "android")]
#[derive(Default)]
pub struct AndroidInterface(pub(crate) Option<AndroidApp>);

#[cfg(target_os = "android")]
pub type AndroidApp = winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
impl AndroidInterface {
    pub fn new(app: AndroidApp) -> Self {
        Self(Some(app))
    }
}

pub struct Foliage {
    window_descriptor: Option<WindowDescriptor>,
    leaf_queue: Option<Vec<Leaflet>>,
    render_queue: Option<RenderLeafletStorage>,
    android_interface: AndroidInterface,
}

impl Default for Foliage {
    fn default() -> Self {
        Foliage::new()
    }
}

impl Foliage {
    pub fn new() -> Self {
        let mut this = Self {
            window_descriptor: None,
            leaf_queue: Some(vec![]),
            render_queue: Some(RenderLeafletStorage::new()),
            android_interface: AndroidInterface::default(),
        };
        this.with_renderleaf::<Panel>()
            .with_renderleaf::<Circle>()
            .with_renderleaf::<Rectangle>()
            .with_renderleaf::<Icon>()
            .with_renderleaf::<Text>()
            .with_leaf::<CoordinateLeaf>()
            .with_leaf::<Button>()
    }
    pub fn with_android_interface(mut self, android_interface: AndroidInterface) -> Self {
        self.android_interface = android_interface;
        self
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
    fn with_renderer<T: Render + 'static>(mut self) -> Self {
        self.render_queue
            .as_mut()
            .unwrap()
            .insert(T::render_id(), RenderLeaflet::leaf_fn::<T>());
        self
    }
    pub fn with_renderleaf<T: Leaf + Render + 'static>(self) -> Self {
        self.with_leaf::<T>().with_renderer::<T>()
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
        let mut event_loop_builder = EventLoopBuilder::<()>::with_user_event();
        cfg_if::cfg_if! {
            if #[cfg(target_os = "android")] {
                use winit::platform::android::EventLoopBuilderExtAndroid;
                let event_loop = event_loop_builder
                    .with_android_app(self.android_interface.0.take().unwrap())
                    .build().expect("event-loop");
            } else {
                let event_loop = event_loop_builder
            .build()
            .expect("event-loop");
            }
        }

        let window_desc = self.window_descriptor.unwrap_or_default();
        let mut ginkgo = Ginkgo::new();
        cfg_if::cfg_if! {
            if #[cfg(target_family = "wasm")] {
                let mut window_handle = WindowHandle::some(&event_loop, &window_desc);
                ginkgo.initialize(window_handle.clone()).await;
            } else {
                let mut window_handle = WindowHandle::none();
            }
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
            move |event, event_loop_window_target: &EventLoopWindowTarget<()>| {
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
                            let new_handle = ginkgo.resize(
                                (size.width, size.height).into(),
                                window_handle.scale_factor(),
                            );
                            elm.set_viewport_handle_area(new_handle);
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
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            elm.set_scale_factor(scale_factor as CoordinateUnit);
                            ginkgo.set_scale_factor(scale_factor as CoordinateUnit);
                        }
                        WindowEvent::ThemeChanged(_) => {}
                        WindowEvent::Occluded(_) => {}
                        WindowEvent::RedrawRequested => {
                            if elm.job.resumed() {
                                ginkgo.adjust_viewport_pos(elm.viewport_handle_changes());
                                ash.extract(elm.render_packet_package());
                                ash.prepare(&ginkgo);
                                ash.record(&ginkgo);
                                ash.render(&mut ginkgo);
                                window_handle.value().request_redraw();
                            }
                        }
                    },
                    Event::DeviceEvent { .. } => {}
                    Event::UserEvent(_e) => {}
                    Event::Suspended => {
                        ginkgo.suspend();
                        elm.job.suspend();
                    }
                    Event::Resumed => {
                        if let Some(viewport_area) = ginkgo.resume(
                            event_loop_window_target,
                            &mut window_handle,
                            &window_desc,
                        ) {
                            elm.attach_viewport_handle(viewport_area);
                        }
                        if !elm.initialized() {
                            elm.set_scale_factor(window_handle.scale_factor());
                            elm.attach_leafs(self.leaf_queue.take().unwrap());
                            ash.establish(&ginkgo, self.render_queue.take().unwrap());
                            elm.finish_initialization();
                        }
                        elm.job.resume();
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