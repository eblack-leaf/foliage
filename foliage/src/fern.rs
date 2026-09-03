//! Fern -- the frame.
//!
//! The one sequence every other subsystem is ordered by. There is a single [`run`], called by the
//! platform's loop and by the headless suite, so both answer with the same code.

use tracing::{debug, trace_span};

use crate::aspen::{self, Property};
use crate::elm;
use crate::grove::Grove;
use crate::interaction;
use crate::layout::Layout;
use crate::leaf::Leaf;
use crate::op::{Bud, Op};
use crate::place::Caller;
use crate::pollen::Pollen;
use crate::root::Rooted;
use crate::rowan;

/// One frame, steps 1 through 8. Drawing belongs to the caller.
///
/// ```text
/// 1  intake    window and input events -> input state. The clock is sampled once, here
/// 2  dispatch  hit-test against what was drawn last frame -> Pollen
/// 3  root      the app reads settled state and Pollen, and queues ops
/// 4  drain     the single apply -- every queued op, FIFO by arrival, whatever its origin
/// 5  animate   tweens advance
/// 6  resolve   grid -> location -> section -> extent -> scroll -> clip -> rank
/// 7  settle    the inherited products, the box stack, focus
/// 8  extract   changed state -> render instances
/// 9  draw
/// ```
pub(crate) fn run(grove: &mut Grove, app: Option<&mut (dyn Rooted + '_)>) {
    grove.frames += 1;
    let _frame = trace_span!("frame", n = grove.frames).entered();
    grove.again = false;
    intake(grove);
    interaction::dispatch(grove);
    root(grove, app);
    drain(grove);
    aspen::run(grove);
    rowan::run(grove);
    elm::run(grove);
}

/// Step 1. Window and input events become input state, and the clock is fixed for the frame.
fn intake(grove: &mut Grove) {
    let _step = trace_span!("intake").entered();
    grove.clock.sample();
    if let Some(viewport) = grove.pending_resize.take() {
        grove.viewport = viewport;
        grove.drift.resized = Some(viewport);
        debug!(width = viewport.width, height = viewport.height, "resized");
        breakpoints(grove);
    }
}

/// The responsive state the whole tree is read against, re-derived from the viewport.
fn breakpoints(grove: &mut Grove) {
    let layout = Layout::of(grove.viewport);
    let short = grove.short.next(grove.viewport);
    if layout != grove.layout {
        debug!(from = ?grove.layout, to = ?layout, "breakpoint");
        grove.layout = layout;
    }
    if short != grove.short {
        debug!(from = ?grove.short, to = ?short, "short");
        grove.short = short;
    }
}

/// Step 3. The app reads settled state and this frame's [`Pollen`], and queues ops.
fn root(grove: &mut Grove, app: Option<&mut (dyn Rooted + '_)>) {
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
            Op::Plant { leaf, growth, bud } => {
                refuse_sown_cycle(grove, leaf, &bud);
                if grove.tree.grow(leaf, growth, None, bud) {
                    debug!(leaf = leaf.id(), "planted");
                } else {
                    dropped("plant", leaf, "name is already grown");
                }
            }
            Op::Branch {
                leaf,
                growth,
                under,
                bud,
            } => {
                refuse_sown_cycle(grove, leaf, &bud);
                if !grove.tree.is_live(under) {
                    dropped("branch", leaf, "trunk is not live");
                } else if grove.tree.grow(leaf, growth, Some(under), bud) {
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
                grove.aspen.wither(&gone);
                grove.drift.withered.extend(gone);
            }
            Op::Place { leaf, location } => {
                if !grove.tree.is_live(leaf) {
                    dropped("at", leaf, "not live");
                    continue;
                }
                cancel(grove, "at", leaf, Property::Location);
                grove.tree.set_location(leaf, location);
                debug!(leaf = leaf.id(), "placed");
            }
            Op::Divide { leaf, grid } => {
                if !grove.tree.is_live(leaf) {
                    dropped("grid", leaf, "not live");
                    continue;
                }
                grove.tree.set_grid(leaf, grid);
                debug!(leaf = leaf.id(), "divided");
            }
            Op::Anchor { leaf, to, at } => {
                if !grove.tree.is_live(leaf) {
                    dropped("anchor", leaf, "not live");
                    continue;
                }
                if !grove.tree.is_live(to) {
                    dropped("anchor", leaf, "target is not live");
                    continue;
                }
                refuse_cycle(grove, leaf, to, at, None);
                grove.tree.set_anchor(leaf, to, at);
                debug!(leaf = leaf.id(), to = to.id(), "anchored");
            }
            Op::Elevate { leaf, elevation } => {
                if !grove.tree.is_live(leaf) {
                    dropped("elevate", leaf, "not live");
                    continue;
                }
                grove.tree.set_elevation(leaf, elevation);
                debug!(leaf = leaf.id(), "elevated");
            }
            Op::Recolor { leaf, fill } => {
                if !grove.tree.is_live(leaf) {
                    dropped("color", leaf, "not live");
                    continue;
                }
                cancel(grove, "color", leaf, Property::Fill);
                if grove.tree.set_fill(leaf, fill) {
                    debug!(leaf = leaf.id(), "recolored");
                } else {
                    dropped("color", leaf, "draws nothing to fill");
                }
            }
            Op::Round { leaf, rounding } => {
                if !grove.tree.is_live(leaf) {
                    dropped("round", leaf, "not live");
                    continue;
                }
                if grove.tree.set_rounding(leaf, rounding) {
                    debug!(leaf = leaf.id(), "rounded");
                } else {
                    dropped("round", leaf, "draws nothing to round");
                }
            }
            Op::Disable { leaf, disabled } => {
                if !grove.tree.is_live(leaf) {
                    dropped(
                        if disabled { "disable" } else { "enable" },
                        leaf,
                        "not live",
                    );
                    continue;
                }
                grove.tree.set_disabled(leaf, disabled);
                debug!(leaf = leaf.id(), disabled, "disabled");
            }
            Op::Reveal { leaf, visible } => {
                if !grove.tree.is_live(leaf) {
                    dropped("visible", leaf, "not live");
                    continue;
                }
                grove.tree.set_visible(leaf, visible);
                debug!(leaf = leaf.id(), visible, "revealed");
            }
            Op::Fade { leaf, opacity } => {
                if !grove.tree.is_live(leaf) {
                    dropped("opacity", leaf, "not live");
                    continue;
                }
                cancel(grove, "opacity", leaf, Property::Opacity);
                grove.tree.set_opacity(leaf, opacity);
                debug!(leaf = leaf.id(), opacity, "faded");
            }
            Op::Animate {
                leaf,
                motion,
                timing,
            } => {
                if !grove.tree.is_live(leaf) {
                    dropped("animate", leaf, "not live");
                    continue;
                }
                if aspen::animate(grove, leaf, motion, timing) {
                    debug!(leaf = leaf.id(), "animating");
                } else {
                    dropped("animate", leaf, "draws nothing to fill");
                }
            }
            Op::Channel {
                tween,
                from,
                to,
                timing,
            } => {
                grove.aspen.channel(tween, from, to, timing);
                debug!("tweening");
            }
            Op::Stop(tween) => {
                if grove.aspen.stop(tween) {
                    debug!("tween stopped");
                }
            }
            // Not answered here. Where focus can go depends on geometry and on the inherited state
            // this frame has yet to resolve, so the ask is recorded and settled at step 7 -- which
            // is what lets an app open a drawer and focus into it in one frame.
            Op::Focus(intent) => grove.focus.ask(intent),
            Op::Repaint(scheme) => {
                grove.scheme = scheme;
                debug!("repainted");
            }
        }
    }
}

/// The spawn-time half of the same refusal, for an element described with
/// [`anchored`](crate::Place::anchored).
fn refuse_sown_cycle(grove: &Grove, leaf: Leaf, bud: &Bud) {
    if let Some(anchored) = &bud.placement.anchor {
        refuse_cycle(grove, leaf, anchored.to, anchored.at, Some(bud.at));
    }
}

/// Refuses an anchor that would close a cycle, naming both ends and the write that made it.
///
/// A ↔ B is a contradiction rather than a scheduling problem, so running more passes would defer it
/// rather than resolve it. Refusing it here leaves the tree acyclic at all times, which is what
/// lets resolution order by dependency without any cycle handling of its own. Hiding the element
/// instead would be absence with extra steps: a placement that cannot resolve has no box, so there
/// is no state to fall back to, and it would swallow a mistake that has no correct recovery.
///
/// `planted` names where the element being anchored was written, for the case where it does not
/// exist yet to be asked.
fn refuse_cycle(grove: &Grove, leaf: Leaf, to: Leaf, at: Caller, planted: Option<Caller>) {
    if !grove.tree.reaches(to, leaf) {
        return;
    }
    let written = |leaf: Leaf, known: Option<Caller>| match known.or(grove.tree.spawned_at(leaf)) {
        Some(caller) => caller.to_string(),
        None => "an unknown callsite".to_string(),
    };
    panic!(
        "anchor cycle: leaf {leaf} cannot anchor to leaf {to}, which already reaches back to it\n  \
         leaf {leaf} was planted at {}\n  \
         leaf {to} was planted at {}\n  \
         the anchor was written at {at}",
        written(leaf, planted),
        written(to, None),
        leaf = leaf.id(),
        to = to.id(),
    );
}

/// F8. A direct write cancels any motion on the property it writes.
///
/// The drain runs before `animate`, so by the time a tween would be advanced no property has both a
/// pending write and a running one. That is what makes the old advisory rule -- "if a property is
/// animated anywhere, animate it everywhere" -- structural, and gone.
fn cancel(grove: &mut Grove, verb: &'static str, leaf: Leaf, property: Property) {
    if grove.aspen.cancel(leaf, property) {
        debug!(verb, leaf = leaf.id(), "tween cancelled");
    }
}

fn dropped(verb: &'static str, leaf: Leaf, reason: &'static str) {
    debug!(verb, leaf = leaf.id(), reason, "op dropped");
}
