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
//! | R4 | scroll | top-down | [`Drawn`]: [`Placed`] less every scrolling ancestor's offset |
//! | R6 | rank | dependency order | `ResolvedElevation`: elevation accumulated, then tie-broken |
//!
//! Both halves of R2 call the same pure resolver, once per axis. That is only safe because it is
//! pure: there is no accumulated state for a second call to corrupt.

use std::collections::HashMap;

use bevy_ecs::component::Component;
use tracing::trace_span;

use crate::coordinate::{Axis, Position, Section};
use crate::elevation::ResolvedElevation;
use crate::grove::Grove;
use crate::leaf::Leaf;
use crate::placement::grid::Tracks;
use crate::placement::location::Location;
use crate::placement::resolve::{Basis, Context, resolve};
use crate::tree::Tree;

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

/// Step 6. Declared placement becomes resolved geometry, for everything.
pub(crate) fn run(grove: &mut Grove) {
    let order = order(&grove.tree);
    let _step = trace_span!("resolve", elements = order.len()).entered();
    let viewport = Section::new(Position::default(), grove.viewport);
    let fallback = Location::default();
    let mut boxes: HashMap<Leaf, Section> = HashMap::with_capacity(order.len());

    for axis in [Axis::Horizontal, Axis::Vertical] {
        for &leaf in &order {
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

    // R4. Nothing scrolls yet, so every accumulated offset is zero and an element appears exactly
    // where the layout put it.
    for (&leaf, &section) in &boxes {
        grove.tree.settle(leaf, section, section);
    }

    rank(grove, &order);
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
