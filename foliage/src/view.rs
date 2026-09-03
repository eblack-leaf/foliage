//! Scrolling regions -- what a gesture hands itself to, and where it goes afterwards.
//!
//! An element scrolls because it said so. A grid divides an element's box for its children and says
//! nothing about scrolling, and an axis that was not declared does not scroll, has no extent, and
//! cannot be moved: it is not a scrolling axis with a range of zero, which is a different and
//! simpler thing to reason about.
//!
//! Three values, each written by one pass:
//!
//! | | Pass | Holds |
//! |---|---|---|
//! | [`Extent`] | R3, bottom-up | how far the content reaches from the region's own near edges |
//! | [`Offset`] | R4, top-down | how far the region has been moved, clamped to what it can reach |
//! | [`Clipped`] | R5, top-down | the box a scrolling ancestor leaves visible |
//!
//! # One unit: logical pixels
//!
//! Offset is logical pixels, read and written. [`ScrollTo`] still accepts other framings, and each
//! of them **names its unit at the call site** -- which is what was missing when a fraction was the
//! only thing that could be written and everything else in the system was in pixels.
//!
//! # Coast is a view behaviour
//!
//! A drag released with speed keeps the region moving. The gesture supplies its initial velocity
//! and is then done; the decay, the clamp against the extent, and whether reaching an end while
//! coasting chains outward or absorbs are all here. It is deliberately not an
//! [`Aspen`](crate::aspen) tween: there is no target and no duration, and an integration that runs
//! until it settles is a different shape from an interpolation between two known endpoints.

use core::time::Duration;
use std::collections::HashMap;

use bevy_ecs::component::Component;
use tracing::debug;

use crate::aspen::blend;
use crate::coordinate::{Area, Axes, Axis, Position, Section};
use crate::grove::Grove;
use crate::interaction;
use crate::leaf::Leaf;

/// Which axes an element scrolls, and what each of them does at its end.
///
/// Absent, the element does not scroll, and a drag inside it goes on outward to something that
/// does.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Scrolls(pub(crate) Scroll);

/// What a region declares about scrolling: the axes it moves on, and which of them absorb.
///
/// The two are one value rather than two flags, because a boundary policy on an axis the region
/// does not scroll describes nothing, and two declarations that can disagree about which axes are
/// in play is exactly the failure the per-axis split exists to avoid.
///
/// ```no_run
/// # use foliage::{Axes, Place, Scroll, Stem};
/// // Reaching the end hands the gesture outward to the next scrolling ancestor.
/// Stem::new().scrolls(Axes::Vertical);
/// // The region absorbs it instead. Nothing outside moves.
/// Stem::new().scrolls(Scroll::new(Axes::Vertical).contain(Axes::Vertical));
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Scroll {
    axes: Axes,
    contains: Option<Axes>,
}

impl Scroll {
    /// A region scrolling `axes`, each of them **chaining**.
    ///
    /// Reaching the end of a chaining axis hands the gesture outward to the next scrolling
    /// ancestor, which is what lets a drag inside a list keep moving the page once the list is
    /// done. It is the default because it is what a reader expects of ordinary content.
    pub fn new(axes: Axes) -> Self {
        Self {
            axes,
            contains: None,
        }
    }

    /// The axes named **contain** rather than chain: the region absorbs a gesture that reaches
    /// their end, and nothing outside it moves.
    ///
    /// For a region that owns its gesture outright -- a map, an editor, a pane inside a fixed
    /// shell -- where reaching the bottom and having the whole page lurch is a bug every time.
    ///
    /// Naming an axis the region does not scroll says nothing, because there is no gesture on that
    /// axis for it to absorb.
    pub fn contain(mut self, axes: Axes) -> Self {
        self.contains = Some(axes);
        self
    }

    /// Whether this region scrolls `axis` at all.
    pub(crate) fn covers(self, axis: Axis) -> bool {
        self.axes.covers(axis)
    }

    /// Whether reaching the end of `axis` stops here rather than passing outward.
    pub(crate) fn absorbs(self, axis: Axis) -> bool {
        self.covers(axis) && self.contains.is_some_and(|axes| axes.covers(axis))
    }
}

impl From<Axes> for Scroll {
    fn from(axes: Axes) -> Self {
        Self::new(axes)
    }
}

