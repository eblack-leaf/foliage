//! Sprig -- a cutting of the tree, carried off the frame.
//!
//! Everything the engine does is done in a frame, on the thread the frame runs on. A `Sprig` is how
//! work that is on neither -- a thread decoding something large, a promise that has resolved, a
//! timer of the host's own -- reaches the tree anyway: it carries [`Grow`](crate::Grow) entire and
//! pushes onto the one queue, so what it writes is drained where every other change is drained and
//! is ordered against them by when it arrived.
//!
//! It registers as well as writes. A name comes from the one allocator and every registry grows to
//! meet one, so a worker that built a face, a field or a picture hands the bytes over as the op a
//! retrieval would have pushed -- which is what makes a thread that *made* an asset and one that
//! *read* one the same case.
//!
//! What it cannot do is **sample**. A read needs the world, and the world belongs to the frame -- so
//! the reads a worker needs are pushed to it instead: [`Conditions`] is what is true of the tree as a
//! whole, published every frame, and [`watch`](Sprig::watch) is how one property of one element joins
//! them.

use core::time::Duration;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use tracing::trace;

use crate::aspen::{Sequence, Tween};
use crate::asset::{Bytes, Destination, Supply, retrieve};
use crate::coordinate::Area;
use crate::grove::Grove;
use crate::icon::Field;
use crate::image::Plate;
use crate::layout::{Layout, Short};
use crate::leaf::{Growth, Leaf};
use crate::naming::Naming;
use crate::op::Op;
use crate::palette::Scheme;
use crate::pollen::Pollen;
use crate::queue::{Queue, Wake};
use crate::text::Font;
use crate::vein::{Sap, Vein};
use crate::verbs::Queues;

/// The engine, reached from off the frame.
///
/// Cloneable and, where the platform has threads, `Send`: hand one to a worker and it can grow,
/// move, fill and take down elements without the frame having to relay anything. It carries
/// [`Grow`](crate::Grow) exactly as [`Grove`] does, so **a change reads identically wherever it is
/// issued** and the same code does the same thing on either side of the boundary.
///
/// ```no_run
/// # use foliage::{Grove, Grow, Stem, Vein};
/// # fn f(grove: &mut Grove) {
/// let mut sprig = grove.sprig();
/// std::thread::spawn(move || {
///     let leaf = sprig.plant(Stem::new());
///     sprig.watch(leaf, Vein::Drawn);
/// });
/// # }
/// ```
///
/// # What is different, and what is not
///
/// An op from here lands in the drain of the frame that was running when it arrived, if it arrived
/// before that frame's drain, and in the next frame's otherwise. That is the whole of the
/// difference, and it is concurrency's rather than the engine's: **nothing about how an op is
/// applied depends on which side it came from.**
///
/// A [`Leaf`] named here may have withered by the time this thread writes to it, which is safe on
/// the same terms it is safe anywhere -- every op naming a withered leaf is dropped.
///
/// # One handle
///
/// Every clone, and every [`Grove::sprig`] the engine hands out, is the same handle: one queue, one
/// stream of reports, one set of watches. Two workers holding one read a single stream between them
/// rather than each being sent a copy of it, which is what keeps the reports finite when nothing is
/// listening and the watches finite when everything is.
#[derive(Clone)]
pub struct Sprig(Arc<Cutting>);

/// What every clone of a handle shares.
struct Cutting {
    queue: Queue,
    /// How the loop is roused when an op arrives from off the frame. Shared with the engine rather
    /// than copied out of it, because a handle is usually taken before there is a loop to rouse.
    wake: Wake,
    naming: Naming,
    /// Last frame's ambient state. Written whether or not anyone is reading, because it is a fixed
    /// handful of `Copy` values -- there is nothing here to gate.
    conditions: Mutex<Option<Conditions>>,
    /// What every standing watch last read. Current rather than a history, so a reader that has not
    /// asked in a while is behind by no more than a frame instead of by everything that happened.
    readings: Mutex<HashMap<(Leaf, Vein), Sap>>,
    /// What the frame has reported and this side has not taken. A history rather than a state,
    /// because a report is about a moment and nothing else will say it happened.
    inbox: Mutex<Vec<Pollen>>,
    /// Set by the first [`pollen`](Sprig::pollen) call. Until then the frame has nowhere to deliver
    /// and does not, which is what keeps a handle that only ever writes from filling an inbox
    /// nothing will read.
    listening: AtomicBool,
}

