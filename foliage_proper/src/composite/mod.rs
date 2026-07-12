use crate::IconId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, Query};

pub(crate) mod button;
pub(crate) mod text_input;
pub(crate) use text_input::keybindings::KeyBindings;

/// A text-bearing entity's public value channel: write it to a `Text` entity (or a widget
/// root that forwards it, like Button/TextInput) and the content follows.
#[derive(Component, Clone, Default)]
pub struct TextValue(pub String);
/// An icon-bearing entity's public value channel, same contract as [`TextValue`].
#[derive(Component, Copy, Clone, Default)]
pub struct IconValue(pub IconId);
/// Points a widget's descendant at its root entity, for reactive systems that can't resolve
/// state locally and need to route back through the root. Use [`Root::resolve`] rather than
/// querying this directly.
#[derive(Component, Copy, Clone)]
pub struct Root(pub Entity);
impl Root {
    /// Walks a `Root` pointer if the entity has one, otherwise returns the entity itself -- for
    /// reactive systems on a widget's descendants that need to route back to the widget root.
    pub fn resolve(entity: Entity, roots: &Query<&Root>) -> Entity {
        roots.get(entity).map(|r| r.0).unwrap_or(entity)
    }
}
