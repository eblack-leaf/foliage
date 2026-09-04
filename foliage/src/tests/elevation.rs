//! R6: elevation accumulated down the tree, and the order that settles what it leaves equal.

use crate::elevation::ResolvedElevation;
use crate::tests::{grove, tick};
use crate::{Boxed, Elevation, Grow, Place, Sap, Stem, Vein};

fn rank(grove: &crate::Grove, leaf: crate::Leaf) -> ResolvedElevation {
    grove.tree.rank(leaf)
}

#[test]
fn an_undeclared_elevation_sits_level_with_its_trunk() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    let branch = grove.branch(trunk, Stem::new());
    tick(&mut grove);
    assert_eq!(rank(&grove, branch).stack, rank(&grove, trunk).stack);
}

/// Level with its trunk still means in front of it, because it was allocated later and that is what
/// the tie-break reads.
#[test]
fn a_branch_sits_in_front_of_the_trunk_it_is_level_with() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    let branch = grove.branch(trunk, Stem::new());
    tick(&mut grove);
    assert!(rank(&grove, branch) > rank(&grove, trunk));
}

#[test]
fn elevation_accumulates_down_the_tree() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new().elevate(Elevation::up(2)));
    let branch = grove.branch(trunk, Stem::new().elevate(Elevation::up(3)));
    let deeper = grove.branch(branch, Stem::new().elevate(Elevation::down(1)));
    tick(&mut grove);
    assert_eq!(rank(&grove, trunk).stack, 2);
    assert_eq!(rank(&grove, branch).stack, 5);
    assert_eq!(rank(&grove, deeper).stack, 4);
}

#[test]
fn zero_has_no_direction() {
    let mut grove = grove();
    let up = grove.plant(Stem::new().elevate(Elevation::up(0)));
    let down = grove.plant(Stem::new().elevate(Elevation::down(0)));
    tick(&mut grove);
    assert_eq!(rank(&grove, up).stack, rank(&grove, down).stack);
}

/// Raising a card carries its whole subtree with it, and nothing inside it is rewritten.
#[test]
fn elevating_a_trunk_carries_its_subtree() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    let branch = grove.branch(trunk, Stem::new().elevate(Elevation::up(1)));
    tick(&mut grove);
    assert_eq!(rank(&grove, branch).stack, 1);

    grove.elevate(trunk, Elevation::up(10));
    tick(&mut grove);
    assert_eq!(rank(&grove, trunk).stack, 10);
    assert_eq!(rank(&grove, branch).stack, 11);
    assert_eq!(
        grove.tap(branch, Vein::Elevation),
        Some(Sap::Elevation(Elevation::up(1)))
    );
}

/// An element that parents things has to carry an elevation even though it draws nothing, because
/// what hangs off it accumulates from it.
#[test]
fn a_stem_that_draws_nothing_still_elevates_what_it_holds() {
    let mut grove = grove();
    let wrapper = grove.plant(Stem::new().elevate(Elevation::up(5)));
    let branch = grove.branch(wrapper, Stem::new());
    tick(&mut grove);
    assert_eq!(rank(&grove, branch).stack, 5);
}

// The tie-break.

#[test]
fn equal_elevation_siblings_rank_by_allocation_order() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    let first = grove.branch(trunk, Stem::new());
    let second = grove.branch(trunk, Stem::new());
    let third = grove.branch(trunk, Stem::new());
    tick(&mut grove);
    assert_eq!(rank(&grove, first).stack, rank(&grove, second).stack);
    assert!(rank(&grove, first) < rank(&grove, second));
    assert!(rank(&grove, second) < rank(&grove, third));
}

/// Allocation order, not the order the drain reached them. The two differ whenever an element is
/// named before one that is grown under an earlier trunk.
#[test]
fn the_tie_break_is_the_order_they_were_named() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    let named_first = grove.branch(trunk, Stem::new());
    let named_second = grove.plant(Stem::new());
    tick(&mut grove);
    assert!(rank(&grove, named_first) < rank(&grove, named_second));
}

/// A name is never reused, so nothing that went can renumber what stayed.
#[test]
fn the_tie_break_survives_a_prune_of_an_earlier_sibling() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    let first = grove.branch(trunk, Stem::new());
    let second = grove.branch(trunk, Stem::new());
    let third = grove.branch(trunk, Stem::new());
    tick(&mut grove);
    let (before_second, before_third) = (rank(&grove, second), rank(&grove, third));

    grove.prune(first);
    tick(&mut grove);
    assert_eq!(rank(&grove, second), before_second);
    assert_eq!(rank(&grove, third), before_third);
    assert!(rank(&grove, second) < rank(&grove, third));
}

/// An elevation is stated once and read back as it was written -- not as the accumulated total,
/// which is a number about the whole ancestry rather than about this element.
#[test]
fn an_elevation_reads_back_as_declared() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new().elevate(Elevation::up(4)));
    let branch = grove.branch(trunk, Stem::new().elevate(Elevation::down(2)));
    tick(&mut grove);
    assert_eq!(
        grove.tap(branch, Vein::Elevation),
        Some(Sap::Elevation(Elevation::down(2)))
    );
    assert_eq!(rank(&grove, branch).stack, 2);
}

#[test]
fn an_undeclared_elevation_reads_back_as_level() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    tick(&mut grove);
    assert_eq!(
        grove.tap(leaf, Vein::Elevation),
        Some(Sap::Elevation(Elevation::up(0)))
    );
}

#[test]
fn elevating_a_withered_leaf_is_dropped() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    tick(&mut grove);
    grove.prune(leaf);
    grove.elevate(leaf, Elevation::up(3));
    tick(&mut grove);
    assert_eq!(grove.tap(leaf, Vein::Elevation), None);
}
