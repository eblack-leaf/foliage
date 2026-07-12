use crate::{EcsExtension, Entity, Sprout, Tree, Trigger};
use bevy_ecs::component::Component;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::observer::IntoEntityObserver;
use bevy_ecs::system::Query;
use bevy_ecs::world::World;

/// Spawns a widget's (or a one-off structure's) children under a common `parent`, auto-filling
/// `.stem(parent)` on any [`Sprout`] -- a named one (`Panel::new()`, `Text::new()`) or a bare
/// `Leaf::sprout()` for a child with no marker component of its own. Every child uses the same
/// `.elevate()/.at()/.with()` chain regardless of which; there's exactly one way to spawn a
/// child here.
///
/// A child's `Location` is whatever `.at(...)` sets, or the empty default if never called: some
/// children (e.g. an icon whose position depends on a reactive property) spawn without one and
/// receive their real `Location` from a [`Children::react`] reaction's first fire, in the same
/// command batch -- never a visible flicker.
pub struct Children<'t, T: EcsExtension> {
    parent: Entity,
    tree: &'t mut T,
}
impl<'t, T: EcsExtension> Children<'t, T> {
    pub fn new(parent: Entity, tree: &'t mut T) -> Self {
        Self { parent, tree }
    }
    pub fn parent(&self) -> Entity {
        self.parent
    }
    pub fn tree(&mut self) -> &mut T {
        self.tree
    }
    pub fn spawn<S: Sprout>(&mut self, spec: S) -> Entity {
        spec.stem(self.parent).photosynthesize(self.tree)
    }
    /// Registers `observer` -- a plain bevy entity-observer watching `Trigger<Insert, C>`, full
    /// `SystemParam` freedom -- on `parent`, then re-fires it once with the current value so
    /// initial state and every later `write_to` run the SAME code path. This is the one door
    /// for everything data-dependent in a widget: values AND structure (a dropdown's option
    /// rows, a hand's cards). What the body does -- respawn, pool, patch -- is entirely the
    /// author's policy. The `_` stands in for the observer's inferred marker:
    /// `kids.react::<TextValue, _>(..)`.
    pub fn react<C: Component + Clone, M>(&mut self, observer: impl IntoEntityObserver<M>) {
        self.tree.subscribe(self.parent, observer);
        self.tree.refire::<(C,)>(self.parent);
    }
    /// [`Children::react`] over a component SET: the observer watches
    /// `Trigger<Insert, (A, B)>` and fires when ANY member is written -- one registration, one
    /// body, for state derived from several inputs (Button's style + engagement). The build-time
    /// re-fire inserts only the first present member: one fire is enough because reaction bodies
    /// read the current state of everything they depend on.
    pub fn react_any<CS: Refire, M>(&mut self, observer: impl IntoEntityObserver<M>) {
        self.tree.subscribe(self.parent, observer);
        self.tree.refire::<CS>(self.parent);
    }
    /// [`Children::react`] specialized to the pure-copy case: `parent`'s `C` is copied to
    /// `target` verbatim, now and on every write. Anything that is NOT a pure copy (a text
    /// whose width also depends on the value) stays an explicit `react` -- the boundary is
    /// visible on purpose.
    pub fn forward<C: Component + Clone>(&mut self, target: Entity) {
        self.react::<C, _>(
            move |trigger: Trigger<Insert, C>, values: Query<&C>, mut tree: Tree| {
                tree.entity(target)
                    .insert(values.get(trigger.event_target()).unwrap().clone());
            },
        );
    }
    /// Spawns per item. `build` receives the item's index, the item itself, and this `Children`
    /// (so it can call `spawn`/`each` again), and decides everything about how that child is
    /// placed and ordered — row, column, bottom-up, chained-stack, whatever. This only removes
    /// the loop/bookkeeping ceremony, never the placement decision.
    pub fn each<I, F, R>(&mut self, items: I, mut build: F) -> Vec<R>
    where
        I: IntoIterator,
        F: FnMut(usize, I::Item, &mut Self) -> R,
    {
        items
            .into_iter()
            .enumerate()
            .map(|(i, item)| build(i, item, self))
            .collect()
    }
}

/// What `react`/`react_any` re-fire at build time: a tuple of `Clone`-able components
/// (arity 1-4; tuple-only so the impls can't collide with a foreign `Component` impl).
/// Re-inserts only the FIRST present member -- one fire is sufficient because reaction
/// bodies read the current state of everything they depend on, that's the pattern.
pub trait Refire: Send + Sync + 'static {
    /// Re-inserts the entity's current value; true if anything was inserted.
    fn refire(entity: Entity, world: &mut World) -> bool;
}
fn refire_one<C: Component + Clone>(entity: Entity, world: &mut World) -> bool {
    if let Some(current) = world.get::<C>(entity).cloned() {
        world.entity_mut(entity).insert(current);
        true
    } else {
        false
    }
}
macro_rules! impl_refire_tuple {
    ($($t:ident),+) => {
        impl<$($t: Component + Clone),+> Refire for ($($t,)+) {
            fn refire(entity: Entity, world: &mut World) -> bool {
                $( if refire_one::<$t>(entity, world) { return true; } )+
                false
            }
        }
    };
}
impl_refire_tuple!(A);
impl_refire_tuple!(A, B);
impl_refire_tuple!(A, B, C);
impl_refire_tuple!(A, B, C, D);
