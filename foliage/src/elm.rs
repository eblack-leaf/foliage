use crate::ginkgo::ViewportHandle;
use crate::willow::Willow;
use crate::{NumericalContext, Position};
use bevy_ecs::world::World;

#[derive(Default)]
pub struct Ecs {
    pub(crate) world: World,
}
#[derive(Default)]
pub struct Elm {
    pub ecs: Ecs,
}
impl Elm {
    pub(crate) fn process(&mut self) {}
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