/// Where a region is asked to move to, with its unit named at the call site.
///
/// ```no_run
/// # use foliage::{Grove, Grow, Leaf, ScrollTo};
/// # fn f(grove: &mut Grove, column: Leaf, section: Leaf) {
/// grove.scroll(column, ScrollTo::px(240.0));
/// grove.scroll(column, ScrollTo::fraction(0.5));
/// grove.scroll(column, ScrollTo::end());
/// grove.scroll(column, ScrollTo::show(section));
/// # }
/// ```
///
/// Every form lands in the same place: a number of logical pixels from the content's origin,
/// clamped to what the region can reach. Reading the offset back afterwards returns those pixels.
///
/// # Which axis it moves
///
/// Every framing but one is stated **relative to the region's own range**, so it means the same
/// thing on either axis and needs no axis named: the end is the end of each, half way is half of
/// each, and bringing a descendant into view is one place in two dimensions.
///
/// [`px`](ScrollTo::px) is the exception, and it is the only absolute distance here: two hundred
/// pixels down and two hundred pixels across are unrelated distances that happen to share a number.
/// So on a region scrolling both axes it names one with [`on`](ScrollTo::on), and an op that does
/// not is dropped rather than moving two axes by a coincidence.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ScrollTo {
    to: Destination,
    /// Which axes it moves. Absent, read off the region.
    axes: Option<Axes>,
}

/// The framings a destination can be stated in.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Destination {
    Px(f32),
    Fraction(f32),
    Start,
    End,
    Show(Leaf),
}

impl ScrollTo {
    /// That many logical pixels from the content's origin. The unit everything else is in.
    pub fn px(offset: f32) -> Self {
        Self::to(Destination::Px(offset))
    }

    /// That fraction of the region's range, where `1.0` is as far as it goes.
    pub fn fraction(of: f32) -> Self {
        Self::to(Destination::Fraction(of))
    }

    /// Back to the content's origin.
    pub fn start() -> Self {
        Self::to(Destination::Start)
    }

    /// As far as the region reaches.
    pub fn end() -> Self {
        Self::to(Destination::End)
    }

    /// The least distance that brings a descendant into view, and no further.
    ///
    /// Jumping to a section is common enough that every app otherwise computes it by hand. A
    /// descendant already in view moves the region nowhere.
    ///
    /// Dropped, like any op naming something it does not apply to, if the element named is not
    /// grown under the region.
    pub fn show(leaf: Leaf) -> Self {
        Self::to(Destination::Show(leaf))
    }

    /// Which axes this moves, where the region scrolls more than one.
    ///
    /// Naming an axis the region does not scroll leaves nothing for the destination to move, and
    /// the op that carried it is dropped rather than doing nothing quietly.
    pub fn on(mut self, axes: Axes) -> Self {
        self.axes = Some(axes);
        self
    }

    fn to(to: Destination) -> Self {
        Self { to, axes: None }
    }

    /// The element this destination is stated against, if it is stated against one.
    pub(crate) fn names(&self) -> Option<Leaf> {
        match self.to {
            Destination::Show(leaf) => Some(leaf),
            _ => None,
        }
    }

    /// Which axes of a region scrolling `scrolls` this destination actually moves.
    ///
    /// `None` is an op with nothing to do, and it covers both ways of getting there: an absolute
    /// distance that named no axis on a region scrolling two, and a named axis the region does not
    /// scroll. Neither is refused quietly -- the drain names it where the write was made.
    pub(crate) fn over(&self, scrolls: Scroll) -> Option<Axes> {
        let named = match self.axes {
            Some(axes) => axes,
            // Stated against the range, so it says the same thing on either axis.
            None if self.relative() => Axes::Both,
            None => match (
                scrolls.covers(Axis::Horizontal),
                scrolls.covers(Axis::Vertical),
            ) {
                (true, false) => Axes::Horizontal,
                (false, true) => Axes::Vertical,
                // Both, and no way to tell which of two unrelated distances was meant.
                _ => return None,
            },
        };
        named.shared(scrolls.axes)
    }

    /// Whether the destination is stated against the region rather than in absolute pixels.
    fn relative(&self) -> bool {
        !matches!(self.to, Destination::Px(_))
    }

