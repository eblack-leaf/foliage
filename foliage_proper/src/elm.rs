use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{event_update_system, Event, EventRegistry, Events};
use bevy_ecs::prelude::{
    apply_deferred, Component, IntoSystemConfigs, IntoSystemSetConfigs, Schedule, SystemSet,
};
use bevy_ecs::schedule::ExecutorKind;
use bevy_ecs::system::Resource;
use bevy_ecs::world::World;

use crate::anim::{animate, Animate};
use crate::ash::{pull_clipping_section, ClippingSectionQueue, Render};
use crate::asset::{await_assets, on_retrieve};
use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::{DeviceContext, LogicalContext};
use crate::differential::{
    differential, visibility_changed, RenderAddQueue, RenderLink, RenderPacket, RenderRemoveQueue,
};
use crate::ginkgo::viewport::ViewportHandle;
use crate::ginkgo::ScaleFactor;
use crate::grid::responsive::anim::calc_diff;
use crate::grid::Grid;
use crate::interaction::{
    FocusedEntity, InteractiveEntity, KeyboardAdapter, MouseAdapter, TouchAdapter,
};
use crate::layout::{viewport_changes_layout, Layout, LayoutGrid};
use crate::web_ext::navigate;
use crate::willow::Willow;

#[derive(Default)]
pub struct Scheduler {
    pub startup: Schedule,
    pub main: Schedule,
}

impl Scheduler {
    pub(crate) fn exec_main(&mut self, ecs: &mut Ecs) {
        self.main
            .set_executor_kind(ExecutorKind::MultiThreaded)
            .run(ecs);
    }
    pub(crate) fn exec_startup(&mut self, ecs: &mut Ecs) {
        self.startup
            .set_executor_kind(ExecutorKind::MultiThreaded)
            .run(ecs);
    }
}

pub type Ecs = World;

#[derive(Default)]
pub struct Elm {
    pub ecs: Ecs,
    pub scheduler: Scheduler,
    initialized: bool,
    root_fns: Vec<fn(&mut Elm)>,
}

#[derive(Resource)]
pub(crate) struct DifferentialLimiter<D>(PhantomData<D>);

impl<D> Default for DifferentialLimiter<D> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
#[derive(Resource)]
pub(crate) struct SignalLimiter<D>(PhantomData<D>);

impl<D> Default for SignalLimiter<D> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
#[derive(Resource)]
pub(crate) struct BranchLimiter<D>(PhantomData<D>);

impl<D> Default for BranchLimiter<D> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
#[derive(Resource)]
pub(crate) struct EventLimiter<D>(PhantomData<D>);

impl<D> Default for EventLimiter<D> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
#[derive(Resource)]
pub(crate) struct RetrieveLimiter<D>(PhantomData<D>);

impl<D> Default for RetrieveLimiter<D> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
#[derive(Resource)]
pub(crate) struct ClipboardRetrieveLimiter<D>(PhantomData<D>);

impl<D> Default for ClipboardRetrieveLimiter<D> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
#[derive(Resource)]
pub(crate) struct DeriveLimiter<D, B>(PhantomData<D>, PhantomData<B>);

impl<D, B> Default for DeriveLimiter<D, B> {
    fn default() -> Self {
        Self(PhantomData, PhantomData)
    }
}
#[derive(Resource)]
pub(crate) struct FilterAttrLimiter<D>(PhantomData<D>);

impl<D> Default for FilterAttrLimiter<D> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
#[derive(Resource)]
pub(crate) struct AnimationLimiter<D>(PhantomData<D>);

