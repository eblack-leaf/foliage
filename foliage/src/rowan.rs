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
//! The accumulations, R3 through R7, carry a running product down the trunks (or up them, for R3)
//! rather than an answer per element, so what each element passes on is held too, beside what it
//! resolved to. An element is visited where something it reads moved: it was written to, restyled or
//! grown, or what it hangs off passes it something new -- and what it passes on is handed to what is
//! under it only where that changed. They record what they actually changed as they go, which is
//! what extraction then states -- and R8, the box stack, is built again only on a frame where one of
//! them changed something it is built from.
//!
//! No pass walks the order to find what it has to do. What each visits is marked, a bit per
//! position ([`Positions`]), and the walk goes from one mark to the next: a frame that wrote nothing
//! walks a few thousand empty words, where it once walked every element, and an element left alone
//! -- hidden, or only sitting there -- costs nothing on any frame that does not reach it.
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
//! | R1 | measure | — | the character cell, and the run each element is drawn from |
//! | R1m | across | **bottom-up** | the max-content width half of the intrinsic size |
//! | R2a | horizontal | dependency order | the horizontal axis |
//! | R2m | wrap | **bottom-up** | the measured-height half of the intrinsic size |
//! | R2b | vertical | dependency order | the vertical axis, and with it the placed box |
//! | R3 | extent | **bottom-up** | [`Extent`](crate::view::Extent): how far a region's content reaches |
//! | R4 | scroll | top-down | the drawn box: the placed box less every scrolling ancestor's offset, and moved by every shift over it |
//! | R5 | clip | top-down | the clip: what a scrolling ancestor leaves visible |
//! | R6 | rank | dependency order | `ResolvedElevation`: elevation accumulated, then tie-broken |
//! | R7 | inherit | top-down | [`Inherited`]: the visible, opacity and disabled products |
//! | R8 | regions | rank order | the box stack the next frame's dispatch reads |
//!
//! Both halves of R2 call the same pure resolver, once per axis. That is only safe because it is
//! pure: there is no accumulated state for a second call to corrupt.
//!
//! R2a and R2b resolve what was written to, and R1 only what says something new -- a run rewritten,
//! an element grown, a breakpoint or a face that changed every cell. R1m and R2m measure what was
//! written to and everything above it, because what an element reaches over is what the elements
//! under it resolved to -- a measure travels toward the trunk where a box travels away from it. R2m
//! also measures what is only being resolved again against something written, since a box that
//! comes out another width may wrap to another height; R1m does not, since nothing it reads is
//! anyone else's box. A measure that moves under an element nothing wrote to makes that element
//! resolve again, which is why each measure sits before the axis it is read on and not after it. And
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
//! placement beside it. From there every pass indexes -- what the accumulations R3 through R7 carry
//! is held in columns the length of the order, and a trunk's contribution is `[..]` rather than a
//! lookup on a map keyed by element.
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
//! Three passes run the other way, and each is a cycle that turns out not to be vicious.
//!
//! R1m and R2m are two of them, and the same one on each axis: a box sized to its contents is sized
//! by boxes resolved against it. Neither is circular, because a child that reads its trunk's extent
//! on the axis being measured is not counted toward it -- and what is left is measured bottom-up
//! before that axis is laid out at all. R1m runs before R2a, when no width is known; R2m between the
//! two halves of R2, the one moment when every width is known and no height is. The order matters
//! for wrapping, which makes a height depend on a width: **width flows down and height flows up** --
//! a monospaced run's widest unwrapped line is free from its character count, so a run's width needs
//! no wrapping measured, and its height is measured at the width R2a gave it.
//!
//! R3 is the third: a child resolves against its parent's box, a scrolling parent's extent comes
//! from where its children landed, and that extent is what its offset is clamped to. Extent affects
//! only the clamp and never the parent's own box -- so a top-down layout sweep, a bottom-up extent
//! sweep and a top-down scroll application resolve it in one pass each.
//!
//! None of them iterates to convergence. Every pass here runs exactly once.

use std::sync::Arc;

use tracing::field::Empty;
use tracing::trace_span;

use crate::aspen::{Departed, blend};
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
use crate::placement::source::Against;
use crate::placement::trace::{Ends, Trace};
use crate::text::shape::Shaped;
use crate::tree::{Placement, Tree, Written};
use crate::view::{self, Clipped, Escape, Scroll, range};

/// No element sits here.
const NOWHERE: u32 = u32::MAX;

/// A position held as a number, read back as one if anything sits there.
fn position(held: u32) -> Option<usize> {
    (held != NOWHERE).then_some(held as usize)
}

