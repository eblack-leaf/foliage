use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hash, Hasher};

/// An opaque name for one element.
///
/// Allocated the moment you ask for it and usable immediately -- as a parent, as the target of a
/// write -- even though the element itself comes into existence when the frame's ops are drained.
///
/// A `Leaf` naming something that has since been pruned is inert rather than dangerous: every op
/// targeting it is dropped and every tap of it reads absent. Nothing panics, and **a name is never
/// reused**, so a stale one cannot come to address whatever grew after it.
///
/// Inside, it is an index and a generation. The index is where the element is held, and goes back
/// to be named again when the element withers; the generation is how many times it has, so a name
/// that came before is never the name that came after.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Leaf {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

impl Hash for Leaf {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.id());
    }
}

/// A set of elements, hashed as the one number an element's name is.
///
/// A name is a generational index and hashes as one, so what the engine keys on elements is hashed
/// in a multiply rather than by a general-purpose hash built to resist inputs chosen against it --
/// which a name, handed out by the engine itself, never is.
pub(crate) type Leaves = HashSet<Leaf, BuildHasherDefault<Fibonacci>>;

/// A hash that leaves a name's index where it is in the low bits and spreads it into the high ones,
/// in one multiply: names numbered together land together in a table, and what a table compares
/// first still differs between them.
///
/// For a set emptied every frame, which is what [`Leaves`] are. A map held across frames is hashed
/// by [`Spread`] instead.
#[derive(Default)]
pub(crate) struct Fibonacci {
    hash: u64,
}

impl Hasher for Fibonacci {
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write_u64(&mut self, value: u64) {
        // The high half is the reciprocal of the golden ratio, which is what spreads the index
        // upward; the low half passes the index through.
        self.hash = value.wrapping_mul(0x9e37_79b9_0000_0001);
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_u64(self.hash.rotate_left(8) ^ u64::from(byte));
        }
    }
}

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

    fn write_u32(&mut self, value: u32) {
        self.write_u64(u64::from(value));
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
        (u64::from(self.generation) << 32) | u64::from(self.index)
    }

    /// Where the element is held, which is what everything kept per element is indexed by.
    pub(crate) fn slot(self) -> usize {
        self.index as usize
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

/// The order the name was allocated in.
///
/// Taken when the [`Leaf`] is handed out rather than when the element is grown, so it is the order
/// `plant` and `branch` were called in and not the order the drain reached them. Monotonic for the
/// life of the run and never reused, which is what lets it settle the elevation tie-break totally
/// and survive a prune of anything allocated earlier.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Growth(pub(crate) u64);
