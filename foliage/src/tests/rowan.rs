//! Resolution, through the frame.
//!
//! Everything here runs the same `Fern` the event loop runs. The resolver's own arithmetic is
//! proven in `placement`; what is proven here is that the passes reach it with the right context
//! and in the right order.

use crate::coordinate::{Area, Axes, Section};
use crate::tests::{advance, grove, resize, section, tick};
use crate::{
    Boxed, Divide, FontSize, Grid, Grove, Grow, Layout, Leaf, Location, Motion, Panel, Place, Sap,
    ScrollTo, Source, Stem, Text, Timing, Vein, anchor, aspect, bottom, center_x, center_y,
    content, left, right, top, trunk,
};

/// A box at a stated place, for the passes that are about where boxes end up rather than about the
/// grammar that put them there.
fn box_at(x: f32, y: f32, width: f32, height: f32) -> Location {
    Location::new().xs(
        left(x.px()).width(width.px()),
        top(y.px()).height(height.px()),
    )
}

#[test]
fn a_top_level_element_resolves_against_the_viewport() {
    let mut grove = grove();
    let leaf = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(20.px()).width(100.px()), top(10.px()).height(40.px()))),
    );
    tick(&mut grove);
    assert_eq!(
        section(&grove, leaf),
        Section::from_edges(20.0, 10.0, 120.0, 50.0)
    );
}

/// No settling frame: the frame that plants an element is the frame it resolves in, because a fresh
/// element is simply part of "everything" by the time R2 runs.
#[test]
fn an_element_resolves_on_the_frame_it_is_planted() {
    let mut grove = grove();
    let trunk = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(50.px()).width(100.px()), top(50.px()).height(100.px()))),
    );
    let branch = grove.branch(
        trunk,
        Stem::new()
            .at(Location::new().xs(left(10.px()).width(20.px()), top(10.px()).height(20.px()))),
    );
    tick(&mut grove);
    assert_eq!(
        section(&grove, branch),
        Section::from_edges(60.0, 60.0, 80.0, 80.0)
    );
}

#[test]
fn an_element_that_says_nothing_fills_its_trunk() {
    let mut grove = grove();
    let trunk = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(50.px()).width(100.px()), top(50.px()).height(100.px()))),
    );
    let branch = grove.branch(trunk, Stem::new());
    tick(&mut grove);
    assert_eq!(section(&grove, branch), section(&grove, trunk));
}

#[test]
fn a_child_addresses_its_trunk_s_grid() {
    let mut grove = grove();
    let trunk = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(200.px()), top(0.px()).height(100.px())))
            .grid(Grid::new().xs(4.columns().gap(8.0), 2.rows())),
    );
    let branch = grove.branch(
        trunk,
        Stem::new()
            .at(Location::new().xs(left(2.col()).right(2.col()), top(1.row()).bottom(1.row()))),
    );
    tick(&mut grove);
    assert_eq!(
        section(&grove, branch),
        Section::from_edges(52.0, 0.0, 96.0, 50.0)
    );
}

// Anchors.

#[test]
fn an_anchored_element_follows_its_target() {
    let mut grove = grove();
    let target =
        grove
            .plant(Stem::new().at(
                Location::new().xs(left(20.px()).width(60.px()), top(20.px()).height(30.px())),
            ));
    let follower = grove.plant(Stem::new().anchored(target).at(Location::new().xs(
        left(anchor().left()).width(anchor().width()),
        top(anchor().bottom() + 8.px()).height(10.px()),
    )));
    tick(&mut grove);
    assert_eq!(
        section(&grove, follower),
        Section::from_edges(20.0, 58.0, 80.0, 68.0)
    );

    grove.at(
        target,
        Location::new().xs(left(100.px()).width(40.px()), top(70.px()).height(30.px())),
    );
    tick(&mut grove);
    assert_eq!(
        section(&grove, follower),
        Section::from_edges(100.0, 108.0, 140.0, 118.0)
    );
}

/// An anchor may point at something that resolves later in tree order. That was never a cycle, and
/// ordering by dependency handles it without comment.
#[test]
fn an_anchor_may_point_forward() {
    let mut grove = grove();
    let follower = grove.plant(Stem::new().at(Location::new().xs(
        left(anchor().right()).width(10.px()),
        top(0.px()).height(10.px()),
    )));
    let target = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(40.px()).width(30.px()), top(0.px()).height(10.px()))),
    );
    grove.anchor(follower, target);
    tick(&mut grove);
    assert_eq!(section(&grove, follower).left(), 70.0);
}

#[test]
fn an_anchor_chain_resolves_in_order() {
    let mut grove = grove();
    let third = grove.plant(Stem::new().at(Location::new().xs(
        left(anchor().right()).width(10.px()),
        top(0.px()).height(10.px()),
    )));
    let second = grove.plant(Stem::new().at(Location::new().xs(
        left(anchor().right()).width(10.px()),
        top(0.px()).height(10.px()),
    )));
    let first = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(5.px()).width(10.px()), top(0.px()).height(10.px()))),
    );
    grove.anchor(third, second);
    grove.anchor(second, first);
    tick(&mut grove);
    assert_eq!(section(&grove, second).left(), 15.0);
    assert_eq!(section(&grove, third).left(), 25.0);
}

#[test]
fn the_anchor_a_leaf_carries_can_be_read_back() {
    let mut grove = grove();
    let target = grove.plant(Stem::new());
    let follower = grove.plant(Stem::new().anchored(target));
    tick(&mut grove);
    assert_eq!(
        grove.tap(follower, Vein::Anchor),
        Some(Sap::Leaf(Some(target)))
    );
    assert_eq!(grove.tap(target, Vein::Anchor), Some(Sap::Leaf(None)));
}

