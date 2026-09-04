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
//!
//! # A press that was held
//!
//! That threshold is a distance, and a distance alone cannot tell a gesture that is sitting still
//! from one that has not moved yet. So resolving has a second way out: held past [`Hold::after`],
//! reported as a gesture fact of its own.
//!
//! ```text
//! opened ──▶ resolving ──▶ claimed ──▶ ended
//!                └──────▶ held ──▶ claimed ──▶ ended
//! ```
//!
//! It is general -- context menus, reorder handles and press-and-hold affordances all want it -- and
//! it is what lets an element scroll like any other and still take a drag, by declaring no drag at
//! all and claiming one only out of a hold.

pub(crate) mod focus;
pub(crate) mod input;
pub(crate) mod stack;

use core::time::Duration;
use std::collections::VecDeque;

use bevy_ecs::component::Component;
use tracing::field::Empty;
use tracing::{debug, trace_span};

use crate::aspen::Property;
use crate::coordinate::{Axes, Axis, Position};
use crate::grove::Grove;
use crate::interaction::focus::Intent;
use crate::interaction::input::{Input, Key, Keystroke};
use crate::leaf::Leaf;
use crate::op::Op;
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

/// How long a press is held before it is a hold rather than a gesture still deciding.
///
/// The lifecycle's other threshold is a distance, so on its own it leaves a gesture that is sitting
/// still indistinguishable from one that has not moved yet. Touch is where that shows: dragging to
/// scroll and dragging to select are the same motion, and what separates them on both platforms is
/// that a plain drag scrolls and selection begins from a press that was **held**.
///
/// A **global tuning value, not a per-element flag**, for the reason [`Claim`] is one. Set it once,
/// before the engine runs:
///
/// ```no_run
/// # use core::time::Duration;
/// # use foliage::{Foliage, Hold};
/// let mut foliage = Foliage::new();
/// foliage.tune(Hold {
///     after: Duration::from_millis(400),
/// });
/// ```
///
/// Not something an element opts into. A hold is part of what a gesture can turn out to be, so it is
/// reported to whatever the press landed on and that element decides whether it means anything --
/// which is the same footing a tap is on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Hold {
    /// How long the press is down, without having become a drag, before it is reported as held.
    pub after: Duration,
}

impl Default for Hold {
    fn default() -> Self {
        Self {
            after: Duration::from_millis(500),
        }
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

/// How far back a release velocity is measured.
///
/// A release lands in one frame, and that frame on its own is a poor sample of the gesture that
/// ended in it: a hand slows as it lifts, a pointer reports nothing at all in the frame the button
/// came up, and a flick can put the whole of its movement into the frame before. Measured over one
/// frame, all three read as a hand that stopped -- and with a minimum speed to clear, reading low is
/// not a coast that starts slow, it is a coast that never starts.
///
/// So the speed handed on is the mean over the last of the gesture rather than the last frame of it.
/// Long enough that a frame or two of nothing does not throw a fling away, short enough that a hand
/// which came to rest before lifting still reads as stopped.
///
/// Not a tuning value: it is how the measurement is taken, not a statement about how a coast feels.
/// [`Momentum`](crate::Momentum) is the half of momentum an app sets.
const WINDOW: Duration = Duration::from_millis(100);

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
    stage: Stage,
    /// When the press landed, on the engine's own clock.
    ///
    /// What a hold is measured from, so what it measures is time the gesture has been open rather
    /// than time anything moved -- a hand that drifted a few pixels and stopped is still resolving,
    /// and is still on its way to a hold.
    since: Duration,
    /// What the gesture did over the last [`WINDOW`], oldest first, which is what a release velocity
    /// is measured from.
    ///
    /// One entry per frame the gesture has been open in, each carrying the interval it stands for,
    /// so the mean is over time rather than over a count of frames. It holds the movement that was
    /// actually *applied*, so the travel spent claiming the gesture is out of it here for the same
    /// reason it is out of every `delta`.
    recent: VecDeque<Sample>,
}

/// One frame's share of a gesture: how long the frame stood for, and how far it moved in it.
struct Sample {
    span: Duration,
    travel: Position,
}

impl Gesture {
    /// Opens this frame's sample and drops what has fallen out of the window.
    fn open(&mut self, span: Duration) {
        // A frame that took no time is no interval to measure a speed over, so it opens nothing and
        // whatever moves in it belongs to the sample already running. The first sample is pushed
        // whatever its span, so a gesture always has somewhere to put its movement.
        if span.is_zero() && !self.recent.is_empty() {
            return;
        }
        self.recent.push_back(Sample {
            span,
            travel: Position::default(),
        });
        // What is kept is the shortest run of frames that still reaches back a whole window, so a
        // window's worth is always covered and never much more. The last sample is never dropped:
        // where one frame is longer than the window, that frame is the whole measurement.
        let mut covered = self.covered();
        while let Some(oldest) = self.recent.front() {
            let without = covered - oldest.span;
            if without < WINDOW {
                break;
            }
            covered = without;
            self.recent.pop_front();
        }
    }

