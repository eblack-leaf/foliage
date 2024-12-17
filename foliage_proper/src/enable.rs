use bevy_ecs::component::Component;

#[derive(Component, Copy, Clone)]
pub struct Enable {}

impl Enable {
    pub fn new() -> Enable {
        Enable {}
    }
}
