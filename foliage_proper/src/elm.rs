use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use crate::action::Actionable;
use crate::ash::Render;
use crate::asset::on_retrieve;
use crate::clipboard::{read_retrieve, Clipboard};
use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::NumericalContext;
use crate::derive::on_derive;
use crate::differential::{
    added_invalidate, differential, RenderAddQueue, RenderLink, RenderPacket, RenderRemoveQueue,
};
use crate::element::IdTable;
use crate::ginkgo::viewport::ViewportHandle;
use crate::ginkgo::ScaleFactor;
use crate::grid::{Grid, Layout, LayoutGrid};
use crate::interaction::{
    FocusedEntity, InteractiveEntity, KeyboardAdapter, MouseAdapter, TouchAdapter,
};
use crate::willow::Willow;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{event_update_system, Event, Events};
use bevy_ecs::prelude::{
    apply_deferred, Component, IntoSystemConfigs, IntoSystemSetConfigs, Schedule, SystemSet,
};
use bevy_ecs::schedule::ExecutorKind;
use bevy_ecs::system::Resource;
use bevy_ecs::world::World;

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
impl Elm {
    pub fn enable_action<A: Actionable>(&mut self) {
        if !self.ecs.world.contains_resource::<ActionLimiter<A>>() {
            // signaled-action setup?
            self.ecs
                .world
                .insert_resource(ActionLimiter::<A>::default());
        }
    }
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
            self.scheduler.main.add_systems((
                differential::<D>.in_set(ScheduleMarkers::Differential),
                added_invalidate::<D>.in_set(ScheduleMarkers::Differential),
            ));
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
        self.ecs.world.insert_resource(IdTable::default());
        self.ecs.world.insert_resource(TouchAdapter::default());
        self.ecs.world.insert_resource(MouseAdapter::default());
        self.ecs.world.insert_resource(KeyboardAdapter::default());
        self.ecs.world.insert_resource(InteractiveEntity::default());
        self.ecs.world.insert_resource(FocusedEntity::default());
        self.ecs.world.insert_non_send_resource(Clipboard::new());
        self.scheduler.main.configure_sets(
            (
                ScheduleMarkers::Events,
                ScheduleMarkers::External,
                ScheduleMarkers::Interaction,
                ScheduleMarkers::SignalConfirmation,
                ScheduleMarkers::Action,
                ScheduleMarkers::SignalStage,
                ScheduleMarkers::StageActions,
                ScheduleMarkers::Spawn,
                ScheduleMarkers::SpawnFiltered,
                ScheduleMarkers::Clean,
                ScheduleMarkers::GridSemantics,
                ScheduleMarkers::Preparation,
                ScheduleMarkers::Config,
                ScheduleMarkers::SignalConfirmationStart,
                ScheduleMarkers::FinalizeCoordinate,
                ScheduleMarkers::Differential,
            )
                .chain(),
        );
        self.scheduler
            .main
            .add_systems((crate::differential::remove.in_set(ScheduleMarkers::Differential),));
        self.scheduler.main.add_systems((
            apply_deferred
                .after(ScheduleMarkers::Events)
                .before(ScheduleMarkers::External),
            apply_deferred
                .after(ScheduleMarkers::External)
                .before(ScheduleMarkers::Interaction),
            apply_deferred
                .after(ScheduleMarkers::Interaction)
                .before(ScheduleMarkers::SignalConfirmation),
            apply_deferred
                .after(ScheduleMarkers::SignalConfirmation)
                .before(ScheduleMarkers::Action),
            apply_deferred
                .after(ScheduleMarkers::Action)
                .before(ScheduleMarkers::SignalStage),
            apply_deferred
                .after(ScheduleMarkers::SignalStage)
                .before(ScheduleMarkers::StageActions),
            apply_deferred
                .after(ScheduleMarkers::StageActions)
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
                .before(ScheduleMarkers::Preparation),
            apply_deferred
                .after(ScheduleMarkers::Preparation)
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
            .resize(willow.actual_area().to_logical(scale_value).to_numerical());
    }
    pub(crate) fn enable_clipboard_retrieve<B: Bundle + Send + Sync + 'static>(&mut self) {
        if !self
            .ecs
            .world
            .contains_resource::<ClipboardRetrieveLimiter<B>>()
        {
            self.scheduler
                .main
                .add_systems(read_retrieve::<B>.in_set(ScheduleMarkers::External));
            self.ecs
                .world
                .insert_resource(ClipboardRetrieveLimiter::<B>::default());
        }
    }
    pub(crate) fn enable_retrieve<B: Bundle + Send + Sync + 'static>(&mut self) {
        if !self.ecs.world.contains_resource::<RetrieveLimiter<B>>() {
            self.scheduler
                .main
                .add_systems(on_retrieve::<B>.in_set(ScheduleMarkers::Spawn));
            self.ecs
                .world
                .insert_resource(RetrieveLimiter::<B>::default());
        }
    }
    pub(crate) fn enable_derive<
        D: Resource + Send + Sync + 'static + Clone,
        B: Bundle + Send + Sync + 'static + Clone,
    >(
        &mut self,
    ) {
        if !self.ecs.world.contains_resource::<DeriveLimiter<D, B>>() {
            self.scheduler
                .main
                .add_systems(on_derive::<D, B>.in_set(ScheduleMarkers::Preparation));
            self.ecs
                .world
                .insert_resource(DeriveLimiter::<D, B>::default());
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
    StageActions,
    Preparation,
}
