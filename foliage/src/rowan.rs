//! Rowan -- resolution.
//!
//! # Rowan resolves. Elm decides what changed.
//!
//! Every pass runs in the same order over the same elements every frame, and each writes exactly one
//! thing. What a pass may leave alone is an element **nothing it reads was written to**, whose
//! answer is the one it already holds -- sitting in the column the pass would have written it to.
//!
//! That is not invalidation and nothing is ever marked stale. The tree records a write where the
//! write happens ([`Tree::declared`](crate::tree::Tree::declared), which every declaration passes
//! through), and [`read`] closes that over the dependencies before any pass runs. So an element
//! resolves again if it was written to, if what it hangs off or is anchored to resolves again, or if
//! a measure moved under it -- and if none of those happened, resolving it again would produce what
//! it already has. A value here can be old. It cannot be wrong.
//!
//! What still runs over everything is the accumulations, R3 through R8: what they carry is a running
//! product rather than an answer per element, so there is no position in them to start from. They
//! record what they actually changed as they go, which is what extraction then states.
//!
//! The read itself is kept between frames and repaired where the tree was written -- growing,
//! withering and anchoring are the only things that change the order, so an ordinary frame reads
//! none of it.
//!
//! Nothing is re-uploaded on account of any of it, because extraction compares against a cached copy
//! and sends only genuine differences.
//!
//! The passes each write exactly one thing, and nothing writes what another owns:
//!
//! | | Pass | Direction | Produces |
//! |---|---|---|---|
//! | R1 | measure | — | [`Cell`], and the max-content width half of [`Intrinsic`] |
//! | R2a | horizontal | dependency order | the horizontal axis |
//! | R2m | wrap | **bottom-up** | the measured-height half of [`Intrinsic`] |
//! | R2b | vertical | dependency order | the vertical axis, and with it [`Placed`] |
//! | R3 | extent | **bottom-up** | [`Extent`](crate::view::Extent): how far a region's content reaches |
//! | R4 | scroll | top-down | [`Drawn`]: [`Placed`] less every scrolling ancestor's offset |
//! | R5 | clip | top-down | [`Clipped`]: what a scrolling ancestor leaves visible |
//! | R6 | rank | dependency order | `ResolvedElevation`: elevation accumulated, then tie-broken |
//! | R7 | inherit | top-down | [`Inherited`]: the visible, opacity and disabled products |
//! | R8 | regions | rank order | the box stack the next frame's dispatch reads |
//!
//! Both halves of R2 call the same pure resolver, once per axis. That is only safe because it is
//! pure: there is no accumulated state for a second call to corrupt.
//!
//! R1, R2a and R2b resolve what was written to. R2m resolves that and everything above it, because
//! what an element reaches over is what the elements under it resolved to -- a measure travels
//! toward the trunk where a box travels away from it. A measure that moves under an element nothing
//! wrote to makes that element resolve again, which is why R2m sits before R2b and not after it.
//!
//! # Every pass reads the order, not the world
//!
//! The passes ask the same questions of the same elements over and over: what an element hangs off,
//! what it is anchored to, and what each of those offers a placement reading it. Asked of the world
//! each time, every one of those is a lookup by entity -- and the arithmetic a pass does is small
//! enough that the lookups, not the arithmetic, are what a frame costs.
//!
//! So the tree is read once, into [`Elements`]: every live element in dependency order, holding what
//! it depends on as **positions in that order** rather than as names, and holding what it offers a
//! placement beside it. From there every pass indexes -- the accumulations R3 through R7 carry are
//! arrays the length of the order, and a trunk's contribution is `[..]` rather than a lookup on a
//! map keyed by element.
//!
//! The measures R1 and R2m state are written there too, and reach the elements themselves in one
//! [`scatter`] once both passes have run. So no pass writes a value another is about to read back
//! out of the world, and during resolution the order is the only place a measure is held.
//!
//! Nothing about what is computed changes, and neither does the ordering that makes it correct.
//!
//! Two passes run the other way, and each is a cycle that turns out not to be vicious.
//!
//! R2m is one: wrapping makes a height depend on a width, and the width comes from the layout, so a
//! box sized to its contents looks circular. It is not, because **width flows down and height flows
//! up** -- a monospaced run's widest unwrapped line is free from its character count, so the
//! down-pass needs nothing measured. R2m sits between the two halves of R2 for that reason: it is
//! the one moment when every width is known and no height is, which is exactly what measuring needs.
//!
//! R3 is the other: a child resolves against its parent's box, a scrolling parent's extent comes
//! from where its children landed, and that extent is what its offset is clamped to. Extent affects
//! only the clamp and never the parent's own box -- so a top-down layout sweep, a bottom-up extent
//! sweep and a top-down scroll application resolve it in one pass each.
//!
//! Neither iterates to convergence. Every pass here runs exactly once.

use std::collections::HashMap;

use bevy_ecs::component::Component;
use tracing::field::Empty;
use tracing::trace_span;

use crate::aspen::Departed;
use crate::coordinate::{Area, Axis, Position, Section};
use crate::elevation::ResolvedElevation;
use crate::elm::Chlorophyll;
use crate::grove::Grove;
use crate::interaction::stack::Region;
use crate::leaf::Leaf;
use crate::lifecycle::Inherited;
use crate::line::Stretched;
use crate::placement::grid::Tracks;
use crate::placement::location::Location;
use crate::placement::resolve::{Basis, Context, Span, locate, resolve};
use crate::placement::role::Config;
use crate::view::{self, Clipped, Escape, Scroll, range};

/// Where the layout put an element. What its children resolve against.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Placed(pub(crate) Section);

/// Where an element is on screen: its [`Placed`] box less every scrolling ancestor's accumulated
/// offset. What drawing, clipping and hit-testing read.
///
/// The two are deliberately separate. A change to the first means the layout moved a box and its
/// children have to follow; a change to the second means only that the box moved under a scroll.
///
/// Logical pixels, like every other coordinate. The scale factor is applied in the render backend
/// and nowhere else.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Drawn(pub(crate) Section);

/// The character cell an element's own font and size make, as R1 measured it.
///
/// On every element, because every element may be sized in characters: it is what
/// [`letters`](crate::Source::letters) and a letter-pitched track are measured in. An element that
/// named no font and no size has none, and reads zero.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Cell(pub(crate) Area);

