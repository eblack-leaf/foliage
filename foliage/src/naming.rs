//! Where every name the engine hands out comes from.
//!
//! One source, drawn from by both sides of the boundary: a name taken inside a frame and one taken
//! from a [`Sprig`](crate::Sprig) on another thread can never collide, and the order they were asked
//! for is the order they were asked for. Every counter here is atomic for that reason -- naming takes
//! `&self` wherever it happens, and an op issued off the frame is ordered against the frame's own by
//! nothing but when it arrived.
//!
//! What is named here is exactly what can be named before the thing exists, which is everything a
//! name is handed out for: an element, a channel, a group, a picture, a face, a mark. Each registry
//! holds what has been filled and grows to meet a name it has not seen, so nothing has to reach one
//! to be given a name for it.
//!
//! Every counter is monotonic and **nothing is ever handed out twice**, which is what makes a stale
//! handle inert rather than dangerous: there is no later thing for it to come to address. A
//! generation would be what made reuse safe, and so there is none to carry -- except on an element,
//! where the index behind a [`Leaf`] does return to the allocator when it withers, and the
//! generation `bevy_ecs` keeps on it is what tells the two apart.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use bevy_ecs::entity::RemoteAllocator;

use crate::aspen::{Sequence, Tween};
use crate::icon::Field;
use crate::image::Plate;
use crate::leaf::{Growth, Leaf};
use crate::text::Font;

/// The one source of names, shared by the frame and by every [`Sprig`](crate::Sprig).
#[derive(Clone)]
pub(crate) struct Naming(Arc<Names>);

struct Names {
    /// The world's own entity allocator, owned rather than borrowed from it -- which is what lets a
    /// name be taken where the world cannot be reached.
    entities: RemoteAllocator,
    growth: AtomicU64,
    tweens: AtomicU64,
    sequences: AtomicU64,
    plates: AtomicU32,
    /// Starts past the bundled face, which is [`Font::DEFAULT`] and is registered before anything
    /// can ask for a name.
    faces: AtomicU32,
    marks: AtomicU32,
}

impl Naming {
    pub(crate) fn new(entities: RemoteAllocator) -> Self {
        Self(Arc::new(Names {
            entities,
            growth: AtomicU64::new(0),
            tweens: AtomicU64::new(0),
            sequences: AtomicU64::new(0),
            plates: AtomicU32::new(0),
            faces: AtomicU32::new(Font::DEFAULT.0 + 1),
            marks: AtomicU32::new(0),
        }))
    }

    /// A name for one element, and its place in allocation order.
    ///
    /// Both are taken here rather than at the drain, so the order is the order `plant` and `branch`
    /// were called in and not the order the drain reached them. That order settles the elevation
    /// tie-break, and it is total across the boundary because there is one counter rather than one
    /// per side.
    pub(crate) fn leaf(&self) -> (Leaf, Growth) {
        let leaf = Leaf(self.0.entities.alloc());
        (leaf, Growth(self.0.growth.fetch_add(1, Ordering::Relaxed)))
    }

    /// A name for one channel. Never reused, so a stale one is inert.
    pub(crate) fn tween(&self) -> Tween {
        Tween(self.0.tweens.fetch_add(1, Ordering::Relaxed))
    }

    /// A name for one group.
    pub(crate) fn sequence(&self) -> Sequence {
        Sequence(self.0.sequences.fetch_add(1, Ordering::Relaxed))
    }

    /// A name for one picture, whose pixels may not exist yet.
    ///
    /// Taken from a counter rather than from the registry that holds pictures, because a decode that
    /// finishes on another thread names what it decoded where the registry is out of reach. The
    /// registry grows to meet the name when it is filled.
    pub(crate) fn plate(&self) -> Plate {
        Plate(self.0.plates.fetch_add(1, Ordering::Relaxed))
    }

    /// A name for one typeface, whose bytes may not be parsed yet.
    ///
    /// Never [`Font::DEFAULT`]: the bundled face holds that name for the life of the run, which is
    /// what lets a face that has not arrived be measured as one.
    pub(crate) fn face(&self) -> Font {
        Font(self.0.faces.fetch_add(1, Ordering::Relaxed))
    }

    /// A name for one mark, whose field may not have arrived yet.
    pub(crate) fn mark(&self) -> Field {
        Field(self.0.marks.fetch_add(1, Ordering::Relaxed))
    }
}