    /// Where on `axis` this destination puts a region that currently sits at `at` and can travel
    /// `span`.
    ///
    /// `shown` is the box of the element [`show`](ScrollTo::show) named, in the same layout
    /// coordinates as `region` -- which is what makes the arithmetic a subtraction rather than a
    /// walk.
    fn landing(&self, axis: Axis, at: f32, span: f32, region: Section, shown: Section) -> f32 {
        let landing = match self.to {
            Destination::Px(offset) => offset,
            Destination::Fraction(of) => of.clamp(0.0, 1.0) * span,
            Destination::Start => 0.0,
            Destination::End => span,
            Destination::Show(_) => {
                let (near, far, seen) = match axis {
                    Axis::Horizontal => (
                        shown.left() - region.left(),
                        shown.right() - region.left(),
                        region.width(),
                    ),
                    Axis::Vertical => (
                        shown.top() - region.top(),
                        shown.bottom() - region.top(),
                        region.height(),
                    ),
                };
                // The least movement that brings it in, and none at all if it is already there.
                if near < at {
                    near
                } else if far > at + seen {
                    far - seen
                } else {
                    at
                }
            }
        };
        landing.clamp(0.0, span)
    }
}

/// How a released drag coasts.
///
/// A **global tuning value**, like every other number behind how the engine feels. Set it once,
/// before the engine runs:
///
/// ```no_run
/// # use core::time::Duration;
/// # use foliage::{Foliage, Momentum};
/// let mut foliage = Foliage::new();
/// foliage.tune(Momentum {
///     half_life: Duration::from_millis(350),
///     minimum: 40.0,
/// });
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Momentum {
    /// How long a coast takes to lose half its speed.
    ///
    /// Stated as a half-life because that is the one form of it a person can read and predict: a
    /// coast at half speed after this long, a quarter after twice as long, and travelling in total
    /// a little under its release speed times this. The decay is continuous, so the ground a fling
    /// covers does not depend on how often frames happen to run.
    ///
    /// Zero is no coast at all: a release stops the region where it was.
    pub half_life: Duration,
    /// The speed, in logical pixels per second, below which a release starts no coast and a
    /// running one stops.
    pub minimum: f32,
}

impl Default for Momentum {
    fn default() -> Self {
        Self {
            half_life: Duration::from_millis(350),
            minimum: 40.0,
        }
    }
}

/// How far a region has been moved from its content's origin, in logical pixels.
///
/// One unit throughout, read and written the same way. Positive is content moved toward the near
/// edge -- what a drag away from that edge produces.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Offset(pub(crate) Position);

/// How far a region's content reaches, measured outward from the region's own near edges.
///
/// Derived from where the children landed, never from what is currently drawn: a child scrolled out
/// of sight is the content the extent exists to describe, so culling has no say in it. Never
/// smaller than the region's own box, so an empty region has a range of zero rather than a negative
/// one.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Extent(pub(crate) Area);

/// How far out of the regions above it a floating element reaches.
///
/// Stated rather than defaulted, because both answers are right somewhere and neither is right
/// everywhere. A menu on a row of a table that has its own scrollbar, inside a settings panel that
/// has another, wants out of both. A tooltip on a file in a list inside a sidebar wants out of the
/// list and *not* out of the sidebar, or it paints across the page beside it. Nothing about the
/// two pictures differs to the engine, so the callsite says which.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Escape {
    /// Out of the region it is grown in, and no further: whatever clips that region clips this too.
    Region,
    /// Out of every region above it, held only by the surface.
    Surface,
    /// Out of every region up to the one named, which still holds it.
    ///
    /// The general case the other two are shorthands for, and the answer where neither of them is:
    /// three regions deep, leaving two and staying inside the third. It names the element rather
    /// than counting regions, because a count is a fact about the tree's current shape and would
    /// quietly mean something else the moment a wrapper was added.
    ///
    /// An element that is not one of this one's ancestors holds nothing, so it falls back to
    /// [`Region`](Escape::Region) rather than escaping further than it was told to.
    Within(Leaf),
}