impl Sprig {
    pub(crate) fn new(queue: Queue, wake: Wake, naming: Naming) -> Self {
        Self(Arc::new(Cutting {
            queue,
            wake,
            naming,
            conditions: Mutex::new(None),
            readings: Mutex::new(HashMap::new()),
            inbox: Mutex::new(Vec::new()),
            listening: AtomicBool::new(false),
        }))
    }

    /// Registers a font and hands back the name elements compose in.
    ///
    /// [`Grove::font`] for a thread that cannot reach the registry: bytes a worker holds, or an
    /// [`Origin`](crate::Origin) to read them from, and the name comes back at once either way. A
    /// face that has yet to land is composed in the bundled one until it does.
    ///
    /// # What differs from the frame's
    ///
    /// **Nothing here panics.** [`Grove::font`] refuses a proportional face outright when it was
    /// given the bytes, because bytes written at a callsite are a statement the program made and the
    /// mistake is worth stopping for. A worker holds bytes it built or was sent rather than ones it
    /// stated, and a panic on a thread ends the thread rather than naming a callsite -- so a face
    /// that turns out to be proportional is reported as [`missing`](crate::Pollen::missing) here, as
    /// a fetched one is. What lands is [`loaded`](crate::Pollen::loaded), on the same terms.
    pub fn font(&mut self, bytes: impl Into<Bytes>) -> Font {
        let font = self.0.naming.face();
        self.supply(Destination::Face(font), bytes.into());
        font
    }

    /// Registers a mark and hands back the name elements draw it by.
    ///
    /// [`Grove::icon`] for a thread that cannot reach the registry, and refusing rather than
    /// asserting for the reason [`font`](Sprig::font) does: a field smaller than it was said to be
    /// is reported as [`missing`](crate::Pollen::missing).
    pub fn icon(&mut self, field: impl Into<Bytes>, side: u32, range: f32) -> Field {
        let mark = self.0.naming.mark();
        self.supply(Destination::Mark(mark, side, range), field.into());
        mark
    }

    /// Registers a picture and hands back the name elements draw it by.
    ///
    /// [`Grove::image`] for a thread that cannot reach the registry: PNG or JPEG, decoded when it is
    /// drained. Pixels the worker made itself are [`pixels`](crate::Grow::pixels), which needs no
    /// decode and states a size because there is nothing to read one from.
    pub fn image(&mut self, bytes: impl Into<Bytes>) -> Plate {
        let plate = self.0.naming.plate();
        self.supply(Destination::Picture(plate), bytes.into());
        plate
    }

    /// One road for both ways bytes are given.
    ///
    /// Bytes a worker holds arrive as the op a read would have pushed, and an
    /// [`Origin`](crate::Origin) is read exactly as the frame reads one -- a retrieval needs the
    /// queue and the wake, and this holds both. So a registration is drained in the frame it
    /// reached, whichever way it was supplied, and a worker that fetches writes the same line as one
    /// that already has the bytes.
    fn supply(&mut self, destination: Destination, bytes: Bytes) {
        match bytes.0 {
            Supply::Held(bytes) => self.queue(Op::Arrived {
                destination,
                bytes: Ok(bytes),
            }),
            Supply::At(origin) => retrieve(&self.0.queue, &self.0.wake, destination, origin),
        }
    }

    /// What is true of the tree as a whole, as of the last frame, or `None` before the first one has
    /// run.
    ///
    /// The reads that need no watching: not per-element and not large, so the frame publishes them
    /// outright. Taken together from one frame, so the viewport and the breakpoint in hand always
    /// agree with each other.
    pub fn conditions(&self) -> Option<Conditions> {
        *self.lock(&self.0.conditions)
    }

