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
//! A write that moves nothing is recorded apart from one that can
//! ([`Tree::restyled`](crate::tree::Tree::restyled)): a fill, a rounding, a tint or a shape is read
//! by extraction alone, and whether an element is shown, how opaque, whether disabled and how far
//! forward are read by the passes below that run over every element anyway. Such an element is read
//! again and drawn again, and nothing is placed again on its account.
//!
//! What still runs over everything is the accumulations, R3 through R7: what they carry is a running
//! product rather than an answer per element, so there is no position in them to start from. They
//! record what they actually changed as they go, which is what extraction then states -- and R8, the
//! box stack, is built again only on a frame where one of them changed something it is built from.
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
//! | R1 | measure | — | the character cell, and the max-content width half of the intrinsic size |
//! | R2a | horizontal | dependency order | the horizontal axis |
//! | R2m | wrap | **bottom-up** | the measured-height half of the intrinsic size |
//! | R2b | vertical | dependency order | the vertical axis, and with it the placed box |
//! | R3 | extent | **bottom-up** | [`Extent`](crate::view::Extent): how far a region's content reaches |
//! | R4 | scroll | top-down | the drawn box: the placed box less every scrolling ancestor's offset |
//! | R5 | clip | top-down | the clip: what a scrolling ancestor leaves visible |
//! | R6 | rank | dependency order | `ResolvedElevation`: elevation accumulated, then tie-broken |
//! | R7 | inherit | top-down | [`Inherited`]: the visible, opacity and disabled products |
//! | R8 | regions | rank order | the box stack the next frame's dispatch reads |
//!
//! Both halves of R2 call the same pure resolver, once per axis. That is only safe because it is
//! pure: there is no accumulated state for a second call to corrupt.
//!
//! R2a and R2b resolve what was written to, and R1 only what says something new -- a run rewritten,
//! an element grown, a breakpoint or a face that changed every cell. R2m resolves what was written to
//! and everything above it, because
//! what an element reaches over is what the elements under it resolved to -- a measure travels
//! toward the trunk where a box travels away from it. A measure that moves under an element nothing
//! wrote to makes that element resolve again, which is why R2m sits before R2b and not after it. And
//! an element measuring again because one of its children moved asks that child alone: how far each
//! of the others reaches is held from when it last resolved, which is the answer asking would give.
//!
//! # Every pass reads the order, not the world
//!
//! The passes ask the same questions of the same elements over and over: what an element hangs off,
//! what it is anchored to, what is grown under it, and what each of those offers a placement reading
//! it. Asked of the world each time, every one of those is a lookup by entity -- and the arithmetic a
//! pass does is small enough that the lookups, not the arithmetic, are what a frame costs.
//!
//! So the tree is read once, into [`Elements`]: every live element in dependency order, holding what
//! it depends on as **positions in that order** rather than as names, and holding what it offers a
//! placement beside it. From there every pass indexes -- the accumulations R3 through R7 carry are
//! arrays the length of the order, and a trunk's contribution is `[..]` rather than a lookup on a
//! map keyed by element.
//!
//! **What resolution produces is held there and nowhere else.** The placed and drawn boxes, the
//! measures, the clip, the rank and the inherited products are columns of the order, kept between
//! frames and read by position -- by extraction, which walks the same order -- or by name through
//! [`Elements`]'s accessors, by anything that asks about one element. None of it is written back
//! into the world, so no pass pays to store a value per element per frame that the next pass would
//! have to look up again.
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

use std::sync::Arc;

use tracing::field::Empty;
use tracing::trace_span;

use crate::aspen::{Departed, blend};
use crate::coordinate::{Area, Axis, Position, Section};
use crate::elevation::{Elevation, ResolvedElevation};
use crate::elm::Chlorophyll;
use crate::grove::Grove;
use crate::interaction::Gestures;
use crate::interaction::stack::Region;
use crate::leaf::Leaf;
use crate::lifecycle::Inherited;
use crate::line::Stretched;
use crate::placement::grid::Tracks;
use crate::placement::location::Location;
use crate::placement::resolve::{Basis, Context, Span, locate, resolve};
use crate::placement::role::Config;
use crate::placement::trace::{Ends, Trace};
use crate::text::shape::Shaped;
use crate::tree::Placement;
use crate::view::{self, Clipped, Escape, Scroll, range};

/// Everything the passes after the axes read about an element, and nothing that places it.
///
/// Held beside the order, one per element, so the passes that run over every element on every frame
/// read a column rather than look each of these up. Read out of the tree when the order is built,
/// and again for an element only where one of these was written -- which is a
/// [`restyled`](crate::tree::Tree::restyled) write, since the rest of it is fixed when the element
/// is grown.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Standing {
    /// What it draws. Fixed at growth.
    pub(crate) chlorophyll: Chlorophyll,
    /// Where it came in allocation order, which is what settles a tie in the stack. Fixed at growth.
    pub(crate) growth: u64,
    /// What it declared about gestures. Fixed at growth.
    pub(crate) gestures: Gestures,
    /// What it declared about scrolling, if it scrolls. Fixed at growth.
    pub(crate) scrolls: Option<Scroll>,
    /// Whether it stays put while its region's content slides under it. Fixed at growth.
    pub(crate) pinned: bool,
    /// How far it floats out of the regions above it. Fixed at growth.
    pub(crate) floats: Option<Escape>,
    /// Whether the app hid it, as against an ancestor of it.
    pub(crate) visible: bool,
    /// How opaque it was told to be, before its ancestry is taken into account.
    pub(crate) opacity: f32,
    /// Whether it was disabled in its own right.
    pub(crate) disabled: bool,
    /// How far in front of its trunk it was told to sit.
    pub(crate) elevation: Elevation,
}