/// The element sits over its region rather than in it.
///
/// The trunk decides what takes an element down, what it stacks among, and what clips it; an
/// [`anchor`](crate::Place::anchored) decides where it sits. That split is what lets an element be
/// positioned by one element while living under another -- and it is also what leaves an element
/// positioned *outside* its trunk cut off at the trunk's edge, which is the whole point of having
/// put it out there.
///
/// This is how an element says it is not part of the region it is grown in. Two consequences, and
/// they are one question asked twice, so they get one answer:
///
/// - it is **not clipped** by that region -- [`Escape`] says whether anything further out still is
/// - it **contributes nothing** to that region's extent, because it is not content
///
/// The extent half takes no [`Escape`], and needs none: a region contributes its own box and never
/// its content to whatever contains it, so there is no second region for an overlay to be excluded
/// from. The choice is about clipping alone.
///
/// What it keeps is the region's offset: it travels with the content it is anchored to, which is
/// what makes a menu follow the row that opened it. That is the whole difference from
/// [`pinned`](crate::Place::pinned), which keeps the clip and escapes the movement.
///
/// Both are relative to the **nearest** scrolling ancestor: the region the element is grown in is
/// the one it is not part of.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Floats(pub(crate) Escape);

/// The element does not travel with its region's content.
///
/// *Moving with the content* and *counting toward extent* are the same question, so they get one
/// answer: a pinned element receives no offset from its nearest scrolling ancestor in R4 and
/// contributes nothing to that region's extent in R3. Two separate flags could drift out of
/// agreement with each other; one cannot.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Pinned;

/// The box a scrolling ancestor leaves visible: the intersection of every one of them.
///
/// A rect, and only a rect. Whether an element is *culled* is a decision extraction makes by
/// comparing this against the element's box, and is never recorded on the element -- which is what
/// keeps culling out of anything that reads state, extent first among them.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub(crate) struct Clipped(pub(crate) Section);

impl Clipped {
    /// Nothing clipped: what an element with no scrolling ancestor carries.
    pub(crate) fn unbounded() -> Self {
        Self(Section::from_edges(
            f32::MIN / 2.0,
            f32::MIN / 2.0,
            f32::MAX / 2.0,
            f32::MAX / 2.0,
        ))
    }
}

impl Default for Clipped {
    fn default() -> Self {
        Self::unbounded()
    }
}

/// Every region still moving from a release, and how fast.
///
/// Held beside the tree rather than on the elements, for the same reason the running tweens are:
/// what is coasting is a small set and almost never the tree, so the loop can ask whether anything
/// is (F9) without a pass over anything.
///
/// Keyed per axis, because the two axes reach their ends independently and each hands outward on
/// its own.
#[derive(Default)]
pub(crate) struct Coasting {
    running: HashMap<(Leaf, Axis), Coast>,
}

/// One region still moving.
#[derive(Copy, Clone, Debug)]
struct Coast {
    /// Its speed, in logical pixels of *offset* per second.
    velocity: f32,
    /// Whether a frame's delta has been charged to it yet.
    ///
    /// A coast begins at the dispatch of the frame that released it, and that frame's delta is the
    /// interval *ending* there -- the interval the drag was still being made in, and which the drag
    /// has already been paid for. Charging it as well would move the region twice for one stretch
    /// of time. The same rule a tween created at the drain follows, for the same reason.
    charged: bool,
}

impl Coasting {
    /// Sets a region coasting, replacing whatever it was doing on that axis.
    fn start(&mut self, leaf: Leaf, axis: Axis, velocity: f32) {
        self.running.insert(
            (leaf, axis),
            Coast {
                velocity,
                charged: false,
            },
        );
    }

    /// Puts one back, as it now stands.
    fn resume(&mut self, leaf: Leaf, axis: Axis, coast: Coast) {
        self.running.insert((leaf, axis), coast);
    }

    /// Stops a region on one axis, and says whether it was moving.
    pub(crate) fn halt(&mut self, leaf: Leaf, axis: Axis) -> bool {
        self.running.remove(&(leaf, axis)).is_some()
    }

    /// Stops a region on both axes, which is what taking hold of it does, and says whether it
    /// caught anything -- a hand that stopped something has spent its press on stopping it.
    pub(crate) fn stop(&mut self, leaf: Leaf) -> bool {
        Axis::BOTH
            .into_iter()
            .fold(false, |caught, axis| self.halt(leaf, axis) || caught)
    }

    /// Drops every coast on elements that have gone.
    pub(crate) fn wither(&mut self, gone: &[Leaf]) {
        if self.running.is_empty() {
            return;
        }
        for leaf in gone {
            self.stop(*leaf);
        }
    }

    /// Whether nothing is coasting, which is one of the clauses in whether the loop owes a frame.
    pub(crate) fn idle(&self) -> bool {
        self.running.is_empty()
    }