#[test]
#[should_panic(expected = "anchor cycle")]
fn an_anchor_that_closes_a_cycle_is_refused() {
    let mut grove = grove();
    let first = grove.plant(Stem::new());
    let second = grove.plant(Stem::new().anchored(first));
    grove.anchor(first, second);
    tick(&mut grove);
}

#[test]
#[should_panic(expected = "anchor cycle")]
fn an_element_may_not_anchor_to_itself() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    grove.anchor(leaf, leaf);
    tick(&mut grove);
}

#[test]
#[should_panic(expected = "anchor cycle")]
fn a_longer_cycle_is_refused_too() {
    let mut grove = grove();
    let first = grove.plant(Stem::new());
    let second = grove.plant(Stem::new().anchored(first));
    let third = grove.plant(Stem::new().anchored(second));
    grove.anchor(first, third);
    tick(&mut grove);
}

/// A dropped op is not a cycle: an anchor naming something that has withered goes the way every
/// other stale write does.
#[test]
fn an_anchor_naming_a_withered_leaf_is_dropped() {
    let mut grove = grove();
    let target = grove.plant(Stem::new());
    let leaf = grove.plant(Stem::new());
    tick(&mut grove);
    grove.prune(target);
    grove.anchor(leaf, target);
    tick(&mut grove);
    assert_eq!(grove.tap(leaf, Vein::Anchor), Some(Sap::Leaf(None)));
}

// Writes to a live element.

#[test]
fn a_placement_written_this_frame_lands_this_frame() {
    let mut grove = grove();
    let leaf = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(10.px()), top(0.px()).height(10.px()))),
    );
    tick(&mut grove);
    grove.at(
        leaf,
        Location::new().xs(left(30.px()).width(20.px()), top(40.px()).height(20.px())),
    );
    tick(&mut grove);
    assert_eq!(
        section(&grove, leaf),
        Section::from_edges(30.0, 40.0, 50.0, 60.0)
    );
}

#[test]
fn redividing_a_grid_moves_the_children_addressing_it() {
    let mut grove = grove();
    let trunk = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(200.px()), top(0.px()).height(100.px())))
            .grid(Grid::new().xs(2.columns(), 1.rows())),
    );
    let branch = grove.branch(
        trunk,
        Stem::new()
            .at(Location::new().xs(left(2.col()).right(2.col()), top(0.px()).height(10.px()))),
    );
    tick(&mut grove);
    assert_eq!(section(&grove, branch).left(), 100.0);

    grove.grid(trunk, Grid::new().xs(4.columns(), 1.rows()));
    tick(&mut grove);
    assert_eq!(section(&grove, branch).left(), 50.0);
}

/// A grid nothing wrote to is still the grid, wherever the order now puts the element it is on.
///
/// The order is built again when something is grown or pruned, and an element nothing wrote to
/// keeps what it resolved to -- including its grid, which has to be found at the position the
/// element now holds and not at the one it held before. Enough siblings that the write does not
/// simply resolve everything, and pruned from before the trunk so that it moves.
#[test]
fn a_grid_survives_the_order_being_built_again() {
    let mut grove = grove();
    let trunk = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(200.px()), top(0.px()).height(100.px())))
            .grid(Grid::new().xs(2.columns(), 1.rows())),
    );
    let siblings: Vec<_> = (0..12)
        .map(|_| grove.plant(Stem::new().at(box_at(0.0, 0.0, 10.0, 10.0))))
        .collect();
    tick(&mut grove);
    for sibling in &siblings[..3] {
        grove.prune(*sibling);
    }
    tick(&mut grove);

    let branch = grove.branch(
        trunk,
        Stem::new()
            .at(Location::new().xs(left(2.col()).right(2.col()), top(0.px()).height(10.px()))),
    );
    tick(&mut grove);
    assert_eq!(section(&grove, branch).left(), 100.0);
}

// The surface changing.

#[test]
fn a_resize_re_resolves_everything() {
    let mut grove = grove();
    let leaf =
        grove
            .plant(Stem::new().at(
                Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(10.px())),
            ));
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).width(), 400.0);

    resize(&mut grove, Area::new(800.0, 600.0));
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).width(), 800.0);
}

#[test]
fn crossing_a_breakpoint_takes_the_other_configuration() {
    let mut grove = Grove::new(Area::new(400.0, 800.0));
    let leaf = grove.plant(
        Stem::new().at(Location::new()
            .xs(left(0.px()).width(10.px()), top(0.px()).height(10.px()))
            .md(left(0.px()).width(60.px()), top(0.px()).height(10.px()))),
    );
    tick(&mut grove);
    assert_eq!(grove.layout(), Layout::Xs);
    assert_eq!(section(&grove, leaf).width(), 10.0);

    resize(&mut grove, Area::new(700.0, 800.0));
    tick(&mut grove);
    assert_eq!(grove.layout(), Layout::Md);
    assert_eq!(section(&grove, leaf).width(), 60.0);
}

/// The deadband is asymmetric on purpose, so a viewport resting near the boundary does not thrash.
#[test]
fn shortness_is_sticky() {
    let mut grove = Grove::new(Area::new(400.0, 800.0));
    tick(&mut grove);
    assert_eq!(grove.short(), crate::Short::No);

    resize(&mut grove, Area::new(400.0, 390.0));
    tick(&mut grove);
    assert_eq!(grove.short(), crate::Short::Yes);

    resize(&mut grove, Area::new(400.0, 420.0));
    tick(&mut grove);
    assert_eq!(grove.short(), crate::Short::Yes);

    resize(&mut grove, Area::new(400.0, 440.0));
    tick(&mut grove);
    assert_eq!(grove.short(), crate::Short::No);
}

