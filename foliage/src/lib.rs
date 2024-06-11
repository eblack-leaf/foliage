pub use bevy_ecs;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Commands, Component, Entity};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Command, Query};
pub use wgpu;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use willow::Willow;

use crate::ash::{Ash, Render};
use crate::coordinate::area::Area;
use crate::coordinate::{Coordinates, DeviceContext};
use crate::elm::Elm;
use crate::ginkgo::Ginkgo;
use crate::icon::Icon;
use crate::image::Image;
use crate::panel::Panel;
use crate::signal::{Signal, Signaler, TargetComponents, TriggerTarget, TriggeredAttribute};
use crate::view::{
    CurrentViewStage, SignalConfirmation, SignalHandle, Stage, StagedSignal, View, ViewActive,
    ViewComponents, ViewHandle, ViewStage,
};
use signal::LayoutFilter;

pub mod ash;
pub mod asset;
pub mod color;
pub mod coordinate;
mod differential;
pub mod elm;
pub mod ginkgo;
pub mod grid;
pub mod icon;
pub mod image;
pub mod instances;
pub mod panel;
pub mod signal;
pub mod view;
pub mod willow;

pub struct Foliage {
    willow: Willow,
    ash: Ash,
    ginkgo: Ginkgo,
    elm: Elm,
    worker_path: String,
    android_connection: AndroidConnection,
    leaf_fns: Vec<Box<fn(&mut Elm)>>,
    leaves_fns: Vec<Box<fn(&mut Foliage)>>,
}

