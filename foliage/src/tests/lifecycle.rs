use super::{Observer, grove, section, tick, tick_with};
use crate::coordinate::Section;
use crate::grove::Grove;
use crate::leaf::{Leaf, Presence};
use crate::placement::grid::Grid;
use crate::placement::location::Location;
use crate::stem::Stem;
use crate::vein::{Sap, Vein};
use crate::verbs::Grow;
use crate::{Divide, Source, left, top};

#[test]
fn a_name_is_planted_until_its_op_drains() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());

    assert_eq!(grove.presence(leaf), Presence::Planted);
    assert_eq!(grove.tap(leaf, Vein::Branches), None);

    tick(&mut grove);

    assert_eq!(grove.presence(leaf), Presence::Live);
    assert_eq!(grove.tap(leaf, Vein::Branches), Some(Sap::Leaves(vec![])));
}

#[test]
fn a_leaf_is_usable_as_a_trunk_the_instant_it_is_handed_out() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    let branch = grove.branch(trunk, Stem::new());
    tick(&mut grove);

    assert_eq!(grove.presence(branch), Presence::Live);
    assert_eq!(grove.tap(branch, Vein::Trunk), Some(Sap::Leaf(Some(trunk))));
}

#[test]
fn branches_read_back_in_the_order_they_were_grown() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    let first = grove.branch(trunk, Stem::new());
    let second = grove.branch(trunk, Stem::new());
    tick(&mut grove);

    assert_eq!(
        grove.tap(trunk, Vein::Branches),
        Some(Sap::Leaves(vec![first, second]))
    );
    assert_eq!(grove.tap(trunk, Vein::Trunk), Some(Sap::Leaf(None)));
}

#[test]
fn prune_takes_the_whole_subtree() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    let branch = grove.branch(trunk, Stem::new());
    let twig = grove.branch(branch, Stem::new());
    tick(&mut grove);

    grove.prune(trunk);
    tick(&mut grove);

    for leaf in [trunk, branch, twig] {
        assert_eq!(grove.presence(leaf), Presence::Withered);
        assert_eq!(grove.tap(leaf, Vein::Branches), None);
    }
}

#[test]
fn prune_reports_every_name_that_went() {
    let mut grove = grove();
    let mut app = Observer::default();
    let trunk = grove.plant(Stem::new());
    let branch = grove.branch(trunk, Stem::new());
    let twig = grove.branch(branch, Stem::new());
    tick_with(&mut grove, &mut app);

    grove.prune(trunk);
    tick_with(&mut grove, &mut app);

    // The drain withers, and the pollen it releases was sealed before it ran.
    for leaf in [trunk, branch, twig] {
        assert!(!app.last().withered(leaf));
    }

    tick_with(&mut grove, &mut app);

    for leaf in [trunk, branch, twig] {
        assert!(app.last().withered(leaf));
    }
}

#[test]
fn ops_naming_a_withered_leaf_are_dropped() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    tick(&mut grove);
    grove.prune(trunk);
    tick(&mut grove);

    let orphan = grove.branch(trunk, Stem::new());
    grove.prune(trunk);
    tick(&mut grove);

    assert_eq!(grove.presence(trunk), Presence::Withered);
    assert_eq!(grove.presence(orphan), Presence::Planted);
}

#[test]
fn pruning_a_withered_leaf_is_a_no_op() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    let keep = grove.plant(Stem::new());
    tick(&mut grove);

    grove.prune(trunk);
    tick(&mut grove);

    grove.prune(trunk);
    tick(&mut grove);

    assert_eq!(grove.presence(trunk), Presence::Withered);
    assert_eq!(grove.presence(keep), Presence::Live);
}

#[test]
fn a_leaf_is_reported_withered_once_and_not_again() {
    let mut grove = grove();
    let mut app = Observer::default();
    let trunk = grove.plant(Stem::new());
    tick_with(&mut grove, &mut app);

    grove.prune(trunk);
    tick_with(&mut grove, &mut app);
    tick_with(&mut grove, &mut app);
    assert!(app.last().withered(trunk));

    grove.prune(trunk);
    tick_with(&mut grove, &mut app);
    tick_with(&mut grove, &mut app);
    assert!(!app.last().withered(trunk));
}

/// An op under a name whose own grow was dropped is dropped in turn: the trunk never became live,
/// so there is nothing to branch off.
#[test]
fn an_op_beneath_a_dropped_op_is_dropped_too() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    tick(&mut grove);

    grove.prune(trunk);
    let orphan = grove.branch(trunk, Stem::new());
    let twig = grove.branch(orphan, Stem::new());
    tick(&mut grove);

    assert_eq!(grove.presence(trunk), Presence::Withered);
    assert_eq!(grove.presence(orphan), Presence::Planted);
    assert_eq!(grove.presence(twig), Presence::Planted);
}

