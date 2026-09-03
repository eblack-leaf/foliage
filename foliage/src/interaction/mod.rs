//! Interaction -- hit-testing, gestures, and focus.
//!
//! # Two questions, not one
//!
//! 1. **What is at this point?** Geometry, answered for every element, always. That is
//!    [`stack`], and it is where the law of this module lives.
//! 2. **Who receives this gesture?** Intent, answered only for elements that asked. That is
//!    [`interactive`](crate::Place::interactive), and it is one bit.
//!
//! Answering both with one mechanism is what made decoration steal taps, because the only way to
//! stop it stealing them was to take it out of the geometry -- which is the same geometry a drag
//! needs to find its scrolling region.
//!
//! # Gestures are claimed, not declared
//!
//! Nothing at spawn time knows what a gesture will turn out to mean. A drag beginning on a slider
//! inside a scrolling column belongs to the slider if it moves along the slider and to the column
//! if it moves down.
//!
//! So a gesture opens **unclaimed**. The target it landed on holds it while it resolves; if it
//! resolves to a drag the target does not take, the target yields and it passes to the nearest
//! scrolling ancestor. Contention runs up the tree and never sideways -- a target contends only
//! with the things containing it, because targeting has already picked one element over every other
//! one at that point.
//!
//! A **tap** is what a gesture that ended without ever resolving to a drag emits. Nothing is
//! cancelled and nothing is retracted, because nothing was issued early: the threshold is not a
//! rule about taking a click back, it is the point at which the kind of the gesture becomes known.

pub(crate) mod focus;
pub(crate) mod input;
pub(crate) mod stack;

use bevy_ecs::component::Component;
use tracing::field::Empty;
use tracing::{debug, trace_span};

use crate::aspen::Property;
use crate::coordinate::{Axes, Axis, Position};
use crate::grove::Grove;
use crate::interaction::input::Input;
use crate::leaf::Leaf;
use crate::view::{self, consumable, range};

/// What an element declared about gestures.
///
/// Two of these state what cannot be derived from anything else, and the other two are the shape of
/// the element as a hand meets it rather than as it is drawn. What is *not* here is any prediction
/// of what a gesture will mean: that is claimed at the time, from the gesture itself.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Gestures {
    /// Declared [`interactive`](crate::Place::interactive): this element receives.
    pub(crate) receives: bool,
    /// Declared [`pass_through`](crate::Place::pass_through): never the top of the stack.
    pub(crate) transparent: bool,
    /// Which drags the element takes. Absent, it takes none -- so it holds a gesture only until
    /// that gesture becomes a drag, and then yields.
    pub(crate) drags: Option<Axes>,
    pub(crate) shape: Shape,
}

/// The shape a hit is tested against inside an element's box.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Shape {
    #[default]
    Box,
    /// The ellipse inscribed in the box, from
    /// [`round_hit_area`](crate::Place::round_hit_area).
    Round,
}

/// How far a gesture travels before it is claimed as a drag, per axis, in logical pixels.
///
/// The two axes compete at different scales, so one number is too eager for one and too reluctant
/// for the other. On touch, scrolling down is the dominant gesture and wants an eager claim; a
/// claim across contends with it and wants a larger threshold, or every attempt to scroll steals
/// into a carousel.
///
/// A **global tuning value, not a per-element flag**. This is input feel, and input feel that
/// varies element to element is what makes an app feel unpredictable. Set it once, before the
/// engine runs:
///
/// ```no_run
/// # use foliage::{Claim, Foliage};
/// let mut foliage = Foliage::new();
/// foliage.tune(Claim {
///     horizontal: 18.0,
///     vertical: 8.0,
/// });
/// ```
///
/// Callers wanting the two the same set them the same.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Claim {
    /// How far across before a drag is claimed as a drag across.
    pub horizontal: f32,
    /// How far down before a drag is claimed as a drag down.
    pub vertical: f32,
}

impl Default for Claim {
    fn default() -> Self {
        Self {
            horizontal: 16.0,
            vertical: 8.0,
        }
    }
}

impl Claim {
    /// Which axis this travel has claimed, or `None` while the gesture could still be a tap.
    ///
    /// A tie goes down, because down is the gesture a hand makes without meaning anything by it.
    fn claimed(&self, travel: Position) -> Option<Axis> {
        let (across, down) = (travel.x.abs(), travel.y.abs());
        if down >= self.vertical && down >= across {
            return Some(Axis::Vertical);
        }
        if across >= self.horizontal && across > down {
            return Some(Axis::Horizontal);
        }
        None
    }
}

