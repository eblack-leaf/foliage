use crate::ash::Render;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, Schedule};
use bevy_ecs::schedule::ExecutorKind;
use bevy_ecs::world::World;
use std::collections::HashSet;

use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::NumericalContext;
use crate::differential::{RenderAddQueue, RenderLink, RenderPacket, RenderRemoveQueue};
use crate::ginkgo::ViewportHandle;
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
}

impl Elm {
    pub(crate) fn initialized(&self) -> bool {
        self.initialized
    }
    pub(crate) fn initialize(&mut self, window_area: Area<NumericalContext>) {
        // attach leafs?
        self.ecs
            .world
            .insert_resource(ViewportHandle::new(window_area));
        self.ecs.world.insert_resource(RenderRemoveQueue::default());
        self.scheduler.exec_startup(&mut self.ecs);
        self.initialized = true;
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
        self.elm
            .ecs
            .world
            .get_resource_mut::<RenderAddQueue<D>>()
            .expect("render-queue")
            .queue
            .get_mut(&RenderLink::new::<R>())
            .expect("render-queue")
            .drain()
            .map(|a| a.into())
            .collect()
    }
}
