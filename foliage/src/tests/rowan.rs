//! Resolution, through the frame.
//!
//! Everything here runs the same `Fern` the event loop runs. The resolver's own arithmetic is
//! proven in `placement`; what is proven here is that the passes reach it with the right context
//! and in the right order.

use crate::coordinate::{Area, Axes, Section};
use crate::tests::{grove, resize, section, tick};
use crate::{
    Divide, Grid, Grove, Grow, Layout, Location, Panel, Place, Sap, ScrollTo, Source, Stem, Vein,
    anchor, bottom, center_x, center_y, left, right, top,
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
    assert_eq!(grove.tree.drawn(leaf), grove.tree.placed(leaf));
}

/// R4 accumulates down the tree, so an element inside two moved regions is drawn where the layout
/// put it less both of them -- while `Placed`, which is what its own children resolve against, is
/// untouched. That the two are separate values is the whole reason a scroll does not re-run a
/// layout.
#[test]
fn nested_regions_accumulate_offsets() {
    let mut grove = grove();
    let outer = grove.plant(Stem::new().at(box_at(0.0, 0.0, 200.0, 200.0)).scrolls(Axes::Vertical));
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
    assert_eq!(grove.tree.placed(deep).top(), 20.0);
}

/// R5 intersects each element's clip with its ancestors', so an element several regions deep sees
/// what all of them agree on. A rect, and only a rect: whether it is culled is extraction's
/// decision from this and is recorded nowhere.
#[test]
fn clip_rects_intersect_through_several_levels() {
    let mut grove = grove();
    let outer = grove.plant(Stem::new().at(box_at(0.0, 0.0, 200.0, 200.0)).scrolls(Axes::Vertical));
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
        grove.tree.clip(deep),
        Section::from_edges(20.0, 40.0, 180.0, 120.0)
    );
    // A region does not clip itself, only what is grown inside it.
    assert_eq!(
        grove.tree.clip(middle),
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