/// What a drag has done, as the element holding it reads it.
///
/// `delta` is this frame's movement and nothing else, so an element following a pointer adds it and
/// an element reading an absolute position takes `current`. The travel spent deciding the gesture
/// was a drag at all is not in any `delta`: it is the distance the claim cost, and paying it out
/// afterwards would make everything that claims start with a jump.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Drag {
    /// Where the gesture began.
    pub start: Position,
    /// Where the pointer is now.
    pub current: Position,
    /// How far it moved this frame.
    pub delta: Position,
}

/// The gesture in progress.
pub(crate) struct Gesture {
    /// Where it began, which is where its tap would be.
    at: Position,
    /// Where the pointer is now.
    to: Position,
    /// The element receiving it, while one is. Cleared when a target yields, so nothing that has
    /// let go of a gesture hears any more about it.
    target: Option<Leaf>,
    /// The scrolling ancestors of wherever it landed, innermost first. Fixed at the press: where a
    /// gesture lands is where it looks for a region, whatever it passes over afterwards.
    chain: Vec<Leaf>,
    held: Held,
    /// How far the gesture moved during this frame, reset at the top of every dispatch.
    ///
    /// One frame's worth, because a release velocity is a speed and a speed needs an interval. It
    /// is the movement that was actually *applied*, so the travel spent claiming the gesture is out
    /// of it here for the same reason it is out of every `delta`.
    travelled: Position,
}

/// Who is holding the gesture.
enum Held {
    /// Opened and not yet a drag. The target holds it, and a release now is a tap.
    Resolving,
    /// A drag the target took, because it declared [`drags`](crate::Place::drags) on this axis.
    Target,
    /// A drag a scrolling region took. The index only ever moves outward.
    Region { index: usize, axis: Axis },
    /// A drag on an axis nothing in the chain scrolls. Nothing moves, and nothing will.
    Nobody,
}

/// Step 2. The frame's input, resolved against what the last frame drew.
pub(crate) fn dispatch(grove: &mut Grove) {
    let step = trace_span!("dispatch", inputs = Empty, stack = grove.stack.len());
    let _entered = step.enter();
    let pending = core::mem::take(&mut grove.pointer.pending);
    step.record("inputs", pending.len());
    // A frame's worth of movement is what a release velocity is measured over, so the count starts
    // again here whether or not anything arrives.
    if let Some(gesture) = grove.pointer.gesture.as_mut() {
        gesture.travelled = Position::default();
    }
    for input in pending {
        match input {
            Input::Pressed(at) => pressed(grove, at),
            Input::Moved(to) => moved(grove, to),
            Input::Released(at) => released(grove, at),
            Input::Cancelled => close(grove, false),
            Input::Wheeled { at, delta } => wheeled(grove, at, delta),
        }
    }
}

/// The one read of the stack, and everything that follows from it.
fn pressed(grove: &mut Grove, at: Position) {
    // A press arriving with a gesture still open ends that one first. It was not released, so it
    // earned no tap.
    close(grove, false);
    let mut gesture = Gesture {
        at,
        to: at,
        target: None,
        chain: Vec::new(),
        held: Held::Resolving,
        travelled: Position::default(),
    };
    if let Some(region) = grove.stack.top(at) {
        if region.disabled {
            // Swallowed. A disabled element is present and inert: it takes nothing itself, and it
            // does not pass the gesture on to what is behind it or to a region containing it.
            debug!(leaf = region.leaf.id(), "gesture swallowed");
        } else {
            if region.receives {
                gesture.target = Some(region.leaf);
                grove.drift.engaged.insert(region.leaf);
                debug!(leaf = region.leaf.id(), "engaged");
            }
            gesture.chain = chain(grove, region.leaf);
            // Taking hold of something still coasting stops it where the hand met it. A coast is
            // the reader's own last gesture carrying on, so catching it is how it is meant to end.
            for &region in &gesture.chain {
                grove.coasting.stop(region);
            }
        }
    }
    grove.pointer.gesture = Some(gesture);
}

