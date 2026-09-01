//! Fern -- the frame.
//!
//! The one sequence every other subsystem is ordered by. There is a single [`run`], called by the
//! platform's loop and by the headless suite, so both answer with the same code.

use tracing::{debug, trace_span};

use crate::grove::Grove;
use crate::leaf::Leaf;
use crate::op::Op;
use crate::pollen::Pollen;
use crate::root::Rooted;

/// One frame, steps 1 through 8. Drawing belongs to the caller.
///
/// ```text
/// 1  intake    window and input events -> input state. The clock is sampled once, here
/// 2  dispatch  hit-test against what was drawn last frame -> Pollen
/// 3  root      the app reads settled state and Pollen, and queues ops
/// 4  drain     the single apply -- every queued op, FIFO by arrival, whatever its origin
/// 5  animate   tweens advance
/// 6  resolve   grid -> location -> section -> clip -> elevation rank
/// 7  settle    visibility, opacity, view extents
/// 8  extract   changed state -> render instances
/// 9  draw
/// ```
pub(crate) fn run(grove: &mut Grove, app: Option<&mut dyn Rooted>) {
    grove.frames += 1;
    let _frame = trace_span!("frame", n = grove.frames).entered();
    grove.again = false;
    intake(grove);
    root(grove, app);
    drain(grove);
}

/// Step 1. Window and input events become input state, and the clock is fixed for the frame.
fn intake(grove: &mut Grove) {
    let _step = trace_span!("intake").entered();
    grove.clock.sample();
    if let Some(viewport) = grove.pending_resize.take() {
        grove.viewport = viewport;
        grove.drift.resized = Some(viewport);
        debug!(width = viewport.width, height = viewport.height, "resized");
    }
}

/// Step 3. The app reads settled state and this frame's [`Pollen`], and queues ops.
fn root(grove: &mut Grove, app: Option<&mut dyn Rooted>) {
    let _step = trace_span!("root").entered();
    let pollen = Pollen::seal(core::mem::take(&mut grove.drift));
    if let Some(app) = app {
        app.frame(grove, pollen);
    }
}

/// Step 4. The single apply: every queued op, in arrival order, whatever its origin.
///
/// An op naming something that has withered is dropped silently, and reported to the trace.
fn drain(grove: &mut Grove) {
    let ops = grove.queue.take();
    let _step = trace_span!("drain", ops = ops.len()).entered();
    for op in ops {
        match op {
            Op::Plant { leaf, bud } => {
                if grove.tree.grow(leaf, None, bud) {
                    debug!(leaf = leaf.id(), "planted");
                } else {
                    dropped("plant", leaf, "name is already grown");
                }
            }
            Op::Branch { leaf, under, bud } => {
                if !grove.tree.is_live(under) {
                    dropped("branch", leaf, "trunk is not live");
                } else if grove.tree.grow(leaf, Some(under), bud) {
                    debug!(leaf = leaf.id(), under = under.id(), "branched");
                } else {
                    dropped("branch", leaf, "name is already grown");
                }
            }
            Op::Prune(leaf) => {
                if !grove.tree.is_live(leaf) {
                    dropped("prune", leaf, "not live");
                    continue;
                }
                let gone = grove.tree.wither(leaf);
                debug!(leaf = leaf.id(), withered = gone.len(), "pruned");
                grove.drift.withered.extend(gone);
            }
        }
    }
}

fn dropped(verb: &'static str, leaf: Leaf, reason: &'static str) {
    debug!(verb, leaf = leaf.id(), reason, "op dropped");
}