impl Default for Standing {
    fn default() -> Self {
        Self {
            chlorophyll: Chlorophyll::None,
            growth: 0,
            gestures: Gestures::default(),
            scrolls: None,
            pinned: false,
            floats: None,
            visible: true,
            opacity: 1.0,
            disabled: false,
            elevation: Elevation::default(),
        }
    }
}

/// No element sits here.
const NOWHERE: u32 = u32::MAX;

/// Every live element in dependency order, with what each one resolves against, what each one offers
/// a placement that reads it, and everything resolution made of it.
///
/// The frame's one read of the tree, and what every pass indexes into. A trunk and an anchor are
/// asked for on both axes and on four of the passes after them; what an element offers is asked for
/// twice per element on each axis. Out of the world each time, every one of those is a lookup by
/// entity -- resolved once to a position in the order, each is an index.
///
/// The columns are as long as the order and are read at the position an element sits at, so a pass
/// that already knows where it is never asks a second time. They are kept between frames, which is
/// what lets an element nothing wrote to keep the answer it already has.
#[derive(Default)]
pub(crate) struct Elements {
    /// Every live element, ordered so that nothing resolves before what it depends on.
    pub(crate) order: Vec<Leaf>,
    /// Where each element sits in the order, by the index of its name. A slot may be stale -- one
    /// left by a name that withered -- which is why a lookup checks the order holds the name it
    /// asked for.
    slots: Vec<u32>,
    /// What each element hangs off, as a position in the order.
    trunk: Vec<Option<usize>>,
    /// What each element is anchored to, as a position in the order.
    anchor: Vec<Option<usize>>,
    /// What is grown directly under each element, as positions in the order: the children of the
    /// element at `at` are `children[branches[at]..branches[at + 1]]`. One array for the whole tree
    /// rather than one per element, and built with the order, so a pass that asks what is under an
    /// element never asks the world.
    branches: Vec<u32>,
    children: Vec<u32>,
    /// What each element declared that the passes after the axes read.
    pub(crate) standing: Vec<Standing>,
    /// The box, as far as the axes have resolved it: where the layout put the element, which is
    /// what its children resolve against.
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
    /// shaping cache worth sweeping.
    restated: bool,
    /// Whether anything extraction reads about the element moved this frame.
    ///
    /// Seeded from [`dirty`](Elements::dirty), because an element being resolved again may land
    /// somewhere else, and from a [`restyled`](crate::tree::Tree::restyled) write, and set by R4
    /// through R7 where what they wrote differs from what was held. An element it is false for is
    /// drawn exactly as the backend already has it.
    pub(crate) moved: Vec<bool>,
    /// Whether R1 has to measure the element again: it was grown, or what it says was rewritten, or
    /// every cell in the tree moved.
    lettered: Vec<bool>,
    /// Whether the element has to measure again, before R2m walks that up the trunks.
    ///
    /// [`dirty`](Elements::dirty) is most of it. The rest is an element something withered from,
    /// which has one fewer thing to reach over and nothing left below it to say so.
    measures: Vec<bool>,
    /// What the element measured to, which is what [`content()`](crate::content) reads. R1 writes
    /// the width -- max-content, free in a monospaced font -- and R2m the height it wrapped to at
    /// the width R2a gave it, which is the whole of width-down and height-up.
    intrinsic: Vec<Area>,
    /// How far the element reaches below the top of what it is grown under, as R2m last found it
    /// there, or `None` where its placement does not count toward that measure.
    ///
    /// What each element contributes to its trunk's [`reach`], held like any other answer so that a
    /// trunk measuring again because one of its children moved asks the rest nothing. A child's
    /// reach reads its own placement and the horizontal half of what it and its trunk and anchor
    /// resolved to, and each of those that moves makes it dirty -- so an entry is found again
    /// exactly where its element is, and a container of thousands with one child written pays for
    /// one child.
    reached: Vec<Option<f32>>,
    /// The element's own grid, divided at the breakpoint in force.
    tracks: Vec<Tracks>,
    /// The character cell the element's own font and size make, as R1 measured it. What
    /// [`letters`](crate::Source::letters) and a letter-pitched track are measured in; an element
    /// that named no font and no size has none.
    cell: Vec<Area>,
    /// The element's run as R1 shaped it, where it has one: what R2m wraps, what a placement
    /// reading a [`character`](crate::Anchor::character) of the element resolves against, and what
    /// extraction draws. Shared with the shaping cache rather than copied out of it, and what tells
    /// the cache the run is still stated.
    pub(crate) composed: Vec<Option<Arc<Shaped>>>,
    /// Where the layout put a stroke's two ends, as R2 settled them and before any scrolling
    /// ancestor moved them. `None` on everything placed by a box.
    ///
    /// The box cannot stand in for it: a box is the rectangle around the two ends grown by half the
    /// weight, and which of its two diagonals the stroke runs along is a fact the rectangle does not
    /// carry. A stroke in motion is settled at the blend, so this is also what a retarget
    /// snapshots.
    spanned: Vec<Option<Stretched>>,
    /// Where the element is on screen: its placed box less every scrolling ancestor's accumulated
    /// offset. What drawing, clipping and hit-testing read.
    ///
    /// Deliberately apart from the placed box. A change to that means the layout moved a box and
    /// its children have to follow; a change to this means only that the box moved under a scroll.
    pub(crate) drawn: Vec<Section>,
    /// Where a stroke's two ends landed: the spanned ends less the same offset the drawn box is.
    pub(crate) stretched: Vec<Option<Stretched>>,
    /// What a scrolling ancestor leaves visible of the element.
    pub(crate) clip: Vec<Section>,
    /// Where the element sits in the one stack.
    pub(crate) rank: Vec<ResolvedElevation>,
    /// What the three off-states resolved to over the element's whole ancestry.
    pub(crate) inherited: Vec<Inherited>,
    /// Whether anything the box stack is built from moved this frame: an element grown or withered,
    /// or a drawn box, a clip, a rank or an inherited product that changed. What R8 waits on.
    restack: bool,
    /// Every position, front-most first: the order the box stack is read in. Sorted again only when
    /// a rank moved, which is the only thing it is sorted on.
    stacking: Vec<u32>,
    /// Whether a rank moved this frame, or the order was built again.
    reranked: bool,
    /// Every element that withered since the order was last built, and what it drew. Read by
    /// extraction, which has to let go of whatever it was holding for them.
    pub(crate) gone: Vec<(Leaf, Chlorophyll)>,
}