fn moved(grove: &mut Grove, to: Position) {
    let Some(mut gesture) = grove.pointer.gesture.take() else {
        return;
    };
    let delta = gesture.to.to(to);
    gesture.to = to;
    if matches!(gesture.held, Held::Resolving) {
        match grove.claim.claimed(gesture.at.to(to)) {
            Some(axis) => claim(grove, &mut gesture, axis),
            // Still short of both thresholds, so this is still a gesture that could end as a tap.
            None => {
                grove.pointer.gesture = Some(gesture);
                return;
            }
        }
    }
    gesture.travelled = gesture.travelled.moved(delta);
    apply(grove, &mut gesture, delta);
    grove.pointer.gesture = Some(gesture);
}

fn released(grove: &mut Grove, at: Position) {
    // The last of the movement, which can itself be what claims the gesture: a flick can carry one
    // move and a release and nothing else.
    moved(grove, at);
    close(grove, true);
}

/// The gesture has become a drag. Who takes it is settled here, once.
fn claim(grove: &mut Grove, gesture: &mut Gesture, axis: Axis) {
    if let Some(target) = gesture.target {
        if grove
            .tree
            .gestures(target)
            .drags
            .is_some_and(|axes| axes.covers(axis))
        {
            gesture.held = Held::Target;
            grove.drift.drag_started.insert(target);
            debug!(leaf = target.id(), ?axis, "drag claimed");
            return;
        }
        // The target takes no drag on this axis, so it yields. It is no longer holding the gesture
        // -- which is what makes a button inside a scrolling list behave: press it and it holds,
        // drag and it lets go, so the list scrolls and the button gets no tap.
        grove.drift.disengaged.insert(target);
        debug!(leaf = target.id(), ?axis, "target yielded");
        gesture.target = None;
    }
    gesture.held = match scrolling(grove, &gesture.chain, 0, axis) {
        Some(index) => Held::Region { index, axis },
        None => Held::Nobody,
    };
}

/// This frame's movement, delivered to whoever is holding the gesture.
fn apply(grove: &mut Grove, gesture: &mut Gesture, delta: Position) {
    match gesture.held {
        Held::Resolving | Held::Nobody => {}
        Held::Target => {
            let Some(target) = gesture.target else {
                return;
            };
            grove.drift.dragged(
                target,
                Drag {
                    start: gesture.at,
                    current: gesture.to,
                    delta,
                },
            );
        }
        Held::Region { index, axis } => {
            // Content moves against the pointer: dragging toward the near edge carries the content
            // that way, which is the region moving further into its own extent.
            let wanted = -delta.along(axis);
            // A move that went nowhere on this axis is not a region declining to move: nothing was
            // asked of it. Reading it as a refusal is what would hand the claim outward on the
            // release itself, which re-delivers the last position and so always has a delta of
            // zero -- and the region that ends up holding the gesture is the region that is handed
            // the coast.
            if wanted == 0.0 {
                return;
            }
            let mut index = index;
            loop {
                if scroll(grove, gesture.chain[index], axis, wanted) != 0.0 {
                    break;
                }
                // This one can no longer consume, so it yields and the claim passes outward. The
                // outermost region keeps it and moves nothing: a claim never travels back inward,
                // or a drag would hand itself between regions every time it reversed.
                match outward(grove, &gesture.chain, index, axis) {
                    Some(outward) => {
                        debug!(
                            from = gesture.chain[index].id(),
                            to = gesture.chain[outward].id(),
                            "scroll handed outward"
                        );
                        index = outward;
                    }
                    None => break,
                }
            }
            gesture.held = Held::Region { index, axis };
        }
    }
}

/// Ends the open gesture, taking the tap it earned if it earned one and handing on the speed it
/// ended with.
fn close(grove: &mut Grove, released: bool) {
    let Some(gesture) = grove.pointer.gesture.take() else {
        return;
    };
    if released {
        launch(grove, &gesture);
    }
    let Some(target) = gesture.target else {
        return;
    };
    // A tap is what a gesture that ended without ever resolving to a drag emits. A gesture that
    // became a drag is not a tap that was taken back; it was never a tap.
    if released && matches!(gesture.held, Held::Resolving) {
        grove.drift.clicked.insert(target);
        debug!(leaf = target.id(), "tapped");
    }
    grove.drift.disengaged.insert(target);
}

