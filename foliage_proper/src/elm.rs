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
use crate::grid::{resolve_grid_locations, Grid};
use crate::interaction::{
    FocusedEntity, InteractiveEntity, KeyboardAdapter, MouseAdapter, TouchAdapter,
};
use crate::layout::{viewport_changes_layout, Layout, LayoutGrid};
use crate::leaf::{
    apply_triggered, change_stem, clear_trigger_signal, dependent_elevation, interaction_enable,
    recursive_removal, recursive_visibility, update_stem_deps,
};
use crate::opacity::opacity;
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
            .run(&mut ecs.world);
    }
    pub(crate) fn exec_startup(&mut self, ecs: &mut Ecs) {
        self.startup
            .set_executor_kind(ExecutorKind::MultiThreaded)
            .run(&mut ecs.world);
    }
}

#[derive(Default)]
pub struct Ecs {
    pub(crate) world: World,
}

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
        if !self.ecs.world.contains_resource::<EventLimiter<E>>() {
            self.ecs.world.insert_resource(Events::<E>::default());
            EventRegistry::register_event::<E>(&mut self.ecs.world);
            self.scheduler
                .main
                .add_systems(apply_triggered::<E>.in_set(ScheduleMarkers::ApplyTriggerSignal));
            self.ecs.world.insert_resource(EventLimiter::<E>::default());
        }
    }
    pub fn enable_animation<A: Animate>(&mut self) {
        if !self.ecs.world.contains_resource::<AnimationLimiter<A>>() {
            self.scheduler
                .main
                .add_systems(animate::<A>.in_set(ScheduleMarkers::Animation));
            self.ecs
                .world
                .insert_resource(AnimationLimiter::<A>::default());
        }
    }
    pub fn enable_differential<R: Render, D: Component + PartialEq + Clone>(&mut self) {
        if !self.ecs.world.contains_resource::<DifferentialLimiter<D>>() {
            self.scheduler.main.add_systems((
                differential::<D>.in_set(ScheduleMarkers::Differential),
                visibility_changed::<D>.in_set(ScheduleMarkers::Differential),
            ));
            self.ecs
                .world
                .insert_resource(DifferentialLimiter::<D>::default())
        }
        if !self.ecs.world.contains_resource::<RenderAddQueue<D>>() {
            self.ecs
                .world
                .insert_resource(RenderAddQueue::<D>::default());
        }
        let link = RenderLink::new::<R>();
        self.ecs
            .world
            .get_resource_mut::<RenderAddQueue<D>>()
            .unwrap()
            .queue
            .insert(link, HashMap::new());
        self.ecs
            .world
            .get_resource_mut::<RenderAddQueue<D>>()
            .unwrap()
            .cache
            .insert(link, HashMap::new());
        self.ecs
            .world
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
        self.ecs.world.insert_resource(ViewportHandle::new(
            window_area.to_logical(scale_factor.value()),
        ));
        self.ecs.world.insert_resource(scale_factor);
        self.ecs.world.insert_resource(RenderRemoveQueue::default());
        self.ecs
            .world
            .insert_resource(LayoutGrid::new(Grid::new(4, 4)));
        self.ecs.world.insert_resource(Layout::SQUARE);
        self.ecs.world.insert_resource(TouchAdapter::default());
        self.ecs.world.insert_resource(MouseAdapter::default());
        self.ecs.world.insert_resource(KeyboardAdapter::default());
        self.ecs.world.insert_resource(InteractiveEntity::default());
        self.ecs.world.insert_resource(FocusedEntity::default());
        self.ecs
            .world
            .insert_resource(ClippingSectionQueue::default());
        self.scheduler.main.configure_sets(
            (
                ScheduleMarkers::Events,
                ScheduleMarkers::External,
                ScheduleMarkers::Interaction,
                ScheduleMarkers::Animation,
                ScheduleMarkers::ApplyTriggerSignal,
                ScheduleMarkers::Unused2,
                ScheduleMarkers::Unused3,
                ScheduleMarkers::Spawn,
                ScheduleMarkers::Unused4,
                ScheduleMarkers::Clean,
                ScheduleMarkers::GridSemantics,
                ScheduleMarkers::Unused6,
                ScheduleMarkers::Preparation,
                ScheduleMarkers::Resolve,
                ScheduleMarkers::Config,
                ScheduleMarkers::Unused5,
                ScheduleMarkers::FinalizeCoordinate,
                ScheduleMarkers::Differential,
            )
                .chain(),
        );
        self.scheduler.main.add_systems((
            event_update_system.in_set(ScheduleMarkers::Events),
            (viewport_changes_layout, await_assets, navigate).in_set(ScheduleMarkers::External),
            (change_stem, update_stem_deps)
                .chain()
                .in_set(ScheduleMarkers::Clean)
                .before(recursive_removal)
                .before(recursive_visibility),
            (recursive_removal, recursive_visibility)
                .in_set(ScheduleMarkers::Clean)
                .before(crate::leaf::remove),
            (crate::leaf::remove, interaction_enable).in_set(ScheduleMarkers::Clean),
            dependent_elevation.in_set(ScheduleMarkers::GridSemantics),
            resolve_grid_locations.in_set(ScheduleMarkers::GridSemantics),
            opacity.in_set(ScheduleMarkers::Resolve),
            pull_clipping_section.in_set(ScheduleMarkers::FinalizeCoordinate),
            clear_trigger_signal.after(ScheduleMarkers::Differential),
        ));
        self.scheduler.main.add_systems((
            apply_deferred
                .after(ScheduleMarkers::Events)
                .before(ScheduleMarkers::External),
            apply_deferred
                .after(ScheduleMarkers::External)
                .before(ScheduleMarkers::Interaction),
            apply_deferred
                .after(ScheduleMarkers::Interaction)
                .before(ScheduleMarkers::Animation),
            apply_deferred
                .after(ScheduleMarkers::Animation)
                .before(ScheduleMarkers::ApplyTriggerSignal),
            apply_deferred
                .after(ScheduleMarkers::ApplyTriggerSignal)
                .before(ScheduleMarkers::Unused2),
            apply_deferred
                .after(ScheduleMarkers::Unused2)
                .before(ScheduleMarkers::Unused3),
            apply_deferred
                .after(ScheduleMarkers::Unused3)
                .before(ScheduleMarkers::Spawn),
            apply_deferred
                .after(ScheduleMarkers::Spawn)
                .before(ScheduleMarkers::Unused4),
            apply_deferred
                .after(ScheduleMarkers::Unused4)
                .before(ScheduleMarkers::Clean),
            apply_deferred
                .after(ScheduleMarkers::Clean)
                .before(ScheduleMarkers::GridSemantics),
            apply_deferred
                .after(ScheduleMarkers::GridSemantics)
                .before(ScheduleMarkers::Unused6),
            apply_deferred
                .after(ScheduleMarkers::Unused6)
                .before(ScheduleMarkers::Preparation),
            apply_deferred
                .after(ScheduleMarkers::Preparation)
                .before(ScheduleMarkers::Resolve),
            apply_deferred
                .after(ScheduleMarkers::Resolve)
                .before(ScheduleMarkers::Config),
            apply_deferred
                .after(ScheduleMarkers::Config)
                .before(ScheduleMarkers::Unused5),
            apply_deferred
                .after(ScheduleMarkers::Unused5)
                .before(ScheduleMarkers::FinalizeCoordinate),
            apply_deferred
                .after(ScheduleMarkers::FinalizeCoordinate)
                .before(ScheduleMarkers::Differential),
        ));
    }
    pub(crate) fn process(&mut self) {
        self.scheduler.exec_main(&mut self.ecs);
    }
    pub(crate) fn viewport_handle_changes(&mut self) -> Option<Position<LogicalContext>> {
        self.ecs
            .world
            .get_resource_mut::<ViewportHandle>()?
            .changes()
    }
    pub(crate) fn adjust_viewport_handle(&mut self, willow: &Willow) {
        let scale_value = self
            .ecs
            .world
            .get_resource::<ScaleFactor>()
            .unwrap()
            .value();
        self.ecs
            .world
            .get_resource_mut::<ViewportHandle>()
            .unwrap()
            .resize(willow.actual_area().to_logical(scale_value));
    }
    pub fn enable_retrieve<B: Bundle + Send + Sync + 'static>(&mut self) {
        if !self.ecs.world.contains_resource::<RetrieveLimiter<B>>() {
            self.scheduler
                .main
                .add_systems(on_retrieve::<B>.in_set(ScheduleMarkers::Spawn));
            self.ecs
                .world
                .insert_resource(RetrieveLimiter::<B>::default());
        }
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
            .world
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
            .world
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
pub enum ScheduleMarkers {
    FinalizeCoordinate,
    Differential,
    Config,
    Spawn,
    Unused4,
    ApplyTriggerSignal,
    Unused2,
    External,
    GridSemantics,
    Animation,
    Unused5,
    Clean,
    Interaction,
    Events,
    Unused3,
    Preparation,
    Unused6,
    Resolve,
}
