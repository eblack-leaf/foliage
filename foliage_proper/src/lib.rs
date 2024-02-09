#![allow(clippy::type_complexity)]

use ash::identification::RenderIdentification;
use ash::leaflet::RenderLeaflet;
#[doc(hidden)]
pub use bevy_ecs;
use bevy_ecs::prelude::Resource;
use elm::leaf::{Leaf, Leaflet};
#[doc(hidden)]
pub use wgpu;
use window::{WindowDescriptor, WindowHandle};
use winit::event::{Event, Ime, KeyEvent, MouseButton, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
#[doc(hidden)]
pub use winit::keyboard::NamedKey;

use self::ash::leaflet::RenderLeafletStorage;
use crate::ash::render::Render;
use crate::ash::Ash;
use crate::asset::AssetContainer;
use crate::circle::Circle;
use crate::clipboard::Clipboard;
use crate::compositor::Compositor;
use crate::coordinate::position::Position;
use crate::coordinate::CoordinateUnit;
use crate::elm::leaf::Leaves;
use crate::elm::Elm;
use crate::ginkgo::viewport::ViewportHandle;
use crate::ginkgo::Ginkgo;
use crate::icon::Icon;
use crate::image::Image;
use crate::interaction::{
    Interaction, InteractionEvent, InteractionId, InteractionPhase, KeyboardAdapter, KeyboardEvent,
    MouseAdapter,
};
use crate::media::Media;
use crate::panel::Panel;
use crate::rectangle::Rectangle;
use crate::system_message::{ResponseMessage, SystemMessageResponse};
use crate::text::Text;
use crate::time::Time;
use crate::virtual_keyboard::VirtualKeyboardAdapter;
use crate::workflow::{Workflow, WorkflowConnectionBase};
use animate::trigger::Trigger;

mod animate;
pub mod ash;
pub mod asset;
pub mod circle;
pub mod clipboard;
pub mod color;
pub mod compositor;
pub mod coordinate;
pub mod differential;
pub mod elm;
mod generator;
pub mod ginkgo;
pub mod icon;
pub mod image;
pub mod instance;
pub mod interaction;
pub mod job;
pub mod media;
pub mod panel;
pub mod rectangle;
pub mod scene;
pub(crate) mod system_message;
pub mod text;
pub mod texture;
pub mod time;
pub mod virtual_keyboard;
pub mod window;
pub mod workflow;

#[cfg(not(target_os = "android"))]
#[derive(Default, Clone, Resource)]
pub struct AndroidInterface();

#[cfg(target_os = "android")]
#[derive(Default, Resource, Clone)]
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
    worker_path: String,
}