impl Foliage {
    pub fn new() -> Self {
        Self {
            willow: Willow::default(),
            ash: Ash::default(),
            ginkgo: Ginkgo::default(),
            elm: Elm::default(),
            worker_path: "".to_string(),
            android_connection: AndroidConnection::default(),
            leaf_fns: vec![],
            leaves_fns: vec![],
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
    pub fn attach_leaf<L: Leaf>(&mut self) {
        self.leaf_fns.push(Box::new(|e| {
            L::attach(e);
        }));
    }
    pub fn attach_leaves<L: Leaves>(&mut self) {
        self.leaves_fns.push(Box::new(|f| {
            L::attach(f);
        }));
    }
    pub fn add_renderer<R: Render>(&mut self) {
        self.ash.add_renderer::<R>();
    }
    pub fn create_view(&mut self) -> ViewConfig {
        let handle = self.elm.ecs.world.spawn(ViewComponents::new()).id();
        ViewConfig {
            root: handle,
            reference: &mut self.elm,
        }
    }
    pub fn view(&mut self, vh: ViewHandle) -> ViewReference {
        ViewReference {
            root: vh.0,
            reference: &mut self.elm,
        }
    }
    pub fn create_action<A: Command + Clone + Send + Sync + 'static>(
        &mut self,
        a: A,
    ) -> ActionHandle {
        self.elm.checked_add_action_fns::<A>();
        let handle = self
            .elm
            .ecs
            .world
            .spawn((TriggeredAction(a), Signal::default()))
            .id();
        // signal that matches to a triggered-action (command) that spawns it on self + gives to a cmd to execute
        // can be added to OnClick(...) to set logic triggers
        ActionHandle(handle)
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

    fn leaves_attach(&mut self) {
        for leaves_fn in self
            .leaves_fns
            .drain(..)
            .collect::<Vec<Box<fn(&mut Foliage)>>>()
        {
            (leaves_fn)(self);
        }
    }
}
#[derive(Copy, Clone)]
pub struct ActionHandle(pub(crate) Entity);
#[derive(Component)]
pub(crate) struct TriggeredAction<A: Command + Send + Sync + 'static + Clone>(pub(crate) A);
pub(crate) fn engage_action<A: Command + Send + Sync + 'static + Clone>(
    actions: Query<(&Signal, &TriggeredAction<A>), Changed<Signal>>,
    mut cmd: Commands,
) {
    for (signal, action) in actions.iter() {
        if signal.should_spawn() {
            cmd.add(action.0.clone());
        }
    }
}
pub struct ViewConfig<'a> {
    root: Entity,
    reference: &'a mut Elm,
}
impl<'a> ViewConfig<'a> {
    pub fn template(mut self) -> Self {
        self
    }
    pub fn padding(mut self) -> Self {
        self
    }
    pub fn handle(self) -> ViewHandle {
        ViewHandle(self.root)
    }
}
pub struct ViewReference<'a> {
    root: Entity,
    reference: &'a mut Elm,
}
pub struct TargetReference<'a> {
    root: Entity,
    this: Entity,
    reference: &'a mut Elm,
}
pub struct StageReference<'a> {
    root: Entity,
    reference: &'a mut Elm,
    stage: Stage,
}
pub struct SignalReference<'a> {
    root: Entity,
    this: Entity,
    reference: &'a mut Elm,
    stage: Stage,
}
impl<'a> StageReference<'a> {
    pub fn add_signal(mut self, target: TriggerTarget) -> SignalReference<'a> {
        let signal = self.reference.ecs.world.spawn(Signaler::new(target)).id();
        self.reference
            .ecs
            .world
            .get_mut::<View>(self.root)
            .expect("no-view")
            .stages
            .get_mut(self.stage.0 as usize)
            .expect("invalid-stage")
            .signals
            .insert(
                signal,
                StagedSignal {
                    handle: SignalHandle(signal),
                    state_on_stage_start: Signal::default(),
                },
            );
        SignalReference {
            root: self.root,
            this: signal,
            reference: self.reference,
            stage: self.stage,
        }
    }
    pub fn on_end(mut self, action_handle: ActionHandle) -> Self {
        // action to hook to when the stage is confirmed done
        self.reference
            .ecs
            .world
            .get_mut::<View>(self.root)
            .expect("no-view")
            .stages
            .get_mut(self.stage.0 as usize)
            .expect("no-stage")
            .on_end
            .replace(action_handle);
        self
    }
}
impl<'a> SignalReference<'a> {
    pub fn with_attribute<A: Bundle + 'static + Clone + Send + Sync>(
        mut self,
        a: A,
        filter: Option<LayoutFilter>,
    ) -> Self {
        self.reference.checked_add_signal_fns::<A>();
        self.reference
            .ecs
            .world
            .entity_mut(self.this)
            .insert(TriggeredAttribute(a, filter));
        self
    }
    pub fn clean(mut self) {
        // set Signal::clean() when stage fires instead of Signal::spawn()
        self.reference
            .ecs
            .world
            .get_mut::<View>(self.root)
            .expect("no-view")
            .stages
            .get_mut(self.stage.0 as usize)
            .expect("no-stage")
            .signals
            .get_mut(&self.this)
            .expect("no-signal")
            .state_on_stage_start = Signal::clean();
    }
    pub fn with_transition(mut self) -> Self {
        // TODO self.reference.checked_add_transition_fns::<T>();
        self
    }
    pub fn filter_signal(mut self, layout_filter: LayoutFilter) -> Self {
        self.reference
            .ecs
            .world
            .entity_mut(self.this)
            .insert(layout_filter);
        self
    }
}
impl<'a> TargetReference<'a> {
    pub fn handle(self) -> TriggerTarget {
        TriggerTarget(self.this)
    }
}
impl<'a> ViewReference<'a> {
    pub fn add_target(mut self) -> TargetReference<'a> {
        let target = self
            .reference
            .ecs
            .world
            .spawn(TargetComponents::default())
            .id();
        self.reference
            .ecs
            .world
            .get_mut::<View>(self.root)
            .expect("no-view")
            .targets
            .insert(TriggerTarget(target));
        TargetReference {
            root: self.root,
            this: target,
            reference: self.reference,
        }
    }
    pub fn set_initial_stage(mut self, stage: Stage) {
        self.reference
            .ecs
            .world
            .get_mut::<CurrentViewStage>(self.root)
            .expect("no-current")
            .stage = stage;
    }
    pub fn activate(mut self) {
        self.reference
            .ecs
            .world
            .get_mut::<ViewActive>(self.root)
            .expect("no-active")
            .0 = true;
    }
    pub fn create_stage(&mut self) -> Stage {
        let stage = self
            .reference
            .ecs
            .world
            .entity(self.root)
            .get::<View>()
            .expect("no-view")
            .stages
            .len();
        self.reference
            .ecs
            .world
            .entity_mut(self.root)
            .get_mut::<View>()
            .expect("no-view")
            .stages
            .push(ViewStage::default());
        Stage(stage as i32)
    }
    pub fn stage(&mut self, stage: Stage) -> StageReference {
        StageReference {
            root: self.root,
            stage,
            reference: self.reference,
        }
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
            self.elm.configure(
                self.willow.actual_area().to_numerical(),
                self.ginkgo.configuration().scale_factor,
            );
            self.leaves_attach();
            self.elm.initialize(self.leaf_fns.drain(..).collect());
            self.ash.initialize(&self.ginkgo);
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
            self.elm.configure(
                self.willow.actual_area().to_numerical(),
                self.ginkgo.configuration().scale_factor,
            );
            self.leaves_attach();
            self.elm.initialize(self.leaf_fns.drain(..).collect());
            self.ash.initialize(&self.ginkgo);
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
                self.elm.adjust_viewport_handle(&self.willow);
                self.ginkgo.configure_view(&self.willow);
                self.ginkgo.size_viewport(&self.willow);
                self.willow.window().request_redraw();
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
                self.elm.adjust_viewport_handle(&self.willow);
                self.ginkgo.configure_view(&self.willow);
                self.ginkgo.size_viewport(&self.willow);
            }
            WindowEvent::ThemeChanged(_) => {}
            WindowEvent::Occluded(_) => {}
            WindowEvent::RedrawRequested => {
                if !self.ash.drawn {
                    if let Some(vc) = self.elm.viewport_handle_changes() {
                        self.ginkgo.position_viewport(vc);
                    }
                    self.ash.render(&self.ginkgo, &mut self.elm);
                    self.ash.drawn = true;
                }
            }
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.ash.drawn && self.elm.initialized() {
            self.elm.process();
            self.willow.window().request_redraw();
            self.ash.drawn = false;
        }
    }
}

#[cfg(not(target_os = "android"))]
#[derive(Default, Copy, Clone)]
pub struct AndroidConnection();

#[cfg(target_os = "android")]
pub struct AndroidConnection(pub AndroidApp);

#[cfg(target_os = "android")]
pub type AndroidApp = winit::platform::android::activity::AndroidApp;

pub trait Leaf {
    fn attach(elm: &mut Elm);
}
pub trait Leaves {
    fn attach(foliage: &mut Foliage);
}
pub struct CoreLeaves;
impl Leaves for CoreLeaves {
    fn attach(foliage: &mut Foliage) {
        foliage.attach_leaf::<Panel>();
        foliage.add_renderer::<Panel>();
        foliage.attach_leaf::<Coordinates>();
        foliage.attach_leaf::<Icon>();
        foliage.add_renderer::<Icon>();
        foliage.attach_leaf::<Image>();
        foliage.add_renderer::<Image>();
    }
}
