use crate::boundary::leaf::Leaf;
use crate::boundary::op::Op;
use crate::boundary::bloom::Bloom;
use crate::coordinate::section::Section;
use crate::{Layout, Logical, TimeDelta};
use bevy_ecs::entity::RemoteAllocator;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// A piece of the tree you can carry off the main thread.
///
/// `Send + Clone`, so a background thread -- or your own ECS, at whatever version you like,
/// since nothing but plain data crosses -- can grow and change elements without ever touching
/// the engine's world. Carries the same verbs as [`Canopy`](crate::Canopy) and no way to
/// sample: reads happen at the frame callsite, where the engine is quiescent.
///
/// Emissions come back the same way: [`blooms`](Sprig::blooms) hands over everything the tree
/// has reported since it was last called, so a worker can act on a click on an element it grew
/// without the [`Root`](crate::Root) having to relay it. Nothing is collected until the first
/// call, so a handle that only ever pushes never accumulates one.
///
/// Everything queued here is applied at the top of the next frame, ahead of that frame's own
/// commands, in the order it arrived.
#[derive(Clone)]
pub struct Sprig {
    queue: Arc<Mutex<Vec<Op>>>,
    /// What the frame has reported and this side has not taken yet. Shared by every clone,
    /// exactly like `queue` is: handles are the same handle, so they read the one stream
    /// rather than each getting a copy of it.
    inbox: Arc<Mutex<Vec<Bloom>>>,
    /// Set by the first [`blooms`](Sprig::blooms) call. Until then the frame has nowhere to
    /// deliver and skips the clone entirely -- which is what keeps a worker that only pushes
    /// from growing an inbox it will never read.
    listening: Arc<AtomicBool>,
    /// Last frame's ambient state. A fixed handful of `Copy` values, so unlike the inbox this
    /// is written whether or not anyone is reading -- there is nothing to gate.
    conditions: Arc<Mutex<Option<Conditions>>>,
    /// The same allocator the root uses, so a name minted here can never collide
    /// with one minted there. Owned rather than borrowed from the world -- which is what
    /// lets this be `Send` at all.
    allocator: RemoteAllocator,
}

impl Sprig {
    pub(crate) fn new(allocator: RemoteAllocator) -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            inbox: Arc::new(Mutex::new(Vec::new())),
            listening: Arc::new(AtomicBool::new(false)),
            conditions: Arc::new(Mutex::new(None)),
            allocator,
        }
    }
    /// The ambient state as of the last frame, or `None` before the first one has run.
    ///
    /// The one set of reads that needs no watching: it is neither large nor per-element, so
    /// the frame publishes it outright. Taken together from a single frame, so the viewport
    /// and the breakpoint in hand always agree with each other.
    pub fn conditions(&self) -> Option<Conditions> {
        *self
            .conditions
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
    /// Hands over this frame's ambient state.
    pub(crate) fn publish(&self, conditions: Conditions) {
        *self
            .conditions
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(conditions);
    }
    /// Everything the tree has reported since this was last called, oldest first.
    ///
    /// The same emissions the [`Root`](crate::Root) is handed, delivered as the frame collects
    /// them -- so a worker sees a click on an element it grew one frame after the frame did,
    /// and reads the queue on its own schedule rather than the engine's. Empty until the frame
    /// after the first call, which is what arms delivery.
    ///
    /// A [`Leaf`] named here may already have withered by the time this thread acts on it,
    /// which is safe: every command naming a withered `Leaf` is a no-op.
    pub fn blooms(&self) -> Vec<Bloom> {
        self.listening.store(true, Ordering::Relaxed);
        let mut inbox = self.inbox.lock().unwrap_or_else(|e| e.into_inner());
        core::mem::take(&mut *inbox)
    }
    /// Hands this frame's emissions to whoever is listening. Costs a clone only once someone
    /// actually is.
    pub(crate) fn deliver(&self, blooms: &[Bloom]) {
        if blooms.is_empty() || !self.listening.load(Ordering::Relaxed) {
            return;
        }
        let mut inbox = self.inbox.lock().unwrap_or_else(|e| e.into_inner());
        inbox.extend_from_slice(blooms);
    }
    pub(crate) fn allocator(&self) -> RemoteAllocator {
        self.allocator.clone()
    }
    /// Moves everything queued since the last drain into `into`.
    pub(crate) fn drain_into(&self, into: &mut Vec<Op>) {
        let mut queue = self.queue.lock().unwrap_or_else(|e| e.into_inner());
        into.append(&mut queue);
    }
}

/// What is true of the tree as a whole, for a thread that cannot sample it.
///
/// The same values [`Canopy`](crate::Canopy) answers for the viewport, the breakpoint and the
/// clock -- which is what a worker positioning anything needs before it can state a location
/// in something other than hardcoded pixels.
#[derive(Copy, Clone, Debug)]
pub struct Conditions {
    /// The visible area.
    pub viewport: Section<Logical>,
    /// The current breakpoint.
    pub layout: Layout,
    /// Whether the viewport is vertically cramped.
    pub short: bool,
    /// Device pixels per logical pixel.
    pub scale_factor: f32,
    /// How long the last frame took.
    pub frame_time: TimeDelta,
}

impl crate::boundary::verbs::Queues for Sprig {
    fn push(&mut self, op: Op) {
        // A poisoned queue means another thread panicked mid-push. The ops already in it are
        // still well-formed, so draining them is better than propagating the panic into the
        // event loop.
        let mut queue = self.queue.lock().unwrap_or_else(|e| e.into_inner());
        queue.push(op);
    }
    fn allocate(&self) -> Leaf {
        Leaf(self.allocator.alloc())
    }
}