    /// How many coasts are running. Reported to the trace.
    pub(crate) fn len(&self) -> usize {
        self.running.len()
    }

    /// Takes every running coast, leaving nothing. Whatever is still moving is put back.
    fn take(&mut self) -> Vec<(Leaf, Axis, Coast)> {
        self.running
            .drain()
            .map(|((leaf, axis), coast)| (leaf, axis, coast))
            .collect()
    }
}

/// How far a region can be moved on one axis: what its content reaches, less the box it is seen
/// through.
pub(crate) fn range(extent: Area, own: Area, axis: Axis) -> f32 {
    (extent.along(axis) - own.along(axis)).max(0.0)
}

/// How much of `delta` a region sitting at `offset` can still take.
///
/// The whole of what "can no longer consume" means, and the reason it needs no flag: a region at its
/// end returns zero for a delta going further out and the full delta for one coming back, so the
/// answer is about where the region currently sits rather than about what it was told at spawn.
pub(crate) fn consumable(offset: f32, range: f32, delta: f32) -> f32 {
    (offset + delta).clamp(0.0, range) - offset
}

/// How far a coast travelling at `velocity` gets in `elapsed` seconds, and the speed it leaves,
/// given a `half_life` in seconds.
///
/// The decay is continuous, so the distance covered is the integral of the speed rather than one
/// frame's rectangle. That is what makes a fling reach the same place at thirty frames a second as
/// at a hundred and twenty, which a per-frame multiply does not.
pub(crate) fn coasted(velocity: f32, half_life: f32, elapsed: f32) -> (f32, f32) {
    // No half-life at all is a coast that stops dead. No *time* is not: a frame that took no time
    // costs a coast neither distance nor speed, or a run of them would settle one that never moved.
    if half_life <= 0.0 {
        return (0.0, 0.0);
    }
    if elapsed <= 0.0 {
        return (0.0, velocity);
    }
    let remaining = 0.5f32.powf(elapsed / half_life);
    let travelled = velocity * half_life * (1.0 - remaining) / core::f32::consts::LN_2;
    (travelled, velocity * remaining)
}

/// How far through its range a region sits on one axis, as the `0.0..=1.0` a scrollbar reads.
///
/// Derived rather than held. A region with nowhere to go reads zero, which is the honest answer:
/// there is no progress through a range of nothing.
pub(crate) fn progress(offset: Position, extent: Area, own: Area, axis: Axis) -> f32 {
    let range = range(extent, own, axis);
    match range > 0.0 {
        true => (offset.along(axis) / range).clamp(0.0, 1.0),
        false => 0.0,
    }
}

/// Hands a coast its initial velocity, in logical pixels of offset per second.
///
/// The whole of what a gesture has to do with one: everything after this -- the decay, the clamp,
/// and whether an end chains outward or absorbs -- is the region's.
pub(crate) fn launch(grove: &mut Grove, leaf: Leaf, axis: Axis, velocity: f32) {
    if grove.momentum.half_life.is_zero() || velocity.abs() < grove.momentum.minimum {
        return;
    }
    debug!(leaf = leaf.id(), ?axis, velocity, "coasting");
    grove.coasting.start(leaf, axis, velocity);
}

/// The offset each region is asked for this frame, before R4 clamps it and carries it down.
///
/// Three ways to ask, all of them writes to one value: a coast still running from a release, a
/// [`scroll`](crate::Grow::scroll) written this frame, and a
/// [`Motion::Scroll`](crate::Motion::Scroll) part way through. They never contend, because each of
/// them ends the others where it starts (F8).
///
/// All three are answered here rather than where they were written, because all three need the
/// extent -- and the extent is R3's, one pass earlier in this same frame. Asking to scroll to the
/// end therefore lands at the end of the content as it is *now*, in the frame the content changed,
/// with no settling frame.
pub(crate) fn asked(grove: &mut Grove, boxes: &HashMap<Leaf, Section>) {
    coast(grove, boxes);
    sought(grove, boxes);
    animated(grove, boxes);
}

