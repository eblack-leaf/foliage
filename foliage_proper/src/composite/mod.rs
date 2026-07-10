use crate::{Color, EcsExtension, IconId};
use bevy_ecs::entity::Entity;
use crate::IntoTargets;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::prelude::Component;
use bevy_ecs::world::DeferredWorld;

pub(crate) mod button;
pub(crate) mod carousel;
pub(crate) mod dropdown;
pub(crate) mod list;
pub(crate) mod pagination;
pub(crate) mod prompt;
pub(crate) mod text_input;
pub(crate) use text_input::keybindings::KeyBindings;

/// Position of a generated child (row, dot, page, option) within its composite.
#[derive(Component, Copy, Clone, Eq, PartialEq, Debug)]
pub struct ItemIndex(pub usize);

/// The one event item-based composites (List, Pagination, Dropdown, Carousel) emit at their
/// root when the current item changes. Subscribe with `tree.subscribe(composite, ...)`.
#[derive(bevy_ecs::event::EntityEvent, Copy, Clone, Debug)]
pub struct Selected {
    entity: Entity,
    pub index: usize,
}
impl Selected {
    pub fn new(index: usize) -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            index,
        }
    }
}
crate::targeted_event!(Selected);
pub trait Composite {
    type Handle: Component;
    fn remove(handle: &Self::Handle) -> impl IntoTargets + Send + Sync + 'static;
}
pub fn handle_replace<C: Composite>(mut world: DeferredWorld, ctx: HookContext) {
    let this = ctx.entity;
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
#[derive(Component, Copy, Clone)]
pub(crate) struct Root(pub(crate) Entity);
