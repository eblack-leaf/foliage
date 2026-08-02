use crate::boundary::leaf::Leaf;
use crate::boundary::op::Op;
use bevy_ecs::entity::RemoteAllocator;
use std::sync::Arc;
use std::sync::Mutex;

/// A piece of the tree you can carry off the main thread.
///
/// `Send + Clone`, so a background thread -- or your own ECS, at whatever version you like,
/// since nothing but plain data crosses -- can grow and change elements without ever touching
/// the engine's world. Carries the same verbs as [`Canopy`](crate::Canopy) and no way to
/// sample: reads happen at the frame callsite, where the engine is quiescent.
///
/// Everything queued here is applied at the top of the next frame, ahead of that frame's own
/// commands, in the order it arrived.
#[derive(Clone)]
pub struct Sprig {
    queue: Arc<Mutex<Vec<Op>>>,
    /// The same allocator the frame closure uses, so a name minted here can never collide
    /// with one minted there. Owned rather than borrowed from the world -- which is what
    /// lets this be `Send` at all.
    allocator: RemoteAllocator,
}

impl Sprig {
    pub(crate) fn new(allocator: RemoteAllocator) -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            allocator,
        }
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