/// What an element measured to, which is what [`content()`](crate::content) reads.
///
/// The two halves are written by different passes and mean different questions, which is the whole
/// of width-down and height-up:
///
/// - **width** is max-content, written by R1 -- the widest the element would like to be, unwrapped.
///   In a monospaced font that is a character count times a cell, so it is free and is available
///   before any layout has happened.
/// - **height** is measured, written by R2m -- what the element turned out to be at the width R2a
///   gave it.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Intrinsic(pub(crate) Area);

/// Every live element in dependency order, with what each one resolves against and what each one
/// offers a placement that reads it.
///
/// The frame's one read of the tree, and what every pass indexes into. A trunk and an anchor are
/// asked for on both axes and on four of the passes after them; what an element offers is asked for
/// twice per element on each axis. Out of the world each time, every one of those is a lookup by
/// entity -- resolved once to a position in the order, each is an index.
///
/// The columns are as long as the order and are read at the position an element sits at, so a pass
/// that already knows where it is never asks a second time.
#[derive(Default)]
pub(crate) struct Elements {
    /// Every live element, ordered so that nothing resolves before what it depends on.
    order: Vec<Leaf>,
    /// Where each element sits in the order.
    index: HashMap<Leaf, usize>,
    /// What each element hangs off, as a position in the order.
    trunk: Vec<Option<usize>>,
    /// What each element is anchored to, as a position in the order.
    anchor: Vec<Option<usize>>,
    /// The box, as far as the axes have resolved it.
    section: Vec<Section>,
    /// Whether the element has a box worth reading yet. True from the start for one nothing has
    /// written to, whose box is the one it settled at last frame; false for one being resolved
    /// again, until an axis reaches it -- which is what answers a cycle's remainder with the
    /// fallback rather than with a box that has not been computed.
    resolved: Vec<bool>,
    /// Whether the element has to be resolved again this frame.
    ///
    /// True where something it reads was written -- its own declarations, or those of what it hangs
    /// off or is anchored to. False where nothing was, in which case the columns already hold what
    /// it resolved to and the passes that only rewrite that have nothing to do.
    dirty: Vec<bool>,
    /// Whether any run stopped being stated this frame, which is the one thing that makes the
    /// shaping cache worth sweeping -- and, because a sweep drops what this frame did not state,
    /// the one thing that makes every run worth shaping again.
    restated: bool,
    /// Whether anything extraction reads about the element moved this frame.
    ///
    /// Seeded from [`dirty`](Elements::dirty), because an element being resolved again may land
    /// somewhere else, and set by R4 through R7 where what they wrote differs from what was held.
    /// An element it is false for is drawn exactly as the backend already has it.
    moved: Vec<bool>,
    /// What the element draws, or [`Chlorophyll::None`] where it draws nothing.
    chlorophyll: Vec<Chlorophyll>,
    /// Whether the element has to measure again, before R2m walks that up the trunks.
    ///
    /// [`dirty`](Elements::dirty) is most of it. The rest is an element something withered from,
    /// which has one fewer thing to reach over and nothing left below it to say so.
    measures: Vec<bool>,
    /// What the element measured to. R1 writes the width and R2m the height.
    intrinsic: Vec<Area>,
    /// The element's own grid, divided at the breakpoint in force.
    tracks: Vec<Tracks>,
    /// The element's character cell.
    cell: Vec<Area>,
}

impl Elements {
    /// How many elements are live.
    fn len(&self) -> usize {
        self.order.len()
    }

    /// Where `leaf` sits, or `None` if it is not live.
    fn at(&self, leaf: Leaf) -> Option<usize> {
        self.index.get(&leaf).copied()
    }

    /// The box `leaf` resolved to, or a zero one if it is not live.
    ///
    /// Resolution reads a box by position, because it holds every element in order. A coast, a
    /// sought region and a running motion each name one element instead, so they ask the way an app
    /// asks.
    /// Every live element in the order resolution used, with what it draws and whether anything
    /// extraction reads about it moved this frame.
    pub(crate) fn drawing(&self) -> impl Iterator<Item = (Leaf, Chlorophyll, bool)> + '_ {
        (0..self.len()).map(|at| (self.order[at], self.chlorophyll[at], self.moved[at]))
    }

    pub(crate) fn of(&self, leaf: Leaf) -> Section {
        self.at(leaf).map(|at| self.section[at]).unwrap_or_default()
    }
}

/// Steps 6 and 7. Declared state becomes resolved geometry and resolved products, for everything.
pub(crate) fn run(grove: &mut Grove) {
    let step = trace_span!("resolve", elements = Empty, dirty = Empty);
    let resolving = step.enter();
    // Taken off the grove so the passes can hold it while they write to the rest, and put back at
    // the end of resolution: what it holds is what this frame settled, and what the next one starts
    // from rather than reads again.
    let mut elements = core::mem::take(&mut grove.elements);
    read(&mut elements, grove);
    // Forgotten the moment it is read, so a write made by a pass after this belongs to the frame
    // that would resolve it.
    grove.tree.taken();
    step.record("elements", elements.len());
    step.record(
        "dirty",
        elements.dirty.iter().filter(|dirty| **dirty).count(),
    );
    measure(grove, &mut elements);
    let ends = axes(grove, &mut elements);
    scatter(grove, &elements);
    // Both of the passes that shape have run, so what is not held now is a run nothing states. Only
    // on a frame that shaped every one of them, which is the frame after one stopped being stated:
    // a sweep over a frame that shaped some of them would drop the rest.
    if elements.restated {
        grove.shaping.sweep();
    }
    extent(grove, &elements);
    scroll(grove, &mut elements, &ends);
    clip(grove, &mut elements);
    rank(grove, &mut elements);
    // Resolution ends here. What settles after it is its own step, and is timed as one.
    drop(resolving);
    let _step = trace_span!("settle").entered();
    inherit(grove, &mut elements);
    regions(grove, &elements);
    grove.elements = elements;
}