impl Elements {
    /// How many elements are live.
    pub(crate) fn len(&self) -> usize {
        self.order.len()
    }

    /// Where `leaf` sits, or `None` if it is not in the order.
    fn at(&self, leaf: Leaf) -> Option<usize> {
        let at = *self.slots.get(leaf.0.index_u32() as usize)? as usize;
        (self.order.get(at) == Some(&leaf)).then_some(at)
    }

    /// The positions of everything grown directly under the element at `at`.
    fn children(&self, at: usize) -> impl Iterator<Item = usize> + '_ {
        self.children[self.branches[at] as usize..self.branches[at + 1] as usize]
            .iter()
            .map(|child| *child as usize)
    }

    /// Where the layout put `leaf`, which is what its children resolve against. A zero box for
    /// anything resolution has not reached.
    ///
    /// Resolution reads a box by position, because it holds every element in order. A coast, a
    /// sought region, a running motion and an app each name one element instead, so they ask this.
    pub(crate) fn placed(&self, leaf: Leaf) -> Section {
        self.at(leaf).map(|at| self.section[at]).unwrap_or_default()
    }

    /// Where `leaf` is on screen, which is what an app reads and what a hit test runs against.
    pub(crate) fn drawn(&self, leaf: Leaf) -> Section {
        self.at(leaf).map(|at| self.drawn[at]).unwrap_or_default()
    }

    /// What a scrolling ancestor leaves visible of `leaf`.
    #[allow(unused)]
    pub(crate) fn clip(&self, leaf: Leaf) -> Section {
        self.at(leaf)
            .map(|at| self.clip[at])
            .unwrap_or_else(|| Clipped::unbounded().0)
    }

    /// Where `leaf` sits in the one stack, as R6 last resolved it.
    #[allow(unused)]
    pub(crate) fn rank(&self, leaf: Leaf) -> ResolvedElevation {
        self.at(leaf).map(|at| self.rank[at]).unwrap_or_default()
    }

    /// What the three off-states resolved to over `leaf`'s whole ancestry, as R7 last computed it.
    pub(crate) fn inherited(&self, leaf: Leaf) -> Inherited {
        self.at(leaf)
            .map(|at| self.inherited[at])
            .unwrap_or_default()
    }

    /// What R1 shaped `leaf`'s run as, character for character: what is drawn, which a masked
    /// field's value is not.
    #[cfg(test)]
    pub(crate) fn shaped_text(&self, leaf: Leaf) -> Option<String> {
        self.composed[self.at(leaf)?]
            .as_ref()
            .map(|shaped| shaped.text())
    }

    /// The character cell of `leaf`'s own font, at its own size, as R1 measured it.
    ///
    /// Per element rather than per engine: an app registers as many fonts as it likes and each
    /// element chooses, so `8.letters()` is eight cells of *that* element's font.
    pub(crate) fn cell(&self, leaf: Leaf) -> Area {
        self.at(leaf).map(|at| self.cell[at]).unwrap_or_default()
    }

    /// Where the layout put `leaf`'s two ends, before R4 moved them, or `None` if it is placed by
    /// a box.
    pub(crate) fn spanned(&self, leaf: Leaf) -> Option<Stretched> {
        self.spanned[self.at(leaf)?]
    }

    /// Where `leaf`'s two ends landed, as R4 moved them, or `None` if it is placed by a box.
    pub(crate) fn stretched(&self, leaf: Leaf) -> Option<Stretched> {
        self.stretched[self.at(leaf)?]
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
    // Every element now holds the run it states, so a run nothing holds is one nothing states. Only
    // on a frame where a run stopped being stated, which is the only way the cache comes to hold
    // one nothing wants.
    if elements.restated {
        grove.shaping.sweep();
    }
    axes(grove, &mut elements);
    extent(grove, &elements);
    scroll(grove, &mut elements);
    clip(&mut elements);
    rank(&mut elements);
    // Resolution ends here. What settles after it is its own step, and is timed as one.
    drop(resolving);
    let _step = trace_span!("settle").entered();
    inherit(&mut elements);
    regions(grove, &mut elements);
    grove.elements = elements;
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
        // column is already holding. What it reads is its own run and face and nothing else, so an
        // element that only moved is one of these.
        if !elements.lettered[at] {
            continue;
        }
        let leaf = elements.order[at];
        // No font and no size is no cell, and nothing measured in one. What such an element last
        // measured to stands.
        let Some(typeface) = tree.typeface(leaf) else {
            continue;
        };
        let size = typeface.size.at(*layout, *short);
        let cell = fonts.cell(typeface.font, size);
        // The run, shaped, is kept beside the cell: it is what a placement reading a character of
        // this element resolves against, and this is the one pass that shapes.
        // A masked run is shaped as its dots, so the value itself is never a key in the shaping
        // cache, and nothing drawn from here can say it.
        let masked = tree.masked(leaf);
        let composed = tree.lettering(leaf).map(|value| match masked {
            true => shaping
                .shape(fonts, typeface.font, size, &crate::text_input::dots(value))
                .clone(),
            false => shaping.shape(fonts, typeface.font, size, value).clone(),
        });
        let width = composed
            .as_ref()
            .map(|shaped| shaped.max_content())
            .unwrap_or_default();
        elements.cell[at] = cell;
        elements.composed[at] = composed;
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
fn axes(grove: &mut Grove, elements: &mut Elements) {
    axis(grove, elements, Axis::Horizontal);
    wrap(grove, elements);
    axis(grove, elements, Axis::Vertical);
}

/// One axis of the whole tree, in dependency order, through the one pure resolver.
fn axis(grove: &Grove, elements: &mut Elements, axis: Axis) {
    let _pass = trace_span!("axis", vertical = (axis == Axis::Vertical)).entered();
    let viewport = Section::new(Position::default(), grove.viewport);
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
        let (span, stretched) = geometry(grove, leaf, &context, axis);
        // A stroke's ends are settled one axis at a time, into what it held: an element a measure
        // reached between the two axes is resolved on the second alone, and keeps the first.
        if let Some((from, to)) = stretched {
            elements.spanned[at]
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
    context: &Context,
    axis: Axis,
) -> (Span, Option<(f32, f32)>) {
    let (trace, stroke) = match grove.tree.placement(leaf) {
        Some(Placement::Traced(trace, stroke)) => (trace, stroke),
        Some(Placement::Boxed(location)) => {
            return (span(grove, leaf, location, context, axis), None);
        }
        // Every live element is grown with a placement, so this is an element the order holds and
        // the world does not -- which nothing that runs between the two can make.
        None => return (span(grove, leaf, &Location::default(), context, axis), None),
    };
    let half = stroke.half();
    let (from, to) = ends(grove, leaf, trace, context, axis);
    (
        Span {
            near: from.min(to) - half,
            far: from.max(to) + half,
        },
        Some((from, to)),
    )
}

/// One axis of where `leaf`'s two ends currently are: what it declares, blended with the ends a
/// motion left.
///
/// [`span`] for a trace. Each end blends on its own and the box is taken from the blended pair,
/// rather than the two boxes blending: a stroke part way between a rising and a falling diagonal is
/// a stroke, and a box that blended separately would not be the rectangle around it.
fn ends(grove: &Grove, leaf: Leaf, trace: &Trace, context: &Context, axis: Axis) -> (f32, f32) {
    let (from, to) = located(pinned_ends(trace, grove), context);
    match grove.aspen.trace(leaf) {
        Some((departed, at)) => {
            let (left_from, left_to) = match departed {
                Departed::Declared(trace) => located(pinned_ends(trace, grove), context),
                Departed::Snapshot(stretched) => stretched.along(axis),
            };
            (blend(left_from, from, at), blend(left_to, to, at))
        }
        None => (from, to),
    }
}

/// One axis of both of a pair of ends, through the resolver.
fn located(ends: &Ends, context: &Context) -> (f32, f32) {
    (locate(&ends.from, context), locate(&ends.to, context))
}

/// Which pair of ends a trace states, at the breakpoint in force. [`pinned`] for a trace.
fn pinned_ends<'a>(trace: &'a Trace, grove: &Grove) -> &'a Ends {
    trace.ends(grove.layout, grove.short)
}

