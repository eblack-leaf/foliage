use crate::ginkgo::ViewportHandle;
use crate::willow::Willow;
use crate::{Area, NumericalContext, Position};
use bevy_ecs::prelude::Schedule;
use bevy_ecs::schedule::ExecutorKind;
use bevy_ecs::world::World;
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