/// The measures, written back to the elements themselves.
///
/// R1 and R2m state them into the order, because everything that reads them *during* resolution
/// reads them there. This is the one write, after both passes that state them have run: what an
/// element measured to is one value, and a frame never holds it in two places.
fn scatter(grove: &mut Grove, elements: &Elements) {
    let _pass = trace_span!("scatter").entered();
    for at in 0..elements.len() {
        let leaf = elements.order[at];
        grove.tree.set_cell(leaf, elements.cell[at]);
        grove.tree.set_intrinsic(leaf, elements.intrinsic[at]);
    }
}

/// R1. What an element's own font makes of it: its character cell, and the widest its content would
/// like to be.
///
/// Ahead of every other pass, and reading no geometry at all, because neither answer has anything to
/// do with where the element ended up. A monospaced run's max-content width is its longest line's
/// character count times its cell, so the whole of the down-pass's input is available before the
/// down-pass runs -- which is what leaves only the up-pass with anything to measure.
fn measure(grove: &mut Grove, elements: &mut Elements) {
    let _pass = trace_span!("measure").entered();
    let Grove {
        tree,
        fonts,
        shaping,
        layout,
        short,
        ..
    } = grove;
    for at in 0..elements.order.len() {
        // Nothing it reads was written, so it measures to what it measured to, which is what the
        // column was read out of the tree holding.
        if !elements.dirty[at] {
            continue;
        }
        let leaf = elements.order[at];
        // No font and no size is no cell, and nothing measured in one. What such an element last
        // measured to stands, which is what it was read out of the tree holding.
        let Some(typeface) = tree.typeface(leaf) else {
            continue;
        };
        let size = typeface.size.at(*layout, *short);
        let cell = fonts.cell(typeface.font, size);
        let width = match tree.lettering(leaf) {
            Some(value) => shaping
                .shape(fonts, typeface.font, size, value)
                .max_content(),
            None => 0.0,
        };
        elements.cell[at] = cell;
        // The height half is R2m's, and is written before anything reads it.
        elements.intrinsic[at] = Area::new(width, 0.0);
    }
}

/// R2a, R2m and R2b: the horizontal axis, then the measure it makes possible, then the vertical one.
///
/// An element with a placement in motion is resolved twice on each axis -- once for each endpoint,
/// in the *same* context -- and the two answers are blended. That is what the resolver's purity
/// buys: the endpoints are consistent with each other because they were asked at the same position
/// in the same dependency order, against the same settled ancestors, and neither is remembered
/// afterwards.
fn axes(grove: &mut Grove, elements: &mut Elements) -> Vec<Option<Stretched>> {
    let mut ends: Vec<Option<Stretched>> = vec![None; elements.len()];
    axis(grove, elements, Axis::Horizontal, &mut ends);
    wrap(grove, elements);
    axis(grove, elements, Axis::Vertical, &mut ends);
    ends
}

/// One axis of the whole tree, in dependency order, through the one pure resolver.
fn axis(grove: &Grove, elements: &mut Elements, axis: Axis, ends: &mut [Option<Stretched>]) {
    let _pass = trace_span!("axis", vertical = (axis == Axis::Vertical)).entered();
    let viewport = Section::new(Position::default(), grove.viewport);
    let fallback = Location::default();
    for at in 0..elements.len() {
        // R2m may have found a measure that moved under an element nothing was written to, between
        // this axis and the last. The order puts what an element reads before it, so spreading that
        // here reaches everything resolved against it.
        elements.dirty[at] |= elements.trunk[at].is_some_and(|on| elements.dirty[on])
            || elements.anchor[at].is_some_and(|on| elements.dirty[on]);
        // Nothing it or anything it resolves against was written, so it resolves to the box it
        // already holds. Its column keeps that box, so what reads it here reads the right one.
        if !elements.dirty[at] {
            continue;
        }
        let leaf = elements.order[at];
        let context = context(elements, viewport, at, axis);
        let (span, stretched) = geometry(grove, leaf, &fallback, &context, axis);
        if let Some((from, to)) = stretched {
            ends[at]
                .get_or_insert_with(Stretched::default)
                .set(axis, from, to);
        }
        let section = &mut elements.section[at];
        match axis {
            Axis::Horizontal => {
                section.position.x = span.near;
                section.area.width = span.extent();
            }
            Axis::Vertical => {
                section.position.y = span.near;
                section.area.height = span.extent();
            }
        }
        elements.resolved[at] = true;
    }
}

/// One axis of `leaf`, whichever of the two ways its placement is stated.
///
/// A box resolves to a span directly. A **trace** resolves to two positions, and the span is the
/// distance between them grown by half the stroke's weight on each side -- which is what gives a
/// rule, whose ends share a coordinate on one axis, a box on that axis at all. Both ends come back
/// too, because the span cannot say which of its diagonals they are.
fn geometry(
    grove: &Grove,
    leaf: Leaf,
    fallback: &Location,
    context: &Context,
    axis: Axis,
) -> (Span, Option<(f32, f32)>) {
    let Some(traced) = grove.tree.traced(leaf) else {
        return (span(grove, leaf, fallback, context, axis), None);
    };
    let half = grove.tree.stroke(leaf).unwrap_or_default().half();
    let (from, to) = (locate(&traced.from, context), locate(&traced.to, context));
    (
        Span {
            near: from.min(to) - half,
            far: from.max(to) + half,
        },
        Some((from, to)),
    )
}

/// One axis of where `leaf` currently is: what it declares, blended with the endpoint a motion left.
///
/// The one place a placement becomes a span, so a measure and a layout are answering the same
/// question -- an element in motion is measured where it *is* rather than where it is going.
fn span(grove: &Grove, leaf: Leaf, fallback: &Location, context: &Context, axis: Axis) -> Span {
    let location = grove.tree.location(leaf).unwrap_or(fallback);
    let target = resolve(pinned(location, grove, axis), context);
    match grove.aspen.location(leaf) {
        Some((departed, at)) => departure(departed, grove, context, axis).blend(target, at),
        None => target,
    }
}

/// How one axis of a placement is pinned down, at the breakpoint in force.
///
/// A lookup rather than arithmetic, which is why it is here and not in the resolver: picking a
/// configuration out of a placement is not part of resolving one.
fn pinned<'a>(location: &'a Location, grove: &Grove, axis: Axis) -> &'a Config {
    let axes = location.axes(grove.layout, grove.short);
    match axis {
        Axis::Horizontal => &axes.horizontal,
        Axis::Vertical => &axes.vertical,
    }
}

