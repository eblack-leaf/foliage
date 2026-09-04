//! Extraction, through the frame.
//!
//! What is proven here is that a change reaches the backend exactly once and that everything else
//! costs nothing -- which is what the whole recompute-totally thesis is bought with.

use crate::coordinate::Section;
use crate::elm::Key;
use crate::panel::PanelInstance;
use crate::tests::{grove, tick};
use crate::{
    Boxed,
    Color, Corner, Corners, Fill, Grove, Grow, Leaf, Location, Palette, Panel, Place, Rounding,
    Sap, Scheme, Side, Source, Stem, Vein, left, top,
};

/// A panel filling a box of a stated size, so its radii have something to resolve against.
fn panel(size: f32) -> Panel {
    Panel::new()
        .at(Location::new().xs(left(0.px()).width(size.px()), top(0.px()).height(size.px())))
}

fn held(grove: &Grove, leaf: Leaf) -> PanelInstance {
    grove
        .elm
        .panels
        .holding(leaf)
        .expect("the backend is holding this panel")
}

#[test]
fn a_panel_is_written_on_the_frame_it_is_planted() {
    let mut grove = grove();
    let leaf = grove.plant(panel(40.0));
    tick(&mut grove);
    assert_eq!(grove.elm.panels.written.len(), 1);
    assert_eq!(grove.elm.panels.written[0].key, Key::from(leaf));
    assert_eq!(
        held(&grove, leaf).section,
        Section::from_edges(0.0, 0.0, 40.0, 40.0)
    );
}

/// Carrying no renderer is the whole of what makes an element a stem, so extraction routes past it
/// on the decision alone -- never on which components it happens to have.
#[test]
fn a_stem_is_not_extracted() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    tick(&mut grove);
    assert_eq!(grove.elm.panels.len(), 0);
    assert_eq!(grove.elm.panels.holding(leaf), None);
}

/// The claim the whole phase rests on. Rowan has recomputed everything three more times and the
/// backend is told nothing, because the comparison is against what it holds.
#[test]
fn an_unchanged_frame_writes_nothing() {
    let mut grove = grove();
    let trunk = grove.plant(panel(40.0));
    for _ in 0..8 {
        grove.branch(trunk, panel(10.0));
    }
    tick(&mut grove);
    assert_eq!(grove.elm.panels.written.len(), 9);

    for _ in 0..3 {
        tick(&mut grove);
        assert!(grove.elm.panels.written.is_empty());
        assert!(grove.elm.panels.withdrawn.is_empty());
    }
    assert_eq!(grove.elm.panels.len(), 9);
}

#[test]
fn a_recolor_writes_the_panel_again() {
    let mut grove = grove();
    let leaf = grove.plant(panel(40.0).color(Palette::Muted));
    tick(&mut grove);
    let before = held(&grove, leaf).color;

    grove.color(leaf, Palette::Accent);
    tick(&mut grove);
    assert_eq!(grove.elm.panels.written.len(), 1);
    assert_ne!(held(&grove, leaf).color, before);
}

/// Only the element that changed. A sibling that did not is not re-sent on its account.
#[test]
fn a_recolor_writes_only_the_element_it_named() {
    let mut grove = grove();
    let one = grove.plant(panel(40.0));
    let two = grove.plant(panel(40.0));
    tick(&mut grove);

    grove.color(one, Palette::Accent);
    tick(&mut grove);
    assert_eq!(grove.elm.panels.written.len(), 1);
    assert_eq!(grove.elm.panels.written[0].key, Key::from(one));
    assert!(grove.elm.panels.holding(two).is_some());
}

#[test]
fn a_moved_panel_is_written_again() {
    let mut grove = grove();
    let leaf = grove.plant(panel(40.0));
    tick(&mut grove);

    grove.at(
        leaf,
        Location::new().xs(left(10.px()).width(20.px()), top(10.px()).height(20.px())),
    );
    tick(&mut grove);
    assert_eq!(grove.elm.panels.written.len(), 1);
    assert_eq!(
        held(&grove, leaf).section,
        Section::from_edges(10.0, 10.0, 30.0, 30.0)
    );
}

#[test]
fn a_pruned_panel_is_withdrawn() {
    let mut grove = grove();
    let leaf = grove.plant(panel(40.0));
    tick(&mut grove);

    grove.prune(leaf);
    tick(&mut grove);
    assert_eq!(grove.elm.panels.withdrawn, vec![Key::from(leaf)]);
    assert_eq!(grove.elm.panels.len(), 0);
    assert_eq!(grove.elm.panels.holding(leaf), None);
}

