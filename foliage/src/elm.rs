use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{
    apply_deferred, Component, IntoSystemConfigs, IntoSystemSetConfigs, Schedule, SystemSet,
};
use bevy_ecs::schedule::ExecutorKind;
use bevy_ecs::system::Resource;
use bevy_ecs::world::World;

use crate::ash::Render;
use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::NumericalContext;
use crate::differential::{
    differential, RenderAddQueue, RenderLink, RenderPacket, RenderRemoveQueue,
};
use crate::ginkgo::{ScaleFactor, ViewportHandle};
use crate::willow::Willow;

#[derive(Default)]
pub struct Scheduler {
    pub(crate) startup: Schedule,
    pub(crate) main: Schedule,
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

impl Elm {
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
        self.scheduler.main.configure_sets(
            (
                ScheduleMarkers::Config,
                ScheduleMarkers::Coordinate,
                ScheduleMarkers::Differential,
            )
                .chain(),
        );
        self.scheduler.main.add_systems((
            apply_deferred
                .after(ScheduleMarkers::Config)
                .before(ScheduleMarkers::Coordinate),
            apply_deferred
                .after(ScheduleMarkers::Coordinate)
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
    Coordinate,
    Differential,
    Config,
}