/// One axis of the endpoint a motion left.
///
/// A placement it left is resolved in the context the target was resolved in, so anything that
/// moves one end moves both. A snapshot is already an answer: it is the box the element was blended
/// to when it was retargeted, and it never corresponded to a placement that could be re-resolved.
fn departure(
    departed: &Departed<Location, Section>,
    grove: &Grove,
    context: &Context,
    axis: Axis,
) -> Span {
    match departed {
        Departed::Declared(location) => resolve(pinned(location, grove, axis), context),
        Departed::Snapshot(section) => Span::of(*section, axis),
    }
}

/// R2m. How tall each element's contents turned out at the width R2a gave it.
///
/// Bottom-up, so everything inside an element is measured before the element asks. Two things are
/// measured, and an element takes the greater of them, because both answer the one question
/// [`content()`](crate::content) asks -- *how large is what is inside me*:
///
/// - a run of glyphs wraps at its own resolved width, and its lines times its cell is its height
/// - anything with elements grown under it takes the furthest any of them reaches down
///
/// The second is resolved in the element's own space, with its vertical extent taken as **not yet
/// known**, which is exactly what it is at this point in the frame. So a child that reads that
/// extent -- `100.pct()`, a row of a grid, an anchor's edge -- contributes nothing to the measure
/// and is given its real height by R2b like anything else. That is the correct reading rather than a
/// limitation: a child sized to its trunk cannot also be what sizes it, and nothing is asked to
/// converge.
/// The two halves are separate walks because only one of them has an order. A run wraps at its own
/// width and against its own font, so what an element shapes to reads nothing but the element --
/// while what reaches below it reads the measures of everything under it, and so has to run after
/// them.
fn wrap(grove: &mut Grove, elements: &mut Elements) {
    let _pass = trace_span!("wrap").entered();
    // What has to measure again: what was written to, and everything above it, because what an
    // element reaches over is what the elements under it resolved to. Up the order rather than down
    // it -- a measure travels toward the trunk and stops there, where a box travels away from it.
    let mut measuring = elements.measures.clone();
    for at in (0..elements.len()).rev() {
        if let (true, Some(trunk)) = (measuring[at], elements.trunk[at]) {
            measuring[trunk] = true;
        }
    }
    let shaped = shaped(grove, elements, &measuring);
    let _half = trace_span!("reach").entered();
    let fallback = Location::default();
    for at in (0..elements.len()).rev() {
        if !measuring[at] {
            continue;
        }
        let height = shaped[at].max(reach(grove, elements, &fallback, at));
        if height != elements.intrinsic[at].height {
            elements.intrinsic[at].height = height;
            // The measure moved under an element nothing was written to, and a placement reading
            // that measure has to be resolved against the new one. R2b runs after this and spreads
            // it, which is why it is safe to find out here.
            elements.dirty[at] = true;
        }
    }
}

/// How tall each element's own run turned out at the width R2a gave it.
///
/// In any order, because no element's answer is another's: this is the half of R2m that reaches into
/// the world for a typeface and a run, and the only half that touches the shaping cache.
///
/// Shaping a run is what states that the run is still wanted, and the sweep drops what this frame did
/// not state -- so on a frame that is going to sweep, every element is shaped whether or not it is
/// measuring again. A frame where no run stopped being stated has nothing to sweep, and shapes only
/// what it is measuring.
fn shaped(grove: &mut Grove, elements: &Elements, measuring: &[bool]) -> Vec<f32> {
    let _half = trace_span!("shaped").entered();
    let mut shaped = vec![0.0; elements.len()];
    for at in 0..elements.len() {
        if !(measuring[at] || elements.restated) {
            continue;
        }
        shaped[at] = wrapped(grove, elements.order[at], elements.section[at].width());
    }
    shaped
}

/// How tall `leaf`'s own run of glyphs is at `width`, or zero if it says nothing.
fn wrapped(grove: &mut Grove, leaf: Leaf, width: f32) -> f32 {
    let Grove {
        tree,
        fonts,
        shaping,
        layout,
        short,
        ..
    } = grove;
    let Some(typeface) = tree.typeface(leaf) else {
        return 0.0;
    };
    let Some(value) = tree.lettering(leaf) else {
        return 0.0;
    };
    let size = typeface.size.at(*layout, *short);
    shaping
        .shape(fonts, typeface.font, size, value)
        .measure(width)
}

/// How far the elements grown under `leaf` reach below its top edge.
///
/// Only the children that describe their own extent are counted. One that reads a vertical box --
/// a percentage of this element, a row of its grid, an anchor's edge -- is asking how tall
/// something else is, so it cannot be what decides how tall this is. See
/// [`Config::measurable`](crate::placement::role::Config::measurable).
fn reach(grove: &Grove, elements: &Elements, fallback: &Location, at: usize) -> f32 {
    let mut reach: f32 = 0.0;
    for child in grove.tree.branched(elements.order[at]) {
        let Some(child_at) = elements.at(child) else {
            continue;
        };
        if !measurable(grove, child, fallback) {
            continue;
        }
        let context = raised(elements, at, child_at);
        reach = reach.max(
            geometry(grove, child, fallback, &context, Axis::Vertical)
                .0
                .far,
        );
    }
    reach
}

/// Whether `leaf`'s vertical placement describes its own extent, and so counts toward the measure
/// of what it is grown under.
///
/// One question with two spellings, because a placement has two. A box asks it of its vertical
/// configuration; a trace asks it of both of its ends, since either one reading a vertical box is
/// enough to make the answer circular.
fn measurable(grove: &Grove, leaf: Leaf, fallback: &Location) -> bool {
    match grove.tree.traced(leaf) {
        Some(traced) => traced.from.measurable() && traced.to.measurable(),
        None => pinned(
            grove.tree.location(leaf).unwrap_or(fallback),
            grove,
            Axis::Vertical,
        )
        .measurable(),
    }
}

