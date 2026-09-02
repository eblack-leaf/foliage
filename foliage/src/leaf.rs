use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;

/// An opaque name for one element.
///
/// Allocated the moment you ask for it and usable immediately -- as a parent, as the target of a
/// write -- even though the element itself comes into existence when the frame's ops are drained.
///
/// A `Leaf` naming something that has since been pruned is inert rather than dangerous: every op
/// targeting it is dropped and every tap of it reads absent. Nothing panics, and **a name is never
/// reused**, so a stale one cannot come to address whatever grew after it.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Leaf(pub(crate) Entity);

impl Leaf {
    /// A stable number for this element, for logging or as a map key. Not an address -- there is
    /// nothing to be done with it but tell two elements apart.
    pub fn id(&self) -> u64 {
        self.0.to_bits()
    }
}

/// What a [`Leaf`] names right now.
///
/// The three states are ordered and terminal at the end: `Planted` -> `Live` -> `Withered`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Presence {
    /// Named, and the op that grows it has not been drained yet -- the normal state for the
    /// remainder of the frame that planted it. Taps read absent until it lands.
    ///
    /// Also what a name reads as when the op that would have grown it was dropped because its
    /// parent had withered by the time the drain reached it.
    Planted,
    /// Live in the tree.
    Live,
    /// Pruned, or taken down with an ancestor. Terminal: growing again means a new `Leaf`.
    Withered,
}

/// Marks an element the app grew itself, as opposed to one foliage spawned underneath it.
#[derive(Component, Copy, Clone, Default)]
pub(crate) struct Grown;

/// The order the name was allocated in.
///
/// Taken when the [`Leaf`] is handed out rather than when the element is grown, so it is the order
/// `plant` and `branch` were called in and not the order the drain reached them. Monotonic for the
/// life of the run and never reused, which is what lets it settle the elevation tie-break totally
/// and survive a prune of anything allocated earlier.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Growth(pub(crate) u64);

/// Where the element was written into existence.
///
/// What a refusal names, so the panic points at the call that caused it rather than at an entity
/// inside a resolve pass.
#[derive(Component, Copy, Clone, Debug)]
pub(crate) struct SpawnedAt(pub(crate) crate::place::Caller);