/// One axis of where `leaf` currently is: what it declares, blended with the endpoint a motion left.
///
/// The one place a placement becomes a span, so a measure and a layout are answering the same
/// question -- an element in motion is measured where it *is* rather than where it is going.
fn span(grove: &Grove, leaf: Leaf, location: &Location, context: &Context, axis: Axis) -> Span {
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
    for at in (0..elements.len()).rev() {
        if let (true, Some(trunk)) = (elements.measures[at], elements.trunk[at]) {
            elements.measures[trunk] = true;
        }
    }
    let shaped = shaped(elements);
    let _half = trace_span!("reach").entered();
    for at in (0..elements.len()).rev() {
        if !elements.measures[at] {
            continue;
        }
        let height = shaped[at].max(reach(grove, elements, at));
        if height != elements.intrinsic[at].height {
            elements.intrinsic[at].height = height;
            // The measure moved under an element nothing was written to, and a placement reading
            // that measure has to be resolved against the new one. R2b runs after this and spreads
            // it, which is why it is safe to find out here.
            elements.dirty[at] = true;
        }
    }
}

/// How tall each element's own run turned out at the width R2a gave it, or zero where it says
/// nothing.
///
/// In any order, because no element's answer is another's. The run is the one R1 left the element
/// holding: an element measuring again because something under it moved says what it said last
/// frame, and one that says something new was shaped for it by R1 this frame.
fn shaped(elements: &Elements) -> Vec<f32> {
    let _half = trace_span!("shaped").entered();
    elements
        .measures
        .iter()
        .zip(&elements.composed)
        .zip(&elements.section)
        .map(|((measures, run), section)| match (measures, run) {
            (true, Some(run)) => run.measure(section.width()),
            _ => 0.0,
        })
        .collect()
}