/// What one child resolves its vertical axis against while its trunk is being measured.
///
/// The same [`Context`] R2b will build, with every vertical reading taken as zero: no box on this
/// axis has resolved yet, which is the point of measuring. Every horizontal reading is real -- the
/// child's own width included -- so a height stated in columns, read off a width, or held in
/// proportion to its own still answers.
fn raised(elements: &Elements, trunk: usize, child: usize) -> Context {
    let offered = |at: usize| Basis {
        section: flattened(elements.section[at]),
        intrinsic: elements.intrinsic[at],
        tracks: elements.tracks[at],
        cell: elements.cell[at],
    };
    Context {
        axis: Axis::Vertical,
        own: own(elements, child, Axis::Vertical),
        trunk: offered(trunk),
        anchor: match elements.anchor[child] {
            Some(anchor) => offered(anchor),
            None => Basis::default(),
        },
    }
}

/// R3. How far each region's content reaches, from where its children landed.
///
/// Bottom-up, so a subtree is measured before whatever contains it. Two things are deliberately not
/// consulted: what is currently drawn, because content scrolled out of sight is exactly what an
/// extent describes and has to remain reachable; and the offset, because measuring against it would
/// make the extent depend on the clamp that depends on the extent.
///
/// A child that scrolls in its own right contributes its box and not its content -- what overflows
/// inside it is its own to reach, and is already reachable there.
///
/// A **visible child that is simply far away is content**, and is counted. That is the answer
/// `views.md` gives to the parking footgun, and it is deliberately not a guess about intent: if it
/// is visible and out there it is reachable, which is coherent where the old behaviour was a
/// surprise. What removes a child from the extent is the app saying so -- [`visible(false)`] takes
/// it and its subtree out, and [`pinned`] takes out something that is inside the region without
/// travelling with it.
///
/// [`visible(false)`]: crate::Place::visible
/// [`pinned`]: crate::Place::pinned
fn extent(grove: &mut Grove, elements: &Elements) {
    let _pass = trace_span!("extent").entered();
    // The far corner of everything under an element, in absolute coordinates. Absent where the
    // element was skipped, which is what keeps a hidden subtree out of what contains it.
    let mut reach: Vec<Option<Position>> = vec![None; elements.len()];
    for at in (0..elements.len()).rev() {
        let leaf = elements.order[at];
        // A hidden element is not content, and neither is anything under it: that is what makes
        // hiding the whole answer for parking something out of the way, rather than half of one.
        if !grove.tree.visible(leaf).0 {
            continue;
        }
        let section = elements.section[at];
        let mut far = Position::new(section.right(), section.bottom());
        for child in grove.tree.branched(leaf) {
            // The two ways an element grown inside a region is not part of its content, and each
            // is left out here by the same one declaration that says the rest of what it means --
            // which is what keeps the halves from disagreeing.
            //
            // A pinned child does not move with the content. A floating one moves with it but sits
            // over the region rather than in it, so it is not content either: an overlay that
            // invented room to scroll to is the scrollbar nobody ordered.
            if grove.tree.pinned(child) || grove.tree.floats(child).is_some() {
                continue;
            }
            let Some(child) = elements.at(child).and_then(|child| reach[child]) else {
                continue;
            };
            far = Position::new(far.x.max(child.x), far.y.max(child.y));
        }
        if let Some(scroll) = grove.tree.scrolls(leaf) {
            grove.tree.set_extent(leaf, reached(scroll, section, far));
            far = Position::new(section.right(), section.bottom());
        }
        reach[at] = Some(far);
    }
}

/// How far a region's content reaches, on each axis.
///
/// Three rules, and each of them is what stops a scrollbar nobody ordered:
///
/// - **only along a declared axis.** An axis the region does not scroll reads its own box and
///   nothing else, so a child extending sideways out of a column that scrolls down is simply out of
///   frame. Most accidental extent came from the axis nobody was scrolling.
/// - **measured outward from the content origin, clamped at the near side.** A child at a negative
///   offset creates no range to scroll *back* into: content above the origin is a layout mistake,
///   and the previous behaviour turned it into a feature.
/// - **never smaller than the region's own box**, so an empty region has a range of zero rather
///   than a negative one.
///
fn reached(scroll: Scroll, section: Section, far: Position) -> Area {
    let along = |axis: Axis, reach: f32, near: f32| {
        let own = section.area.along(axis);
        if !scroll.covers(axis) {
            return own;
        }
        (reach - near).max(own)
    };
    Area::new(
        along(Axis::Horizontal, far.x, section.left()),
        along(Axis::Vertical, far.y, section.top()),
    )
}

/// R4. Where each element is drawn: where the layout put it, less what its scrolling ancestors have
/// moved.
///
/// A region's own box does not move under its own offset -- what moves is everything grown inside
/// it. The offset is clamped here, against the extent R3 just measured, so a region whose content
/// shrank under it comes back into range on the next frame rather than staying somewhere it can no
/// longer reach.
///
/// The three ways a region is *asked* to move -- a coast still running from a release, a
/// [`scroll`](crate::Grow::scroll) written this frame, and a
/// [`Motion::Scroll`](crate::Motion::Scroll) part way through -- are answered first, here rather
/// than where they were written, because all three need the extent and the extent is one pass old.
fn scroll(grove: &mut Grove, elements: &mut Elements, ends: &[Option<Stretched>]) {
    let _pass = trace_span!("scroll", coasting = grove.coasting.len()).entered();
    view::asked(grove, elements);
    let mut accumulated: Vec<Position> = vec![Position::default(); elements.len()];
    // What a *pinned* child of each element receives, which is the accumulation with its nearest
    // scrolling ancestor left out of it.
    let mut unpinned: Vec<Position> = vec![Position::default(); elements.len()];
    for at in 0..elements.len() {
        let leaf = elements.order[at];
        let placed = elements.section[at];
        let trunk = elements.trunk[at];
        let inherited = trunk.map(|trunk| accumulated[trunk]).unwrap_or_default();
        let outside = trunk.map(|trunk| unpinned[trunk]).unwrap_or_default();
        // A pinned element does not receive its nearest scrolling ancestor's offset, and receives
        // every offset outside that one: pinning is relative to the region the element sits in and
        // says nothing about what contains that region.
        let applied = match grove.tree.pinned(leaf) {
            true => outside,
            false => inherited,
        };
        let drawn = Section::new(
            Position::new(placed.left() - applied.x, placed.top() - applied.y),
            placed.area,
        );
        elements.moved[at] |= grove.tree.settle(leaf, placed, drawn);
        // A stroke's ends travel with its box, by the same offset, because they are the same
        // geometry said two ways.
        if let Some(stretched) = &ends[at] {
            grove.tree.set_stretched(leaf, stretched.less(applied));
        }
        let (carried, escaped) = match grove.tree.scrolls(leaf) {
            Some(scroll) => {
                let clamped = clamp(grove, leaf, scroll, placed);
                (
                    Position::new(applied.x + clamped.x, applied.y + clamped.y),
                    // A pinned child of this element is pinned to *this* region, so its offset is
                    // the accumulation as it stood before this region's own.
                    applied,
                )
            }
            // Nothing here to be pinned against, so a pinned child of this element is pinned to
            // whatever region contains it, exactly as a pinned sibling of this element would be.
            None => (applied, outside),
        };
        accumulated[at] = carried;
        unpinned[at] = escaped;
    }
}