/// Nothing may depend on the order a map iterates, so two identical runs have to withdraw
/// identically.
#[test]
fn withdrawals_are_in_a_stable_order() {
    let withdraw = || {
        let mut grove = grove();
        let leaves = (0..16)
            .map(|_| grove.plant(panel(40.0)))
            .collect::<Vec<_>>();
        tick(&mut grove);
        for leaf in &leaves {
            grove.prune(*leaf);
        }
        tick(&mut grove);
        grove.elm.panels.withdrawn.clone()
    };
    let once = withdraw();
    assert_eq!(once.len(), 16);
    assert!(once.windows(2).all(|pair| pair[0] < pair[1]));
}

/// A whole subtree goes in one prune, and every one of them is withdrawn rather than left claimed.
#[test]
fn withering_a_subtree_withdraws_all_of_it() {
    let mut grove = grove();
    let trunk = grove.plant(panel(40.0));
    let branch = grove.branch(trunk, panel(20.0));
    let deeper = grove.branch(branch, panel(10.0));
    tick(&mut grove);
    assert_eq!(grove.elm.panels.len(), 3);

    grove.prune(trunk);
    tick(&mut grove);
    assert_eq!(grove.elm.panels.withdrawn.len(), 3);
    for leaf in [trunk, branch, deeper] {
        assert_eq!(grove.elm.panels.holding(leaf), None);
    }
}

// Rounding.

#[test]
fn one_bracket_rounds_every_corner() {
    let mut grove = grove();
    let leaf = grove.plant(panel(100.0).rounding(Rounding::Md));
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).radii, [12.0; 4]);
}

/// Ordered top-left, top-right, bottom-right, bottom-left, which is the order the shader indexes.
#[test]
fn a_side_rounds_the_two_corners_on_it() {
    let mut grove = grove();
    let leaf = grove.plant(panel(100.0).rounding(Corners::none().side(Side::Left, Rounding::Sm)));
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).radii, [8.0, 0.0, 0.0, 8.0]);
}

#[test]
fn a_corner_rounds_only_itself() {
    let mut grove = grove();
    let leaf = grove
        .plant(panel(100.0).rounding(Corners::none().corner(Corner::BottomRight, Rounding::Lg)));
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).radii, [0.0, 0.0, 16.0, 0.0]);
}

/// A box smaller than its own bracket cannot ask for more curve than it has room for.
#[test]
fn a_bracket_is_clamped_to_half_the_shorter_side() {
    let mut grove = grove();
    let leaf = grove.plant(panel(10.0).rounding(Rounding::Lg));
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).radii, [5.0; 4]);
}

#[test]
fn a_full_rounding_is_half_the_shorter_side() {
    let mut grove = grove();
    let leaf = grove.plant(
        Panel::new()
            .at(Location::new().xs(left(0.px()).width(80.px()), top(0.px()).height(24.px())))
            .rounding(Rounding::Full),
    );
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).radii, [12.0; 4]);
}

#[test]
fn rounding_written_after_the_fact_is_extracted() {
    let mut grove = grove();
    let leaf = grove.plant(panel(100.0));
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).radii, [0.0; 4]);

    grove.round(leaf, Corners::none().side(Side::Top, Rounding::Xs));
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).radii, [4.0, 4.0, 0.0, 0.0]);
}

// Reading back what was declared.

#[test]
fn a_fill_reads_back() {
    let mut grove = grove();
    let leaf = grove.plant(panel(40.0).color(Palette::Accent));
    tick(&mut grove);
    assert_eq!(
        grove.tap(leaf, Vein::Color),
        Some(Sap::Color(Fill::Role(Palette::Accent)))
    );
}

#[test]
fn a_rounding_reads_back() {
    let mut grove = grove();
    let rounding = Corners::none().corner(Corner::TopLeft, Rounding::Md);
    let leaf = grove.plant(panel(40.0).rounding(rounding));
    tick(&mut grove);
    assert_eq!(
        grove.tap(leaf, Vein::Rounding),
        Some(Sap::Rounding(rounding))
    );
}

/// An undeclared fill is a role like any other, so it reads back as one.
#[test]
fn an_undeclared_fill_reads_back_as_surface() {
    let mut grove = grove();
    let leaf = grove.plant(panel(40.0));
    tick(&mut grove);
    assert_eq!(
        grove.tap(leaf, Vein::Color),
        Some(Sap::Color(Fill::Role(Palette::Surface)))
    );
}