/// How far the elements grown under the element at `at` reach below its top edge.
///
/// Only the children that describe their own extent are counted. One that reads a vertical box --
/// a percentage of this element, a row of its grid, an anchor's edge -- is asking how tall
/// something else is, so it cannot be what decides how tall this is. See
/// [`Config::measurable`](crate::placement::role::Config::measurable).
///
/// A child is asked only where it is dirty, and its answer kept. Every other child reaches exactly
/// as far as it did when it last was, which is what [`reached`](Elements::reached) holds for it.
fn reach(grove: &Grove, elements: &mut Elements, at: usize) -> f32 {
    let mut reach: f32 = 0.0;
    for index in elements.branches[at] as usize..elements.branches[at + 1] as usize {
        let child_at = elements.children[index] as usize;
        if elements.dirty[child_at] {
            let child = elements.order[child_at];
            let far = measurable(grove, child).then(|| {
                let context = raised(elements, at, child_at);
                geometry(grove, child, &context, Axis::Vertical).0.far
            });
            elements.reached[child_at] = far;
        }
        if let Some(far) = elements.reached[child_at] {
            reach = reach.max(far);
        }
    }
    reach
}

/// Whether `leaf`'s vertical placement describes its own extent, and so counts toward the measure
/// of what it is grown under.
///
/// One question with two spellings, because a placement has two. A box asks it of its vertical
/// configuration; a trace asks it of both of its ends, since either one reading a vertical box is
/// enough to make the answer circular.
fn measurable(grove: &Grove, leaf: Leaf) -> bool {
    match grove.tree.placement(leaf) {
        Some(Placement::Traced(trace, _)) => {
            let ends = pinned_ends(trace, grove);
            ends.from.measurable() && ends.to.measurable()
        }
        Some(Placement::Boxed(location)) => pinned(location, grove, Axis::Vertical).measurable(),
        None => pinned(&Location::default(), grove, Axis::Vertical).measurable(),
    }
}