// The harness's own obligations, for this slice.

#[test]
fn resolution_is_idempotent() {
    let mut grove = grove();
    let trunk = grove.plant(
        Stem::new()
            .at(Location::new().xs(
                left(10.px()).right(100.pct() - 10.px()),
                top(10.px()).bottom(100.pct() - 10.px()),
            ))
            .grid(Grid::new().xs(3.columns().gap(4.0), 3.rows().gap(4.0))),
    );
    let branch = grove.branch(
        trunk,
        Stem::new().at(Location::new().xs(
            center_x(2.col()).width(50.pct()),
            bottom(3.row()).height(1.row()),
        )),
    );
    tick(&mut grove);
    let first = (section(&grove, trunk), section(&grove, branch));
    tick(&mut grove);
    tick(&mut grove);
    assert_eq!(first, (section(&grove, trunk), section(&grove, branch)));
}

#[test]
fn two_identical_scripts_resolve_identically() {
    let script = |grove: &mut Grove| {
        let trunk = grove.plant(
            Stem::new()
                .at(Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(200.px())))
                .grid(Grid::new().xs(5.columns().gap(2.0), 1.rows())),
        );
        let branch = grove.branch(
            trunk,
            Stem::new().at(Location::new().xs(
                right(4.col()).width(2.col()),
                center_y(50.pct()).height(30.px()),
            )),
        );
        (trunk, branch)
    };
    let mut one = grove();
    let (trunk_one, branch_one) = script(&mut one);
    tick(&mut one);
    let mut two = grove();
    let (trunk_two, branch_two) = script(&mut two);
    tick(&mut two);
    assert_eq!(section(&one, trunk_one), section(&two, trunk_two));
    assert_eq!(section(&one, branch_one), section(&two, branch_two));
}

/// Where the layout put a box and where it is on screen are separate values. Nothing scrolls yet, so
/// they agree -- which is the statement a scrolling slice has to keep true for everything that does
/// not scroll.
#[test]
fn an_unscrolled_element_is_on_screen_where_the_layout_put_it() {
    let mut grove = grove();
    let leaf =
        grove
            .plant(Stem::new().at(
                Location::new().xs(left(12.px()).width(34.px()), top(56.px()).height(78.px())),
            ));
    tick(&mut grove);
    assert_eq!(grove.elements.drawn(leaf), grove.elements.placed(leaf));
}

/// R4 accumulates down the tree, so an element inside two moved regions is drawn where the layout
/// put it less both of them -- while `Placed`, which is what its own children resolve against, is
/// untouched. That the two are separate values is the whole reason a scroll does not re-run a
/// layout.
#[test]
fn nested_regions_accumulate_offsets() {
    let mut grove = grove();
    let outer = grove.plant(
        Stem::new()
            .at(box_at(0.0, 0.0, 200.0, 200.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(outer, Panel::new().at(box_at(0.0, 0.0, 200.0, 600.0)));
    let inner = grove.branch(
        outer,
        Stem::new()
            .at(box_at(0.0, 20.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    let deep = grove.branch(inner, Panel::new().at(box_at(0.0, 0.0, 200.0, 300.0)));
    tick(&mut grove);

    grove.scroll(outer, ScrollTo::px(40.0));
    grove.scroll(inner, ScrollTo::px(30.0));
    tick(&mut grove);

    // The inner region carries only what is outside it; what is in it carries both.
    assert_eq!(section(&grove, inner).top(), -20.0);
    assert_eq!(section(&grove, deep).top(), -50.0);
    // And the layout is where it always was, which is what the inner region's children resolved
    // against.
    assert_eq!(grove.elements.placed(deep).top(), 20.0);
}

/// R5 intersects each element's clip with its ancestors', so an element several regions deep sees
/// what all of them agree on. A rect, and only a rect: whether it is culled is extraction's
/// decision from this and is recorded nowhere.
#[test]
fn clip_rects_intersect_through_several_levels() {
    let mut grove = grove();
    let outer = grove.plant(
        Stem::new()
            .at(box_at(0.0, 0.0, 200.0, 200.0))
            .scrolls(Axes::Vertical),
    );
    let middle = grove.branch(
        outer,
        Stem::new()
            .at(box_at(20.0, 40.0, 160.0, 300.0))
            .scrolls(Axes::Vertical),
    );
    let inner = grove.branch(
        middle,
        Stem::new()
            .at(box_at(0.0, 0.0, 400.0, 80.0))
            .scrolls(Axes::Vertical),
    );
    let deep = grove.branch(inner, Panel::new().at(box_at(0.0, 0.0, 400.0, 400.0)));
    tick(&mut grove);

    // Each level narrows what the one below it may show: the middle one is cut off at the outer
    // one's bottom, and the inner one is cut off at the middle one's right.
    assert_eq!(
        grove.elements.clip(deep),
        Section::from_edges(20.0, 40.0, 180.0, 120.0)
    );
    // A region does not clip itself, only what is grown inside it.
    assert_eq!(
        grove.elements.clip(middle),
        Section::from_edges(0.0, 0.0, 200.0, 200.0)
    );
}

#[test]
fn a_withered_element_reads_no_section() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    tick(&mut grove);
    grove.prune(leaf);
    tick(&mut grove);
    assert_eq!(grove.tap(leaf, Vein::Drawn), None);
}

// A height in proportion to the element's own width.

/// The width read is the one R2a produced, not the one stated: the clamp is in it, and so is the
/// viewport a percentage resolved against.
#[test]
fn an_aspect_height_follows_the_width_the_horizontal_pass_produced() {
    let mut grove = grove();
    let clamped = grove.plant(Stem::new().at(Location::new().xs(
        left(0.px()).width(1000.px()).at_most(300.px()),
        top(0.px()).height(aspect(1.5)),
    )));
    let filling = grove.plant(Stem::new().at(Location::new().xs(
        left(0.px()).right(100.pct()),
        top(0.px()).height(aspect(2.0)),
    )));
    tick(&mut grove);
    assert_eq!(
        section(&grove, clamped),
        Section::from_edges(0.0, 0.0, 300.0, 200.0)
    );
    assert_eq!(
        section(&grove, filling),
        Section::from_edges(0.0, 0.0, 400.0, 200.0)
    );

    resize(&mut grove, Area::new(800.0, 300.0));
    tick(&mut grove);
    assert_eq!(
        section(&grove, filling),
        Section::from_edges(0.0, 0.0, 800.0, 400.0)
    );
}

/// An element in proportion describes its own height, in terms the measure can read -- its width is
/// settled before anything is measured -- so it counts toward a trunk sized to its contents.
#[test]
fn a_container_grows_to_fit_a_child_held_in_proportion() {
    let mut grove = grove();
    let container = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(200.px()), top(0.px()).height(content()))),
    );
    let child = grove.branch(
        container,
        Stem::new().at(Location::new().xs(
            left(0.px()).right(100.pct()),
            top(0.px()).height(aspect(2.0)),
        )),
    );
    tick(&mut grove);
    assert_eq!(section(&grove, child).height(), 100.0);
    assert_eq!(section(&grove, container).height(), 100.0);
}

/// Both ends of a motion resolve in one context, and that context's own width is the blended one
/// R2a just wrote -- so an element in proportion stays in proportion the whole way.
#[test]
fn an_aspect_height_follows_an_animated_width_mid_motion() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new().at(Location::new().xs(
        left(0.px()).width(100.px()),
        top(0.px()).height(aspect(1.0)),
    )));
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).area, Area::new(100.0, 100.0));

    grove.animate(
        leaf,
        Motion::Location(Location::new().xs(
            left(0.px()).width(300.px()),
            top(0.px()).height(aspect(1.0)),
        )),
        Timing::ms(200),
    );
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).area, Area::new(200.0, 200.0));

    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).area, Area::new(300.0, 300.0));
}