// Every verb against a name that is not live.
//
// The write is dropped where the drain reaches it: not refused, not panicked, and not held over for
// a frame in which the name might mean something. That is the whole of what makes a stale handle
// inert rather than dangerous, and it is why an app never checks presence before writing.
//
// A name is not live for two separate reasons, and both are covered: it has withered, or its own
// grow was dropped and it never became anything. `plant` is absent from the set because it cannot
// be reached -- it allocates its own name, and a name is never reused, so there is no occupied name
// for one to land on.

/// A name that was grown and then taken down.
fn withered(grove: &mut Grove) -> Leaf {
    let leaf = grove.plant(Stem::new());
    tick(grove);
    grove.prune(leaf);
    tick(grove);
    leaf
}

/// A name whose grow was dropped, so it never became anything and never will.
fn never_grown(grove: &mut Grove) -> Leaf {
    let trunk = grove.plant(Stem::new());
    tick(grove);
    grove.prune(trunk);
    let orphan = grove.branch(trunk, Stem::new());
    tick(grove);
    orphan
}

/// A placement distinct from the default, so a write that landed would be visible.
fn somewhere() -> Location {
    Location::new().xs(left(10.px()).width(20.px()), top(30.px()).height(40.px()))
}

#[test]
fn placing_a_withered_leaf_is_a_no_op() {
    let mut grove = grove();
    let leaf = withered(&mut grove);

    grove.at(leaf, somewhere());
    tick(&mut grove);

    assert_eq!(grove.presence(leaf), Presence::Withered);
    assert_eq!(grove.tap(leaf, Vein::Drawn), None);
}

#[test]
fn placing_a_leaf_that_was_never_grown_is_a_no_op() {
    let mut grove = grove();
    let orphan = never_grown(&mut grove);

    grove.at(orphan, somewhere());
    tick(&mut grove);

    assert_eq!(grove.presence(orphan), Presence::Planted);
    assert_eq!(grove.tap(orphan, Vein::Drawn), None);
}

#[test]
fn dividing_a_withered_leaf_is_a_no_op() {
    let mut grove = grove();
    let leaf = withered(&mut grove);

    grove.grid(leaf, Grid::new().xs(4.columns(), 4.rows()));
    tick(&mut grove);

    assert_eq!(grove.presence(leaf), Presence::Withered);
    assert_eq!(grove.tap(leaf, Vein::Branches), None);
}

#[test]
fn dividing_a_leaf_that_was_never_grown_is_a_no_op() {
    let mut grove = grove();
    let orphan = never_grown(&mut grove);

    grove.grid(orphan, Grid::new().xs(4.columns(), 4.rows()));
    tick(&mut grove);

    assert_eq!(grove.presence(orphan), Presence::Planted);
    assert_eq!(grove.tap(orphan, Vein::Branches), None);
}

#[test]
fn anchoring_a_withered_leaf_is_a_no_op() {
    let mut grove = grove();
    let target = grove.plant(Stem::new());
    let leaf = withered(&mut grove);

    grove.anchor(leaf, target);
    tick(&mut grove);

    assert_eq!(grove.presence(leaf), Presence::Withered);
    assert_eq!(grove.tap(leaf, Vein::Anchor), None);
}

#[test]
fn anchoring_a_leaf_that_was_never_grown_is_a_no_op() {
    let mut grove = grove();
    let target = grove.plant(Stem::new());
    let orphan = never_grown(&mut grove);

    grove.anchor(orphan, target);
    tick(&mut grove);

    assert_eq!(grove.presence(orphan), Presence::Planted);
    assert_eq!(grove.tap(orphan, Vein::Anchor), None);
}

/// A dropped op is skipped, not a stop. The drain is total, so everything queued behind one still
/// applies -- which is what keeps a stale handle from taking the rest of the frame's work with it.
#[test]
fn a_dropped_op_does_not_stop_the_drain() {
    let mut grove = grove();
    let gone = withered(&mut grove);
    let live = grove.plant(Stem::new());
    tick(&mut grove);

    grove.at(gone, somewhere());
    grove.grid(gone, Grid::new().xs(4.columns(), 4.rows()));
    grove.anchor(gone, live);
    grove.at(live, somewhere());
    tick(&mut grove);

    assert_eq!(
        section(&grove, live),
        Section::from_edges(10.0, 30.0, 30.0, 70.0)
    );
}

#[test]
fn a_name_is_never_reused() {
    let mut grove = grove();
    let mut gone = Vec::new();
    for _ in 0..32 {
        let leaf = grove.plant(Stem::new());
        tick(&mut grove);
        grove.prune(leaf);
        tick(&mut grove);
        gone.push(leaf);
    }

    for _ in 0..32 {
        let leaf = grove.plant(Stem::new());
        tick(&mut grove);
        assert!(!gone.contains(&leaf));
        assert_eq!(grove.presence(leaf), Presence::Live);
    }

    for leaf in gone {
        assert_eq!(grove.presence(leaf), Presence::Withered);
    }
}