/// What one child resolves its vertical axis against while its trunk is being measured.
///
/// The same [`Context`] R2b will build, with every vertical reading taken as zero: no box on this
/// axis has resolved yet, which is the point of measuring. Every horizontal reading is real -- the
/// child's own width included -- so a height stated in columns, read off a width, or held in
/// proportion to its own still answers.
fn raised<'a>(elements: &'a Elements, trunk: usize, child: usize) -> Context<'a> {
    let offered = |at: usize| Basis {
        section: flattened(elements.section[at]),
        intrinsic: elements.intrinsic[at],
        tracks: elements.tracks[at],
        cell: elements.cell[at],
        run: elements.composed[at].as_deref(),
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
        let standing = &elements.standing[at];
        // A hidden element is not content, and neither is anything under it: that is what makes
        // hiding the whole answer for parking something out of the way, rather than half of one.
        if !standing.visible {
            continue;
        }
        let section = elements.section[at];
        let mut far = Position::new(section.right(), section.bottom());
        for child in elements.children(at) {
            // The two ways an element grown inside a region is not part of its content, and each
            // is left out here by the same one declaration that says the rest of what it means --
            // which is what keeps the halves from disagreeing.
            //
            // A pinned child does not move with the content. A floating one moves with it but sits
            // over the region rather than in it, so it is not content either: an overlay that
            // invented room to scroll to is the scrollbar nobody ordered.
            let held = &elements.standing[child];
            if held.pinned || held.floats.is_some() {
                continue;
            }
            let Some(child) = reach[child] else {
                continue;
            };
            far = Position::new(far.x.max(child.x), far.y.max(child.y));
        }
        if let Some(scroll) = standing.scrolls {
            grove
                .tree
                .set_extent(elements.order[at], reached(scroll, section, far));
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
fn scroll(grove: &mut Grove, elements: &mut Elements) {
    let _pass = trace_span!("scroll", coasting = grove.coasting.len()).entered();
    view::asked(grove, elements);
    let mut accumulated: Vec<Position> = vec![Position::default(); elements.len()];
    // What a *pinned* child of each element receives, which is the accumulation with its nearest
    // scrolling ancestor left out of it.
    let mut unpinned: Vec<Position> = vec![Position::default(); elements.len()];
    for at in 0..elements.len() {
        let placed = elements.section[at];
        let trunk = elements.trunk[at];
        let standing = elements.standing[at];
        let inherited = trunk.map(|trunk| accumulated[trunk]).unwrap_or_default();
        let outside = trunk.map(|trunk| unpinned[trunk]).unwrap_or_default();
        // A pinned element does not receive its nearest scrolling ancestor's offset, and receives
        // every offset outside that one: pinning is relative to the region the element sits in and
        // says nothing about what contains that region.
        let applied = match standing.pinned {
            true => outside,
            false => inherited,
        };
        let drawn = Section::new(
            Position::new(placed.left() - applied.x, placed.top() - applied.y),
            placed.area,
        );
        if drawn != elements.drawn[at] {
            elements.drawn[at] = drawn;
            elements.moved[at] = true;
            elements.restack = true;
        }
        // A stroke's ends travel with its box, by the same offset, because they are the same
        // geometry said two ways. A stroke nothing wrote to keeps the ends it settled at, and moves
        // under the offset exactly as its box does.
        if let Some(spanned) = elements.spanned[at] {
            let stretched = Some(spanned.less(applied));
            if stretched != elements.stretched[at] {
                elements.stretched[at] = stretched;
                elements.moved[at] = true;
            }
        }
        let (carried, escaped) = match standing.scrolls {
            Some(scroll) => {
                let clamped = clamp(grove, elements.order[at], scroll, placed);
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
fn clip(elements: &mut Elements) {
    let _pass = trace_span!("clip").entered();
    let unbounded = Clipped::unbounded().0;
    let mut passed: Vec<Section> = vec![unbounded; elements.len()];
    // What a *floating* child of each element is clipped to, which is the intersection with its
    // nearest scrolling ancestor's own rect left out of it. The same second accumulation R4 carries
    // for a pinned child, and for the same reason: both say "not part of this region", and both
    // have to say it about the region the element is actually in rather than about all of them.
    let mut escaped: Vec<Section> = vec![unbounded; elements.len()];
    for at in 0..elements.len() {
        let trunk = elements.trunk[at];
        let standing = elements.standing[at];
        let inherited = trunk.map(|trunk| passed[trunk]).unwrap_or(unbounded);
        let outside = trunk.map(|trunk| escaped[trunk]).unwrap_or(unbounded);
        // A floating element is positioned outside the region on purpose, so cutting it off at the
        // region's edge would undo the placement that put it there. How far out it reaches is the
        // element's own statement, because no one answer is right everywhere.
        let applied = match standing.floats {
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
        if applied != elements.clip[at] {
            elements.clip[at] = applied;
            elements.moved[at] = true;
            elements.restack = true;
        }
        let scrolls = standing.scrolls.is_some();
        passed[at] = match scrolls {
            true => applied.intersect(elements.drawn[at]),
            false => applied,
        };
        escaped[at] = match scrolls {
            true => applied,
            false => outside,
        };
    }
}

/// What `named` leaves visible of what it holds, if it is above the element at `at` at all.
///
/// The walk is up the trunks from the element, so one that names something beside it rather than
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

/// R6. Declared elevation accumulates down the tree, and allocation order settles what it leaves
/// equal.
///
/// One walk in the same dependency order the axes used, which puts every trunk before what hangs
/// off it -- so a trunk's rank is already this frame's when what hangs off it reads it. Nothing here
/// reads a box: where an element sits in the stack has nothing to do with where it sits on the
/// surface.
fn rank(elements: &mut Elements) {
    let _pass = trace_span!("rank").entered();
    for at in 0..elements.len() {
        let trunk = elements.trunk[at]
            .map(|trunk| elements.rank[trunk].stack)
            .unwrap_or_default();
        let standing = &elements.standing[at];
        let rank = ResolvedElevation {
            stack: standing.elevation.accumulate(trunk),
            growth: standing.growth,
        };
        if rank != elements.rank[at] {
            elements.rank[at] = rank;
            elements.moved[at] = true;
            elements.restack = true;
            elements.reranked = true;
        }
    }
}

/// R7. The three off-states, resolved over each element's whole ancestry.
///
/// One walk, in the same order the axes used, which puts every trunk before what hangs off it.
/// Nothing has a cascade to write: an element grown under a disabled trunk is disabled on its first
/// frame because the pass does not care when it arrived, and enabling that trunk leaves anything
/// disabled in its own right disabled because the product is over the whole ancestry rather than a
/// single bit that was overwritten on the way down.
fn inherit(elements: &mut Elements) {
    let _pass = trace_span!("inherit").entered();
    for at in 0..elements.len() {
        let trunk = elements.trunk[at]
            .map(|trunk| elements.inherited[trunk])
            .unwrap_or_default();
        let standing = &elements.standing[at];
        let product =
            Inherited::under(trunk, standing.visible, standing.opacity, standing.disabled);
        if product != elements.inherited[at] {
            elements.inherited[at] = product;
            elements.moved[at] = true;
            elements.restack = true;
        }
    }
}

/// R8. The box stack the next frame's dispatch reads.
///
/// Membership is universal: an element is here because it is there. What is left out is only what
/// is not there at all -- hidden, fully transparent, or clipped away by a region it sits inside.
/// `intangible` is not a way out of the stack; it is carried on the region and decides what may be
/// the top of it.
///
/// Built only on a frame where something it is built from moved. Every one of those is a column R4
/// through R7 compared as they wrote it, or the order itself being built again -- so a frame where
/// one hover fades, or nothing happens at all, keeps the stack the last one built.
fn regions(grove: &mut Grove, elements: &mut Elements) {
    if !elements.restack {
        return;
    }
    let _pass = trace_span!("regions").entered();
    // The order is total -- a resolved elevation carries allocation order as its tie-break -- so two
    // identical runs read the same element at the same point, and sorting without stability loses
    // nothing.
    if elements.reranked {
        let rank = &elements.rank;
        elements.stacking.clear();
        elements.stacking.extend(0..elements.len() as u32);
        elements
            .stacking
            .sort_unstable_by(|left, right| rank[*right as usize].cmp(&rank[*left as usize]));
    }
    grove
        .stack
        .settle(elements.stacking.iter().filter_map(|&at| {
            let at = at as usize;
            let inherited = elements.inherited[at];
            if !inherited.present() {
                return None;
            }
            let section = elements.drawn[at];
            let clip = elements.clip[at];
            if section.intersect(clip).is_empty() {
                return None;
            }
            let gestures = elements.standing[at].gestures;
            Some(Region {
                leaf: elements.order[at],
                section,
                clip,
                shape: gestures.shape,
                tangible: !gestures.intangible,
                receives: gestures.receives,
                disabled: inherited.disabled,
            })
        }));
}

/// Everything one axis of the element at `at` resolves against.
fn context<'a>(elements: &'a Elements, viewport: Section, at: usize, axis: Axis) -> Context<'a> {
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
fn own<'a>(elements: &'a Elements, at: usize, axis: Axis) -> Basis<'a> {
    Basis {
        section: match axis {
            Axis::Horizontal => Section::default(),
            Axis::Vertical => flattened(elements.section[at]),
        },
        intrinsic: elements.intrinsic[at],
        tracks: Tracks::default(),
        cell: elements.cell[at],
        // Where its own characters stand is a question about a box that is still being solved.
        run: None,
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
fn basis<'a>(
    elements: &'a Elements,
    at: Option<usize>,
    fallback: Section,
    axis: Axis,
) -> Basis<'a> {
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
        run: elements.composed[at].as_deref(),
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
/// frame reads none of it. When it is built again, what every element already resolved to is
/// carried to wherever the element now sits, so building it again loses nothing but the elements
/// that went.
fn structure(elements: &mut Elements, grove: &Grove) {
    let _pass = trace_span!("structure").entered();
    let tree = &grove.tree;
    let leaves = tree.leaves();
    let count = leaves.len();
    // Where each element sits among the leaves, by the index of its name, which is dense: a lookup
    // is an index rather than a hash.
    let names = leaves
        .iter()
        .map(|leaf| leaf.0.index_u32() as usize + 1)
        .max()
        .unwrap_or_default();
    let mut among: Vec<u32> = vec![NOWHERE; names];
    for (at, leaf) in leaves.iter().enumerate() {
        among[leaf.0.index_u32() as usize] = at as u32;
    }
    let find = |leaf: Leaf| -> Option<usize> {
        let at = *among.get(leaf.0.index_u32() as usize)? as usize;
        (leaves.get(at) == Some(&leaf)).then_some(at)
    };
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
        let on = [tree.trunk(leaf), tree.anchor(leaf)].map(|on| on.and_then(find));
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
    // Where each element sat in the order being replaced, which is where what it resolved to is
    // carried from. Whatever sat there and is not carried anywhere withered.
    let from: Vec<Option<usize>> = order.iter().map(|&at| elements.at(leaves[at])).collect();
    let mut carried = vec![false; elements.len()];
    for at in from.iter().flatten() {
        carried[*at] = true;
    }
    elements.gone.extend(
        (0..elements.len())
            .filter(|at| !carried[*at])
            .map(|at| (elements.order[at], elements.standing[at].chlorophyll)),
    );
    carry(&mut elements.section, &from, Section::default());
    carry(&mut elements.intrinsic, &from, Area::default());
    carry(&mut elements.reached, &from, None);
    carry(&mut elements.cell, &from, Area::default());
    carry(&mut elements.composed, &from, None);
    carry(&mut elements.spanned, &from, None);
    carry(&mut elements.drawn, &from, Section::default());
    carry(&mut elements.stretched, &from, None);
    carry(&mut elements.clip, &from, Clipped::unbounded().0);
    carry(&mut elements.rank, &from, ResolvedElevation::default());
    carry(&mut elements.inherited, &from, Inherited::default());
    // What an element declared is carried like what it resolved to. A grid is divided again by
    // `read` wherever the element is dirty, which an element just grown always is; its standing is
    // read here, and read again by `read` wherever it was restyled.
    carry(&mut elements.tracks, &from, Tracks::default());
    carry(&mut elements.standing, &from, Standing::default());
    elements.order.clear();
    elements.trunk.clear();
    elements.anchor.clear();
    elements.slots.clear();
    elements.slots.resize(names, NOWHERE);
    for (position, &at) in order.iter().enumerate() {
        let leaf = leaves[at];
        let [trunk, anchor] = depends[at];
        elements.slots[leaf.0.index_u32() as usize] = position as u32;
        elements.order.push(leaf);
        elements.trunk.push(trunk.map(|on| ranked[on]));
        elements.anchor.push(anchor.map(|on| ranked[on]));
        if from[position].is_none() {
            elements.standing[position] = tree.standing(leaf);
        }
    }
    // What is grown under each element, by position: counted, summed into where each element's run
    // starts, and filled.
    elements.branches.clear();
    elements.branches.resize(count + 1, 0);
    for trunk in elements.trunk.iter().flatten() {
        elements.branches[trunk + 1] += 1;
    }
    for at in 0..count {
        elements.branches[at + 1] += elements.branches[at];
    }
    let mut filling = elements.branches.clone();
    elements.children.clear();
    elements.children.resize(count, 0);
    for at in 0..count {
        if let Some(trunk) = elements.trunk[at] {
            elements.children[filling[trunk] as usize] = at as u32;
            filling[trunk] += 1;
        }
    }
    // Everything the stack is built from may have moved, since what is in it did.
    elements.restack = true;
    elements.reranked = true;
}

/// Rebuilds one column in the new order: each element's value carried from where it sat, and
/// `fresh` for one that was not in the order before.
fn carry<T: Clone>(column: &mut Vec<T>, from: &[Option<usize>], fresh: T) {
    let held = core::mem::take(column);
    column.extend(from.iter().map(|at| match at {
        Some(at) => held[*at].clone(),
        None => fresh.clone(),
    }));
}

/// What this frame has to resolve, and what it may leave alone.
///
/// An element resolves again if it was written to, or if what it hangs off or is anchored to does.
/// That closure is one walk of the order rather than a search, because the order already puts a
/// dependency before what depends on it. What was written is found by name and marked where it
/// sits, so the walk costs the order and the marking costs what was written.
///
/// An element that was restyled is read again and marked as moved, and resolves again only if
/// something it is placed by was written as well.
fn read(elements: &mut Elements, grove: &Grove) {
    let _pass = trace_span!("elements").entered();
    let written = grove.tree.written();
    elements.gone.clear();
    elements.restack = false;
    elements.reranked = false;
    if written.restructured {
        structure(elements, grove);
    }
    // What the restyled elements declared is read again where they sit.
    for &leaf in written.restyled {
        if let Some(at) = elements.at(leaf) {
            elements.standing[at] = grove.tree.standing(leaf);
        }
    }
    let count = elements.len();
    elements.restated = written.restated;
    elements.dirty.clear();
    elements.dirty.resize(count, written.all);
    elements.moved.clear();
    elements.moved.resize(count, false);
    elements.measures.clear();
    elements.measures.resize(count, false);
    elements.resolved.clear();
    elements.resolved.resize(count, false);
    elements.lettered.clear();
    elements.lettered.resize(count, written.all);
    if !written.all {
        for &leaf in written.lettered {
            if let Some(at) = elements.at(leaf) {
                elements.lettered[at] = true;
            }
        }
    }
    if !written.all {
        for &leaf in written.declared {
            if let Some(at) = elements.at(leaf) {
                elements.dirty[at] = true;
            }
        }
    }
    for &leaf in written.measures {
        if let Some(at) = elements.at(leaf) {
            elements.measures[at] = true;
        }
    }
    if written.repainted {
        elements.moved.fill(true);
    }
    for &leaf in written.restyled {
        if let Some(at) = elements.at(leaf) {
            elements.moved[at] = true;
        }
    }
    for at in 0..count {
        // A dependency the order could not put first is one a cycle left in the remainder, and it
        // has no answer yet. Resolving such an element is the same fallback the rest of resolution
        // gives it: it is answered against what its dependency last was, so it is answered again.
        let decided = |on: Option<usize>| on.is_some_and(|on| on >= at || elements.dirty[on]);
        let dirty =
            elements.dirty[at] || decided(elements.trunk[at]) || decided(elements.anchor[at]);
        elements.dirty[at] = dirty;
        elements.moved[at] |= dirty;
        elements.measures[at] |= dirty;
        // A clean element offers the box it settled at, and offers it from the start, because
        // nothing this frame is going to compute it again. One being resolved again offers nothing
        // until an axis reaches it, which is what it did when every element was resolved.
        elements.resolved[at] = !dirty;
        // The grid is divided at the breakpoint in force, so it is read again wherever the element
        // is -- a breakpoint that moved is every element written to, and nothing else can change a
        // division without writing the grid it divides.
        if dirty {
            elements.tracks[at] = grove
                .tree
                .grid(elements.order[at])
                .unwrap_or_default()
                .tracks(grove.layout, grove.short);
        }
    }
}