    /// Adds movement to the frame being measured.
    fn record(&mut self, delta: Position) {
        if let Some(sample) = self.recent.back_mut() {
            sample.travel = sample.travel.moved(delta);
        }
    }

    /// How much time the samples reach back over.
    fn covered(&self) -> Duration {
        self.recent.iter().map(|sample| sample.span).sum()
    }

    /// The mean velocity of the pointer over the window, in logical pixels per second, or `None`
    /// where no time has passed to measure one over.
    fn velocity(&self) -> Option<Position> {
        let covered = self.covered().as_secs_f32();
        if covered <= 0.0 {
            return None;
        }
        let travel = self
            .recent
            .iter()
            .fold(Position::default(), |sum, sample| sum.moved(sample.travel));
        Some(Position::new(travel.x / covered, travel.y / covered))
    }

    /// Whether this gesture could still become a hold.
    ///
    /// F9's question about a press that is not moving. A hold is the one thing that happens to a
    /// gesture with nothing arriving to make it happen, so the frames that would notice it are owed
    /// while this is true.
    ///
    /// A gesture that landed on nothing which receives is not one of them. A hold is a fact reported
    /// to whoever is holding the gesture, and where nobody is, there is nothing for it to be a fact
    /// about -- so such a press stays resolving and ends as the tap it would always have been.
    fn awaiting_hold(&self) -> bool {
        matches!(self.stage, Stage::Resolving) && self.target.is_some()
    }
}

/// Where the gesture has reached, and with it who is holding it.
enum Stage {
    /// Opened and not yet a drag. The target holds it, and a release now is a tap.
    Resolving,
    /// Down past [`Hold::after`] without having become a drag. The target still holds it, and a
    /// release now is not a tap -- the press has already turned into something else.
    Held,
    /// A drag the target took, because it declared [`drags`](crate::Place::drags) on this axis or
    /// because it took the hold this drag came out of.
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
    let pending = core::mem::take(&mut grove.incoming.pending);
    step.record("inputs", pending.len());
    // Every frame an open gesture lives through is one the window is measured across, so a frame
    // opens a sample whether or not anything arrives in it: a frame nothing happened in is a hand
    // that held still, and it counts against the mean exactly as much as one that moved.
    let span = grove.clock.delta();
    if let Some(gesture) = grove.incoming.gesture.as_mut() {
        gesture.open(span);
    }
    // Read before this frame's input, because by the time any of that arrived the press had already
    // been down this long. A hold is the transition nothing arrives to make.
    holding(grove);
    for input in pending {
        match input {
            Input::Pressed(at) => pressed(grove, at),
            Input::Moved(to) => moved(grove, to),
            Input::Released(at) => released(grove, at),
            Input::Cancelled => close(grove, false),
            Input::Wheeled { at, delta } => wheeled(grove, at, delta),
            Input::Keyed(key) => keyed(grove, key),
            Input::Modifiers(modifiers) => grove.incoming.modifiers = modifiers,
        }
    }
}

/// The second way out of resolving: a press that has been down long enough to be a statement of its
/// own.
///
/// Read at the top of every frame an open gesture lives through, against the one clock the frame
/// shares. Every other transition is an event -- a move that crossed a threshold, a release, a
/// cancel -- and this is the one that is a duration, so there is nothing to answer it but the frame
/// itself.
///
/// Reported and then done with: the gesture leaves resolving, so it is reported once however long
/// the press goes on.
fn holding(grove: &mut Grove) {
    let (elapsed, after) = (grove.clock.elapsed(), grove.hold.after);
    let Some(gesture) = grove.incoming.gesture.as_mut() else {
        return;
    };
    if !gesture.awaiting_hold() || elapsed.saturating_sub(gesture.since) < after {
        return;
    }
    let Some(target) = gesture.target else {
        return;
    };
    let at = gesture.at;
    gesture.stage = Stage::Held;
    grove.drift.held.insert(target, at);
    debug!(leaf = target.id(), "held");
}

/// A keystroke, delivered to whatever it is about.
///
/// `Tab` and `Escape` are focus's own and are answered wherever focus is, including nowhere --
/// which is what makes keyboard navigation work on a page with no field on it at all. Everything
/// else goes to whatever holds focus, and what an element makes of a key is that element's: this
/// pass knows which keys steer focus and nothing else about any of them.
///
/// Queued rather than applied, so a keystroke is drained in arrival order beside every other change
/// and what it produces is reported on F7's own terms.
fn keyed(grove: &mut Grove, key: Key) {
    let stroke = Keystroke {
        key,
        modifiers: grove.incoming.modifiers,
    };
    if key == Key::Tab {
        grove.queue.push(Op::Focus(match stroke.modifiers.shift {
            true => Intent::Previous,
            false => Intent::Next,
        }));
        debug!(shift = stroke.modifiers.shift, "tabbed");
        return;
    }
    if key == Key::Escape {
        grove.queue.push(Op::Focus(Intent::Away));
        debug!("escaped");
        return;
    }
    let Some(leaf) = grove.focus.held() else {
        return;
    };
    grove.queue.push(Op::Type { leaf, stroke });
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
        stage: Stage::Resolving,
        since: grove.clock.elapsed(),
        recent: VecDeque::new(),
    };
    // Opened here rather than at the top of the next dispatch, because a flick can press, move and
    // release inside one frame and that movement has to land somewhere.
    gesture.open(grove.clock.delta());
    if let Some(region) = grove.stack.top(at) {
        if region.disabled {
            // Swallowed. A disabled element is present and inert: it takes nothing itself, and it
            // does not pass the gesture on to what is behind it or to a region containing it.
            debug!(leaf = region.leaf.id(), "gesture swallowed");
        } else {
            gesture.chain = chain(grove, region.leaf);
            // Taking hold of something still coasting stops it where the hand met it. A coast is
            // the reader's own last gesture carrying on, so catching it is how it is meant to end.
            let caught = gesture.chain.iter().fold(false, |caught, &region| {
                grove.coasting.stop(region) || caught
            });
            if caught {
                // And the press is spent on the catch. What is under the hand hears nothing of it,
                // or stopping a moving list would also be a press on whatever it happened to stop
                // over -- which is the one thing a reader reaching for it did not mean. The gesture
                // stays open and keeps its chain, so a drag out of the catch scrolls as any other.
                debug!(leaf = region.leaf.id(), "coast caught");
            } else if region.receives {
                gesture.target = Some(region.leaf);
                grove.drift.engaged.insert(region.leaf);
                debug!(leaf = region.leaf.id(), "engaged");
            }
        }
    }
    grove.incoming.gesture = Some(gesture);
}

