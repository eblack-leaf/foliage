//! Rowan -- resolution.
//!
//! # Rowan recomputes. Elm decides what changed.
//!
//! Resolution runs over everything, every frame. There is no dirty tracking, no invalidation, and
//! no mechanism by which a value can be stale because something forgot to mark it. The cost is
//! arithmetic over a handful of numbers per element; the saving is a whole class of bug. Nothing is
//! re-uploaded on account of it, because extraction compares against a cached copy and sends only
//! genuine differences.
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
use tracing::trace_span;

use crate::aspen::Departed;
use crate::coordinate::{Area, Axis, Position, Section};
use crate::elevation::ResolvedElevation;
use crate::grove::Grove;
use crate::interaction::focus;
use crate::interaction::stack::Region;
use crate::leaf::Leaf;
use crate::lifecycle::Inherited;
use crate::placement::grid::Tracks;
use crate::placement::location::Location;
use crate::placement::resolve::{Basis, Context, Span, resolve};
use crate::placement::role::Config;
use crate::tree::Tree;
use crate::view::{Clipped, range};

/// Where the layout put an element. What its children resolve against.
#[derive(Component, Copy, Clone, Debug, Default)]
pub(crate) struct Placed(pub(crate) Section);

/// Where an element is on screen: its [`Placed`] box less every scrolling ancestor's accumulated
/// offset. What drawing, clipping and hit-testing read.
///
/// The two are deliberately separate. A change to the first means the layout moved a box and its
/// children have to follow; a change to the second means only that the box moved under a scroll.
///
/// Logical pixels, like every other coordinate. The scale factor is applied in the render backend
/// and nowhere else.
#[derive(Component, Copy, Clone, Debug, Default)]
pub(crate) struct Drawn(pub(crate) Section);

/// The character cell an element's own font and size make, as R1 measured it.
///
/// On every element, because every element may be sized in characters: it is what
/// [`letters`](crate::Source::letters) and a letter-pitched track are measured in. An element that
/// named no font and no size has none, and reads zero.
#[derive(Component, Copy, Clone, Debug, Default)]
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
#[derive(Component, Copy, Clone, Debug, Default)]
pub(crate) struct Intrinsic(pub(crate) Area);

/// Steps 6 and 7. Declared state becomes resolved geometry and resolved products, for everything.
pub(crate) fn run(grove: &mut Grove) {
    let order = order(&grove.tree);
    {
        let _step = trace_span!("resolve", elements = order.len()).entered();
        measure(grove, &order);
        let boxes = axes(grove, &order);
        // Both of the passes that shape have run, so what is not held now is a run nothing states.
        grove.shaping.sweep();
        extent(grove, &order, &boxes);
        scroll(grove, &order, &boxes);
        clip(grove, &order);
        rank(grove, &order);
    }
    let _step = trace_span!("settle").entered();
    inherit(grove, &order);
    regions(grove, &order);
    focus::settle(grove);
}

/// R1. What an element's own font makes of it: its character cell, and the widest its content would
/// like to be.
///
/// Ahead of every other pass, and reading no geometry at all, because neither answer has anything to
/// do with where the element ended up. A monospaced run's max-content width is its longest line's
/// character count times its cell, so the whole of the down-pass's input is available before the
/// down-pass runs -- which is what leaves only the up-pass with anything to measure.
fn measure(grove: &mut Grove, order: &[Leaf]) {
    let _pass = trace_span!("measure").entered();
    let Grove {
        tree,
        fonts,
        shaping,
        layout,
        short,
        ..
    } = grove;
    for &leaf in order {
        // No font and no size is no cell, and nothing measured in one.
        let Some(typeface) = tree.typeface(leaf) else {
            continue;
        };
        let size = typeface.size.at(*layout, *short);
        let cell = fonts.cell(typeface.font, size);
        let width = match tree.lettering(leaf) {
            Some(value) => shaping.shape(fonts, typeface.font, size, value).max_content(),
            None => 0.0,
        };
        tree.set_cell(leaf, cell);
        // The height half is R2m's, and is written before anything reads it.
        tree.set_intrinsic(leaf, Area::new(width, 0.0));
    }
}

/// R2a, R2m and R2b: the horizontal axis, then the measure it makes possible, then the vertical one.
///
/// An element with a placement in motion is resolved twice on each axis -- once for each endpoint,
/// in the *same* context -- and the two answers are blended. That is what the resolver's purity
/// buys: the endpoints are consistent with each other because they were asked at the same position
/// in the same dependency order, against the same settled ancestors, and neither is remembered
/// afterwards.
fn axes(grove: &mut Grove, order: &[Leaf]) -> HashMap<Leaf, Section> {
    let mut boxes: HashMap<Leaf, Section> = HashMap::with_capacity(order.len());
    axis(grove, order, Axis::Horizontal, &mut boxes);
    wrap(grove, order, &boxes);
    axis(grove, order, Axis::Vertical, &mut boxes);
    boxes
}

