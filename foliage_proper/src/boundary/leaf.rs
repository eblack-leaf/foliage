use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;

/// An opaque name for one element.
///
/// Allocated the moment you ask for it and usable immediately -- as a parent, as the target
/// of a write -- even though the element itself comes into existence when the frame's
/// commands are applied.
///
/// A `Leaf` naming something that has since been pruned is inert rather than dangerous: every
/// command targeting it is dropped and every sample of it reads absent. Nothing panics, and a
/// name is never reused, so a stale one cannot silently address whatever grew after it.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Leaf(pub(crate) Entity);

impl Leaf {
    /// A stable number for this element, for logging or as a map key. Not an address --
    /// there is nothing to be done with it but tell two elements apart.
    pub fn id(&self) -> u64 {
        self.0.to_bits()
    }
}

/// Marks an element the app grew itself, as opposed to one foliage spawned underneath it --
/// a text input's caret, a polyline's segments.
///
/// This is what emissions are attributed through: a click landing on a widget's inner panel
/// climbs to the nearest ancestor carrying this, so it is reported against the element the
/// app actually named. A marker rather than a lookup table, so attribution costs a component
/// check on a handful of ancestors instead of a hash of every entity in the world.
#[derive(Component, Copy, Clone, Default)]
pub(crate) struct Grown;

/// What a [`Leaf`] names right now.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Presence {
    /// Named, and the command that grows it has not been applied yet -- the normal state for
    /// the remainder of the frame that grew it. Samples read absent until it lands.
    Planted,
    /// Live in the tree.
    Live,
    /// Pruned, or taken down with an ancestor. Terminal: growing again means a new `Leaf`.
    Withered,
}