/// Every live element in dependency order, with what each one resolves against, what each one offers
/// a placement that reads it, and everything resolution made of it.
///
/// The frame's one read of the tree, and what every pass indexes into. A trunk and an anchor are
/// asked for on both axes and on four of the passes after them; what an element offers is asked for
/// twice per element on each axis. Out of the world each time, every one of those is a lookup by
/// entity -- resolved once to a position in the order, each is an index.
///
/// What the order is made of -- what each element hangs off, what is under it, what every pass
/// visits -- is held by position, since that is what a walk goes by. What resolution made of each
/// element is held by its slot, the index of its name, which is where the tree holds what it
/// declared: so an order made again, or repaired for what was grown and what withered, moves none of
/// it, and an element new to the order has only its own slot to clear. The columns are kept between
/// frames, which is what lets an element nothing wrote to keep the answer it already has.
#[derive(Default)]
pub(crate) struct Elements {
    /// Every live element, ordered so that nothing resolves before what it depends on.
    pub(crate) order: Vec<Leaf>,
    /// Where each element sits in the order, by its slot. One may be stale -- left by a name that
    /// withered -- which is why a lookup checks the order holds the name it asked for.
    sits: Vec<u32>,
    /// What each element hangs off, as a position in the order, or [`NOWHERE`]. See
    /// [`trunk`](Elements::trunk).
    trunk: Vec<u32>,
    /// What each element is anchored to, as a position in the order, or [`NOWHERE`]. See
    /// [`anchor`](Elements::anchor).
    anchor: Vec<u32>,
    /// What is grown directly under each element, as positions in the order: the children of the
    /// element at `at` are `children[branches[at]..branches[at + 1]]`. One array for the whole tree
    /// rather than one per element, and built with the order, so a pass that asks what is under an
    /// element never asks the world.
    branches: Vec<u32>,
    children: Vec<u32>,
    /// What is anchored to each element, as positions in the order, held the way
    /// [`children`](Elements::children) are: the elements anchored to the one at `at` are
    /// `anchored[anchoring[at]..anchoring[at + 1]]`. With the children, everything resolved against
    /// an element -- what a box that moved reaches.
    anchoring: Vec<u32>,
    anchored: Vec<u32>,
    /// The positions whose trunk or anchor sits at or after them: a cycle's remainder, resolved
    /// against what its dependency last was, and so visited by every pass on every frame. Almost
    /// always empty.
    forward: Vec<u32>,
    /// The positions of every element that scrolls. What a region scrolled to is not written to it
    /// -- a coast, a drag and a clamp each move it -- so R4 and R5 visit each of these on every
    /// frame. There are few. Found again wherever what scrolls changed.
    scrolling: Vec<u32>,
    /// The positions of every element that floats out to an element it names, which may sit any
    /// distance above it: R5 visits each on every frame, rather than following the chain between.
    within: Vec<u32>,
    /// The elements grown inside a region that are not part of its content, which R3 asks of every
    /// child of whatever it measures -- so it is a bit here rather than a read of what each child
    /// declared. There are two ways, each left out by the same one declaration that says the rest
    /// of what it means, which is what keeps the halves from disagreeing: a pinned child does not
    /// move with the content, and a floating one moves with it but sits over the region rather
    /// than in it -- an overlay that invented room to scroll to is the scrollbar nobody ordered.
    /// Found again wherever either is written.
    aside: Positions,
    /// The box, as far as the axes have resolved it: where the layout put the element, which is
    /// what its children resolve against.
    section: Vec<Section>,
    /// Whether the element is being resolved again and no axis has reached it yet: false from the
    /// start for one nothing has written to, whose box is the one it settled at last frame. What
    /// answers a cycle's remainder with the fallback rather than with a box that has not been
    /// computed.
    unresolved: Positions,
    /// Whether the element has to be resolved again this frame.
    ///
    /// True where something it reads was written -- its own declarations, or those of what it hangs
    /// off or is anchored to. False where nothing was, in which case the columns already hold what
    /// it resolved to and the passes that only rewrite that have nothing to do.
    dirty: Positions,
    /// [`dirty`](Elements::dirty) as [`read`] closed it, before any measure moved: everything
    /// resolved against one of these is one of these, so an axis reaching it has nothing to mark.
    closed: Positions,
    /// Whether the element was restyled this frame: what R3, R4, R6 and R7 read of it may be
    /// different, and nothing that places it is.
    restyled: Positions,
    /// Whether what the element passes on has to be found again whatever it reads: it is new to the
    /// order this frame, with nothing it passes on yet -- or, on a frame the order was built again or
    /// everything is resolved again, every element is.
    fresh: Positions,
    /// Whether any run stopped being stated this frame, which is the one thing that makes the
    /// shaping cache worth sweeping.
    restated: bool,
    /// Whether what scrolls, what is pinned or what floats changed this frame. A restyled element
    /// may have come to scroll or stopped, and what R5 passes on from it with it, so R5 visits what
    /// was restyled as well -- and it may have come to count toward its trunk's extent or stopped,
    /// so R3 visits what each hangs off.
    rescoped: bool,
    /// Whether anything extraction reads about the element moved this frame.
    ///
    /// Seeded from [`dirty`](Elements::dirty), because an element being resolved again may land
    /// somewhere else, and from a [`restyled`](crate::tree::Tree::restyled) write, and set by R4
    /// through R7 where what they wrote differs from what was held. An element it is false for is
    /// drawn exactly as the backend already has it.
    pub(crate) moved: Positions,
    /// Whether what the element offers what is resolved against it may be different this frame,
    /// which is what makes those resolve again.
    ///
    /// Seeded from [`dirty`](Elements::dirty) as [`read`] left it -- an element written to, and
    /// everything resolved against one, which are all resolved again whatever they land on -- and
    /// set by R2 where an element resolved again for any other reason came out with another box.
    /// An element resolved again only because a measure moved under it, and that lands where it
    /// was, offers what it offered: what is resolved against it is resolved again only where it
    /// reads that measure. See [`remeasured`](Elements::remeasured).
    answers: Positions,
    /// Whether the element's measured extent moved this frame, found by R1m across and R2m down.
    ///
    /// What makes the element itself resolve again, and what is resolved against it where it reads
    /// the measure: a [`content`](crate::content) of it. Nothing else reads a measure, so nothing
    /// else follows it unless the box moved too.
    remeasured: Positions,
    /// Whether R1 has to measure the element again: it was grown, or what it says was rewritten, or
    /// every cell in the tree moved.
    lettered: Positions,
    /// Whether the element has to measure again, before R2m walks that up the trunks.
    ///
    /// [`dirty`](Elements::dirty) is most of it. The rest is an element something withered from,
    /// which has one fewer thing to reach over and nothing left below it to say so.
    measures: Positions,
    /// Whether the element's own declarations were written: [`dirty`](Elements::dirty) as the
    /// tree recorded it, before anything it depends on spread to it -- and, once R1m has measured
    /// it, whether what it measured to across moved.
    ///
    /// What a trunk measuring across asks a child again for. A child's reach across reads its own
    /// placement and its own measure and nothing of where its trunk or its anchor sits, so a child
    /// that is only being resolved against something that moved reaches exactly as far as it did.
    touched: Positions,
    /// Whether R1m has to measure the element again: it was written to, or something under it was,
    /// or went.
    ///
    /// Narrower than [`measures`](Elements::measures), for the reason
    /// [`touched`](Elements::touched) is narrower than `dirty`: nothing a measure across reads is
    /// moved by an element's trunk moving, so a subtree under a box in motion measures to what it
    /// did and is not asked.
    measures_across: Positions,
    /// Whether anything is to be measured across this frame at all: something was written to, or
    /// went, or every measure has to be taken again. Known from what the tree recorded before any
    /// element is looked at, so a frame that wrote nothing -- a hover's fade, a frame where nothing
    /// happened -- walks nothing for R1m.
    across: bool,
    /// What the element measured to, which is what [`content()`](crate::content) reads. R1m writes
    /// the width -- the wider of its run's max-content, free in a monospaced font, and how far right
    /// anything grown under it reaches -- and R2m the height, the taller of what its run wrapped to
    /// at the width R2a gave it and how far down anything grown under it reaches. Width first and
    /// height after is the whole of width-down and height-up.
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
    /// Whether the element's placement describes its own height, and so counts toward the measure
    /// of what it is grown under: [`measurable`] on the vertical axis, as R2a last found it.
    ///
    /// Found where R2a already holds the placement, so R2m asks no element for it a second time.
    /// It is a fact about the placement and the breakpoint in force, either of which changing makes
    /// the element dirty -- so every element R2m asks about was found again by R2a this frame, or
    /// holds what it was found to be when last it was.
    upright: Vec<bool>,
    /// How far the element reaches right of the left edge of what it is grown under, as R1m last
    /// found it there, or `None` where its placement does not count toward that measure.
    ///
    /// [`reached`](Elements::reached) across: what each element contributes to its trunk's
    /// [`spread`], held so that a trunk measuring again because one of its children moved asks the
    /// rest nothing.
    spread: Vec<Option<f32>>,
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
    /// What R3 found the element reaches to: the far corner of it and everything under it that is
    /// content, or `None` where it is hidden. What its trunk measures its own from.
    extends: Vec<Option<Position>>,
    /// What R4 passes to what is grown under the element: the offset everything above it scrolled
    /// and shifted by, its own included -- and, for a pinned child, the same with the nearest
    /// scrolling region's own offset left out.
    accumulated: Vec<Position>,
    unpinned: Vec<Position>,
    /// What R5 passes to what is grown under the element: what is left visible of it -- and, for a
    /// floating child, the same with the nearest scrolling region's own rect left out.
    passed: Vec<Section>,
    escaped: Vec<Section>,
    /// Whether anything the box stack is built from moved this frame: an element grown or withered,
    /// or a drawn box, a clip, a rank or an inherited product that changed. What R8 waits on.
    restack: bool,
    /// The position of every tangible element, front-most first: the order the box stack is read
    /// in. Nothing intangible is ever in the stack, so nothing intangible is sorted for it. Sorted
    /// again only when a tangible element's rank moved, or the order was made again.
    stacking: Vec<u32>,
    /// Whether some element is anchored to one that was not live when the order was last built, so
    /// the order has an edge still to come: the frame that grows what it waits on builds the order
    /// again rather than repairing it, since repairing only places what is grown after what it
    /// reads, and here what is grown is what something already placed reads.
    waits: bool,
    /// Whether a tangible element's rank moved this frame, or the order was built again.
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
        let at = *self.sits.get(leaf.slot())? as usize;
        (self.order.get(at) == Some(&leaf)).then_some(at)
    }

    /// Where what the element at `at` hangs off sits, if it hangs off anything.
    fn trunk(&self, at: usize) -> Option<usize> {
        position(self.trunk[at])
    }

    /// Where what the element at `at` is anchored to sits, if it is anchored to anything.
    fn anchor(&self, at: usize) -> Option<usize> {
        position(self.anchor[at])
    }

    /// The slot of the element at `at`: where what resolution made of it is held.
    fn slot(&self, at: usize) -> usize {
        self.order[at].slot()
    }

    /// The slot of `leaf`, or `None` if it is not in the order.
    fn held(&self, leaf: Leaf) -> Option<usize> {
        self.at(leaf).map(|_| leaf.slot())
    }

    /// The positions of everything grown directly under the element at `at`.
    fn children(&self, at: usize) -> impl Iterator<Item = usize> + '_ {
        self.children[self.branches[at] as usize..self.branches[at + 1] as usize]
            .iter()
            .map(|child| *child as usize)
    }

    /// Marks in `marks` everything resolved against the element at `at` that comes after it in the
    /// order -- what is grown under it, and what is anchored to it -- which is everything a walk
    /// still to reach them will visit. One that comes before it is a cycle's remainder, which every
    /// pass visits anyway.
    fn dependents(&self, at: usize, marks: &mut Positions) {
        let anchored = &self.anchored[self.anchoring[at] as usize..self.anchoring[at + 1] as usize];
        for dependent in self
            .children(at)
            .chain(anchored.iter().map(|on| *on as usize))
        {
            if dependent > at {
                marks.set(dependent);
            }
        }
    }

    /// Nothing marked, as long as the order, with what `seeds` mark and what every pass visits on
    /// every frame marked: where a pass that walks the order starts from.
    fn visiting(&self, seeds: &[&Positions], every: &[&[u32]]) -> Positions {
        let mut marks = Positions::default();
        marks.reset(self.len());
        for seed in seeds {
            marks.add(seed);
        }
        for at in every.iter().copied().flatten().chain(&self.forward) {
            marks.set(*at as usize);
        }
        marks
    }

    /// Where the layout put `leaf`, which is what its children resolve against. A zero box for
    /// anything resolution has not reached.
    ///
    /// Resolution reads a box where it goes, because it holds every element in order. A coast, a
    /// sought region, a running motion and an app each name one element instead, so they ask this.
    pub(crate) fn placed(&self, leaf: Leaf) -> Section {
        self.held(leaf)
            .map(|slot| self.section[slot])
            .unwrap_or_default()
    }

    /// Where `leaf` is on screen, which is what an app reads and what a hit test runs against.
    pub(crate) fn drawn(&self, leaf: Leaf) -> Section {
        self.held(leaf)
            .map(|slot| self.drawn[slot])
            .unwrap_or_default()
    }

    /// What a scrolling ancestor leaves visible of `leaf`.
    #[allow(unused)]
    pub(crate) fn clip(&self, leaf: Leaf) -> Section {
        self.held(leaf)
            .map(|slot| self.clip[slot])
            .unwrap_or_else(|| Clipped::unbounded().0)
    }

    /// Where `leaf` sits in the one stack, as R6 last resolved it.
    #[allow(unused)]
    pub(crate) fn rank(&self, leaf: Leaf) -> ResolvedElevation {
        self.held(leaf)
            .map(|slot| self.rank[slot])
            .unwrap_or_default()
    }

    /// What the three off-states resolved to over `leaf`'s whole ancestry, as R7 last computed it.
    pub(crate) fn inherited(&self, leaf: Leaf) -> Inherited {
        self.held(leaf)
            .map(|slot| self.inherited[slot])
            .unwrap_or_default()
    }

    /// How far `leaf` and the content under it reach, as R3 last found it.
    #[cfg(test)]
    pub(crate) fn extends(&self, leaf: Leaf) -> Option<Position> {
        self.extends[self.held(leaf)?]
    }

    /// Whether `leaf` was resolved again by the last frame.
    #[cfg(test)]
    pub(crate) fn resolved_again(&self, leaf: Leaf) -> bool {
        self.at(leaf).is_some_and(|at| self.dirty.get(at))
    }

    /// What R1 shaped `leaf`'s run as, character for character: what is drawn, which a masked
    /// field's value is not.
    #[cfg(test)]
    pub(crate) fn shaped_text(&self, leaf: Leaf) -> Option<String> {
        self.composed[self.held(leaf)?]
            .as_ref()
            .map(|shaped| shaped.text())
    }

    /// The character cell of `leaf`'s own font, at its own size, as R1 measured it.
    ///
    /// Per element rather than per engine: an app registers as many fonts as it likes and each
    /// element chooses, so `8.letters()` is eight cells of *that* element's font.
    pub(crate) fn cell(&self, leaf: Leaf) -> Area {
        self.held(leaf)
            .map(|slot| self.cell[slot])
            .unwrap_or_default()
    }

    /// Where the layout put `leaf`'s two ends, before R4 moved them, or `None` if it is placed by
    /// a box.
    pub(crate) fn spanned(&self, leaf: Leaf) -> Option<Stretched> {
        self.spanned[self.held(leaf)?]
    }

    /// Where `leaf`'s two ends landed, as R4 moved them, or `None` if it is placed by a box.
    pub(crate) fn stretched(&self, leaf: Leaf) -> Option<Stretched> {
        self.stretched[self.held(leaf)?]
    }
}

