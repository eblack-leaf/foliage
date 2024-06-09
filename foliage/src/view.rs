use bevy_ecs::component::Component;
use bevy_ecs::prelude::Entity;

#[derive(Clone, Copy)]
pub struct ViewHandle(pub(crate) Entity);
pub struct Stage(pub(crate) i32);
#[derive(Component)]
pub struct View {
    pub(crate) stages: Vec<ViewStage>,
    current: Stage,
}
impl View {
    pub(crate) fn new() -> Self {
        Self {
            stages: vec![],
            current: Stage(0),
        }
    }
}
pub struct ViewStage {
    signals: Vec<SignalHandle>,
    num_confirmed: i32,
}
impl Default for ViewStage {
    fn default() -> Self {
        ViewStage {
            signals: vec![],
            num_confirmed: 0,
        }
    }
}
pub struct SignalHandle {
    repr: Entity,
}
#[derive(Component)]
pub struct SignalConfirmation(pub(crate) bool);
impl SignalHandle {
    pub fn repr(&self) -> Entity {
        self.repr
    }
}
