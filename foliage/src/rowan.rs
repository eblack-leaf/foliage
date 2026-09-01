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
//! | R2b | vertical | dependency order | the vertical axis, and with it [`LayoutSection`] |
//! | R4 | scroll | top-down | [`Screen`]: `LayoutSection` less every scrolling ancestor's offset |
//!
//! Both halves of R2 call the same pure resolver, once per axis. That is only safe because it is
//! pure: there is no accumulated state for a second call to corrupt.

use std::collections::HashMap;

use bevy_ecs::component::Component;
use tracing::trace_span;

use crate::coordinate::{Area, Axis, Position, Section};
use crate::grove::Grove;
use crate::leaf::Leaf;
use crate::placement::location::Location;
use crate::placement::resolve::{Context, resolve};
use crate::tree::Tree;

/// Where the layout put an element.
#[derive(Component, Copy, Clone, Debug, Default)]
pub(crate) struct LayoutSection(pub(crate) Section);

/// Where an element currently appears: its [`LayoutSection`] less every scrolling ancestor's
/// accumulated offset.
///
/// The two are deliberately separate. A write to the first means the layout moved a box and its
/// children have to follow; a write to the second means only that the box moved on screen, which is
/// what clipping and hit-testing read.
#[derive(Component, Copy, Clone, Debug, Default)]
pub(crate) struct Screen(pub(crate) Section);

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
    for (leaf, section) in boxes {
        grove.tree.settle(leaf, section, section);
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
    let trunk = grove.tree.trunk(leaf);
    let parent = trunk
        .and_then(|trunk| boxes.get(&trunk).copied())
        .unwrap_or(viewport);
    let tracks = trunk
        .and_then(|trunk| grove.tree.grid(trunk))
        .unwrap_or_default()
        .tracks(grove.layout, grove.short);
    let anchor = grove
        .tree
        .anchor(leaf)
        .and_then(|anchor| boxes.get(&anchor).copied())
        .unwrap_or_default();
    Context {
        axis,
        parent,
        anchor,
        // Measured content lands with text; an element with nothing in it is intrinsically empty.
        intrinsic: Area::default(),
        tracks,
        cell: grove.tree.cell(leaf),
        parent_cell: trunk.map(|trunk| grove.tree.cell(trunk)).unwrap_or_default(),
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