// -- Resolving only what was written -----------------------------------------------------------

/// A stream of numbers that is the same on every run, so a script that wanders is one script.
struct Wander(u64);

impl Wander {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn px(&mut self, most: u64) -> f32 {
        (self.next() % most) as f32
    }
}

/// Everything a wandering script grew, in the order it grew it. The same script run on two groves
/// names its elements in the same order, which is what the two are compared by.
#[derive(Default)]
struct Garden {
    columns: Vec<Leaf>,
    rows: Vec<Leaf>,
    labels: Vec<Leaf>,
    grown: Vec<Leaf>,
}

/// Where a row goes: mostly somewhere that states its own extent, which is what a column sized to
/// its contents reaches over, and sometimes somewhere read off the column, which counts for nothing.
fn row_at(wander: &mut Wander) -> Location {
    let across = left(wander.px(20).px()).right(100.pct() - wander.px(20).px());
    let y = wander.px(360);
    match wander.below(4) {
        0 | 1 => Location::new().xs(across, top(y.px()).height(content())),
        2 => Location::new().xs(across, top(y.px()).height((wander.px(60) + 4.0).px())),
        _ => Location::new().xs(across, top(wander.px(100).pct()).height(10.pct())),
    }
}

/// A run of a few words, which wraps to a different height at every width.
fn words(wander: &mut Wander) -> String {
    let count = wander.below(14);
    vec!["leaf"; count].join(" ")
}

/// Grows a row under one of the columns, with a label on it half the time.
fn sprout(grove: &mut Grove, wander: &mut Wander, garden: &mut Garden) {
    let column = garden.columns[wander.below(garden.columns.len())];
    let row = grove.branch(column, Stem::new().at(row_at(wander)));
    garden.rows.push(row);
    garden.grown.push(row);
    if wander.below(2) == 0 {
        let label = grove.branch(
            row,
            Text::new(words(wander))
                .font_size(FontSize::new().xs(14))
                .at(Location::new().xs(
                    left(4.px()).right(100.pct() - 4.px()),
                    top(4.px()).height(content()),
                )),
        );
        garden.labels.push(label);
        garden.grown.push(label);
    }
}

/// Three columns sized to what is in them, and a fourth grown in the first as one of its rows, so a
/// measure travels up more than one trunk.
fn garden(grove: &mut Grove, wander: &mut Wander) -> Garden {
    let mut garden = Garden::default();
    let page = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(content()))),
    );
    garden.grown.push(page);
    for n in 0..3 {
        let column = grove.branch(
            page,
            Stem::new().at(Location::new().xs(
                left((n as f32 * 130.0).px()).width(120.px()),
                top(0.px()).height(content()),
            )),
        );
        garden.columns.push(column);
        garden.grown.push(column);
    }
    let nested = grove.branch(
        garden.columns[0],
        Stem::new().at(Location::new().xs(
            left(0.px()).right(100.pct()),
            top(wander.px(200).px()).height(content()),
        )),
    );
    garden.columns.push(nested);
    garden.grown.push(nested);
    for _ in 0..24 {
        sprout(grove, wander, &mut garden);
    }
    garden
}

