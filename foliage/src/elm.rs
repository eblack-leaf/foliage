use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{event_update_system, Event, Events};
use bevy_ecs::prelude::{
    apply_deferred, Component, IntoSystemConfigs, IntoSystemSetConfigs, Schedule, SystemSet,
};
use bevy_ecs::schedule::ExecutorKind;
use bevy_ecs::system::{Command, Resource};
use bevy_ecs::world::World;

use crate::ash::Render;
use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::NumericalContext;
use crate::differential::{
    differential, RenderAddQueue, RenderLink, RenderPacket, RenderRemoveQueue,
};
use crate::ginkgo::viewport::ViewportHandle;
use crate::ginkgo::ScaleFactor;
use crate::grid::{place_on_grid, viewport_changes_layout, Grid, Layout, LayoutGrid};
use crate::interaction::{
    FocusedEntity, InteractiveEntity, KeyboardAdapter, MouseAdapter, TouchAdapter,
};
use crate::signal::engage_action;
use crate::signal::{
    clean, clear_signal, filter_signal, filtered_signaled_spawn, signaled_clean, signaled_spawn,
};
use crate::view::{
    adjust_view_grid_on_change, attempt_to_confirm, cleanup_view, on_target_grid_placement_change,
    on_view_grid_change, resignal_on_layout_change, signal_confirmation, signal_stage,
};
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
    leaf_fns: Vec<Box<fn(&mut Elm)>>,
}

#[derive(Resource)]
pub(crate) struct DifferentialScheduleLimiter<D>(PhantomData<D>);

impl<D> Default for DifferentialScheduleLimiter<D> {
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
pub(crate) struct ActionLimiter<D>(PhantomData<D>);

impl<D> Default for ActionLimiter<D> {
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
impl Elm {
    pub fn enable_event<E: Event + Send + Sync + 'static>(&mut self) {
        if !self.ecs.world.contains_resource::<EventLimiter<E>>() {
            self.ecs.world.insert_resource(Events::<E>::default());
            self.scheduler
                .main
                .add_systems((event_update_system::<E>.in_set(ScheduleMarkers::Events),));
            self.ecs.world.insert_resource(EventLimiter::<E>::default());
        }
    }
    pub fn enable_differential<R: Render, D: Component + PartialEq + Clone>(&mut self) {
        if !self
            .ecs
            .world
            .contains_resource::<DifferentialScheduleLimiter<D>>()
        {
            self.scheduler
                .main
                .add_systems((differential::<D>.in_set(ScheduleMarkers::Differential),));
            self.ecs
                .world
                .insert_resource(DifferentialScheduleLimiter::<D>::default())
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
    pub(crate) fn checked_add_signal_fns<A: Bundle + Clone + 'static + Send + Sync>(&mut self) {
        if !self.ecs.world.contains_resource::<SignalLimiter<A>>() {
            self.scheduler
                .main
                .add_systems(signaled_spawn::<A>.in_set(ScheduleMarkers::Spawn));
            self.ecs
                .world
                .insert_resource(SignalLimiter::<A>::default());
        }
    }
    pub(crate) fn checked_add_filtered_signal_fns<A: Bundle + Clone + 'static + Send + Sync>(
        &mut self,
    ) {
        if !self.ecs.world.contains_resource::<SignalLimiter<A>>() {
            self.scheduler
                .main
                .add_systems(filtered_signaled_spawn::<A>.in_set(ScheduleMarkers::SpawnFiltered));
            self.ecs
                .world
                .insert_resource(SignalLimiter::<A>::default());
        }
    }
    pub(crate) fn checked_add_action_fns<A: Command + Clone + 'static + Send + Sync>(&mut self) {
        if !self.ecs.world.contains_resource::<ActionLimiter<A>>() {
            self.scheduler
                .main
                .add_systems(engage_action::<A>.in_set(ScheduleMarkers::Action));
            self.ecs
                .world
                .insert_resource(ActionLimiter::<A>::default());
        }
    }
    pub(crate) fn initialize(&mut self, leaf_fns: Vec<Box<fn(&mut Elm)>>) {
        for leaf_fn in leaf_fns {
            (leaf_fn)(self);
        }
        self.scheduler.exec_startup(&mut self.ecs);
        self.initialized = true;
    }