/// Every running coast, advanced by this frame's share of its decay.
fn coast(grove: &mut Grove, boxes: &HashMap<Leaf, Section>) {
    if grove.coasting.idle() {
        return;
    }
    let elapsed = grove.clock.delta().as_secs_f32();
    let momentum = grove.momentum;
    for (leaf, axis, mut coast) in grove.coasting.take() {
        if !coast.charged {
            coast.charged = true;
            grove.coasting.resume(leaf, axis, coast);
            continue;
        }
        let section = boxes.get(&leaf).copied().unwrap_or_default();
        let span = range(grove.tree.extent(leaf), section.area, axis);
        let offset = grove.tree.offset(leaf);
        let (travelled, speed) =
            coasted(coast.velocity, momentum.half_life.as_secs_f32(), elapsed);
        let at = offset.along(axis);
        let wanted = at + travelled;
        let landed = wanted.clamp(0.0, span);
        if landed != at {
            grove.tree.set_offset(leaf, offset.set(axis, landed));
        }
        // An end reached while coasting chains outward or absorbs exactly as a drag would, because
        // it is the same question asked of the same region at the same place in its own extent.
        if landed != wanted {
            match handed(grove, leaf, axis) {
                Some(next) => {
                    debug!(from = leaf.id(), to = next.id(), "coast handed outward");
                    // Already mid-flight: this frame's time is spent, and the region it came from
                    // is what spent it.
                    grove.coasting.resume(
                        next,
                        axis,
                        Coast {
                            velocity: speed,
                            charged: true,
                        },
                    );
                }
                None => debug!(leaf = leaf.id(), "coast absorbed"),
            }
            continue;
        }
        if speed.abs() < momentum.minimum {
            debug!(leaf = leaf.id(), "coast settled");
            continue;
        }
        grove.coasting.resume(
            leaf,
            axis,
            Coast {
                velocity: speed,
                charged: true,
            },
        );
    }
}

/// The region a coast passes to when `leaf` can move no further, or `None` where it absorbs.
fn handed(grove: &Grove, leaf: Leaf, axis: Axis) -> Option<Leaf> {
    let chain = interaction::chain(grove, leaf);
    // A coasting region scrolls, so it is the innermost link of its own chain -- unless it stopped
    // being one under the coast, in which case there is nothing to hand anything to.
    if chain.first() != Some(&leaf) {
        return None;
    }
    interaction::outward(grove, &chain, 0, axis).map(|index| chain[index])
}

/// The one-shot destinations written this frame.
fn sought(grove: &mut Grove, boxes: &HashMap<Leaf, Section>) {
    for (leaf, to) in core::mem::take(&mut grove.sought) {
        if let Some(landed) = destination(grove, boxes, leaf, to) {
            grove.tree.set_offset(leaf, landed);
            debug!(leaf = leaf.id(), x = landed.x, y = landed.y, "scrolled");
        }
    }
}

/// The destinations a motion is part way to.
///
/// Both ends are answered every frame, in this frame's context: the target re-resolves against the
/// extent R3 just measured, so a motion toward the end of a list that grew under it still lands on
/// the end. What it left is a number of pixels and is carried as one, because the offset is a
/// resolved value rather than a declaration that could be resolved again.
fn animated(grove: &mut Grove, boxes: &HashMap<Leaf, Section>) {
    for (leaf, from, to, at) in grove.aspen.scrolling() {
        let Some(target) = destination(grove, boxes, leaf, to) else {
            continue;
        };
        grove.tree.set_offset(
            leaf,
            Position::new(
                blend(from.x, target.x, at),
                blend(from.y, target.y, at),
            ),
        );
    }
}

/// Where `to` puts `leaf`, on each axis the destination speaks for.
///
/// `None` where there is no such axis, which is the same refusal the drain already named, read one
/// pass later.
fn destination(
    grove: &Grove,
    boxes: &HashMap<Leaf, Section>,
    leaf: Leaf,
    to: ScrollTo,
) -> Option<Position> {
    let moves = to.over(grove.tree.scrolls(leaf)?)?;
    let section = boxes.get(&leaf).copied().unwrap_or_default();
    let extent = grove.tree.extent(leaf);
    let offset = grove.tree.offset(leaf);
    let shown = to
        .names()
        .and_then(|named| boxes.get(&named).copied())
        .unwrap_or_default();
    let mut landed = offset;
    for axis in Axis::BOTH {
        if !moves.covers(axis) {
            continue;
        }
        let span = range(extent, section.area, axis);
        landed = landed.set(
            axis,
            to.landing(axis, offset.along(axis), span, section, shown),
        );
    }
    Some(landed)
}
