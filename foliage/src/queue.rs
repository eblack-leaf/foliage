use crate::op::Op;
use std::sync::{Arc, Mutex, MutexGuard};

/// The one op queue.
///
/// Every op lands here whichever side of the boundary issued it, and its position is fixed by
/// when it arrived and by nothing else.
#[derive(Clone, Default)]
pub(crate) struct Queue(Arc<Mutex<Vec<Op>>>);

impl Queue {
    pub(crate) fn push(&self, op: Op) {
        self.lock().push(op);
    }

    /// Everything queued since the last drain, in arrival order.
    pub(crate) fn take(&self) -> Vec<Op> {
        core::mem::take(&mut *self.lock())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.lock().is_empty()
    }

    fn lock(&self) -> MutexGuard<'_, Vec<Op>> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
