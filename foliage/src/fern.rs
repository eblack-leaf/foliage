//! Fern -- the frame.
//!
//! The one sequence every other subsystem is ordered by. There is a single [`run`], called by the
//! platform's loop and by the headless suite, so both answer with the same code.

use tracing::{debug, trace_span};

use crate::aspen::{self, Motion, Property};
use crate::elm;
use crate::grove::Grove;
use crate::interaction;
use crate::interaction::focus;
use crate::layout::Layout;
use crate::leaf::Leaf;
use crate::frond::{self, Sprouts};
use crate::op::{Bud, Op};
use crate::place::Caller;
use crate::pollen::Pollen;
use crate::root::Rooted;
use crate::rowan;
use crate::text_input;
use crate::view::ScrollTo;

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
    // What this frame's gestures meant to the leaves that are divided. After dispatch, because that
    // is where they are reported; before the app, because what it queues is written afterwards and
    // has the last word.
    frond::gestured(grove);
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
            Op::Plant {
                leaf,
                growth,
                mut bud,
            } => {
                refuse_sown_cycle(grove, leaf, &bud);
                // Taken before the bud is spent, because a frond's leaflets are grown under the leaf
                // this is about to grow and so cannot be grown until it exists.
                let sprout = bud.sprout.take();
                if grove.tree.grow(leaf, growth, None, bud) {
                    sprouted(grove, leaf, sprout);
                    debug!(leaf = leaf.id(), "planted");
                } else {
                    dropped("plant", leaf, "name is already grown");
                }
            }
            Op::Branch {
                leaf,
                growth,
                under,
                mut bud,
            } => {
                refuse_sown_cycle(grove, leaf, &bud);
                let sprout = bud.sprout.take();
                if !grove.tree.is_live(under) {
                    dropped("branch", leaf, "trunk is not live");
                } else if grove.tree.grow(leaf, growth, Some(under), bud) {
                    sprouted(grove, leaf, sprout);
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
                grove.coasting.wither(&gone);
                grove.drift.withered.extend(gone);
            }
            Op::Place { leaf, location } => {
                if !grove.tree.is_live(leaf) {
                    dropped("at", leaf, "not live");
                    continue;
                }
                cancel(grove, "at", leaf, Property::Location);
                if grove.tree.set_location(leaf, location) {
                    debug!(leaf = leaf.id(), "placed");
                } else {
                    dropped("at", leaf, "is placed by its ends; use `between`");
                }
            }
            Op::Trace { leaf, from, to } => {
                if !grove.tree.is_live(leaf) {
                    dropped("between", leaf, "not live");
                    continue;
                }
                // The same property `at` writes: a stroke's ends and a box are two ways of saying
                // where an element is, so a motion moving one is cancelled by a write to the other.
                cancel(grove, "between", leaf, Property::Location);
                if grove.tree.set_traced(leaf, from, to) {
                    debug!(leaf = leaf.id(), "traced");
                } else {
                    dropped("between", leaf, "is placed by a box; use `at`");
                }
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
            Op::Letter { leaf, value } => {
                if !grove.tree.is_live(leaf) {
                    dropped("text", leaf, "not live");
                    continue;
                }
                // A field is addressed as one element and made of four, so a write reaching one
                // goes to the run that holds its value and takes the caret to the end of what it
                // wrote. Anything else is the run itself.
                match grove.tree.parts(leaf) {
                    Some(parts) => {
                        text_input::lettered(grove, leaf, parts, value);
                        debug!(leaf = leaf.id(), "lettered");
                    }
                    None if grove.tree.set_lettering(leaf, value) => {
                        debug!(leaf = leaf.id(), "lettered");
                    }
                    None => dropped("text", leaf, "says nothing to rewrite"),
                }
            }
            Op::Keyed { target, stroke } => {
                let Some(leaf) = target else {
                    grove.drift.root_keys.push(stroke);
                    debug!(key = ?stroke.key, "keyed with no focus");
                    continue;
                };
                if !grove.tree.is_live(leaf) {
                    dropped("key", leaf, "not live");
                    continue;
                }
                // Reported to whoever holds focus whatever that element is, and then given to the
                // element for whatever it makes of it. An app hears the key it was sent either way:
                // what a seed does with one is not a claim on it.
                grove.drift.keys.entry(leaf).or_default().push(stroke);
                debug!(leaf = leaf.id(), key = ?stroke.key, "keyed");
                text_input::typed(grove, leaf, stroke);
            }
            Op::Select { leaf, range } => {
                if !grove.tree.is_live(leaf) {
                    dropped("select", leaf, "not live");
                    continue;
                }
                text_input::select(grove, leaf, range);
            }
            Op::Tint { leaf, tints } => {
                if !grove.tree.is_live(leaf) {
                    dropped("tint", leaf, "not live");
                    continue;
                }
                if grove.tree.set_tints(leaf, tints) {
                    debug!(leaf = leaf.id(), "tinted");
                } else {
                    dropped("tint", leaf, "says nothing to tint");
                }
            }
            Op::Reshape { leaf, shape } => {
                if !grove.tree.is_live(leaf) {
                    dropped("reshape", leaf, "not live");
                    continue;
                }
                cancel(grove, "reshape", leaf, Property::Shape);
                if grove.tree.set_shape(leaf, shape) {
                    debug!(leaf = leaf.id(), "reshaped");
                } else {
                    dropped("reshape", leaf, "has no shape to reshape");
                }
            }
            // Names no element, like a repaint: a picture belongs to the program rather than to any
            // of the elements drawing it, and every one of them follows when it arrives.
            Op::Load {
                plate,
                pixels,
                size,
            } => {
                if grove.plates.load(plate, &pixels, size) {
                    debug!(plate = plate.0, "loaded");
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
            Op::Scroll { leaf, to } => {
                if !grove.tree.is_live(leaf) {
                    dropped("scroll", leaf, "not live");
                    continue;
                }
                if !reaches(grove, "scroll", leaf, &to) {
                    continue;
                }
                // A direct write to where the region sits: it cancels a motion moving the same
                // region, and ends the coast the reader's last release left running.
                cancel(grove, "scroll", leaf, Property::Scroll);
                grove.coasting.stop(leaf);
                // Answered in R4, against the extent this frame's R3 measures rather than the one
                // the last frame left.
                grove.sought.push((leaf, to));
                debug!(leaf = leaf.id(), "scrolling");
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
                // The one motion whose target is a statement about the element it names rather than
                // a value, so it is checked the same way the verb writing it directly is.
                if let Motion::Scroll(to) = &motion {
                    if !reaches(grove, "animate", leaf, to) {
                        continue;
                    }
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
            // Answered here, in arrival order like every other write. Whether a target can take
            // focus is a walk of what has been declared, which this drain has already applied --
            // so an app can show a drawer and focus into it in the same frame, and focus is final
            // before anything downstream of it resolves.
            Op::Focus(intent) => focus::moved(grove, intent),
            Op::Repaint(scheme) => {
                grove.scheme = scheme;
                debug!("repainted");
            }
        }
    }
    // Anything the drain hid, disabled or pruned may have been holding focus, and none of those
    // writes is obliged to think about it.
    focus::sweep(grove);
    // Focus and every queued write are final, so a divided leaf puts its leaflets in step with
    // state that cannot change again this frame -- as ordinary writes, resolved by ordinary passes.
    frond::settled(grove);
}

/// Grows a [`Frond`](crate::frond)'s leaflets, for a leaf that is divided.
///
/// Here rather than in [`Tree::grow`](crate::tree::Tree::grow) because the leaflets are elements in
/// their own right: they are grown by the same call, in the same drain, with names from the same
/// allocator, and nothing downstream can tell them from anything else grown this frame.
///
/// What they are is [`Sprouts`]'s, not this pass's.
fn sprouted(grove: &mut Grove, leaf: Leaf, sprout: Option<Box<dyn Sprouts>>) {
    if let Some(sprout) = sprout {
        sprout.sprout(grove, leaf);
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

/// Whether a destination has anywhere to move `leaf`, naming the reason where it has not.
///
/// One check for both ways a destination is written -- the verb and the motion -- because both are
/// the same statement about the same region, and a refusal belongs where the write was made rather
/// than three passes later where the answer would simply be missing.
fn reaches(grove: &Grove, verb: &'static str, leaf: Leaf, to: &ScrollTo) -> bool {
    let Some(scroll) = grove.tree.scrolls(leaf) else {
        dropped(verb, leaf, "does not scroll");
        return false;
    };
    if to.over(scroll).is_none() {
        dropped(verb, leaf, "no axis to move; name one with `on`");
        return false;
    }
    match to.names() {
        Some(shown) if !grove.tree.grown_under(shown, leaf) => {
            dropped(verb, leaf, "shows an element not grown under it");
            false
        }
        _ => true,
    }
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