    /// Asks to be told what one property of an element is, and told again whenever it changes.
    ///
    /// The read path for a thread that cannot sample: the value is pushed rather than asked for, so
    /// nothing here borrows the world or waits on a frame. Read it with [`tap`](Sprig::tap).
    ///
    /// The first reading is taken at the end of the frame this is drained in, with the value as it
    /// already stands -- so a watch does not have to be seeded by waiting for something to move.
    /// Watching the same property twice is one watch, and it ends when the element withers or
    /// [`unwatch`](Sprig::unwatch) says so.
    pub fn watch(&mut self, leaf: Leaf, vein: Vein) {
        self.queue(Op::Watch { leaf, vein });
    }

    /// Ends a [`watch`](Sprig::watch). Unwatching what was never watched does nothing.
    ///
    /// The reading goes with it: [`tap`](Sprig::tap) answers `None` from the frame this is drained
    /// in, rather than going on answering with a value nothing is keeping current.
    pub fn unwatch(&mut self, leaf: Leaf, vein: Vein) {
        self.queue(Op::Unwatch { leaf, vein });
    }

    /// What a watched property last read, or `None` if it is not watched, has not been read yet, or
    /// the element does not carry it.
    ///
    /// [`Grove::tap`] for a thread with no world to read: the same question, answered from what the
    /// last frame published rather than from the tree. Everything read here comes from a single
    /// frame, so two readings never disagree about which frame they are from.
    ///
    /// A property an element does not carry reads `None` exactly as it does at the frame's own
    /// callsite. A watch that has never been drained reads `None` too, which is the ordinary state
    /// for the moment between asking and the frame that answers.
    pub fn tap(&self, leaf: Leaf, vein: Vein) -> Option<Sap> {
        self.lock(&self.0.readings).get(&(leaf, vein)).cloned()
    }

    /// Every frame's report since this was last called, oldest first.
    ///
    /// The same [`Pollen`] the [`Root`](crate::Root) is handed, delivered as each frame hands it
    /// over -- so a worker hears about a gesture on an element it grew without the app having to
    /// relay it, and reads on its own schedule rather than the engine's.
    ///
    /// **Nothing is collected until the first call**, which is what arms delivery: a handle that
    /// only writes never accumulates a report. So the first call answers with nothing, whatever has
    /// happened, and every call after it answers with the frames since.
    ///
    /// What is reported and what [`tap`](Sprig::tap) reads are the two halves of one frame, arriving
    /// together as they do at the frame's own callsite. A reading is never older than the last
    /// report and may be one frame newer, which is the ordinary condition of reading a running
    /// engine from beside it.
    pub fn pollen(&self) -> Vec<Pollen> {
        self.0.listening.store(true, Ordering::Relaxed);
        core::mem::take(&mut *self.lock(&self.0.inbox))
    }

    /// Hands this frame's ambient state over.
    pub(crate) fn ambient(&self, conditions: Conditions) {
        *self.lock(&self.0.conditions) = Some(conditions);
    }