/// A region's offset, held to what it can actually reach.
///
/// An axis that was not declared does not scroll, so nothing can have moved along it -- and a
/// region whose extent shrank under it is brought back into range here rather than left somewhere
/// it can no longer get to.
fn clamp(grove: &mut Grove, leaf: Leaf, scroll: Scroll, placed: Section) -> Position {
    let offset = grove.tree.offset(leaf);
    let extent = grove.tree.extent(leaf);
    let mut clamped = Position::default();
    for axis in Axis::BOTH {
        if scroll.covers(axis) {
            clamped = clamped.set(
                axis,
                offset
                    .along(axis)
                    .clamp(0.0, range(extent, placed.area, axis)),
            );
        }
    }
    if clamped != offset {
        grove.tree.set_offset(leaf, clamped);
    }
    clamped
}

/// R5. What a scrolling ancestor leaves visible of each element.
///
/// A rect, and nothing else. A region does not clip itself, only what is grown inside it, and an
/// element with no scrolling ancestor is clipped by nothing at all. Whether an element is *culled*
/// is extraction's decision from this rect, and is never recorded on the element -- so there is no
/// state saying "currently clipped away" for anything else, extent first among them, to read.
fn clip(grove: &mut Grove, elements: &mut Elements) {
    let _pass = trace_span!("clip").entered();
    let unbounded = Clipped::unbounded().0;
    let mut passed: Vec<Section> = vec![unbounded; elements.len()];
    // What a *floating* child of each element is clipped to, which is the intersection with its
    // nearest scrolling ancestor's own rect left out of it. The same second accumulation R4 carries
    // for a pinned child, and for the same reason: both say "not part of this region", and both
    // have to say it about the region the element is actually in rather than about all of them.
    let mut escaped: Vec<Section> = vec![unbounded; elements.len()];
    for at in 0..elements.len() {
        let leaf = elements.order[at];
        let trunk = elements.trunk[at];
        let inherited = trunk.map(|trunk| passed[trunk]).unwrap_or(unbounded);
        let outside = trunk.map(|trunk| escaped[trunk]).unwrap_or(unbounded);
        // A floating element is positioned outside the region on purpose, so cutting it off at the
        // region's edge would undo the placement that put it there. How far out it reaches is the
        // element's own statement, because no one answer is right everywhere.
        let applied = match grove.tree.floats(leaf) {
            // Held by whatever holds the region it left.
            Some(Escape::Region) => outside,
            // Held by nothing, which extraction reads as the surface: a clip is never wider than
            // what is being drawn on.
            Some(Escape::Surface) => unbounded,
            // Held by the element named, which is what that element passes down to what it holds.
            Some(Escape::Within(named)) => match within(elements, &passed, at, named) {
                Some(clip) => clip,
                // Named something that is not above it, so it holds nothing. Falling back to the
                // near answer rather than the far one: an element escapes no further than it was
                // told to, and a mis-named ancestor is not permission to leave everything.
                None => outside,
            },
            None => inherited,
        };
        elements.moved[at] |= grove.tree.set_clip(leaf, applied);
        let scrolls = grove.tree.scrolls(leaf).is_some();
        passed[at] = match scrolls {
            true => applied.intersect(grove.tree.drawn(leaf)),
            false => applied,
        };
        escaped[at] = match scrolls {
            true => applied,
            false => outside,
        };
    }
}

/// What `named` leaves visible of what it holds, if it is above `leaf` at all.
///
/// The walk is up the trunks from `leaf`, so an element that names something beside it rather than
/// above it gets `None` and is answered as though it had named the region it is in. Naming the
/// element rather than counting regions is what makes this survive a wrapper being added between
/// the two, which a count would not.
fn within(elements: &Elements, passed: &[Section], at: usize, named: Leaf) -> Option<Section> {
    let named = elements.at(named)?;
    let mut step = elements.trunk[at];
    while let Some(above) = step {
        // An ancestor resolves before what hangs off it, so one found here has already passed its
        // own clip down and the read is never of a position the pass has yet to reach.
        if above == named {
            return Some(passed[above]);
        }
        step = elements.trunk[above];
    }
    None
}

/// R7. The three off-states, resolved over each element's whole ancestry.
///
/// One walk, in the same order the axes used, which puts every trunk before what hangs off it.
/// Nothing has a cascade to write: an element grown under a disabled trunk is disabled on its first
/// frame because the pass does not care when it arrived, and enabling that trunk leaves anything
/// disabled in its own right disabled because the product is over the whole ancestry rather than a
/// single bit that was overwritten on the way down.
fn inherit(grove: &mut Grove, elements: &mut Elements) {
    let _pass = trace_span!("inherit").entered();
    let mut products: Vec<Inherited> = vec![Inherited::default(); elements.len()];
    for at in 0..elements.len() {
        let leaf = elements.order[at];
        let trunk = elements.trunk[at]
            .map(|trunk| products[trunk])
            .unwrap_or_default();
        let product = Inherited::under(
            trunk,
            grove.tree.visible(leaf),
            grove.tree.opacity(leaf),
            grove.tree.disabled(leaf),
        );
        products[at] = product;
        elements.moved[at] |= grove.tree.set_inherited(leaf, product);
    }
}

