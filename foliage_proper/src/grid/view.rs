use crate::Component;
use bevy_ecs::entity::Entity;

pub struct View {}
impl View {
    pub fn new() -> View {
        View {}
    }
    pub fn context(entity: Entity) -> ViewContext {
        todo!()
    }
}
#[derive(Copy, Clone, Default, Component)]
pub struct ViewContext {
    pub entity: Option<Entity>,
}