/// Positions in the order, a bit each: what a pass has to visit, walked from one mark to the next.
///
/// What lets a pass cost what it visits rather than what is there. A position left unmarked is a
/// sixty-fourth of a word looked at, so a walk over an order of three hundred thousand elements with
/// nothing marked reads under five thousand words -- and a walk forward that marks a position ahead
/// of it, as a trunk marks what hangs off it, reaches that position in the same walk.
#[derive(Clone, Default)]
pub(crate) struct Positions {
    words: Vec<u64>,
}

impl Positions {
    /// Nothing marked, over `len` positions.
    fn reset(&mut self, len: usize) {
        self.words.clear();
        self.words.resize(len.div_ceil(64), 0);
    }

    /// Every one of `len` positions marked.
    pub(crate) fn fill(&mut self, len: usize) {
        self.words.clear();
        self.words.resize(len / 64, u64::MAX);
        if !len.is_multiple_of(64) {
            self.words.push((1 << (len % 64)) - 1);
        }
    }

    /// What `other` marks, and nothing else.
    pub(crate) fn copy_from(&mut self, other: &Positions) {
        self.words.clear();
        self.words.extend_from_slice(&other.words);
    }

    /// Whether `at` is marked. A position past the end is not.
    pub(crate) fn get(&self, at: usize) -> bool {
        self.words
            .get(at / 64)
            .is_some_and(|word| (word >> (at % 64)) & 1 == 1)
    }

    fn set(&mut self, at: usize) {
        self.words[at / 64] |= 1 << (at % 64);
    }

    fn unset(&mut self, at: usize) {
        self.words[at / 64] &= !(1 << (at % 64));
    }

    /// Marks everything `other` marks, as far as both reach.
    fn add(&mut self, other: &Positions) {
        for (word, theirs) in self.words.iter_mut().zip(&other.words) {
            *word |= theirs;
        }
    }

    /// How many positions are marked.
    pub(crate) fn count(&self) -> usize {
        self.words
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum()
    }

    /// The first marked position at or after `from`.
    fn next(&self, from: usize) -> Option<usize> {
        let mut index = from / 64;
        let mut word = *self.words.get(index)? & (u64::MAX << (from % 64));
        loop {
            if word != 0 {
                return Some(index * 64 + word.trailing_zeros() as usize);
            }
            index += 1;
            word = *self.words.get(index)?;
        }
    }

    /// The last marked position before `end`.
    fn last_before(&self, end: usize) -> Option<usize> {
        let last = end.min(self.words.len() * 64).checked_sub(1)?;
        let mut index = last / 64;
        let mut word = self.words[index] & (u64::MAX >> (63 - last % 64));
        loop {
            if word != 0 {
                return Some(index * 64 + 63 - word.leading_zeros() as usize);
            }
            index = index.checked_sub(1)?;
            word = self.words[index];
        }
    }

    /// Every marked position, first to last.
    pub(crate) fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.words.iter().enumerate().flat_map(|(index, &word)| {
            core::iter::successors((word != 0).then_some(word), |word| {
                let rest = word & (word - 1);
                (rest != 0).then_some(rest)
            })
            .map(move |word| index * 64 + word.trailing_zeros() as usize)
        })
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
    step.record("dirty", elements.dirty.count());
    measure(grove, &mut elements);
    // Every element now holds the run it states, so a run nothing holds is one nothing states. Only
    // on a frame where a run stopped being stated, which is the only way the cache comes to hold
    // one nothing wants.
    if elements.restated {
        grove.shaping.sweep();
    }
    axes(grove, &mut elements);
    extent(grove, &mut elements);
    scroll(grove, &mut elements);
    clip(&grove.tree, &mut elements);
    rank(&grove.tree, &mut elements);
    // Resolution ends here. What settles after it is its own step, and is timed as one.
    drop(resolving);
    let _step = trace_span!("settle").entered();
    inherit(&grove.tree, &mut elements);
    regions(grove, &mut elements);
    grove.elements = elements;
}

/// R1. What an element's own font makes of it: its character cell, and its run shaped in it.
///
/// Ahead of every other pass, and reading no geometry at all, because neither answer has anything to
/// do with where the element ended up. A monospaced run's max-content width is its longest line's
/// character count times its cell, so what a run asks of the down-pass is known before the
/// down-pass runs -- R1m reads it off the run left here.
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
    // An element left unmarked measures to what it measured to, which is what the column is already
    // holding: what it reads is its own run and face and nothing else, so one that only moved is
    // one of these.
    let lettered = core::mem::take(&mut elements.lettered);
    for at in lettered.iter() {
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
        elements.cell[leaf.slot()] = cell;
        // Both halves of what it measures to are read off this, by R1m and R2m, which write them
        // before anything reads either.
        elements.composed[leaf.slot()] = composed;
    }
    elements.lettered = lettered;
}

/// R1m, R2a, R2m and R2b: the measure across, the horizontal axis, then the measure down it makes
/// possible, then the vertical axis.
///
/// An element with a placement in motion is resolved twice on each axis -- once for each endpoint,
/// in the *same* context -- and the two answers are blended. That is what the resolver's purity
/// buys: the endpoints are consistent with each other because they were asked at the same position
/// in the same dependency order, against the same settled ancestors, and neither is remembered
/// afterwards.
fn axes(grove: &mut Grove, elements: &mut Elements) {
    across(grove, elements);
    axis(grove, elements, Axis::Horizontal);
    wrap(grove, elements);
    axis(grove, elements, Axis::Vertical);
}