/// R8. The box stack the next frame's dispatch reads.
///
/// Membership is universal: an element is here because it is there. What is left out is only what
/// is not there at all -- hidden, fully transparent, or clipped away by a region it sits inside.
/// `intangible` is not a way out of the stack; it is carried on the region and decides what may be
/// the top of it.
fn regions(grove: &mut Grove, elements: &Elements) {
    let _pass = trace_span!("regions").entered();
    let mut ranked = Vec::with_capacity(elements.len());
    for &leaf in &elements.order {
        let inherited = grove.tree.inherited(leaf);
        if !inherited.present() {
            continue;
        }
        let section = grove.tree.drawn(leaf);
        let clip = grove.tree.clip(leaf);
        if section.intersect(clip).is_empty() {
            continue;
        }
        let gestures = grove.tree.gestures(leaf);
        ranked.push((
            grove.tree.rank(leaf),
            Region {
                leaf,
                section,
                clip,
                shape: gestures.shape,
                tangible: !gestures.intangible,
                receives: gestures.receives,
                disabled: inherited.disabled,
            },
        ));
    }
    grove.stack.settle(ranked);
}

/// R6. Declared elevation accumulates down the tree, and allocation order settles what it leaves
/// equal.
///
/// One walk in the same dependency order the axes used, which puts every trunk before what hangs
/// off it. Nothing here reads a box: where an element sits in the stack has nothing to do with
/// where it sits on the surface.
fn rank(grove: &mut Grove, elements: &mut Elements) {
    let _pass = trace_span!("rank").entered();
    let mut stacks: Vec<i32> = vec![0; elements.len()];
    for at in 0..elements.len() {
        let leaf = elements.order[at];
        let trunk = elements.trunk[at]
            .map(|trunk| stacks[trunk])
            .unwrap_or_default();
        let stack = grove.tree.elevation(leaf).accumulate(trunk);
        stacks[at] = stack;
        elements.moved[at] |= grove.tree.set_rank(
            leaf,
            ResolvedElevation {
                stack,
                growth: grove.tree.growth(leaf).0,
            },
        );
    }
}

/// Everything one axis of the element at `at` resolves against.
fn context(elements: &Elements, viewport: Section, at: usize, axis: Axis) -> Context {
    Context {
        axis,
        // Its box on this axis is the answer being computed, and its own grid divides it for its
        // children rather than for itself, so neither is readable here. Its box on the *other* axis
        // is readable exactly when that axis has already run, which is what [`own`] hands over.
        own: own(elements, at, axis),
        // A top-level element has no trunk, and fills the viewport instead.
        trunk: basis(elements, elements.trunk[at], viewport, axis),
        // A placement that reads an anchor it has not been given resolves against a zero box.
        anchor: basis(elements, elements.anchor[at], Section::default(), axis),
    }
}

/// What the element at `at` offers its own placement.
///
/// Its measured extent and its character cell on either axis, and its width on the vertical one:
/// the horizontal pass has run for the whole tree by then, so the width is settled where the
/// height is the answer being computed. That is what [`aspect`](crate::aspect) reads, and it is
/// offered flattened the way [`basis`] flattens a resolved box -- with nothing on the axis being
/// resolved -- so nothing about the box that is still open can be read back.
///
/// Nothing on the horizontal pass, where neither axis of it is known.
fn own(elements: &Elements, at: usize, axis: Axis) -> Basis {
    Basis {
        section: match axis {
            Axis::Horizontal => Section::default(),
            Axis::Vertical => flattened(elements.section[at]),
        },
        intrinsic: elements.intrinsic[at],
        tracks: Tracks::default(),
        cell: elements.cell[at],
    }
}

/// A box with its vertical half taken away: its left edge and its width, and nothing on the other
/// axis. What a box is worth to a reader while no height is known.
fn flattened(section: Section) -> Section {
    Section::new(
        Position::new(section.left(), 0.0),
        Area::new(section.width(), 0.0),
    )
}

/// What the element at `at` offers a placement reading it, or `fallback` in place of a box when
/// there is no such element or it has not resolved yet.
///
/// **A resolved box offers no vertical reading on the horizontal axis.** R2a runs before any height
/// is known, so a box it offers is flattened to the horizontal -- which is what one held anyway when
/// every element was resolved every frame, and what it has to go on holding now that an element
/// nothing wrote to comes into the pass carrying the whole of last frame's box.
///
/// The fallback is not flattened, because it is not a resolved box: the viewport a top-level element
/// fills is as tall on this axis as on the other one.
fn basis(elements: &Elements, at: Option<usize>, fallback: Section, axis: Axis) -> Basis {
    let Some(at) = at else {
        return Basis {
            section: fallback,
            ..Basis::default()
        };
    };
    Basis {
        section: match (elements.resolved[at], axis) {
            (true, Axis::Horizontal) => flattened(elements.section[at]),
            (true, Axis::Vertical) => elements.section[at],
            (false, _) => fallback,
        },
        intrinsic: elements.intrinsic[at],
        tracks: elements.tracks[at],
        cell: elements.cell[at],
    }
}