/// One frame's worth of writes: rows grown, moved, set in motion, hidden and pruned, labels
/// rewritten, and columns narrowed so everything in them wraps again.
fn tend(grove: &mut Grove, wander: &mut Wander, garden: &mut Garden) {
    for _ in 0..1 + wander.below(4) {
        let row = garden.rows[wander.below(garden.rows.len())];
        match wander.below(8) {
            0 => sprout(grove, wander, garden),
            1 | 2 => grove.at(row, row_at(wander)),
            3 => {
                let label = garden.labels[wander.below(garden.labels.len())];
                grove.text(label, words(wander));
            }
            4 => {
                let to = row_at(wander);
                grove.animate(row, Motion::Location(to), Timing::ms(80));
            }
            5 => {
                let column = garden.columns[wander.below(garden.columns.len())];
                let width = 60.0 + wander.px(90);
                grove.at(
                    column,
                    Location::new().xs(
                        left(wander.px(260).px()).width(width.px()),
                        top(wander.px(40).px()).height(content()),
                    ),
                );
            }
            6 => grove.visible(row, wander.below(2) == 0),
            _ => grove.prune(row),
        }
    }
}

/// What resolution leaves alone is exactly what resolving it again would have produced. The same
/// wandering script runs on two groves, one of them made to resolve everything every frame, and
/// every element has to be where the other one put it after every frame -- which is the claim the
/// whole of incremental resolution rests on, and the one each cache it keeps has to keep.
#[test]
fn resolving_only_what_was_written_lands_where_resolving_everything_does() {
    let mut incremental = grove();
    let mut entire = grove();
    let mut one = Wander(0x5eed_1eaf);
    let mut two = Wander(0x5eed_1eaf);
    let mut ours = garden(&mut incremental, &mut one);
    let mut theirs = garden(&mut entire, &mut two);
    for frame in 0..400 {
        tend(&mut incremental, &mut one, &mut ours);
        tend(&mut entire, &mut two, &mut theirs);
        advance(&mut incremental, 16);
        advance(&mut entire, 16);
        entire.tree.invalidate();
        tick(&mut incremental);
        tick(&mut entire);
        assert_eq!(ours.grown.len(), theirs.grown.len());
        for (mine, reference) in ours.grown.iter().zip(&theirs.grown) {
            assert_eq!(
                incremental.tap(*mine, Vein::Drawn),
                entire.tap(*reference, Vein::Drawn),
                "frame {frame}: {mine:?} is not where resolving everything puts it",
            );
        }
    }
}

// -- Measuring only what was written, across -------------------------------------------------

/// Everything a wandering script grew on shelves sized across, in the order it grew it.
#[derive(Default)]
struct Shelves {
    shelves: Vec<Leaf>,
    items: Vec<Leaf>,
    labels: Vec<Leaf>,
    grown: Vec<Leaf>,
}

/// Where an item goes across: mostly somewhere that states its own extent -- in pixels, from either
/// edge or the middle, or as wide as its own label -- which is what a shelf sized to its contents
/// reaches over, and sometimes somewhere read off the shelf, which counts for nothing.
fn item_at(wander: &mut Wander) -> Location {
    let x = wander.px(160);
    let width = 4.0 + wander.px(80);
    let across = match wander.below(6) {
        0 | 1 => left(x.px()).width(width.px()),
        2 => right(x.px()).width(width.px()),
        3 => center_x(x.px()).width(width.px()),
        4 => left(x.px()).width(content()),
        _ => left(wander.px(50).pct()).width(20.pct()),
    };
    Location::new().xs(across, top(wander.px(200).px()).height(content()))
}

/// Grows an item on one of the shelves, with a label on it half the time: one the item may hug, or
/// one that fills the item and wraps at whatever width the item comes to -- which is what makes a
/// height depend on a width a shelf's measure moved.
fn stock(grove: &mut Grove, wander: &mut Wander, shelves: &mut Shelves) {
    let shelf = shelves.shelves[wander.below(shelves.shelves.len())];
    let item = grove.branch(shelf, Stem::new().at(item_at(wander)));
    shelves.items.push(item);
    shelves.grown.push(item);
    if wander.below(2) == 0 {
        let across = match wander.below(2) {
            0 => left(4.px()).width(content()),
            _ => left(4.px()).right(100.pct() - 4.px()),
        };
        let label = grove.branch(
            item,
            Text::new(words(wander))
                .font_size(FontSize::new().xs(14).md(20))
                .at(Location::new().xs(across, top(4.px()).height(content()))),
        );
        shelves.labels.push(label);
        shelves.grown.push(label);
    }
}

/// Two shelves sized to what is on them across and down, a third grown on the first so a measure
/// travels up more than one trunk -- and two elements that read a shelf's measure rather than
/// adding to it: one grown on it reading its trunk's, and one anchored to it reading its anchor's.
fn shelving(grove: &mut Grove, wander: &mut Wander) -> Shelves {
    let mut shelves = Shelves::default();
    for n in 0..2 {
        let shelf = grove.plant(Stem::new().at(Location::new().xs(
            left((n as f32 * 200.0).px()).width(content()),
            top(0.px()).height(content()),
        )));
        shelves.shelves.push(shelf);
        shelves.grown.push(shelf);
    }
    let nested = grove.branch(
        shelves.shelves[0],
        Stem::new().at(Location::new().xs(
            left(wander.px(40).px()).width(content()),
            top(wander.px(100).px()).height(content()),
        )),
    );
    shelves.shelves.push(nested);
    shelves.grown.push(nested);
    let reading = grove.branch(
        shelves.shelves[1],
        Stem::new().at(Location::new().xs(
            left(0.px()).width(trunk().content() * 0.5),
            top(0.px()).height(10.px()),
        )),
    );
    shelves.grown.push(reading);
    let beside = grove.plant(
        Stem::new()
            .anchored(shelves.shelves[0])
            .at(Location::new().xs(
                left(anchor().right()).width(anchor().content()),
                top(anchor().bottom()).height(10.px()),
            )),
    );
    shelves.grown.push(beside);
    for _ in 0..24 {
        stock(grove, wander, &mut shelves);
    }
    shelves
}