    pub(crate) fn configure(
        &mut self,
        window_area: Area<NumericalContext>,
        scale_factor: ScaleFactor,
    ) {
        self.ecs
            .world
            .insert_resource(ViewportHandle::new(window_area));
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
        self.scheduler.main.configure_sets(
            (
                ScheduleMarkers::External,
                ScheduleMarkers::SignalConfirmation,
                ScheduleMarkers::Action,
                ScheduleMarkers::SignalStage,
                ScheduleMarkers::Spawn,
                ScheduleMarkers::SpawnFiltered,
                ScheduleMarkers::Clean,
                ScheduleMarkers::GridSemantics,
                ScheduleMarkers::Config,
                ScheduleMarkers::SignalConfirmationStart,
                ScheduleMarkers::FinalizeCoordinate,
                ScheduleMarkers::Differential,
            )
                .chain(),
        );
        self.scheduler.main.add_systems((
            viewport_changes_layout.in_set(ScheduleMarkers::External),
            signal_confirmation.in_set(ScheduleMarkers::SignalConfirmation),
            (signal_stage, resignal_on_layout_change)
                .in_set(ScheduleMarkers::SignalStage)
                .before(filter_signal),
            filter_signal.in_set(ScheduleMarkers::SignalStage),
            (cleanup_view, signaled_clean, apply_deferred, clean)
                .chain()
                .in_set(ScheduleMarkers::Clean),
            place_on_grid.in_set(ScheduleMarkers::GridSemantics),
            adjust_view_grid_on_change
                .in_set(ScheduleMarkers::GridSemantics)
                .after(place_on_grid),
            (on_view_grid_change, on_target_grid_placement_change)
                .in_set(ScheduleMarkers::GridSemantics)
                .after(adjust_view_grid_on_change),
            attempt_to_confirm.in_set(ScheduleMarkers::SignalConfirmationStart),
            clear_signal.after(ScheduleMarkers::Differential),
        ));
        self.scheduler.main.add_systems((
            apply_deferred
                .after(ScheduleMarkers::External)
                .before(ScheduleMarkers::SignalConfirmation),
            apply_deferred
                .after(ScheduleMarkers::SignalConfirmation)
                .before(ScheduleMarkers::Action),
            apply_deferred
                .after(ScheduleMarkers::Action)
                .before(ScheduleMarkers::SignalStage),
            apply_deferred
                .after(ScheduleMarkers::SignalStage)
                .before(ScheduleMarkers::Spawn),
            apply_deferred
                .after(ScheduleMarkers::Spawn)
                .before(ScheduleMarkers::SpawnFiltered),
            apply_deferred
                .after(ScheduleMarkers::SpawnFiltered)
                .before(ScheduleMarkers::Clean),
            apply_deferred
                .after(ScheduleMarkers::Clean)
                .before(ScheduleMarkers::GridSemantics),
            apply_deferred
                .after(ScheduleMarkers::GridSemantics)
                .before(ScheduleMarkers::Config),
            apply_deferred
                .after(ScheduleMarkers::Config)
                .before(ScheduleMarkers::SignalConfirmationStart),
            apply_deferred
                .after(ScheduleMarkers::SignalConfirmationStart)
                .before(ScheduleMarkers::FinalizeCoordinate),
            apply_deferred
                .after(ScheduleMarkers::FinalizeCoordinate)
                .before(ScheduleMarkers::Differential),
        ));
    }
    pub(crate) fn process(&mut self) {
        self.scheduler.exec_main(&mut self.ecs);
    }
    pub(crate) fn viewport_handle_changes(&mut self) -> Option<Position<NumericalContext>> {
        self.ecs
            .world
            .get_resource_mut::<ViewportHandle>()
            .unwrap()
            .changes()
    }
    pub(crate) fn adjust_viewport_handle(&mut self, willow: &Willow) {
        self.ecs
            .world
            .get_resource_mut::<ViewportHandle>()
            .unwrap()
            .resize(willow.actual_area().to_numerical());
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
    SpawnFiltered,
    Action,
    SignalStage,
    External,
    GridSemantics,
    SignalConfirmation,
    SignalConfirmationStart,
    Clean,
    Interaction,
    Events,
}
