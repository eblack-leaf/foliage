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
//! where the index behind a [`Leaf`] does return here when it withers, and the generation it comes
//! back with is what tells the two apart.

use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use crate::aspen::{Sequence, Tween};
use crate::icon::Field;
use crate::image::Plate;
use crate::leaf::{Growth, Leaf};
use crate::text::Font;

/// The one source of names, shared by the frame and by every [`Sprig`](crate::Sprig).
#[derive(Clone)]
pub(crate) struct Naming(Arc<Names>);

struct Names {
    /// One past every index an element has been named by: where a name comes from when none has
    /// gone back.
    fresh: AtomicU32,
    /// The names of elements that withered, each at the generation it is handed out at next. The
    /// last to go back is the first handed out, so what is held per index stays as few indices
    /// long as there have ever been elements at once.
    free: Mutex<Vec<Leaf>>,
    /// How many are in `free`, read without taking it: a frame growing a great deal with nothing
    /// gone back -- the first frame, a page loading -- takes no lock for any of it.
    freed: AtomicUsize,
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
    pub(crate) fn new() -> Self {
        Self(Arc::new(Names {
            fresh: AtomicU32::new(0),
            free: Mutex::new(Vec::new()),
            freed: AtomicUsize::new(0),
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
        let names = &self.0;
        let returned = match names.freed.load(Ordering::Acquire) {
            0 => None,
            _ => {
                let mut free = names.free.lock().unwrap_or_else(PoisonError::into_inner);
                let leaf = free.pop();
                names.freed.store(free.len(), Ordering::Release);
                leaf
            }
        };
        // Nothing went back, or another thread took the last of it between the count and the lock.
        let leaf = returned.unwrap_or_else(|| Leaf {
            index: names.fresh.fetch_add(1, Ordering::Relaxed),
            generation: 0,
        });
        (leaf, Growth(names.growth.fetch_add(1, Ordering::Relaxed)))
    }

    /// Hands the indices of elements that withered back, each at the generation its next name
    /// carries. The frame's alone to call, since only the frame withers anything.
    pub(crate) fn release(&self, leaves: impl IntoIterator<Item = Leaf>) {
        let mut free = self.0.free.lock().unwrap_or_else(PoisonError::into_inner);
        free.extend(leaves);
        self.0.freed.store(free.len(), Ordering::Release);
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