fn moved(grove: &mut Grove, to: Position) {
    let Some(mut gesture) = grove.incoming.gesture.take() else {
        return;
    };
    let delta = gesture.to.to(to);
    gesture.to = to;
    match gesture.stage {
        Stage::Resolving => match grove.claim.claimed(gesture.at.to(to)) {
            Some(axis) => claim(grove, &mut gesture, axis),
            // Still short of both thresholds, so this is still a gesture that could end as a tap.
            None => {
                grove.incoming.gesture = Some(gesture);
                return;
            }
        },
        Stage::Held => claim_hold(grove, &mut gesture),
        Stage::Target | Stage::Region { .. } | Stage::Nobody => {}
    }
    gesture.record(delta);
    apply(grove, &mut gesture, delta);
    grove.incoming.gesture = Some(gesture);
}

fn released(grove: &mut Grove, at: Position) {
    // The last of the movement, which can itself be what claims the gesture: a flick can carry one
    // move and a release and nothing else.
    moved(grove, at);
    close(grove, true);
}

/// The drag out of a hold, which belongs to whoever took the hold.
///
/// Not a second claim, and not one the thresholds have anything left to decide: the hold already
/// settled that this gesture is not a tap and that this element is the one holding it, so the first
/// movement out of it is the drag, however far it went and whichever way. An element that takes no
/// drags at all still takes this one -- which is what lets it scroll like anything else and select
/// from a press that was held.
fn claim_hold(grove: &mut Grove, gesture: &mut Gesture) {
    let Some(target) = gesture.target else {
        return;
    };
    gesture.stage = Stage::Target;
    grove.drift.drag_started.insert(target);
    debug!(leaf = target.id(), "drag claimed");
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
            gesture.stage = Stage::Target;
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
    gesture.stage = match scrolling(grove, &gesture.chain, 0, axis) {
        Some(index) => Stage::Region { index, axis },
        None => Stage::Nobody,
    };
}