/// An element that draws nothing has no fill, which is absent rather than some default.
#[test]
fn a_stem_reads_no_fill() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    tick(&mut grove);
    assert_eq!(grove.tap(leaf, Vein::Color), None);
    assert_eq!(grove.tap(leaf, Vein::Rounding), None);
}

// Repainting.

/// A role belongs to the scheme, not to the elements declaring it, so changing what it resolves to
/// moves everything painted in it and nothing else.
#[test]
fn a_repaint_rewrites_every_element_in_an_affected_role() {
    let mut grove = grove();
    let accented = grove.plant(panel(40.0).color(Palette::Accent));
    let muted = grove.plant(panel(40.0).color(Palette::Muted));
    tick(&mut grove);
    let unaffected = held(&grove, muted).color;

    grove.repaint(Scheme::new().set(Palette::Accent, Color::rgb(1.0, 0.0, 0.0)));
    tick(&mut grove);
    assert_eq!(grove.elm.panels.written.len(), 1);
    assert_eq!(grove.elm.panels.written[0].key, Key::from(accented));
    assert_eq!(held(&grove, accented).color, Color::rgb(1.0, 0.0, 0.0));
    assert_eq!(held(&grove, muted).color, unaffected);
}

/// A literal is an element saying it is not part of the scheme, so a repaint leaves it alone. That
/// is the whole of the difference between naming a role and naming a color, and it is why both are
/// one type: the choice is visible at the callsite and nowhere else.
#[test]
fn a_repaint_leaves_a_fill_named_as_a_color_alone() {
    let mut grove = grove();
    let named = Color::rgb(1.0, 0.0, 0.0);
    let role = grove.plant(panel(40.0).color(Palette::Accent));
    let literal = grove.plant(panel(40.0).color(named));
    tick(&mut grove);
    assert_eq!(held(&grove, literal).color, named);

    grove.repaint(Scheme::new().set(Palette::Accent, Color::rgb(0.0, 1.0, 0.0)));
    tick(&mut grove);
    assert_eq!(grove.elm.panels.written.len(), 1);
    assert_eq!(grove.elm.panels.written[0].key, Key::from(role));
    assert_eq!(held(&grove, literal).color, named);
}

/// The whole tree in one op, and no element rewritten to say so.
#[test]
fn a_repaint_of_every_role_needs_no_element_named() {
    let mut grove = grove();
    for role in [Palette::Surface, Palette::Raised, Palette::Accent] {
        grove.plant(panel(40.0).color(role));
    }
    tick(&mut grove);

    let inverted = Scheme::new()
        .set(Palette::Surface, Color::rgb(1.0, 1.0, 1.0))
        .set(Palette::Raised, Color::rgb(0.9, 0.9, 0.9))
        .set(Palette::Accent, Color::rgb(0.0, 0.2, 0.8));
    grove.repaint(inverted);
    tick(&mut grove);
    assert_eq!(grove.elm.panels.written.len(), 3);
    assert_eq!(grove.scheme(), inverted);
}

/// A scheme that resolves every role to what it already did is not a change, so it is not sent.
#[test]
fn a_repaint_that_changes_nothing_writes_nothing() {
    let mut grove = grove();
    grove.plant(panel(40.0).color(Palette::Accent));
    tick(&mut grove);

    grove.repaint(Scheme::new());
    tick(&mut grove);
    assert!(grove.elm.panels.written.is_empty());
}

/// Nothing an app queues lands while it is still running, and a scheme is no exception.
#[test]
fn a_repaint_is_not_visible_in_the_frame_that_made_it() {
    let mut grove = grove();
    let before = grove.scheme();
    grove.repaint(Scheme::new().set(Palette::Accent, Color::rgb(1.0, 0.0, 0.0)));
    assert_eq!(grove.scheme(), before);
    tick(&mut grove);
    assert_ne!(grove.scheme(), before);
}

/// Filling something that draws nothing is dropped the way every op naming something it does not
/// apply to is: silently, and reported to the trace.
#[test]
fn filling_a_stem_is_dropped() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    tick(&mut grove);
    grove.color(leaf, Palette::Accent);
    tick(&mut grove);
    assert_eq!(grove.tap(leaf, Vein::Color), None);
    assert_eq!(grove.elm.panels.len(), 0);
}