impl Default for Foliage {
    fn default() -> Self {
        Foliage::new()
    }
}
impl Foliage {
    pub fn new() -> Self {
        let this = Self {
            window_descriptor: None,
            leaf_queue: Some(vec![]),
            render_queue: Some(RenderLeafletStorage::new()),
            android_interface: AndroidInterface::default(),
            worker_path: String::default(),
        };
        this.with_renderleaf::<Panel>()
            .with_renderleaf::<Circle>()
            .with_renderleaf::<Rectangle>()
            .with_renderleaf::<Icon>()
            .with_renderleaf::<Text>()
            .with_renderleaf::<Image>()
            .with_leaf::<Trigger>()
            .with_leaf::<Interaction>()
            .with_leaf::<ViewportHandle>()
            .with_leaf::<Compositor>()
            .with_leaf::<Time>()
            .with_leaf::<VirtualKeyboardAdapter>()
            .with_leaf::<Clipboard>()
            .with_leaf::<AssetContainer>()
            .with_leaf::<Media>()
    }
    pub fn with_android_interface(mut self, android_interface: AndroidInterface) -> Self {
        self.android_interface = android_interface;
        self
    }
    pub fn with_window_descriptor(mut self, desc: WindowDescriptor) -> Self {
        self.window_descriptor.replace(desc);
        self
    }
    pub fn with_leaves<L: Leaves>(self) -> Self {
        L::leaves(self)
    }
    pub fn with_leaf<T: Leaf>(mut self) -> Self {
        self.leaf_queue
            .as_mut()
            .unwrap()
            .push(Leaflet::leaf_fn::<T>());
        self
    }
    pub fn with_worker_path<S: Into<String>>(mut self, path: S) -> Self {
        self.worker_path = path.into();
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
    pub fn run<W: Workflow + Default + Send + Sync + 'static>(self) {
        cfg_if::cfg_if! {
            if #[cfg(target_family = "wasm")] {
                wasm_bindgen_futures::spawn_local(self.internal_run::<W>());
            } else {
                let rt = tokio::runtime::Runtime::new();
                rt.unwrap().block_on(
                    self.internal_run::<W>()
                );
            }
        }
    }
    async fn internal_run<W: Workflow + Default + Send + Sync + 'static>(mut self) {
        let mut event_loop_builder = EventLoop::<ResponseMessage<W::Response>>::with_user_event();
        cfg_if::cfg_if! {
            if #[cfg(target_os = "android")] {
                use winit::platform::android::EventLoopBuilderExtAndroid;
                let event_loop = event_loop_builder
                    .with_android_app(self.android_interface.0.clone().unwrap())
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
        let proxy = event_loop.create_proxy();
        let bridge = WorkflowConnectionBase::<W>::new(proxy, self.worker_path);
        elm.container().insert_non_send_resource(bridge);
        elm.container().insert_resource(self.android_interface);
        let mut ash = Ash::new();
        let mut drawn = true;
        let _ = (event_loop_function)(
            event_loop,
            move |event: Event<ResponseMessage<W::Response>>,
                  event_loop_window_target: &EventLoopWindowTarget| {
                if elm.job.can_idle() {
                    tracing::trace!("job-waiting");
                    event_loop_window_target.set_control_flow(ControlFlow::Wait);
                } else {
                    tracing::trace!("job-polling");
                    #[cfg(not(target_family = "wasm"))]
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
                        WindowEvent::KeyboardInput {
                            device_id: _,
                            event,
                            is_synthetic: _,
                        } => match event {
                            KeyEvent {
                                physical_key: _,
                                logical_key,
                                text: _,
                                location: _,
                                state,
                                repeat: _,
                                ..
                            } => {
                                if let Some(e) = elm
                                    .container()
                                    .get_resource_mut::<KeyboardAdapter>()
                                    .unwrap()
                                    .cache_checked(logical_key, state)
                                {
                                    tracing::trace!("sending-keyboard-event: {:?}", e);
                                    elm.send_event::<KeyboardEvent>(e);
                                }
                            }
                        },
                        WindowEvent::ModifiersChanged(modifiers) => {
                            elm.container()
                                .get_resource_mut::<KeyboardAdapter>()
                                .unwrap()
                                .update_modifiers(modifiers);
                        }
                        WindowEvent::Ime(ime) => match ime {
                            Ime::Enabled => {}
                            Ime::Preedit(_, _) => {}
                            Ime::Commit(_string) => {
                                // elm.send_event(KeyboardEvent::new(
                                //     Key::Character(SmolStr::new(content)),
                                //     State::Pressed,
                                //     Mods::default(),
                                // ));
                            }
                            Ime::Disabled => {} // Ime::DeleteSurroundingText { .. } => {}
                        },
                        WindowEvent::CursorMoved {
                            device_id: _,
                            position,
                        } => {
                            let location = Position::from((position.x, position.y));
                            elm.job
                                .container
                                .get_resource_mut::<MouseAdapter>()
                                .unwrap()
                                .update_location(location);
                            if let Some(cached) = elm
                                .job
                                .container
                                .get_resource_mut::<MouseAdapter>()
                                .unwrap()
                                .0
                                .get(&MouseButton::Left)
                            {
                                if cached.is_pressed() {
                                    elm.job.container.send_event(InteractionEvent::new(
                                        InteractionPhase::Moved,
                                        InteractionId(0),
                                        location,
                                    ));
                                }
                            }
                        }
                        WindowEvent::CursorEntered { .. } => {}
                        WindowEvent::CursorLeft { .. } => {}
                        WindowEvent::MouseWheel { .. } => {
                            // scroll wheel event
                        }
                        WindowEvent::MouseInput {
                            device_id: _,
                            state,
                            button,
                        } => {
                            let last_position = elm
                                .job
                                .container
                                .get_resource_mut::<MouseAdapter>()
                                .unwrap()
                                .1;
                            if elm
                                .job
                                .container
                                .get_resource_mut::<MouseAdapter>()
                                .unwrap()
                                .button_pressed(button, state)
                            {
                                if button == MouseButton::Left {
                                    elm.job.container.send_event(InteractionEvent::new(
                                        InteractionPhase::Begin,
                                        button,
                                        last_position,
                                    ));
                                }
                            } else {
                                elm.job.container.send_event(InteractionEvent::new(
                                    InteractionPhase::End,
                                    button,
                                    last_position,
                                ));
                            }
                        }

                        WindowEvent::TouchpadPressure { .. } => {}
                        WindowEvent::AxisMotion { .. } => {}
                        WindowEvent::Touch(t) => {
                            elm.job.container.send_event(InteractionEvent::new(
                                t.phase,
                                t.id,
                                (t.location.x, t.location.y),
                            ));
                        }
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            elm.set_scale_factor(scale_factor as CoordinateUnit);
                            ginkgo.set_scale_factor(scale_factor as CoordinateUnit);
                        }
                        WindowEvent::ThemeChanged(_) => {}
                        WindowEvent::Occluded(_) => {}
                        WindowEvent::RedrawRequested => {
                            if elm.job.resumed() && !drawn {
                                ginkgo.adjust_viewport_pos(elm.viewport_handle_changes());
                                ash.extract(elm.render_packet_package());
                                ash.prepare(&ginkgo);
                                ash.record(&ginkgo);
                                ash.render(&mut ginkgo);
                                window_handle.value().request_redraw();
                                tracing::trace!("ginkgo:ash:redraw-finished");
                                drawn = true;
                            }
                        }
                        WindowEvent::PinchGesture { .. } => {}
                        WindowEvent::DoubleTapGesture { .. } => {}
                        WindowEvent::RotationGesture { .. } => {}
                    },
                    Event::DeviceEvent { .. } => {}
                    Event::UserEvent(ue) => {
                        if let Some(user_message) = ue.0 {
                            W::react(&mut elm, user_message);
                        } else if let Some(system_message) = ue.1 {
                            match system_message {
                                SystemMessageResponse::WasmAsset(id, data) => {
                                    // give to elm assets
                                    elm.store_local_asset(id, data);
                                }
                                SystemMessageResponse::NoOp => {}
                            }
                        }
                    }
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
                            tracing::trace!("elm:attaching-viewport-area");
                        }
                        if !elm.initialized() {
                            elm.set_scale_factor(window_handle.scale_factor());
                            elm.attach_leafs(self.leaf_queue.take().unwrap());
                            ash.establish(&ginkgo, self.render_queue.take().unwrap());
                            elm.finish_initialization();
                            tracing::trace!("elm:finish-initialization");
                        }
                        elm.job.resume();
                    }
                    Event::AboutToWait => {
                        if elm.job.resumed() && drawn {
                            elm.job.exec_main();
                            window_handle.value().request_redraw();
                            tracing::trace!("elm:exec-main");
                            drawn = false;
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