/// This frame's movement, delivered to whoever is holding the gesture.
fn apply(grove: &mut Grove, gesture: &mut Gesture, delta: Position) {
    match gesture.stage {
        Stage::Resolving | Stage::Held | Stage::Nobody => {}
        Stage::Target => {
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
        Stage::Region { index, axis } => {
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
            gesture.stage = Stage::Region { index, axis };
        }
    }
}

/// Ends the open gesture, taking the tap it earned if it earned one and handing on the speed it
/// ended with.
fn close(grove: &mut Grove, released: bool) {
    let Some(gesture) = grove.incoming.gesture.take() else {
        return;
    };
    if released {
        launch(grove, &gesture);
    }
    let Some(target) = gesture.target else {
        // A tap that reached nothing that receives takes focus off whatever held it. That is not a
        // rule about dismissing: it is the same rule as below, reading that what was tapped cannot
        // hold focus -- so nothing does.
        if released && matches!(gesture.stage, Stage::Resolving) {
            focus::moved(grove, Intent::Away);
        }
        return;
    };
    // A tap is what a gesture that ended without ever resolving to a drag emits. A gesture that
    // became a drag is not a tap that was taken back; it was never a tap. Neither is one that was
    // held: a press reported as a hold has already stopped being a gesture that could end as a tap,
    // so it moves no caret and takes no focus either.
    if released && matches!(gesture.stage, Stage::Resolving) {
        grove.drift.clicked.insert(target, gesture.at);
        // Focus goes to what was tapped. There is no second declaration deciding it: `interactive`
        // is already the statement that an element takes input, and focus already rests only on
        // what said that -- so a target of a tap is by definition somewhere focus can be.
        //
        // Applied rather than queued, because it is decided here. That leaves focus final before
        // the drain, so anything following it is an ordinary write on the ordinary path, and an app
        // moving focus elsewhere from `clicked` is drained afterwards and still wins.
        focus::moved(grove, Intent::To(target));
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
/// tap and has no speed to speak of. A hand that came to rest before lifting leaves no speed either:
/// the frames it rested for are in the [`WINDOW`] and are what bring the mean down, which is what
/// makes holding still before lifting stop the list rather than fling it.
fn launch(grove: &mut Grove, gesture: &Gesture) {
    let Stage::Region { index, axis } = gesture.stage else {
        return;
    };
    let Some(velocity) = gesture.velocity() else {
        return;
    };
    // Content moves against the pointer, so the offset's velocity is the pointer's reversed --
    // the same sign the drag itself was applied with.
    let speed = -velocity.along(axis);
    // Reported whether or not it turns out to be enough to coast on, because a fling that did not
    // fling is exactly the question this answers.
    debug!(
        leaf = gesture.chain[index].id(),
        ?axis,
        speed,
        over = gesture.covered().as_secs_f32(),
        frames = gesture.recent.len(),
        "released"
    );
    view::launch(grove, gesture.chain[index], axis, speed);
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

/// Where the pointer is, while `leaf` is the one holding the open drag.
///
/// The gesture itself rather than the moves it reported. A pointer held still produces no event at
/// all, so an element with something to do for as long as a drag is *somewhere* -- selecting toward
/// a point past its own edge -- has nothing to read in a frame that reported nothing. This is what
/// it reads instead, and it is the same answer in a frame that moved.
pub(crate) fn dragging(grove: &Grove, leaf: Leaf) -> Option<Position> {
    let gesture = grove.incoming.gesture.as_ref()?;
    (gesture.target == Some(leaf) && matches!(gesture.stage, Stage::Target)).then_some(gesture.to)
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
    grove
        .tree
        .set_offset(leaf, offset.set(axis, offset.along(axis) + taken));
    taken
}
