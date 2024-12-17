use bevy_ecs::component::Component;

#[derive(Component, Copy, Clone)]
pub struct Disable {}

impl Disable {
    pub fn new() -> Disable {
        Disable {}
    }
}