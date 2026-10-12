use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

use bevy_ecs::component::Component;
use bevy_ecs::entity::{Entity, EntityHash};

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

/// A set of elements, hashed as the one number an element's name is.
///
/// A name is a generational index and hashes as one, so what the engine keys on elements is hashed
/// in a multiply rather than by a general-purpose hash built to resist inputs chosen against it --
/// which a name, handed out by the engine itself, never is.
pub(crate) type Leaves = HashSet<Leaf, EntityHash>;

/// A map keyed on anything that hashes as one name: an element, or what one is held under.
///
/// Hashed by [`Spread`] rather than as [`Leaves`] are. A name's index is dense -- elements grown
/// together are numbered together -- and hashed as itself it puts what was grown together in one
/// unbroken run of the table. That is what a set emptied every frame wants. A map held across
/// frames, whose entries come and go by the hundred thousand as a map's streets do, does not:
/// taking an entry out of a full run leaves a marker rather than a gap, and every lookup for a
/// name not there walks the markers until it finds a gap, further each time entries come and go.
pub(crate) type Named<K, V> = HashMap<K, V, BuildHasherDefault<Spread>>;

/// A hash that mixes every bit of a name into every bit of the hash, so names numbered together
/// land apart: the finaliser of SplitMix64, two multiplies.
#[derive(Default)]
pub(crate) struct Spread {
    hash: u64,
}

impl Hasher for Spread {
    fn finish(&self) -> u64 {
        let mut x = self.hash;
        x ^= x >> 30;
        x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x ^= x >> 27;
        x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
        x ^ (x >> 31)
    }

    fn write_u64(&mut self, value: u64) {
        self.hash = self.hash.rotate_left(29) ^ value;
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_u64(u64::from(byte));
        }
    }
}

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
