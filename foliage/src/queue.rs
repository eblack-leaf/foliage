use crate::op::Op;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

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

/// What is called to rouse the platform's loop when an op arrives from outside a frame.
///
/// The queue is shared and an op may be pushed into it from anywhere, but a loop that sleeps until
/// the platform speaks will never run the frame that drains one: a retrieval finishing on a thread
/// arrives with nothing to wake it. So an arrival pushes its op and then calls this.
///
/// Installed by `photosynthesize` and absent everywhere else -- the headless suite runs frames by
/// hand, and a frame it did not ask for is not a frame it could observe.
///
/// **Shared rather than copied.** A handle taken before the loop is running is one that would
/// otherwise hold an empty wake for the rest of the run, and a [`Sprig`](crate::Sprig) handed out at
/// boot is exactly that: the whole reason it can push an op is that the frame it needs will be run.
#[derive(Clone, Default)]
pub(crate) struct Wake(Arc<OnceLock<Arc<Rouse>>>);

/// A wake is called from wherever the arrival happened, which on every target with threads means
/// from another one. On the web there are none, and requiring `Send` there would only be a bound
/// no caller could satisfy.
#[cfg(not(target_family = "wasm"))]
type Rouse = dyn Fn() + Send + Sync + 'static;
#[cfg(target_family = "wasm")]
type Rouse = dyn Fn() + 'static;

impl Wake {
    /// Installs the one wake, which is the loop's. A second is dropped: there is one loop.
    #[cfg(not(target_family = "wasm"))]
    pub(crate) fn install(&self, rouse: impl Fn() + Send + Sync + 'static) {
        let _ = self.0.set(Arc::new(rouse));
    }

    #[cfg(target_family = "wasm")]
    pub(crate) fn install(&self, rouse: impl Fn() + 'static) {
        let _ = self.0.set(Arc::new(rouse));
    }

    /// Asks for the frame that will drain what was just queued. Nothing, where no loop is running.
    pub(crate) fn rouse(&self) {
        if let Some(rouse) = self.0.get() {
            rouse();
        }
    }
}