    /// Where the watched readings are kept, for the frame to bring up to date.
    fn readings(&self) -> MutexGuard<'_, HashMap<(Leaf, Vein), Sap>> {
        self.lock(&self.0.readings)
    }

    /// Drops a reading whose watch has ended.
    pub(crate) fn forget(&self, leaf: Leaf, vein: Vein) {
        self.readings().remove(&(leaf, vein));
    }

    /// Hands this frame's report to whoever is listening. Costs a clone only once someone is.
    pub(crate) fn deliver(&self, pollen: &Pollen) {
        if !self.0.listening.load(Ordering::Relaxed) {
            return;
        }
        self.lock(&self.0.inbox).push(pollen.clone());
    }

    /// A poisoned lock means another thread panicked while holding it. What is behind these is a
    /// value written whole rather than a structure kept in step across calls, so it is well-formed
    /// either way and going on with it is better than propagating the panic into the event loop.
    fn lock<'a, T>(&self, held: &'a Mutex<T>) -> MutexGuard<'a, T> {
        held.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Queues for Sprig {
    fn queue(&mut self, op: Op) {
        self.0.queue.push(op);
        // The frame that will drain it. An op arriving from off the frame is the one kind nothing
        // else announces: the loop is asleep, and the platform has nothing to say about it.
        self.0.wake.rouse();
    }

    fn allocate(&self) -> (Leaf, Growth) {
        self.0.naming.leaf()
    }

    fn name(&self) -> Tween {
        self.0.naming.tween()
    }

    fn group(&self) -> Sequence {
        self.0.naming.sequence()
    }

    fn picture(&mut self) -> Plate {
        self.0.naming.plate()
    }
}

/// What is true of the tree as a whole.
///
/// Exactly what [`Grove`] answers about the frame rather than about an element, for a thread that
/// cannot ask it: what a worker positioning anything needs before it can state a location in
/// something other than hardcoded pixels.
#[derive(Copy, Clone, PartialEq, Debug)]
#[non_exhaustive]
pub struct Conditions {
    /// The visible area.
    pub viewport: Area,
    /// The width breakpoint in force, which every placement is read against.
    pub layout: Layout,
    /// Whether the viewport is vertically cramped.
    pub short: Short,
    /// What every [`Palette`](crate::Palette) role currently resolves to.
    pub scheme: Scheme,
    /// What holds focus, if anything does.
    pub focused: Option<Leaf>,
    /// How long the last frame took.
    pub frame_time: Duration,
    /// Time since the engine was built.
    pub elapsed: Duration,
}

/// Every standing watch, as the frame holds them.
///
/// Engine-side and not shared: what is asked for crosses the boundary as an op and what is read
/// crosses back as a reading, so the set itself never has to be locked.
#[derive(Default)]
pub(crate) struct Watches(HashSet<(Leaf, Vein)>);

impl Watches {
    pub(crate) fn watch(&mut self, leaf: Leaf, vein: Vein) -> bool {
        self.0.insert((leaf, vein))
    }

    pub(crate) fn unwatch(&mut self, leaf: Leaf, vein: Vein) -> bool {
        self.0.remove(&(leaf, vein))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Publishes what a thread with no world to read would otherwise have to ask for.
///
/// After settle, so what is published is what the frame ended at rather than what it passed
/// through, and before extraction, which is the backend's business and not the boundary's.
pub(crate) fn publish(grove: &mut Grove) {
    grove.sprig.ambient(Conditions {
        viewport: grove.viewport,
        layout: grove.layout,
        short: grove.short,
        scheme: grove.scheme,
        focused: grove.focus.held(),
        frame_time: grove.clock.delta(),
        elapsed: grove.clock.elapsed(),
    });
    if grove.watched.is_empty() {
        return;
    }
    // Taken out so the withered ones can be dropped while the rest are read. Put back whole below,
    // so a frame that drops none allocates nothing here.
    let mut watched = core::mem::take(&mut grove.watched);
    {
        let mut readings = grove.sprig.readings();
        watched.0.retain(|&(leaf, vein)| {
            // A name is never handed out twice, so a watch on something that has withered is one
            // nothing can ever answer again.
            if !grove.tree.is_live(leaf) {
                readings.remove(&(leaf, vein));
                return false;
            }
            match grove.tap(leaf, vein) {
                Some(sap) if readings.get(&(leaf, vein)) != Some(&sap) => {
                    trace!(leaf = leaf.id(), vein = ?vein, "read");
                    readings.insert((leaf, vein), sap);
                }
                Some(_) => {}
                // Live, and not carrying the property. The watch stands -- it ends where every other
                // one does, with the element or with `unwatch` -- and there is nothing to publish.
                None => {
                    readings.remove(&(leaf, vein));
                }
            }
            true
        });
    }
    grove.watched = watched;
}
