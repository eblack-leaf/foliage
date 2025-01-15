use crate::handle_replace;
use crate::{Component, Composite};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::world::DeferredWorld;

#[derive(Component)]
#[component(on_insert = Self::on_insert)]
pub struct Button {
    // args
}
impl Button {
    pub fn new() -> Self {
        Self {}
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let args = world.get::<Button>(this).unwrap();
        // make handle elements
    }
}
impl Composite for Button {
    type Handle = Handle;
    fn remove(handle: &Self::Handle) -> impl TriggerTargets + Send + Sync + 'static {
        [handle.panel, handle.text, handle.icon]
    }
}
#[derive(Component, Copy, Clone)]
#[component(on_replace = handle_replace::<Button>)]
pub struct Handle {
    pub panel: Entity,
    pub icon: Entity,
    pub text: Entity,
}