/// One frame's worth of writes: items grown, moved, set in motion, hidden and pruned, labels
/// rewritten, shelves moved, and the window resized -- inside its breakpoint, which measures nothing
/// across again, and across it, where the labels change size.
fn restock(grove: &mut Grove, wander: &mut Wander, shelves: &mut Shelves) {
    for _ in 0..1 + wander.below(4) {
        let item = shelves.items[wander.below(shelves.items.len())];
        match wander.below(9) {
            8 => {
                let width = [400.0, 412.0, 640.0, 660.0][wander.below(4)];
                resize(grove, Area::new(width, 480.0));
            }
            0 => stock(grove, wander, shelves),
            1 | 2 => grove.at(item, item_at(wander)),
            3 => {
                let label = shelves.labels[wander.below(shelves.labels.len())];
                grove.text(label, words(wander));
            }
            4 => {
                let to = item_at(wander);
                grove.animate(item, Motion::Location(to), Timing::ms(80));
            }
            5 => {
                let shelf = shelves.shelves[wander.below(shelves.shelves.len())];
                grove.at(
                    shelf,
                    Location::new().xs(
                        left(wander.px(200).px()).width(content()),
                        top(wander.px(40).px()).height(content()),
                    ),
                );
            }
            6 => grove.visible(item, wander.below(2) == 0),
            _ => grove.prune(item),
        }
    }
}

/// The same claim as above, across: a shelf measured from what is on it is measured again only
/// where something under it was written, and every box lands where measuring and resolving
/// everything would have put it, after every frame.
#[test]
fn measuring_across_only_what_was_written_lands_where_measuring_everything_does() {
    let mut incremental = grove();
    let mut entire = grove();
    let mut one = Wander(0x5eed_5e1f);
    let mut two = Wander(0x5eed_5e1f);
    let mut ours = shelving(&mut incremental, &mut one);
    let mut theirs = shelving(&mut entire, &mut two);
    for frame in 0..400 {
        restock(&mut incremental, &mut one, &mut ours);
        restock(&mut entire, &mut two, &mut theirs);
        advance(&mut incremental, 16);
        advance(&mut entire, 16);
        entire.tree.invalidate();
        tick(&mut incremental);
        tick(&mut entire);
        assert_eq!(ours.grown.len(), theirs.grown.len());
        for (mine, reference) in ours.grown.iter().zip(&theirs.grown) {
            assert_eq!(
                incremental.tap(*mine, Vein::Drawn),
                entire.tap(*reference, Vein::Drawn),
                "frame {frame}: {mine:?} is not where measuring everything puts it",
            );
        }
    }
}

/// A frame that only grows and withers repairs the order rather than building it again -- what is
/// grown placed after everything already there -- and every element lands exactly where building
/// the order from scratch would put it.
#[test]
fn a_repaired_order_resolves_as_a_rebuilt_one_does() {
    let mut grove = grove();
    let root = grove.plant(Stem::new().at(box_at(0.0, 0.0, 400.0, 300.0)));
    let first = grove.branch(root, Stem::new().at(box_at(10.0, 10.0, 180.0, 120.0)));
    let second = grove.branch(root, Stem::new().at(box_at(200.0, 10.0, 180.0, 120.0)));
    let target = grove.branch(first, Panel::new().at(box_at(20.0, 30.0, 40.0, 20.0)));
    for n in 0..3 {
        grove.branch(
            second,
            Panel::new().at(box_at(n as f32 * 30.0, 5.0, 20.0, 20.0)),
        );
    }
    tick(&mut grove);
    let before = grove.elements.len();

    // One subtree withers and another is grown in the same frame, one of its elements anchored to
    // something that stayed.
    grove.prune(second);
    let third = grove.branch(root, Stem::new().at(box_at(10.0, 150.0, 300.0, 120.0)));
    let inner = grove.branch(
        third,
        Panel::new()
            .at(Location::new().xs(left(10.pct()).width(50.pct()), top(5.px()).height(30.px()))),
    );
    let follower = grove.branch(
        third,
        Stem::new().anchored(target).at(Location::new().xs(
            left(anchor().right() + 5.px()).width(30.px()),
            top(anchor().top()).height(anchor().height()),
        )),
    );
    tick(&mut grove);
    // Repaired: what was grown follows everything that stayed, where building it would have put
    // the new subtree among the rest by name.
    let kept = before - 4;
    let at = |leaf: Leaf| {
        grove
            .elements
            .order
            .iter()
            .position(|placed| *placed == leaf)
    };
    assert!(at(third).is_some_and(|at| at >= kept));
    assert!(at(follower).is_some_and(|at| at > at_of(&grove, third)));
    let live = [root, first, target, third, inner, follower];
    let repaired: Vec<Section> = live.iter().map(|leaf| section(&grove, *leaf)).collect();
    assert_eq!(
        repaired[5],
        Section::from_edges(75.0, 40.0, 105.0, 60.0),
        "the follower reads its anchor through the repaired order"
    );

    // An anchor written again is an edge moved, which builds the order from scratch.
    grove.anchor(follower, target);
    tick(&mut grove);
    let rebuilt: Vec<Section> = live.iter().map(|leaf| section(&grove, *leaf)).collect();
    assert_eq!(repaired, rebuilt);
}

fn at_of(grove: &Grove, leaf: Leaf) -> usize {
    grove
        .elements
        .order
        .iter()
        .position(|placed| *placed == leaf)
        .expect("placed")
}