/// The frame's one read of the tree: every live element, ordered so that nothing resolves before
/// what it depends on, with what it depends on and what it offers read out beside it.
///
/// An anchor may point anywhere -- a later sibling, a cousin, an element in another subtree -- so
/// this is ordered by dependency rather than by tree depth: an element resolves after its parent
/// *and* after whatever it anchors to.
///
/// A chain of anchors cannot close on itself, because one that would is refused where it is written.
/// A trunk and an anchor still can, between them: an element anchored to something grown under it
/// waits on a box that is waiting on its own. That is not the same contradiction -- both boxes
/// resolve, just not both against a settled other -- so the remainder is ordered by allocation order
/// and resolves against what its dependency last was. **Every live element is in the order**, which
/// is the property the passes downstream rely on: an element left out would have no box, no rank and
/// no place in the stack, and nothing would say so.
///
/// Each element is asked what it depends on exactly once here, and every position after that is
/// arithmetic on the answer.
///
/// Kept between frames and repaired rather than built again. The order is what the elements and the
/// edges between them make, and only growing, withering and anchoring change those -- so an ordinary
/// frame reads none of it, and rebuilds only the columns of the elements it is going to resolve.
fn structure(elements: &mut Elements, grove: &Grove) {
    let _pass = trace_span!("structure").entered();
    let tree = &grove.tree;
    let leaves = tree.leaves();
    let count = leaves.len();
    // Where each element sits among the leaves, and below -- once the order is known -- where it
    // sits in the order. Re-pointed rather than rebuilt, so a frame builds one map and not two.
    let mut index: HashMap<Leaf, usize> = HashMap::with_capacity(count);
    for (at, &leaf) in leaves.iter().enumerate() {
        index.insert(leaf, at);
    }
    // What each element waits on, and how many of those have yet to resolve. An element the leaves
    // do not hold is not live, since the leaves are what the world has grown, and a dependency on
    // something not live is no dependency at all.
    let mut depends: Vec<[Option<usize>; 2]> = Vec::with_capacity(count);
    let mut waiting: Vec<u8> = Vec::with_capacity(count);
    // Everything waiting on each element, as one run per element in a single array: an element
    // depends on at most two others, so the whole graph fits in one allocation rather than in a
    // vector per element.
    let mut runs: Vec<u32> = vec![0; count + 1];
    for &leaf in &leaves {
        let on = [tree.trunk(leaf), tree.anchor(leaf)]
            .map(|on| on.and_then(|on| index.get(&on).copied()));
        let mut unresolved = 0;
        for at in on.into_iter().flatten() {
            unresolved += 1;
            runs[at + 1] += 1;
        }
        depends.push(on);
        waiting.push(unresolved);
    }
    for at in 0..count {
        runs[at + 1] += runs[at];
    }
    let mut filling = runs.clone();
    let mut dependents: Vec<u32> = vec![0; runs[count] as usize];
    for (at, on) in depends.iter().enumerate() {
        for &upon in on.iter().flatten() {
            dependents[filling[upon] as usize] = at as u32;
            filling[upon] += 1;
        }
    }
    // Kahn's, with the order under construction serving as its own queue.
    let mut order: Vec<usize> = (0..count).filter(|at| waiting[*at] == 0).collect();
    let mut next = 0;
    while next < order.len() {
        let at = order[next];
        next += 1;
        for &dependent in &dependents[runs[at] as usize..runs[at + 1] as usize] {
            let dependent = dependent as usize;
            waiting[dependent] -= 1;
            if waiting[dependent] == 0 {
                order.push(dependent);
            }
        }
    }
    if order.len() != count {
        order.extend((0..count).filter(|at| waiting[*at] != 0));
    }
    // Where each element ended up, which is what every position held below is in.
    let mut ranked: Vec<usize> = vec![0; count];
    for (position, &at) in order.iter().enumerate() {
        ranked[at] = position;
    }
    for position in index.values_mut() {
        *position = ranked[*position];
    }
    // The positions all moved, so every column is read out of the tree again. What resolution
    // settled is on the elements themselves -- `scatter` and R4 put it there -- so this is a read of
    // last frame's answers into this frame's order, and not a loss of them.
    elements.order.clear();
    elements.trunk.clear();
    elements.anchor.clear();
    elements.section.clear();
    elements.intrinsic.clear();
    elements.cell.clear();
    elements.chlorophyll.clear();
    elements.index = index;
    for &at in &order {
        let leaf = leaves[at];
        let [trunk, anchor] = depends[at];
        elements.order.push(leaf);
        elements.trunk.push(trunk.map(|on| ranked[on]));
        elements.anchor.push(anchor.map(|on| ranked[on]));
        elements.chlorophyll.push(tree.chlorophyll(leaf));
        elements.section.push(tree.placed(leaf));
        elements.intrinsic.push(tree.intrinsic(leaf));
        elements.cell.push(tree.cell(leaf));
    }
    elements.tracks.resize(count, Tracks::default());
    elements.tracks.truncate(count);
}

/// What this frame has to resolve, and what it may leave alone.
///
/// An element resolves again if it was written to, or if what it hangs off or is anchored to does.
/// That closure is one walk of the order rather than a search, because the order already puts a
/// dependency before what depends on it.
///
/// Where most of the tree was written to, working out what to leave alone costs more than resolving
/// it does: a lookup for every element, three columns for the passes to read, and a skip that almost
/// never takes. Past a quarter it is cheaper to resolve everything, and the whole of the closure
/// falls away with one branch.
fn read(elements: &mut Elements, grove: &Grove) {
    let _pass = trace_span!("elements").entered();
    let written = grove.tree.written();
    if written.restructured {
        structure(elements, grove);
    }
    let count = elements.order.len();
    let all = written.all || written.declared.len() * 4 >= count;
    elements.restated = written.restated;
    elements.dirty.clear();
    elements.measures.clear();
    elements.resolved.clear();
    elements.moved.clear();
    for at in 0..count {
        let leaf = elements.order[at];
        // A dependency the order could not put first is one a cycle left in the remainder, and it
        // has no answer yet. Resolving such an element is the same fallback the rest of resolution
        // gives it: it is answered against what its dependency last was, so it is answered again.
        let decided = |on: Option<usize>| {
            on.is_some_and(|on| elements.dirty.get(on).copied().unwrap_or(true))
        };
        let dirty = all
            || written.declared.contains(&leaf)
            || decided(elements.trunk[at])
            || decided(elements.anchor[at]);
        elements.dirty.push(dirty);
        elements.moved.push(dirty);
        elements
            .measures
            .push(dirty || written.measures.contains(&leaf));
        // A clean element offers the box it settled at, and offers it from the start, because
        // nothing this frame is going to compute it again. One being resolved again offers nothing
        // until an axis reaches it, which is what it did when every element was resolved.
        elements.resolved.push(!dirty);
        // The grid is divided at the breakpoint in force, so it is read again wherever the element
        // is -- a breakpoint that moved is every element written to, and nothing else can change a
        // division without writing the grid it divides.
        if dirty {
            elements.tracks[at] = grove
                .tree
                .grid(leaf)
                .unwrap_or_default()
                .tracks(grove.layout, grove.short);
        }
    }
}
