use crate::{Color, EcsExtension, IconId};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::prelude::Component;
use bevy_ecs::world::DeferredWorld;

pub(crate) mod button;
pub trait Composite {
    type Handle: Component;
    fn remove(handle: &Self::Handle) -> impl TriggerTargets + Send + Sync + 'static;
}
pub fn handle_replace<C: Composite>(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
    let handle = world.get::<C::Handle>(this).unwrap();
    let targets = C::remove(handle);
    world.commands().remove(targets);
}
#[derive(Component, Copy, Clone, Default)]
pub struct Primary(pub Color);
#[derive(Component, Copy, Clone, Default)]
pub struct Secondary(pub Color);
#[derive(Component, Copy, Clone, Default)]
pub struct Tertiary(pub Color);
#[derive(Component, Clone, Default)]
pub struct TextValue(pub String);
#[derive(Component, Copy, Clone, Default)]
pub struct IconValue(pub IconId);