/// A measure that moves under a box that stays put re-resolves what reads the measure, and nothing
/// else resolved against the box.
#[test]
fn a_measure_that_moves_reaches_only_what_reads_it() {
    let mut grove = grove();
    let root = grove.plant(Stem::new().at(box_at(0.0, 0.0, 400.0, 300.0)));
    grove.branch(root, Panel::new().at(box_at(0.0, 0.0, 50.0, 20.0)));
    let reader = grove.branch(
        root,
        Panel::new().at(Location::new().xs(
            left(0.px()).width(trunk().content()),
            top(40.px()).height(10.px()),
        )),
    );
    let bystander = grove.branch(root, Stem::new().at(box_at(0.0, 100.0, 40.0, 100.0)));
    let under = grove.branch(bystander, Panel::new().at(box_at(10.0, 10.0, 20.0, 20.0)));
    tick(&mut grove);
    assert_eq!(section(&grove, reader).width(), 50.0);

    // Wider content under the root: its measure moves, and its box, stated in pixels, does not.
    grove.branch(root, Panel::new().at(box_at(0.0, 60.0, 120.0, 20.0)));
    tick(&mut grove);
    assert_eq!(
        section(&grove, root),
        Section::from_edges(0.0, 0.0, 400.0, 300.0)
    );
    assert_eq!(
        section(&grove, reader).width(),
        120.0,
        "what reads the measure follows it"
    );
    assert!(!grove.elements.resolved_again(bystander));
    assert!(!grove.elements.resolved_again(under));
    assert_eq!(
        section(&grove, under),
        Section::from_edges(10.0, 110.0, 30.0, 130.0)
    );
}

// -- Accumulating only what moved --------------------------------------------------------------

/// Everything a wandering script grew in and around two scrolling regions, in the order it grew it.
#[derive(Default)]
struct Regions {
    /// The two regions, the outer first.
    regions: Vec<Leaf>,
    /// What anything may be grown under: the page, the regions, a stem beside them, and every stem
    /// grown since.
    holders: Vec<Leaf>,
    /// What is placed by a box and may be moved, restyled or let go.
    items: Vec<Leaf>,
    /// The strokes, which are moved by their ends.
    strokes: Vec<Leaf>,
    grown: Vec<Leaf>,
}

/// Somewhere in a holder, often past its edge, so a region has somewhere to scroll to.
fn spot(wander: &mut Wander) -> Location {
    box_at(
        wander.px(300),
        wander.px(500),
        10.0 + wander.px(80),
        10.0 + wander.px(80),
    )
}

/// A point inside a holder, as a fraction of it, as a map's strokes are.
fn fraction(wander: &mut Wander) -> crate::Point {
    crate::Point::new(wander.px(100).pct(), wander.px(100).pct())
}

/// Grows one thing under a holder: pinned, floating out of its region or out to the outer one,
/// raised, a stroke, or a stem that more is grown under -- half see-through, or floating out to the
/// surface, so what floats out to the outer region from under it is held by something the stem
/// passes nothing of down.
fn plant_in(grove: &mut Grove, wander: &mut Wander, regions: &mut Regions) {
    use crate::{Elevation, Escape, Line};
    let holder = regions.holders[wander.below(regions.holders.len())];
    let at = spot(wander);
    let leaf = match wander.below(7) {
        0 => grove.branch(holder, Panel::new().at(at).pinned()),
        1 => grove.branch(holder, Panel::new().at(at).floats(Escape::Region)),
        2 => grove.branch(
            holder,
            Panel::new()
                .at(at)
                .floats(Escape::Within(regions.regions[0])),
        ),
        3 => grove.branch(
            holder,
            Panel::new()
                .at(at)
                .elevate(Elevation::up(wander.below(4) as i32)),
        ),
        4 => {
            let (from, to) = (fraction(wander), fraction(wander));
            let stroke = grove.branch(holder, Line::new().weight(2.0).between(from, to));
            regions.strokes.push(stroke);
            regions.grown.push(stroke);
            return;
        }
        held => {
            let stem = match held {
                5 => Stem::new().at(at).opacity(0.5),
                _ => Stem::new().at(at).floats(Escape::Surface),
            };
            let stem = grove.branch(holder, stem);
            regions.holders.push(stem);
            regions.grown.push(stem);
            return;
        }
    };
    regions.items.push(leaf);
    regions.grown.push(leaf);
}

fn plantation(grove: &mut Grove, wander: &mut Wander) -> Regions {
    let mut regions = Regions::default();
    let page = grove.plant(Stem::new().at(box_at(0.0, 0.0, 400.0, 400.0)));
    let outer = grove.branch(
        page,
        Stem::new()
            .at(box_at(10.0, 10.0, 300.0, 300.0))
            .scrolls(Axes::Vertical),
    );
    let inner = grove.branch(
        outer,
        Stem::new()
            .at(box_at(20.0, 40.0, 200.0, 120.0))
            .scrolls(Axes::Both),
    );
    let beside = grove.branch(page, Stem::new().at(box_at(320.0, 0.0, 80.0, 400.0)));
    regions.regions = vec![outer, inner];
    regions.holders = vec![page, outer, inner, beside];
    regions.grown = vec![page, outer, inner, beside];
    for _ in 0..30 {
        plant_in(grove, wander, &mut regions);
    }
    regions
}