/// One axis of the whole tree, in dependency order, through the one pure resolver.
fn axis(grove: &Grove, elements: &mut Elements, axis: Axis) {
    let _pass = trace_span!("axis", vertical = (axis == Axis::Vertical)).entered();
    let viewport = Section::new(Position::default(), grove.viewport);
    // What is to be resolved again, and what is resolved against something that may have moved --
    // marked as the walk finds it, ahead of where the walk is, since the order puts it there.
    let mut visit = elements.visiting(&[&elements.dirty], &[]);
    let mut offered = Offered::default();
    let mut from = 0;
    while let Some(at) = visit.next(from) {
        from = at + 1;
        // A measure may have moved under an element nothing was written to, before this axis: R1m's
        // before this one, R2m's before the vertical. The order puts what an element reads before
        // it, so spreading that here reaches everything resolved against it -- where what it reads
        // changed: a box that moved, or a measure that moved and that it reads.
        let follows = |on: Option<usize>, against: Against| {
            on.is_some_and(|on| {
                elements.answers.get(on)
                    || (elements.remeasured.get(on)
                        && reads_content(grove, elements.order[at], against, axis))
            })
        };
        if follows(elements.trunk(at), Against::Trunk)
            || follows(elements.anchor(at), Against::Anchor)
        {
            elements.dirty.set(at);
        }
        // Nothing it or anything it resolves against was written, so it resolves to the box it
        // already holds. Its column keeps that box, so what reads it here reads the right one.
        if !elements.dirty.get(at) {
            continue;
        }
        // Laid out across again, it may come out another width -- and wrap to another height, or
        // reach somewhere else below it -- so it is measured down again in R2m. Almost all of it
        // already was; what R1m's spreading reached here was not.
        if axis == Axis::Horizontal {
            elements.measures.set(at);
        }
        let leaf = elements.order[at];
        let slot = leaf.slot();
        let placement = grove.tree.placement(leaf);
        if axis == Axis::Horizontal {
            elements.upright[slot] = measurable(grove, placement, Axis::Vertical);
        }
        let context = context(grove, elements, viewport, at, axis, &mut offered);
        let (span, stretched) = geometry(grove, leaf, placement, &context, axis);
        // A stroke's ends are settled one axis at a time, into what it held: an element a measure
        // reached between the two axes is resolved on the second alone, and keeps the first.
        if let Some((from, to)) = stretched {
            elements.spanned[slot]
                .get_or_insert_with(Stretched::default)
                .set(axis, from, to);
        }
        let section = &mut elements.section[slot];
        let landed = match axis {
            Axis::Horizontal => {
                let landed = (section.position.x, section.area.width) == (span.near, span.extent());
                section.position.x = span.near;
                section.area.width = span.extent();
                landed
            }
            Axis::Vertical => {
                let landed =
                    (section.position.y, section.area.height) == (span.near, span.extent());
                section.position.y = span.near;
                section.area.height = span.extent();
                landed
            }
        };
        if !landed {
            elements.answers.set(at);
        }
        elements.unresolved.unset(at);
        offered.resolved(at);
        // What it offers may be different, so what is resolved against it is looked at -- unless
        // it was resolving again from the start, when everything resolved against it already is.
        if !elements.closed.get(at) && (elements.answers.get(at) || elements.remeasured.get(at)) {
            elements.dependents(at, &mut visit);
        }
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
    placement: Option<Placement>,
    context: &Context,
    axis: Axis,
) -> (Span, Option<(f32, f32)>) {
    let (trace, stroke) = match placement {
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

/// R1m. How wide each element's contents would like to be, before any width is laid out.
///
/// R2m across, and the reason [`content()`](crate::content) asks the same question on both axes --
/// *how large is what is inside me*. Bottom-up, so everything inside an element is measured before
/// the element asks, and an element takes the wider of two things:
///
/// - its run's max-content, its widest line unwrapped, which R1 left it holding
/// - how far right anything grown under it reaches, from its own left edge
///
/// The second is resolved with no horizontal box known at all, which is exactly what there is at
/// this point in the frame. So a child that reads one -- `100.pct()`, a column of a grid, an edge of
/// its trunk or of an anchor, a place in a run -- contributes nothing to the measure and is given its
/// real width by R2a like anything else; a child that describes its own extent, in pixels, in
/// letters or by its own content, is what is measured. A width that moved marks the element written,
/// and R2a spreads it to everything resolved against it.
///
/// What has to measure again is what was written to itself, and everything above it -- not what
/// is only being resolved again against something written, which R2m measures as well. Nothing
/// this measure reads is anyone else's box, so a subtree under a box that moved measures across to
/// what it did; see [`touched`](Elements::touched).
fn across(grove: &Grove, elements: &mut Elements) {
    let _pass = trace_span!("across").entered();
    if !elements.across {
        return;
    }
    // Up the order, so everything under an element is measured before it is -- and each one marks
    // what it hangs off, which the walk reaches after it.
    let mut from = elements.len();
    while let Some(at) = elements.measures_across.last_before(from) {
        from = at;
        if let Some(trunk) = elements.trunk(at) {
            elements.measures_across.set(trunk);
        }
        let slot = elements.slot(at);
        let run = elements.composed[slot]
            .as_ref()
            .map_or(0.0, |run| run.max_content());
        let width = run.max(spread(grove, elements, at));
        if width != elements.intrinsic[slot].width {
            elements.intrinsic[slot].width = width;
            // Ahead of R2a, which resolves it and what reads it against the new measure -- and
            // below every trunk it reaches, each of which asks it again on the way up.
            elements.dirty.set(at);
            elements.remeasured.set(at);
            elements.touched.set(at);
        }
    }
}

/// How far right the elements grown under the element at `at` reach from its left edge.
///
/// [`reach`] across: only a child that describes its own extent is counted, and a child is asked
/// only where it was [`touched`](Elements::touched) -- every other one reaches exactly as far as it
/// did when it last was, which is what [`spread`](Elements::spread) holds for it.
fn spread(grove: &Grove, elements: &mut Elements, at: usize) -> f32 {
    let mut spread: f32 = 0.0;
    let divided = tracks(grove, elements, at);
    for index in elements.branches[at] as usize..elements.branches[at + 1] as usize {
        let child_at = elements.children[index] as usize;
        let child = elements.order[child_at];
        if elements.touched.get(child_at) {
            let placement = grove.tree.placement(child);
            let far = measurable(grove, placement, Axis::Horizontal).then(|| {
                let context = unlaid(grove, elements, (at, divided), child_at);
                geometry(grove, child, placement, &context, Axis::Horizontal)
                    .0
                    .far
            });
            elements.spread[child.slot()] = far;
        }
        if let Some(far) = elements.spread[child.slot()] {
            spread = spread.max(far);
        }
    }
    spread
}

/// What one child resolves its horizontal axis against while its trunk is being measured across.
///
/// The same [`Context`] R2a will build, with every box taken as not laid out yet -- which none is,
/// and which a counted child never reads. What is real is what R1 and the measure below the trunk
/// already answered: every cell, and the child's own intrinsic width, so a width in letters or of
/// the child's own content still answers. The trunk's left edge is the origin, so what comes back
/// is how far right of it the child reaches.
///
/// The trunk comes with its grid already divided, since every child of it asks.
fn unlaid<'a>(
    grove: &Grove,
    elements: &'a Elements,
    (trunk, divided): (usize, Tracks),
    child: usize,
) -> Context<'a> {
    let offered = |at: usize, tracks: Tracks| {
        let slot = elements.slot(at);
        Basis {
            section: Section::default(),
            intrinsic: elements.intrinsic[slot],
            tracks,
            cell: elements.cell[slot],
            run: None,
        }
    };
    Context {
        axis: Axis::Horizontal,
        own: own(elements, child, Axis::Horizontal),
        trunk: offered(trunk, divided),
        anchor: match elements.anchor(child) {
            Some(anchor) => offered(anchor, tracks(grove, elements, anchor)),
            None => Basis::default(),
        },
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
/// converge. [`across`] is the same reading on the other axis, taken before any width is known.
///
/// Both halves are taken in the one walk up the order. What reaches below an element reads the
/// measures of everything under it, so it has to come after them -- and a run wraps at its own width
/// and against its own font, reading nothing but its element, so it can be measured wherever the
/// walk finds it. The run is the one R1 left the element holding: an element measuring again
/// because something under it moved says what it said last frame, and one that says something new
/// was shaped for it by R1 this frame.
fn wrap(grove: &mut Grove, elements: &mut Elements) {
    let _pass = trace_span!("wrap").entered();
    // What has to measure again: what was written to, and everything above it, because what an
    // element reaches over is what the elements under it resolved to. Up the order rather than down
    // it -- a measure travels toward the trunk and stops there, where a box travels away from it --
    // so each element marks what it hangs off, which the walk reaches after it.
    let mut from = elements.len();
    while let Some(at) = elements.measures.last_before(from) {
        from = at;
        if let Some(trunk) = elements.trunk(at) {
            elements.measures.set(trunk);
        }
        let slot = elements.slot(at);
        let shaped = elements.composed[slot]
            .as_ref()
            .map_or(0.0, |run| run.measure(elements.section[slot].width()));
        let height = shaped.max(reach(grove, elements, at));
        if height != elements.intrinsic[slot].height {
            elements.intrinsic[slot].height = height;
            // The measure moved under an element nothing was written to, and a placement reading
            // that measure has to be resolved against the new one. R2b runs after this and spreads
            // it, which is why it is safe to find out here.
            elements.dirty.set(at);
            elements.remeasured.set(at);
        }
    }
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
/// Whether a child counts at all is what R2a left in [`upright`](Elements::upright), so one that
/// does not is passed over without its placement being asked for.
fn reach(grove: &Grove, elements: &mut Elements, at: usize) -> f32 {
    let mut reach: f32 = 0.0;
    let divided = tracks(grove, elements, at);
    for index in elements.branches[at] as usize..elements.branches[at + 1] as usize {
        let child_at = elements.children[index] as usize;
        let child = elements.order[child_at];
        if elements.dirty.get(child_at) {
            let far = elements.upright[child.slot()].then(|| {
                let context = raised(grove, elements, (at, divided), child_at);
                let placement = grove.tree.placement(child);
                geometry(grove, child, placement, &context, Axis::Vertical)
                    .0
                    .far
            });
            elements.reached[child.slot()] = far;
        }
        if let Some(far) = elements.reached[child.slot()] {
            reach = reach.max(far);
        }
    }
    reach
}

/// Whether `leaf`'s placement on `axis` describes its own extent, and so counts toward the measure
/// of what it is grown under on that axis.
///
/// One question with two spellings, because a placement has two. A box asks it of its configuration
/// on the axis; a trace asks it of both of its ends, since either one reading a box on the axis is
/// enough to make the answer circular.
fn measurable(grove: &Grove, placement: Option<Placement>, axis: Axis) -> bool {
    match placement {
        Some(Placement::Traced(trace, _)) => {
            let ends = pinned_ends(trace, grove);
            ends.from.measurable(axis) && ends.to.measurable(axis)
        }
        Some(Placement::Boxed(location)) => pinned(location, grove, axis).measurable(axis),
        None => pinned(&Location::default(), grove, axis).measurable(axis),
    }
}

/// Whether `leaf`'s placement on `axis` reads the measured extent of what `against` names -- its
/// trunk's or its anchor's [`content`](crate::content) -- which is all a measure that moved changes
/// for what is resolved against it, where the box it measures stayed where it was.
///
/// An element in motion is resolved against what it left as well as what it declares, and either
/// may read anything, so it is answered yes.
fn reads_content(grove: &Grove, leaf: Leaf, against: Against, axis: Axis) -> bool {
    if grove.aspen.location(leaf).is_some() || grove.aspen.trace(leaf).is_some() {
        return true;
    }
    match grove.tree.placement(leaf) {
        Some(Placement::Boxed(location)) => pinned(location, grove, axis).reads_content(against),
        Some(Placement::Traced(trace, _)) => {
            let ends = pinned_ends(trace, grove);
            ends.from.reads_content(against, axis) || ends.to.reads_content(against, axis)
        }
        None => false,
    }
}

/// What one child resolves its vertical axis against while its trunk is being measured.
///
/// The same [`Context`] R2b will build, with every vertical reading taken as zero: no box on this
/// axis has resolved yet, which is the point of measuring. Every horizontal reading is real -- the
/// child's own width included -- so a height stated in columns, read off a width, or held in
/// proportion to its own still answers.
///
/// The trunk comes with its grid already divided, since every child of it asks.
fn raised<'a>(
    grove: &Grove,
    elements: &'a Elements,
    (trunk, divided): (usize, Tracks),
    child: usize,
) -> Context<'a> {
    let offered = |at: usize, tracks: Tracks| {
        let slot = elements.slot(at);
        Basis {
            section: flattened(elements.section[slot]),
            intrinsic: elements.intrinsic[slot],
            tracks,
            cell: elements.cell[slot],
            run: elements.composed[slot].as_deref(),
        }
    };
    Context {
        axis: Axis::Vertical,
        own: own(elements, child, Axis::Vertical),
        trunk: offered(trunk, divided),
        anchor: match elements.anchor(child) {
            Some(anchor) => offered(anchor, tracks(grove, elements, anchor)),
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
/// What an element reaches to is held, so a walk up the order visits what may reach somewhere else
/// -- what was resolved again, shown or hidden, or lost something under it -- and what hangs over
/// one whose reach moved.
///
/// [`visible(false)`]: crate::Place::visible
/// [`pinned`]: crate::Place::pinned
fn extent(grove: &mut Grove, elements: &mut Elements) {
    let _pass = trace_span!("extent").entered();
    let mut visit = elements.visiting(
        &[
            &elements.dirty,
            &elements.restyled,
            &elements.measures,
            &elements.fresh,
        ],
        &[],
    );
    // A child that came to be pinned or to float, or stopped, came to count toward what it hangs
    // off or stopped, whether or not it reaches anywhere else.
    if elements.rescoped {
        for at in elements.restyled.iter() {
            if let Some(trunk) = elements.trunk(at) {
                visit.set(trunk);
            }
        }
    }
    let mut from = elements.len();
    while let Some(at) = visit.last_before(from) {
        from = at;
        let leaf = elements.order[at];
        let slot = leaf.slot();
        let standing = grove.tree.standing(leaf);
        let (visible, scrolls) = (standing.visible(), standing.scrolls());
        // A hidden element is not content, and neither is anything under it: that is what makes
        // hiding the whole answer for parking something out of the way, rather than half of one.
        // It reaches nowhere, which is what keeps a hidden subtree out of what contains it.
        let extends = visible.then(|| {
            let section = elements.section[slot];
            let mut far = Position::new(section.right(), section.bottom());
            for child in elements.children(at) {
                // Pinned or floating, and so not content. See [`aside`](Elements::aside).
                if elements.aside.get(child) {
                    continue;
                }
                let Some(child) = elements.extends[elements.slot(child)] else {
                    continue;
                };
                far = Position::new(far.x.max(child.x), far.y.max(child.y));
            }
            match scrolls {
                Some(scroll) => {
                    grove.tree.set_extent(leaf, reached(scroll, section, far));
                    Position::new(section.right(), section.bottom())
                }
                None => far,
            }
        });
        if extends != elements.extends[slot] {
            elements.extends[slot] = extends;
            if let Some(trunk) = elements.trunk(at) {
                visit.set(trunk);
            }
        }
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
/// moved, and moved by whatever shift it and they were given.
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
///
/// What each element passes on is held, so the walk visits what was resolved again or restyled,
/// every region -- since what one scrolled to is never written to it -- and what hangs off an element
/// that passes on something new.
fn scroll(grove: &mut Grove, elements: &mut Elements) {
    let _pass = trace_span!("scroll", coasting = grove.coasting.len()).entered();
    view::asked(grove, elements);
    let mut visit = elements.visiting(
        &[&elements.dirty, &elements.restyled, &elements.fresh],
        &[&elements.scrolling],
    );
    let mut from = 0;
    while let Some(at) = visit.next(from) {
        from = at + 1;
        let leaf = elements.order[at];
        let slot = leaf.slot();
        let placed = elements.section[slot];
        let trunk = elements.trunk(at).map(|trunk| elements.slot(trunk));
        let standing = grove.tree.standing(leaf);
        let (shift, pinned, scrolls) = (standing.shift(), standing.pinned(), standing.scrolls());
        // A shift moves the element and everything grown under it, pinned or not, so it comes off
        // both accumulations -- the other way to an offset, which moves content toward the near
        // edge.
        let shifted = |by: Position| Position::new(by.x - shift.x, by.y - shift.y);
        let inherited = shifted(
            trunk
                .map(|trunk| elements.accumulated[trunk])
                .unwrap_or_default(),
        );
        // What a *pinned* element receives, which is the accumulation with its nearest scrolling
        // ancestor left out of it.
        let outside = shifted(
            trunk
                .map(|trunk| elements.unpinned[trunk])
                .unwrap_or_default(),
        );
        // A pinned element does not receive its nearest scrolling ancestor's offset, and receives
        // every offset outside that one: pinning is relative to the region the element sits in and
        // says nothing about what contains that region.
        let applied = match pinned {
            true => outside,
            false => inherited,
        };
        let drawn = Section::new(
            Position::new(placed.left() - applied.x, placed.top() - applied.y),
            placed.area,
        );
        if drawn != elements.drawn[slot] {
            elements.drawn[slot] = drawn;
            elements.moved.set(at);
            elements.restack = true;
        }
        // A stroke's ends travel with its box, by the same offset, because they are the same
        // geometry said two ways. A stroke nothing wrote to keeps the ends it settled at, and moves
        // under the offset exactly as its box does.
        if let Some(spanned) = elements.spanned[slot] {
            let stretched = Some(spanned.less(applied));
            if stretched != elements.stretched[slot] {
                elements.stretched[slot] = stretched;
                elements.moved.set(at);
            }
        }
        let (carried, escaped) = match scrolls {
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
        if (carried, escaped) != (elements.accumulated[slot], elements.unpinned[slot]) {
            elements.accumulated[slot] = carried;
            elements.unpinned[slot] = escaped;
            for child in elements.children(at) {
                visit.set(child);
            }
        }
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
///
/// What each element passes on is held. A clip reads no box but a region's own, and nothing it reads
/// is restyled, so the walk visits what is new, every region and every element floating out to one
/// it names, and what hangs off an element that passes on something new.
fn clip(tree: &Tree, elements: &mut Elements) {
    let _pass = trace_span!("clip").entered();
    let unbounded = Clipped::unbounded().0;
    let restyled = match elements.rescoped {
        true => &elements.restyled,
        false => &Positions::default(),
    };
    let mut visit = elements.visiting(
        &[&elements.fresh, restyled],
        &[&elements.scrolling, &elements.within],
    );
    let mut from = 0;
    while let Some(at) = visit.next(from) {
        from = at + 1;
        let leaf = elements.order[at];
        let slot = leaf.slot();
        let trunk = elements.trunk(at).map(|trunk| elements.slot(trunk));
        let standing = tree.standing(leaf);
        let inherited = trunk
            .map(|trunk| elements.passed[trunk])
            .unwrap_or(unbounded);
        // What a *floating* element is clipped to, which is the intersection with its nearest
        // scrolling ancestor's own rect left out of it. The same second accumulation R4 carries
        // for a pinned child, and for the same reason: both say "not part of this region", and both
        // have to say it about the region the element is actually in rather than about all of them.
        let outside = trunk
            .map(|trunk| elements.escaped[trunk])
            .unwrap_or(unbounded);
        // A floating element is positioned outside the region on purpose, so cutting it off at the
        // region's edge would undo the placement that put it there. How far out it reaches is the
        // element's own statement, because no one answer is right everywhere.
        let applied = match standing.floats() {
            // Held by whatever holds the region it left.
            Some(Escape::Region) => outside,
            // Held by nothing, which extraction reads as the surface: a clip is never wider than
            // what is being drawn on.
            Some(Escape::Surface) => unbounded,
            // Held by the element named, which is what that element passes down to what it holds.
            Some(Escape::Within(named)) => match within(elements, at, named) {
                Some(clip) => clip,
                // Named something that is not above it, so it holds nothing. Falling back to the
                // near answer rather than the far one: an element escapes no further than it was
                // told to, and a mis-named ancestor is not permission to leave everything.
                None => outside,
            },
            None => inherited,
        };
        if applied != elements.clip[slot] {
            elements.clip[slot] = applied;
            elements.moved.set(at);
            elements.restack = true;
        }
        let scrolls = standing.scrolls().is_some();
        let passed = match scrolls {
            true => applied.intersect(elements.drawn[slot]),
            false => applied,
        };
        let escaped = match scrolls {
            true => applied,
            false => outside,
        };
        if (passed, escaped) != (elements.passed[slot], elements.escaped[slot]) {
            elements.passed[slot] = passed;
            elements.escaped[slot] = escaped;
            for child in elements.children(at) {
                visit.set(child);
            }
        }
    }
}

/// What `named` leaves visible of what it holds, if it is above the element at `at` at all.
///
/// The walk is up the trunks from the element, so one that names something beside it rather than
/// above it gets `None` and is answered as though it had named the region it is in. Naming the
/// element rather than counting regions is what makes this survive a wrapper being added between
/// the two, which a count would not.
fn within(elements: &Elements, at: usize, named: Leaf) -> Option<Section> {
    let named = elements.at(named)?;
    let mut step = elements.trunk(at);
    while let Some(above) = step {
        // An ancestor resolves before what hangs off it, so one found here has already passed its
        // own clip down and the read is never of a position the pass has yet to reach.
        if above == named {
            return Some(elements.passed[elements.slot(above)]);
        }
        step = elements.trunk(above);
    }
    None
}

/// R6. Declared elevation accumulates down the tree, and allocation order settles what it leaves
/// equal.
///
/// One walk in the same dependency order the axes used, which puts every trunk before what hangs
/// off it -- so a trunk's rank is already this frame's when what hangs off it reads it. Nothing here
/// reads a box: where an element sits in the stack has nothing to do with where it sits on the
/// surface. What is visited is what is new or restyled, and what hangs off an element whose rank
/// moved.
fn rank(tree: &Tree, elements: &mut Elements) {
    let _pass = trace_span!("rank").entered();
    let mut visit = elements.visiting(&[&elements.fresh, &elements.restyled], &[]);
    let mut from = 0;
    while let Some(at) = visit.next(from) {
        from = at + 1;
        let trunk = elements
            .trunk(at)
            .map(|trunk| elements.rank[elements.slot(trunk)].stack)
            .unwrap_or_default();
        let leaf = elements.order[at];
        let standing = tree.standing(leaf);
        let rank = ResolvedElevation {
            stack: standing.elevation().accumulate(trunk),
            growth: standing.growth(),
        };
        if rank != elements.rank[leaf.slot()] {
            elements.rank[leaf.slot()] = rank;
            elements.moved.set(at);
            elements.restack = true;
            // What the stack is sorted on moved, where the element is in it at all.
            elements.reranked |= !standing.gestures().intangible;
            for child in elements.children(at) {
                visit.set(child);
            }
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
///
/// What is visited is what is new or restyled, and what hangs off an element whose product moved --
/// so hiding a subtree visits all of it once, on the frame it is hidden, and none of it after.
fn inherit(tree: &Tree, elements: &mut Elements) {
    let _pass = trace_span!("inherit").entered();
    let mut visit = elements.visiting(&[&elements.fresh, &elements.restyled], &[]);
    let mut from = 0;
    while let Some(at) = visit.next(from) {
        from = at + 1;
        let trunk = elements
            .trunk(at)
            .map(|trunk| elements.inherited[elements.slot(trunk)])
            .unwrap_or_default();
        let leaf = elements.order[at];
        let standing = tree.standing(leaf);
        let product = Inherited::under(
            trunk,
            standing.visible(),
            standing.opacity(),
            standing.disabled(),
        );
        if product != elements.inherited[leaf.slot()] {
            elements.inherited[leaf.slot()] = product;
            elements.moved.set(at);
            elements.restack = true;
            for child in elements.children(at) {
                visit.set(child);
            }
        }
    }
}

/// R8. The box stack the next frame's dispatch reads.
///
/// Membership is universal: an element is here because it is there. What is left out is what is
/// not there at all -- hidden, fully transparent, or clipped away by a region it sits inside -- and
/// what is `intangible`. The one read of the stack is its top, the front-most tangible element at a
/// point, and a gesture passes through an intangible one to whatever is beneath it: holding it
/// would change no answer the stack gives, and would make the stack the size of everything drawn
/// rather than of what can be pressed.
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
    // nothing. What is sorted is what the order was made holding, every tangible position.
    if elements.reranked {
        let Elements {
            rank,
            order,
            stacking,
            ..
        } = elements;
        let rank = |at: &u32| rank[order[*at as usize].slot()];
        stacking.sort_unstable_by_key(|at| core::cmp::Reverse(rank(at)));
    }
    let Grove { tree, stack, .. } = grove;
    stack.settle(elements.stacking.iter().filter_map(|&at| {
        let leaf = elements.order[at as usize];
        let slot = leaf.slot();
        let gestures = tree.standing(leaf).gestures();
        let inherited = elements.inherited[slot];
        if !inherited.present() {
            return None;
        }
        let section = elements.drawn[slot];
        let clip = elements.clip[slot];
        if section.intersect(clip).is_empty() {
            return None;
        }
        Some(Region {
            leaf,
            section,
            clip,
            shape: gestures.shape,
            receives: gestures.receives,
            disabled: inherited.disabled,
        })
    }));
}

/// Everything one axis of the element at `at` resolves against.
fn context<'a>(
    grove: &Grove,
    elements: &'a Elements,
    viewport: Section,
    at: usize,
    axis: Axis,
    offered: &mut Offered,
) -> Context<'a> {
    let (trunk, anchor) = (elements.trunk(at), elements.anchor(at));
    Context {
        axis,
        // Its box on this axis is the answer being computed, and its own grid divides it for its
        // children rather than for itself, so neither is readable here. Its box on the *other* axis
        // is readable exactly when that axis has already run, which is what [`own`] hands over.
        own: own(elements, at, axis),
        // A top-level element has no trunk, and fills the viewport instead.
        trunk: match trunk {
            Some(trunk) => offered.basis(grove, elements, trunk, viewport, axis),
            None => basis(elements, None, viewport, axis, Tracks::default()),
        },
        // A placement that reads an anchor it has not been given resolves against a zero box.
        anchor: basis(
            elements,
            anchor,
            Section::default(),
            axis,
            anchor.map_or_else(Tracks::default, |anchor| tracks(grove, elements, anchor)),
        ),
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
    let slot = elements.slot(at);
    Basis {
        section: match axis {
            Axis::Horizontal => Section::default(),
            Axis::Vertical => flattened(elements.section[slot]),
        },
        intrinsic: elements.intrinsic[slot],
        tracks: Tracks::default(),
        cell: elements.cell[slot],
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
    tracks: Tracks,
) -> Basis<'a> {
    let Some(at) = at else {
        return Basis {
            section: fallback,
            ..Basis::default()
        };
    };
    let slot = elements.slot(at);
    Basis {
        section: match (elements.unresolved.get(at), axis) {
            (false, Axis::Horizontal) => flattened(elements.section[slot]),
            (false, Axis::Vertical) => elements.section[slot],
            (true, _) => fallback,
        },
        intrinsic: elements.intrinsic[slot],
        tracks,
        cell: elements.cell[slot],
        run: elements.composed[slot].as_deref(),
    }
}

/// What the trunk the last element resolved against offers, held for the next element to ask.
///
/// Siblings sit together in the order and each asks the same trunk the same thing -- and nothing a
/// trunk offers moves once the walk is past it, so what was read for one serves the rest. The one
/// trunk the walk has yet to reach, a cycle's remainder, is let go of when the walk resolves it.
#[derive(Default)]
struct Offered {
    held: Option<(usize, Section, Area, Tracks, Area)>,
}

impl Offered {
    fn basis<'a>(
        &mut self,
        grove: &Grove,
        elements: &'a Elements,
        trunk: usize,
        viewport: Section,
        axis: Axis,
    ) -> Basis<'a> {
        let (section, intrinsic, tracks, cell) = match self.held {
            Some((held, section, intrinsic, tracks, cell)) if held == trunk => {
                (section, intrinsic, tracks, cell)
            }
            _ => {
                let basis = basis(
                    elements,
                    Some(trunk),
                    viewport,
                    axis,
                    tracks(grove, elements, trunk),
                );
                self.held = Some((
                    trunk,
                    basis.section,
                    basis.intrinsic,
                    basis.tracks,
                    basis.cell,
                ));
                (basis.section, basis.intrinsic, basis.tracks, basis.cell)
            }
        };
        Basis {
            section,
            intrinsic,
            tracks,
            cell,
            run: elements.composed[elements.slot(trunk)].as_deref(),
        }
    }

    /// The walk resolved the element at `at`, so what it offers may be different.
    fn resolved(&mut self, at: usize) {
        if self.held.is_some_and(|(held, ..)| held == at) {
            self.held = None;
        }
    }
}

/// The grid of the element at `at`, divided at the breakpoint in force: what a child of it reads a
/// track of.
fn tracks(grove: &Grove, elements: &Elements, at: usize) -> Tracks {
    grove
        .tree
        .standing(elements.order[at])
        .tracks(grove.layout, grove.short)
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
/// frame reads none of it. When it is built again, what every element already resolved to stays
/// where it is held, by slot, so building it again loses nothing but the elements that went.
fn structure(elements: &mut Elements, grove: &Grove) {
    let _pass = trace_span!("structure").entered();
    let tree = &grove.tree;
    let leaves = tree.leaves();
    let count = leaves.len();
    // Where each element sits among the leaves, by the index of its name, which is dense: a lookup
    // is an index rather than a hash.
    let names = leaves
        .iter()
        .map(|leaf| leaf.slot() + 1)
        .max()
        .unwrap_or_default();
    let mut among: Vec<u32> = vec![NOWHERE; names];
    for (at, leaf) in leaves.iter().enumerate() {
        among[leaf.slot()] = at as u32;
    }
    let find = |leaf: Leaf| -> Option<usize> {
        let at = *among.get(leaf.slot())? as usize;
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
    elements.waits = false;
    for &leaf in &leaves {
        let declared = [tree.trunk(leaf), tree.anchor(leaf)];
        let on = declared.map(|on| on.and_then(find));
        elements.waits |= declared[1].is_some() && on[1].is_none();
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
    let placed = order
        .iter()
        .map(|&at| {
            let [trunk, anchor] = depends[at];
            Placed {
                leaf: leaves[at],
                trunk: trunk.map(|on| ranked[on]),
                anchor: anchor.map(|on| ranked[on]),
                from: elements.at(leaves[at]),
            }
        })
        .collect();
    settle(elements, grove, placed, names, false);
}

/// One element's place in an order being made: what it is, the positions of what it hangs off and
/// what it is anchored to, and where it sat in the order being replaced.
struct Placed {
    leaf: Leaf,
    trunk: Option<usize>,
    anchor: Option<usize>,
    from: Option<usize>,
}

/// The order repaired for what was grown and what withered since it was made, rather than made
/// again: what withered is taken out, and what was grown is placed after everything already there,
/// in the order it was grown. Reports whether it could be, having changed nothing where it could
/// not.
///
/// Everything [`structure`] makes the order from is read here too, but only of what changed: what
/// stays keeps its place relative to the rest, so what it depends on is still before it, and is
/// carried without a lookup. What is grown depends on what it was grown under, which was grown
/// before it, and on an anchor -- which has to be placed already. One that is not, or an order whose
/// anchors point forward, is left to [`structure`], which places everything from scratch.
fn patch(elements: &mut Elements, grove: &Grove, written: &Written) -> bool {
    let _pass = trace_span!("patch").entered();
    let tree = &grove.tree;
    let held = elements.len();
    let mut kept = vec![true; held];
    for &(leaf, _) in written.withered {
        if let Some(at) = elements.at(leaf) {
            kept[at] = false;
        }
    }
    let mut moved: Vec<u32> = vec![NOWHERE; held];
    let mut placed: Vec<Placed> = Vec::with_capacity(held + written.grown.len());
    for at in 0..held {
        if !kept[at] {
            continue;
        }
        // Earlier in the order than the element itself, or this is an order a cycle left a
        // remainder in, which only building it again answers.
        let earlier = |on: Option<usize>| match on {
            Some(on) if on < at => Ok((moved[on] != NOWHERE).then_some(moved[on] as usize)),
            Some(_) => Err(()),
            None => Ok(None),
        };
        let (Ok(trunk), Ok(anchor)) = (earlier(elements.trunk(at)), earlier(elements.anchor(at)))
        else {
            return false;
        };
        moved[at] = placed.len() as u32;
        placed.push(Placed {
            leaf: elements.order[at],
            trunk,
            anchor,
            from: Some(at),
        });
    }
    let names = written
        .grown
        .iter()
        .map(|leaf| leaf.slot() + 1)
        .max()
        .unwrap_or_default()
        .max(elements.sits.len());
    // Where each element grown this frame was placed, by the index of its name.
    let placing = trace_span!("grown").entered();
    let mut grown: Vec<u32> = vec![NOWHERE; names];
    for &leaf in written.grown {
        if !tree.is_live(leaf) || elements.at(leaf).is_some() {
            continue;
        }
        let find = |on: Leaf| match elements.at(on) {
            Some(at) => (moved[at] != NOWHERE).then_some(moved[at] as usize),
            None => grown
                .get(on.slot())
                .filter(|at| **at != NOWHERE && placed[**at as usize].leaf == on)
                .map(|at| *at as usize),
        };
        let trunk = match tree.trunk(leaf) {
            Some(on) => match find(on) {
                Some(at) => Some(at),
                None => return false,
            },
            None => None,
        };
        let anchor = match tree.anchor(leaf) {
            Some(on) => match find(on) {
                Some(at) => Some(at),
                None => return false,
            },
            None => None,
        };
        grown[leaf.slot()] = placed.len() as u32;
        placed.push(Placed {
            leaf,
            trunk,
            anchor,
            from: None,
        });
    }
    drop(placing);
    settle(elements, grove, placed, names, true);
    true
}

/// Makes `placed` the order: what is grown under each element found again, and what every pass
/// visits. `names` is one past the largest slot of any element placed.
///
/// Nothing any element resolved to moves, since it is held by slot: an element new to the order has
/// its slot cleared of whatever withered from it, and everything else keeps what it holds.
/// `repaired` says the order was repaired rather than made again -- what stays is in the order it
/// was, with what is new after it -- so only what was grown passes on anything new, where an order
/// made again may hang anything off anything.
fn settle(
    elements: &mut Elements,
    grove: &Grove,
    placed: Vec<Placed>,
    names: usize,
    repaired: bool,
) {
    let _pass = trace_span!("order").entered();
    let tree = &grove.tree;
    let count = placed.len();
    match repaired {
        true => {
            elements.fresh.reset(count);
            for (position, placed) in placed.iter().enumerate() {
                if placed.from.is_none() {
                    elements.fresh.set(position);
                }
            }
        }
        false => elements.fresh.fill(count),
    }
    // A slot past what the columns held is new to them, and already holds what an element starts
    // from. One before it held something that withered.
    let held = elements.section.len();
    columns(elements, &Lengthen(names));
    for placed in &placed {
        let slot = placed.leaf.slot();
        if placed.from.is_none() && slot < held {
            columns(elements, &Clear(slot));
        }
    }
    elements.order.clear();
    elements.trunk.clear();
    elements.anchor.clear();
    elements.sits.clear();
    elements.sits.resize(names, NOWHERE);
    for (position, placed) in placed.iter().enumerate() {
        elements.sits[placed.leaf.slot()] = position as u32;
        elements.order.push(placed.leaf);
        elements
            .trunk
            .push(placed.trunk.map_or(NOWHERE, |on| on as u32));
        elements
            .anchor
            .push(placed.anchor.map_or(NOWHERE, |on| on as u32));
    }
    // What is grown under each element, by position: counted, summed into where each element's run
    // starts, and filled.
    elements.branches.clear();
    elements.branches.resize(count + 1, 0);
    for trunk in elements.trunk.iter().filter_map(|&on| position(on)) {
        elements.branches[trunk + 1] += 1;
    }
    for at in 0..count {
        elements.branches[at + 1] += elements.branches[at];
    }
    let mut filling = elements.branches.clone();
    elements.children.clear();
    elements.children.resize(count, 0);
    for at in 0..count {
        if let Some(trunk) = elements.trunk(at) {
            elements.children[filling[trunk] as usize] = at as u32;
            filling[trunk] += 1;
        }
    }
    // What is anchored to each element, the same way.
    elements.anchoring.clear();
    elements.anchoring.resize(count + 1, 0);
    for anchor in elements.anchor.iter().filter_map(|&on| position(on)) {
        elements.anchoring[anchor + 1] += 1;
    }
    for at in 0..count {
        elements.anchoring[at + 1] += elements.anchoring[at];
    }
    let mut filling = elements.anchoring.clone();
    elements.anchored.clear();
    elements
        .anchored
        .resize(elements.anchoring[count] as usize, 0);
    for at in 0..count {
        if let Some(anchor) = elements.anchor(at) {
            elements.anchored[filling[anchor] as usize] = at as u32;
            filling[anchor] += 1;
        }
    }
    // What every pass visits whatever was written, and what the stack is sorted from.
    elements.forward.clear();
    elements.stacking.clear();
    for at in 0..count {
        let ahead = |on: Option<usize>| on.is_some_and(|on| on >= at);
        if ahead(elements.trunk(at)) || ahead(elements.anchor(at)) {
            elements.forward.push(at as u32);
        }
        if !tree.standing(elements.order[at]).gestures().intangible {
            elements.stacking.push(at as u32);
        }
    }
    scopes(elements, tree);
    // Everything the stack is built from may have moved, since what is in it did.
    elements.restack = true;
    elements.reranked = true;
}

/// Finds again which elements are regions, which float out to an element they name, and which are
/// grown inside a region without being its content: what R3, R4 and R5 read of every element, held
/// beside the order. Wherever the order is made, and wherever what scrolls changed.
fn scopes(elements: &mut Elements, tree: &Tree) {
    elements.scrolling.clear();
    elements.within.clear();
    elements.aside.reset(elements.len());
    for (at, &leaf) in elements.order.iter().enumerate() {
        let standing = tree.standing(leaf);
        if standing.scrolls().is_some() {
            elements.scrolling.push(at as u32);
        }
        let floats = standing.floats();
        if let Some(Escape::Within(_)) = floats {
            elements.within.push(at as u32);
        }
        if standing.pinned() || floats.is_some() {
            elements.aside.set(at);
        }
    }
}

/// Every column held per element, each with what an element new to the order starts from. Listed
/// once, so lengthening them and clearing one element's slot cannot disagree about which there are.
fn columns(elements: &mut Elements, each: &impl Each) {
    each.apply(&mut elements.section, Section::default());
    each.apply(&mut elements.intrinsic, Area::default());
    each.apply(&mut elements.reached, None);
    each.apply(&mut elements.upright, false);
    each.apply(&mut elements.spread, None);
    each.apply(&mut elements.cell, Area::default());
    each.apply(&mut elements.composed, None);
    each.apply(&mut elements.spanned, None);
    each.apply(&mut elements.drawn, Section::default());
    each.apply(&mut elements.stretched, None);
    each.apply(&mut elements.clip, Clipped::unbounded().0);
    each.apply(&mut elements.rank, ResolvedElevation::default());
    each.apply(&mut elements.inherited, Inherited::default());
    // What the accumulations pass on, which an element new to the order is visited for.
    each.apply(&mut elements.extends, None);
    each.apply(&mut elements.accumulated, Position::default());
    each.apply(&mut elements.unpinned, Position::default());
    each.apply(&mut elements.passed, Clipped::unbounded().0);
    each.apply(&mut elements.escaped, Clipped::unbounded().0);
}

/// What is done to every column.
trait Each {
    fn apply<T: Clone>(&self, column: &mut Vec<T>, fresh: T);
}

/// Every column at least this many slots long, each new slot holding what an element starts from.
struct Lengthen(usize);

impl Each for Lengthen {
    fn apply<T: Clone>(&self, column: &mut Vec<T>, fresh: T) {
        if column.len() < self.0 {
            column.resize(self.0, fresh);
        }
    }
}

/// One slot of every column back to what an element starts from, for an element new to the order
/// in a slot something withered from.
struct Clear(usize);

impl Each for Clear {
    fn apply<T: Clone>(&self, column: &mut Vec<T>, fresh: T) {
        column[self.0] = fresh;
    }
}

/// What this frame has to resolve, and what it may leave alone.
///
/// An element resolves again if it was written to, or if what it hangs off or is anchored to does.
/// That closure is one walk of the order rather than a search, because the order already puts a
/// dependency before what depends on it -- and the walk goes from mark to mark, each element
/// resolving again marking what is resolved against it. What was written is found by name and
/// marked where it sits, so the walk costs what resolves again and the marking costs what was
/// written.
///
/// An element that was restyled is read again and marked as moved, and resolves again only if
/// something it is placed by was written as well.
fn read(elements: &mut Elements, grove: &Grove) {
    let _pass = trace_span!("elements").entered();
    let written = grove.tree.written();
    // What withered from the order, and what it drew: what extraction lets go of. Found before the
    // order is repaired, while it still says where each sat -- an element grown and withered between
    // two frames was never in it, and has nothing drawn to let go of.
    let mut gone = core::mem::take(&mut elements.gone);
    gone.clear();
    gone.extend(
        written
            .withered
            .iter()
            .filter(|(leaf, _)| elements.at(*leaf).is_some()),
    );
    // What each held stays in its slot until something is grown there, which is harmless for a
    // value and not for a run: the shaping cache keeps a run for as long as anything holds it.
    for &(leaf, _) in &gone {
        elements.composed[leaf.slot()] = None;
    }
    elements.gone = gone;
    elements.restack = false;
    elements.reranked = false;
    // Nothing is new unless the order is made or repaired below, which marks what is.
    elements.fresh.reset(0);
    if written.restructured
        && (written.reordered || elements.waits || !patch(elements, grove, &written))
    {
        structure(elements, grove);
    }
    let count = elements.len();
    elements.restyled.reset(count);
    for &leaf in written.restyled {
        if let Some(at) = elements.at(leaf) {
            elements.restyled.set(at);
        }
    }
    // An element that came to scroll, or stopped, is restyled, and is a region or is not one from
    // here on.
    elements.rescoped = written.rescoped;
    if written.rescoped {
        scopes(elements, &grove.tree);
    }
    elements.restated = written.restated;
    if written.all {
        elements.fresh.fill(count);
    }
    let marked = |marks: &mut Positions, leaves: &crate::leaf::Leaves| {
        for &leaf in leaves {
            if let Some(at) = elements.at(leaf) {
                marks.set(at);
            }
        }
    };
    let mut dirty = Positions::default();
    let mut lettered = Positions::default();
    match written.all {
        true => {
            dirty.fill(count);
            lettered.fill(count);
        }
        false => {
            dirty.reset(count);
            lettered.reset(count);
            marked(&mut lettered, written.lettered);
            marked(&mut dirty, written.declared);
        }
    }
    let mut measures = Positions::default();
    measures.reset(count);
    marked(&mut measures, written.measures);
    let mut touched = Positions::default();
    if written.across {
        touched.fill(count);
    } else {
        touched.reset(count);
        marked(&mut touched, written.declared);
    }
    elements.dirty = dirty;
    elements.lettered = lettered;
    elements.measures = measures;
    match written.repainted {
        true => elements.moved.fill(count),
        false => elements.moved.reset(count),
    }
    elements.moved.add(&elements.restyled);
    elements.answers.reset(count);
    elements.remeasured.reset(count);
    // What R1m reads, taken before the closure below spreads anything: what was written to each
    // element itself, and with it what something went from. Everything, where what every measure
    // across reads changed; where only the viewport moved, it changed nothing a measure reads, and
    // what was written is what was. Nothing at all, and nothing is filled in, where nothing was
    // written and nothing went.
    elements.across =
        written.across || !written.declared.is_empty() || !written.measures.is_empty();
    if elements.across {
        elements.measures_across.copy_from(&touched);
        elements.measures_across.add(&elements.measures);
        elements.touched = touched;
    }
    // A dependency the order could not put first is one a cycle left in the remainder, and it has
    // no answer yet. Resolving such an element is the same fallback the rest of resolution gives
    // it: it is answered against what its dependency last was, so it is answered again.
    if !written.all {
        let mut dirty = core::mem::take(&mut elements.dirty);
        for &at in &elements.forward {
            dirty.set(at as usize);
        }
        let mut from = 0;
        while let Some(at) = dirty.next(from) {
            from = at + 1;
            elements.dependents(at, &mut dirty);
        }
        elements.dirty = dirty;
    }
    elements.answers.add(&elements.dirty);
    elements.moved.add(&elements.dirty);
    elements.measures.add(&elements.dirty);
    // A clean element offers the box it settled at, and offers it from the start, because nothing
    // this frame is going to compute it again. One being resolved again offers nothing until an
    // axis reaches it, which is what it did when every element was resolved.
    elements.unresolved.copy_from(&elements.dirty);
    elements.closed.copy_from(&elements.dirty);
}
