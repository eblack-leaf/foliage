use crate::IntoTargets;
use crate::Trigger;
use crate::{Color, EcsExtension, IconId, Tree, Update};
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::{HookContext, Insert};
use bevy_ecs::prelude::{Component, Query};
use bevy_ecs::world::DeferredWorld;

pub(crate) mod button;
mod children;
pub(crate) mod text_input;
pub use children::Children;
pub(crate) use text_input::keybindings::KeyBindings;

/// The authoring lead for a widget: `children` builds its child entities (via [`Children`]) and
/// returns the `Handle` that names them; `remove` says which of those entities to despawn when
/// the composite itself is removed.
pub trait Composite: Component {
    type Handle: Component;
    /// Default is an unmigrated stub — real composites override this. It's never invoked unless
    /// a type wires `composite_on_insert::<Self>` into its own `on_insert` hook, so leaving it
    /// unmigrated is inert, not a runtime risk.
    fn children(_this: Entity, _children: &mut Children<DeferredWorld>) -> Self::Handle {
        unimplemented!(
            "Composite::children not migrated for this type; it must keep its own on_insert hook"
        )
    }
    fn remove(handle: &Self::Handle) -> impl IntoTargets + Send + Sync + 'static;
}
/// Wired once per `Composite` type via `#[component(on_insert = composite_on_insert::<X>)]` on
/// the marker component — replaces the hand-written `on_insert` hook every composite used to
/// need just to spawn its children and build its `Handle`.
pub fn composite_on_insert<C: Composite>(mut world: DeferredWorld, ctx: HookContext) {
    let this = ctx.entity;
    let handle = {
        let mut children = Children::new(this, &mut world);
        C::children(this, &mut children)
    };
    world.commands().entity(this).insert(handle);
}
pub fn handle_replace<C: Composite>(mut world: DeferredWorld, ctx: HookContext) {
    let this = ctx.entity;
    let handle = world.get::<C::Handle>(this).unwrap();
    let targets = C::remove(handle);
    world.commands().remove(targets);
}
/// Re-fires `Update::<C>` at the same entity whenever `C` is inserted/changed — the identical
/// half of every composite's old hand-written `forward_X` observer, generic over the property
/// type instead of copy-pasted per property. Register it explicitly, per entity, exactly where
/// the old `.observe(Self::forward_text)` calls used to go (e.g. in a composite's `on_add` hook:
/// `.observe(forward::<TextValue>)`) — scoped to the entities that actually declared they want
/// it, not a blanket global observer over every entity in the app that happens to carry `C`.
pub fn forward<C: Component>(trigger: Trigger<Insert, C>, mut tree: Tree) {
    tree.trigger_targets(Update::<C>::new(), trigger.event_target());
}
/// Walks a `Root` pointer if the entity has one, otherwise returns the entity itself — replaces
/// the hand-rolled "query Root, fall back to self" dance that used to be repeated at every
/// reactive site in a composite whose children can't place/route themselves.
pub fn resolve_root(entity: Entity, roots: &Query<&Root>) -> Entity {
    roots.get(entity).map(|r| r.0).unwrap_or(entity)
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
/// Points a composite's descendant at its root entity, for reactive systems that can't resolve
/// placement/state locally and need to route back through the root's `Handle`. Use
/// [`resolve_root`] rather than querying this directly.
#[derive(Component, Copy, Clone)]
pub struct Root(pub Entity);