/// One frame's worth of writes: things grown, moved and let go, holders shifted, moved, hidden and
/// faded and raised, regions scrolled, and strokes given new ends.
fn stir(grove: &mut Grove, wander: &mut Wander, regions: &mut Regions) {
    use crate::{Elevation, Position};
    for _ in 0..1 + wander.below(4) {
        let holder = regions.holders[wander.below(regions.holders.len())];
        match wander.below(11) {
            0 => plant_in(grove, wander, regions),
            1 | 2 if !regions.items.is_empty() => {
                let item = regions.items[wander.below(regions.items.len())];
                grove.at(item, spot(wander));
            }
            3 => grove.shift(
                holder,
                Position::new(wander.px(40) - 20.0, wander.px(40) - 20.0),
            ),
            4 => {
                let region = regions.regions[wander.below(2)];
                grove.scroll(region, ScrollTo::px(wander.px(300)));
            }
            5 => grove.visible(holder, wander.below(3) != 0),
            6 => grove.opacity(holder, 0.25 + wander.px(4) / 4.0),
            7 => {
                // A holder raises everything under it with it.
                let raised = match regions.items.is_empty() || wander.below(2) == 0 {
                    true => holder,
                    false => regions.items[wander.below(regions.items.len())],
                };
                grove.elevate(raised, Elevation::up(wander.below(5) as i32));
            }
            8 if holder != regions.holders[0] => grove.at(holder, spot(wander)),
            9 if !regions.items.is_empty() => {
                let item = regions.items.swap_remove(wander.below(regions.items.len()));
                grove.prune(item);
            }
            _ if !regions.strokes.is_empty() => {
                let stroke = regions.strokes[wander.below(regions.strokes.len())];
                let (from, to) = (fraction(wander), fraction(wander));
                grove.between(stroke, from, to);
            }
            _ => {}
        }
    }
}

/// What the accumulations leave alone is exactly what carrying them over everything would have
/// produced. The same wandering script runs on two groves, one of them made to resolve everything
/// every frame, and every element has to be drawn where the other one draws it, clipped as it clips
/// it, ranked and faded as it ranks and fades it -- through regions scrolled, holders shifted, hidden
/// and moved, and elements grown and let go under them.
#[test]
fn accumulating_only_what_moved_lands_where_accumulating_everything_does() {
    let mut incremental = grove();
    let mut entire = grove();
    let mut one = Wander(0x0dd_ba11);
    let mut two = Wander(0x0dd_ba11);
    let mut ours = plantation(&mut incremental, &mut one);
    let mut theirs = plantation(&mut entire, &mut two);
    for frame in 0..400 {
        stir(&mut incremental, &mut one, &mut ours);
        stir(&mut entire, &mut two, &mut theirs);
        advance(&mut incremental, 16);
        advance(&mut entire, 16);
        entire.tree.invalidate();
        tick(&mut incremental);
        tick(&mut entire);
        assert_eq!(ours.grown.len(), theirs.grown.len());
        let (mine, reference) = (&incremental.elements, &entire.elements);
        for (&leaf, &other) in ours.grown.iter().zip(&theirs.grown) {
            let said = |what: &str| {
                format!("frame {frame}: {leaf:?}'s {what} is not what carrying everything makes it")
            };
            assert_eq!(
                mine.drawn(leaf),
                reference.drawn(other),
                "{}",
                said("drawn box")
            );
            assert_eq!(mine.clip(leaf), reference.clip(other), "{}", said("clip"));
            assert_eq!(mine.rank(leaf), reference.rank(other), "{}", said("rank"));
            assert_eq!(
                mine.inherited(leaf),
                reference.inherited(other),
                "{}",
                said("inherited")
            );
            assert_eq!(
                mine.stretched(leaf),
                reference.stretched(other),
                "{}",
                said("ends")
            );
            assert_eq!(
                mine.extends(leaf),
                reference.extends(other),
                "{}",
                said("reach")
            );
        }
        for (&leaf, &other) in ours.regions.iter().zip(&theirs.regions) {
            assert_eq!(
                incremental.tree.extent(leaf),
                entire.tree.extent(other),
                "frame {frame}: a region's extent is not what carrying everything makes it"
            );
            assert_eq!(incremental.tree.offset(leaf), entire.tree.offset(other));
        }
    }
}

/// What withers out of a region takes the room it reached over with it, even where the region's own
/// measure never counted it: a child sized off the region is not part of what it measures to, and
/// is part of how far its content reaches.
#[test]
fn a_region_lets_go_of_the_room_what_withered_took() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(box_at(0.0, 0.0, 100.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    let tall = grove.branch(
        region,
        Panel::new()
            .at(Location::new().xs(left(0.px()).width(100.pct()), top(0.px()).height(300.pct()))),
    );
    tick(&mut grove);
    assert_eq!(grove.tree.extent(region), Area::new(100.0, 300.0));

    grove.prune(tall);
    tick(&mut grove);
    assert_eq!(grove.tree.extent(region), Area::new(100.0, 100.0));
}

/// An element floating out to a region it names is clipped by that region however far up it is,
/// and follows it when it moves -- even past a stem between the two that passes nothing down,
/// because it floats out to the surface itself.
#[test]
fn what_floats_out_to_a_region_follows_it_past_what_passes_nothing_down() {
    use crate::Escape;
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(box_at(0.0, 0.0, 200.0, 200.0))
            .scrolls(Axes::Vertical),
    );
    let surfaced = grove.branch(
        region,
        Stem::new()
            .at(box_at(0.0, 0.0, 50.0, 50.0))
            .floats(Escape::Surface),
    );
    let held = grove.branch(
        surfaced,
        Panel::new()
            .at(box_at(0.0, 0.0, 20.0, 20.0))
            .floats(Escape::Within(region)),
    );
    tick(&mut grove);
    assert_eq!(
        grove.elements.clip(held),
        Section::from_edges(0.0, 0.0, 200.0, 200.0)
    );

    grove.at(region, box_at(50.0, 50.0, 200.0, 200.0));
    tick(&mut grove);
    assert_eq!(
        grove.elements.clip(held),
        Section::from_edges(50.0, 50.0, 250.0, 250.0)
    );
}