/// Hands the region that was holding the gesture its release velocity, and stops there.
///
/// This is the whole of interaction's part in momentum. The decay, the clamp against the extent and
/// whether reaching an end chains outward or absorbs are the region's, and are `views.md`'s.
///
/// Only a gesture a *region* was holding leaves one. A target that took the drag owns whatever it
/// was doing with it and coasts it itself if it wants to; a gesture that ended still resolving is a
/// tap and has no speed to speak of. A release in a frame nothing moved in leaves no speed either,
/// which is what makes holding still before lifting stop the list rather than fling it.
fn launch(grove: &mut Grove, gesture: &Gesture) {
    let Held::Region { index, axis } = gesture.held else {
        return;
    };
    let elapsed = grove.clock.delta().as_secs_f32();
    if elapsed <= 0.0 {
        return;
    }
    // Content moves against the pointer, so the offset's velocity is the pointer's reversed --
    // the same sign the drag itself was applied with.
    view::launch(
        grove,
        gesture.chain[index],
        axis,
        -gesture.travelled.along(axis) / elapsed,
    );
}

/// A wheel notch: no gesture, no claim, no lifecycle. It moves what is under it, and it is over.
fn wheeled(grove: &mut Grove, at: Position, delta: Position) {
    let Some(region) = grove.stack.top(at) else {
        return;
    };
    if region.disabled {
        return;
    }
    let axis = if delta.x.abs() > delta.y.abs() {
        Axis::Horizontal
    } else {
        Axis::Vertical
    };
    let chain = chain(grove, region.leaf);
    let wanted = -delta.along(axis);
    let mut next = scrolling(grove, &chain, 0, axis);
    while let Some(index) = next {
        if scroll(grove, chain[index], axis, wanted) != 0.0 {
            break;
        }
        next = outward(grove, &chain, index, axis);
    }
}

/// The scrolling ancestors of `from`, itself included, innermost first.
///
/// Targethood is not consulted. A drag anywhere inside a region must scroll it -- on touch that is
/// the only way to scroll at all -- and that has to work on plain decoration, which asked for
/// nothing. So scrolling is structural, and it is not the reason anything opts in.
pub(crate) fn chain(grove: &Grove, from: Leaf) -> Vec<Leaf> {
    let mut chain = Vec::new();
    let mut step = Some(from);
    while let Some(leaf) = step {
        if grove.tree.scrolls(leaf).is_some() {
            chain.push(leaf);
        }
        step = grove.tree.trunk(leaf);
    }
    chain
}

/// The first element of `chain` at or after `from` that scrolls `axis`.
fn scrolling(grove: &Grove, chain: &[Leaf], from: usize, axis: Axis) -> Option<usize> {
    chain
        .iter()
        .enumerate()
        .skip(from)
        .find_map(|(index, leaf)| {
            grove
                .tree
                .scrolls(*leaf)
                .is_some_and(|scroll| scroll.covers(axis))
                .then_some(index)
        })
}

/// Where a gesture goes when the region at `index` can consume no more of it, or `None` where it
/// stops there.
///
/// Chaining is the default and is what lets a drag inside a list keep moving the page once the list
/// is done. A region that declared [`contain`](crate::Scroll::contain) on this axis owns its
/// gesture outright and is where the walk ends -- reaching its bottom and having the whole page
/// lurch is the bug the declaration exists to prevent.
///
/// The same question for a drag and for a coast, asked of the same region at the same place in its
/// own extent, because there is only one answer to it.
pub(crate) fn outward(grove: &Grove, chain: &[Leaf], index: usize, axis: Axis) -> Option<usize> {
    if grove
        .tree
        .scrolls(chain[index])
        .is_some_and(|scroll| scroll.absorbs(axis))
    {
        return None;
    }
    scrolling(grove, chain, index + 1, axis)
}

/// Moves a region by as much of `wanted` as it can still take, and reports how much that was.
fn scroll(grove: &mut Grove, leaf: Leaf, axis: Axis, wanted: f32) -> f32 {
    let range = range(grove.tree.extent(leaf), grove.tree.placed(leaf).area, axis);
    let offset = grove.tree.offset(leaf);
    let taken = consumable(offset.along(axis), range, wanted);
    if taken == 0.0 {
        return 0.0;
    }
    // The region moved, which makes this a write to where it sits: it ends a coast still running on
    // this axis, and cancels a motion moving the same region (F8). The reader wins over both.
    grove.coasting.halt(leaf, axis);
    if grove.aspen.cancel(leaf, Property::Scroll) {
        debug!(leaf = leaf.id(), "tween cancelled");
    }
    grove.tree.set_offset(leaf, offset.set(axis, offset.along(axis) + taken));
    taken
}