/// One axis of the whole tree, in dependency order, through the one pure resolver.
fn axis(grove: &Grove, order: &[Leaf], axis: Axis, boxes: &mut HashMap<Leaf, Section>) {
    let _pass = trace_span!("axis", vertical = (axis == Axis::Vertical)).entered();
    let viewport = Section::new(Position::default(), grove.viewport);
    let fallback = Location::default();
    for &leaf in order {
        let context = context(grove, boxes, viewport, leaf, axis);
        let span = span(grove, leaf, &fallback, &context, axis);
        let section = boxes.entry(leaf).or_default();
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
    }
}

/// One axis of where `leaf` currently is: what it declares, blended with the endpoint a motion left.
///
/// The one place a placement becomes a span, so a measure and a layout are answering the same
/// question -- an element in motion is measured where it *is* rather than where it is going.
fn span(
    grove: &Grove,
    leaf: Leaf,
    fallback: &Location,
    context: &Context,
    axis: Axis,
) -> Span {
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
fn wrap(grove: &mut Grove, order: &[Leaf], boxes: &HashMap<Leaf, Section>) {
    let _pass = trace_span!("wrap").entered();
    let fallback = Location::default();
    for &leaf in order.iter().rev() {
        let width = boxes.get(&leaf).copied().unwrap_or_default().width();
        let height = wrapped(grove, leaf, width).max(reach(grove, boxes, &fallback, leaf));
        let mut intrinsic = grove.tree.intrinsic(leaf);
        intrinsic.height = height;
        grove.tree.set_intrinsic(leaf, intrinsic);
    }
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
fn reach(
    grove: &Grove,
    boxes: &HashMap<Leaf, Section>,
    fallback: &Location,
    leaf: Leaf,
) -> f32 {
    let mut reach: f32 = 0.0;
    for child in grove.tree.branches(leaf) {
        let location = grove.tree.location(child).unwrap_or(fallback);
        if !pinned(location, grove, Axis::Vertical).measurable() {
            continue;
        }
        let context = raised(grove, boxes, leaf, child);
        reach = reach.max(span(grove, child, fallback, &context, Axis::Vertical).far);
    }
    reach
}

/// What one child resolves its vertical axis against while its trunk is being measured.
///
/// The same [`Context`] R2b will build, with every vertical reading taken as zero: no box on this
/// axis has resolved yet, which is the point of measuring. Every horizontal reading is real, so a
/// height stated in columns or read off a width still answers.
fn raised(
    grove: &Grove,
    boxes: &HashMap<Leaf, Section>,
    trunk: Leaf,
    child: Leaf,
) -> Context {
    let flattened = |leaf: Leaf| {
        let section = boxes.get(&leaf).copied().unwrap_or_default();
        Basis {
            section: Section::new(
                Position::new(section.left(), 0.0),
                Area::new(section.width(), 0.0),
            ),
            intrinsic: grove.tree.intrinsic(leaf),
            tracks: grove
                .tree
                .grid(leaf)
                .unwrap_or_default()
                .tracks(grove.layout, grove.short),
            cell: grove.tree.cell(leaf),
        }
    };
    Context {
        axis: Axis::Vertical,
        own: Basis {
            section: Section::default(),
            intrinsic: grove.tree.intrinsic(child),
            tracks: Tracks::default(),
            cell: grove.tree.cell(child),
        },
        trunk: flattened(trunk),
        anchor: match grove.tree.anchor(child) {
            Some(anchor) => flattened(anchor),
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
fn extent(grove: &mut Grove, order: &[Leaf], boxes: &HashMap<Leaf, Section>) {
    let _pass = trace_span!("extent").entered();
    // The far corner of everything under an element, in absolute coordinates.
    let mut reach: HashMap<Leaf, Position> = HashMap::with_capacity(order.len());
    for &leaf in order.iter().rev() {
        // A hidden element is not content, and neither is anything under it: that is what makes
        // hiding the whole answer for parking something out of the way, rather than half of one.
        if !grove.tree.visible(leaf).0 {
            continue;
        }
        let section = boxes.get(&leaf).copied().unwrap_or_default();
        let mut far = Position::new(section.right(), section.bottom());
        for child in grove.tree.branches(leaf) {
            let Some(child) = reach.get(&child).copied() else {
                continue;
            };
            far = Position::new(far.x.max(child.x), far.y.max(child.y));
        }
        if grove.tree.scrolls(leaf).is_some() {
            // Measured outward from the region's own near edges and never smaller than its own
            // box, so content sitting behind the origin creates no range to scroll back into and an
            // empty region has a range of zero rather than a negative one.
            grove.tree.set_extent(
                leaf,
                Area::new(
                    (far.x - section.left()).max(section.width()),
                    (far.y - section.top()).max(section.height()),
                ),
            );
            far = Position::new(section.right(), section.bottom());
        }
        reach.insert(leaf, far);
    }
}

/// R4. Where each element is drawn: where the layout put it, less what its scrolling ancestors have
/// moved.
///
/// A region's own box does not move under its own offset -- what moves is everything grown inside
/// it. The offset is clamped here, against the extent R3 just measured, so a region whose content
/// shrank under it comes back into range on the next frame rather than staying somewhere it can no
/// longer reach.
fn scroll(grove: &mut Grove, order: &[Leaf], boxes: &HashMap<Leaf, Section>) {
    let _pass = trace_span!("scroll").entered();
    let mut accumulated: HashMap<Leaf, Position> = HashMap::with_capacity(order.len());
    for &leaf in order {
        let placed = boxes.get(&leaf).copied().unwrap_or_default();
        let inherited = grove
            .tree
            .trunk(leaf)
            .and_then(|trunk| accumulated.get(&trunk).copied())
            .unwrap_or_default();
        let drawn = Section::new(
            Position::new(placed.left() - inherited.x, placed.top() - inherited.y),
            placed.area,
        );
        grove.tree.settle(leaf, placed, drawn);
        let mut carried = inherited;
        if let Some(axes) = grove.tree.scrolls(leaf) {
            let offset = grove.tree.offset(leaf);
            let extent = grove.tree.extent(leaf);
            // An axis that was not declared does not scroll, so nothing can have moved along it.
            let clamped = Position::new(
                match axes.covers(Axis::Horizontal) {
                    true => offset
                        .x
                        .clamp(0.0, range(extent, placed.area, Axis::Horizontal)),
                    false => 0.0,
                },
                match axes.covers(Axis::Vertical) {
                    true => offset
                        .y
                        .clamp(0.0, range(extent, placed.area, Axis::Vertical)),
                    false => 0.0,
                },
            );
            if clamped != offset {
                grove.tree.set_offset(leaf, clamped);
            }
            carried = Position::new(carried.x + clamped.x, carried.y + clamped.y);
        }
        accumulated.insert(leaf, carried);
    }
}

/// R5. What a scrolling ancestor leaves visible of each element.
///
/// A rect, and nothing else. A region does not clip itself, only what is grown inside it, and an
/// element with no scrolling ancestor is clipped by nothing at all. Whether an element is *culled*
/// is extraction's decision from this rect, and is never recorded on the element -- so there is no
/// state saying "currently clipped away" for anything else, extent first among them, to read.
fn clip(grove: &mut Grove, order: &[Leaf]) {
    let _pass = trace_span!("clip").entered();
    let mut passed: HashMap<Leaf, Section> = HashMap::with_capacity(order.len());
    for &leaf in order {
        let inherited = grove
            .tree
            .trunk(leaf)
            .and_then(|trunk| passed.get(&trunk).copied())
            .unwrap_or(Clipped::unbounded().0);
        grove.tree.set_clip(leaf, inherited);
        let carried = match grove.tree.scrolls(leaf).is_some() {
            true => inherited.intersect(grove.tree.drawn(leaf)),
            false => inherited,
        };
        passed.insert(leaf, carried);
    }
}

/// R7. The three off-states, resolved over each element's whole ancestry.
///
/// One walk, in the same order the axes used, which puts every trunk before what hangs off it.
/// Nothing has a cascade to write: an element grown under a disabled trunk is disabled on its first
/// frame because the pass does not care when it arrived, and enabling that trunk leaves anything
/// disabled in its own right disabled because the product is over the whole ancestry rather than a
/// single bit that was overwritten on the way down.
fn inherit(grove: &mut Grove, order: &[Leaf]) {
    let _pass = trace_span!("inherit").entered();
    let mut products: HashMap<Leaf, Inherited> = HashMap::with_capacity(order.len());
    for &leaf in order {
        let trunk = grove
            .tree
            .trunk(leaf)
            .and_then(|trunk| products.get(&trunk).copied())
            .unwrap_or_default();
        let product = Inherited::under(
            trunk,
            grove.tree.visible(leaf),
            grove.tree.opacity(leaf),
            grove.tree.disabled(leaf),
        );
        products.insert(leaf, product);
        grove.tree.set_inherited(leaf, product);
    }
}

/// R8. The box stack the next frame's dispatch reads.
///
/// Membership is universal: an element is here because it is there. What is left out is only what
/// is not there at all -- hidden, fully transparent, or clipped away by a region it sits inside.
/// `pass_through` is not a way out of the stack; it is carried on the region and decides what may
/// be the top of it.
fn regions(grove: &mut Grove, order: &[Leaf]) {
    let _pass = trace_span!("regions").entered();
    let mut ranked = Vec::with_capacity(order.len());
    for &leaf in order {
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
                transparent: gestures.transparent,
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
fn rank(grove: &mut Grove, order: &[Leaf]) {
    let _pass = trace_span!("rank").entered();
    let mut stacks: HashMap<Leaf, i32> = HashMap::with_capacity(order.len());
    for &leaf in order {
        let trunk = grove
            .tree
            .trunk(leaf)
            .and_then(|trunk| stacks.get(&trunk).copied())
            .unwrap_or_default();
        let stack = grove.tree.elevation(leaf).accumulate(trunk);
        stacks.insert(leaf, stack);
        grove.tree.set_rank(
            leaf,
            ResolvedElevation {
                stack,
                growth: grove.tree.growth(leaf).0,
            },
        );
    }
}

/// Everything one axis of `leaf` resolves against.
fn context(
    grove: &Grove,
    boxes: &HashMap<Leaf, Section>,
    viewport: Section,
    leaf: Leaf,
    axis: Axis,
) -> Context {
    Context {
        axis,
        // Its box is the answer being computed, and its own grid divides it for its children
        // rather than for itself, so neither is readable here.
        own: Basis {
            section: Section::default(),
            intrinsic: grove.tree.intrinsic(leaf),
            tracks: Tracks::default(),
            cell: grove.tree.cell(leaf),
        },
        // A top-level element has no trunk, and fills the viewport instead.
        trunk: basis(grove, boxes, grove.tree.trunk(leaf), viewport),
        // A placement that reads an anchor it has not been given resolves against a zero box.
        anchor: basis(grove, boxes, grove.tree.anchor(leaf), Section::default()),
    }
}

/// What `leaf` offers a placement reading it, or `fallback` in place of a box when there is no such
/// element or it has not resolved yet.
fn basis(
    grove: &Grove,
    boxes: &HashMap<Leaf, Section>,
    leaf: Option<Leaf>,
    fallback: Section,
) -> Basis {
    let Some(leaf) = leaf else {
        return Basis {
            section: fallback,
            ..Basis::default()
        };
    };
    Basis {
        section: boxes.get(&leaf).copied().unwrap_or(fallback),
        intrinsic: grove.tree.intrinsic(leaf),
        tracks: grove
            .tree
            .grid(leaf)
            .unwrap_or_default()
            .tracks(grove.layout, grove.short),
        cell: grove.tree.cell(leaf),
    }
}

/// Every live element, ordered so that nothing resolves before what it depends on.
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
fn order(tree: &Tree) -> Vec<Leaf> {
    let leaves = tree.leaves();
    let mut waiting: HashMap<Leaf, usize> = HashMap::with_capacity(leaves.len());
    let mut dependents: HashMap<Leaf, Vec<Leaf>> = HashMap::with_capacity(leaves.len());
    for &leaf in &leaves {
        let depends_on = [tree.trunk(leaf), tree.anchor(leaf)];
        let mut count = 0;
        for on in depends_on.into_iter().flatten() {
            if tree.is_live(on) {
                count += 1;
                dependents.entry(on).or_default().push(leaf);
            }
        }
        waiting.insert(leaf, count);
    }
    let mut ready: Vec<Leaf> = leaves
        .iter()
        .copied()
        .filter(|leaf| waiting[leaf] == 0)
        .collect();
    let mut order = Vec::with_capacity(leaves.len());
    let mut next = 0;
    while next < ready.len() {
        let leaf = ready[next];
        next += 1;
        order.push(leaf);
        for dependent in dependents.get(&leaf).into_iter().flatten() {
            let waiting = waiting.get_mut(dependent).expect("a live dependent");
            *waiting -= 1;
            if *waiting == 0 {
                ready.push(*dependent);
            }
        }
    }
    if order.len() != leaves.len() {
        order.extend(leaves.iter().copied().filter(|leaf| waiting[leaf] != 0));
    }
    order
}