impl<D> Default for AnimationLimiter<D> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl Elm {
    pub fn enable_event<E: Event + Clone + Send + Sync + 'static>(&mut self) {
        if !self.ecs.contains_resource::<EventLimiter<E>>() {
            self.ecs.insert_resource(Events::<E>::default());
            EventRegistry::register_event::<E>(&mut self.ecs);
            self.ecs.insert_resource(EventLimiter::<E>::default());
        }
    }
    pub fn enable_animation<A: Animate + Component>(&mut self) {
        if !self.ecs.contains_resource::<AnimationLimiter<A>>() {
            self.scheduler
                .main
                .add_systems(animate::<A>.in_set(InternalStage::Animation));
            self.ecs.insert_resource(AnimationLimiter::<A>::default());
        }
    }
    pub fn enable_differential<R: Render, D: Component + PartialEq + Clone>(&mut self) {
        if !self.ecs.contains_resource::<DifferentialLimiter<D>>() {
            self.scheduler.main.add_systems((
                differential::<D>.in_set(InternalStage::Differential),
                visibility_changed::<D>.in_set(InternalStage::Differential),
            ));
            self.ecs
                .insert_resource(DifferentialLimiter::<D>::default())
        }
        if !self.ecs.contains_resource::<RenderAddQueue<D>>() {
            self.ecs.insert_resource(RenderAddQueue::<D>::default());
        }
        let link = RenderLink::new::<R>();
        self.ecs
            .get_resource_mut::<RenderAddQueue<D>>()
            .unwrap()
            .queue
            .insert(link, HashMap::new());
        self.ecs
            .get_resource_mut::<RenderAddQueue<D>>()
            .unwrap()
            .cache
            .insert(link, HashMap::new());
        self.ecs
            .get_resource_mut::<RenderRemoveQueue>()
            .unwrap()
            .queue
            .insert(link, HashSet::new());
    }
    pub(crate) fn initialized(&self) -> bool {
        self.initialized
    }
    pub(crate) fn initialize(&mut self, leaf_fns: Vec<fn(&mut Elm)>) {
        for leaf_fn in leaf_fns {
            (leaf_fn)(self);
        }
        self.scheduler.exec_startup(&mut self.ecs);
        self.initialized = true;
    }
    pub(crate) fn configure(
        &mut self,
        window_area: Area<DeviceContext>,
        scale_factor: ScaleFactor,
    ) {
        self.ecs.insert_resource(ViewportHandle::new(
            window_area.to_logical(scale_factor.value()),
        ));
        self.ecs.insert_resource(scale_factor);
        self.ecs.insert_resource(RenderRemoveQueue::default());
        self.ecs.insert_resource(LayoutGrid::new(Grid::new(4, 4)));
        self.ecs.insert_resource(Layout::SQUARE);
        self.ecs.insert_resource(TouchAdapter::default());
        self.ecs.insert_resource(MouseAdapter::default());
        self.ecs.insert_resource(KeyboardAdapter::default());
        self.ecs.insert_resource(InteractiveEntity::default());
        self.ecs.insert_resource(FocusedEntity::default());
        self.ecs.insert_resource(ClippingSectionQueue::default());
        self.scheduler.main.configure_sets(
            (
                InternalStage::External,
                InternalStage::Animation,
                InternalStage::Apply,
                ExternalStage::Action,
                InternalStage::Clean,
                InternalStage::DeclarativePass,
                ExternalStage::Configure,
                InternalStage::SecondClean,
                InternalStage::ReactivePass,
                InternalStage::Resolve,
                InternalStage::FinalizeCoordinate,
                InternalStage::Differential,
                InternalStage::Finish,
            )
                .chain(),
        );
        self.scheduler.main.add_systems((
            event_update_system.in_set(InternalStage::External),
            (viewport_changes_layout, await_assets, navigate).in_set(InternalStage::External),
            calc_diff.in_set(InternalStage::Apply),
            on_retrieve.in_set(InternalStage::Clean),
            pull_clipping_section.in_set(InternalStage::FinalizeCoordinate),
        ));
        self.scheduler.main.add_systems((
            apply_deferred
                .after(InternalStage::External)
                .before(InternalStage::Animation),
            apply_deferred
                .after(InternalStage::Animation)
                .before(InternalStage::Apply),
            apply_deferred
                .after(InternalStage::Apply)
                .before(ExternalStage::Action),
            apply_deferred
                .after(ExternalStage::Action)
                .before(InternalStage::Clean),
            apply_deferred
                .after(InternalStage::Clean)
                .before(InternalStage::DeclarativePass),
            apply_deferred
                .after(InternalStage::DeclarativePass)
                .before(ExternalStage::Configure),
            apply_deferred
                .after(ExternalStage::Configure)
                .before(InternalStage::SecondClean),
            apply_deferred
                .after(InternalStage::SecondClean)
                .before(InternalStage::ReactivePass),
            apply_deferred
                .after(InternalStage::ReactivePass)
                .before(InternalStage::Resolve),
            apply_deferred
                .after(InternalStage::Resolve)
                .before(InternalStage::FinalizeCoordinate),
            apply_deferred
                .after(InternalStage::FinalizeCoordinate)
                .before(InternalStage::Differential),
            apply_deferred
                .after(InternalStage::Differential)
                .before(InternalStage::Finish),
        ));
    }
    pub(crate) fn process(&mut self) {
        self.scheduler.exec_main(&mut self.ecs);
    }
    pub(crate) fn viewport_handle_changes(&mut self) -> Option<Position<LogicalContext>> {
        self.ecs.get_resource_mut::<ViewportHandle>()?.changes()
    }
    pub(crate) fn adjust_viewport_handle(&mut self, willow: &Willow) {
        let scale_value = self.ecs.get_resource::<ScaleFactor>().unwrap().value();
        self.ecs
            .get_resource_mut::<ViewportHandle>()
            .unwrap()
            .resize(willow.actual_area().to_logical(scale_value));
    }
}
pub struct RenderQueueHandle<'a> {
    elm: &'a mut Elm,
}
impl<'a> RenderQueueHandle<'a> {
    pub(crate) fn new(elm: &'a mut Elm) -> Self {
        Self { elm }
    }
    pub fn read_removes<R: Render>(&mut self) -> HashSet<Entity> {
        self.elm
            .ecs
            .get_resource_mut::<RenderRemoveQueue>()
            .unwrap()
            .queue
            .get_mut(&RenderLink::new::<R>())
            .expect("remove-queue")
            .drain()
            .collect()
    }
    pub fn read_adds<R: Render, D: Component + Clone + PartialEq>(
        &mut self,
    ) -> Vec<RenderPacket<D>> {
        let mut queue = self
            .elm
            .ecs
            .get_resource_mut::<RenderAddQueue<D>>()
            .expect("render-queue");
        queue
            .queue
            .get_mut(&RenderLink::new::<R>())
            .expect("render-queue")
            .drain()
            .map(|a| a.into())
            .collect()
    }
}
#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum InternalStage {
    External,
    Apply,
    // Action
    Clean,
    DeclarativePass,
    // Configure
    SecondClean,
    ReactivePass,
    Resolve,
    FinalizeCoordinate,
    Differential,
    Finish,
    Animation,
}
#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum ExternalStage {
    Action,
    Configure,
}
