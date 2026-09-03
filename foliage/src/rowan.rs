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
//! | R2a | horizontal | dependency order | the horizontal axis |
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
//! R3 is the one pass that runs the other way, and the reason is a genuine cycle: a child resolves
//! against its parent's box, a scrolling parent's extent comes from where its children landed, and
//! that extent is what its offset is clamped to. The cycle is not vicious, because extent affects
//! only the clamp and never the parent's own box -- so a top-down layout sweep, a bottom-up extent
//! sweep and a top-down scroll application resolve it in one pass each, with no iteration.

use std::collections::HashMap;

use bevy_ecs::component::Component;
use tracing::trace_span;

use crate::coordinate::{Area, Axis, Position, Section};
use crate::elevation::ResolvedElevation;
use crate::grove::Grove;
use crate::interaction::focus;
use crate::interaction::stack::Region;
use crate::leaf::Leaf;
use crate::lifecycle::Inherited;
use crate::placement::grid::Tracks;
use crate::placement::location::Location;
use crate::placement::resolve::{Basis, Context, resolve};
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

/// Steps 6 and 7. Declared state becomes resolved geometry and resolved products, for everything.
pub(crate) fn run(grove: &mut Grove) {
    let order = order(&grove.tree);
    {
        let _step = trace_span!("resolve", elements = order.len()).entered();
        let boxes = axes(grove, &order);
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

/// R2a and R2b. One axis at a time, in dependency order, through the one pure resolver.
fn axes(grove: &Grove, order: &[Leaf]) -> HashMap<Leaf, Section> {
    let viewport = Section::new(Position::default(), grove.viewport);
    let fallback = Location::default();
    let mut boxes: HashMap<Leaf, Section> = HashMap::with_capacity(order.len());
    for axis in [Axis::Horizontal, Axis::Vertical] {
        for &leaf in order {
            let context = context(grove, &boxes, viewport, leaf, axis);
            let location = grove.tree.location(leaf).unwrap_or(&fallback);
            let axes = location.axes(grove.layout, grove.short);
            let config = match axis {
                Axis::Horizontal => &axes.horizontal,
                Axis::Vertical => &axes.vertical,
            };
            let span = resolve(config, &context);
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
    boxes
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
/// *and* after whatever it anchors to. Cycles are refused where they are written, so the graph is a
/// valid one by construction and this needs no cycle handling.
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
    order
}
