use super::{Observer, grove, tick, tick_with};
use crate::leaf::Presence;
use crate::stem::Stem;
use crate::vein::{Sap, Vein};
use crate::verbs::Grow;

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
