use bevy_ecs::component::Component;

#[derive(Component, Copy, Clone)]
pub struct Remove {}

impl Remove {
    pub fn new() -> Self {
        Self {}
    }
}