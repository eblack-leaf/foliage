//! `Elevation`'s resolve/stacking math (`coordinate/elevation.rs`). Worth being precise
//! about: `ResolvedElevation`'s `PartialOrd` is deliberately *inverted* relative to its raw
//! `f32` -- a *smaller* underlying value compares as *greater* elevation, because internally
//! "more in front" is represented by a smaller number (`Elevation::up(n)` subtracts `n` from
//! the parent's resolved value). These tests assert on that comparison operator directly,
//! not just the raw numbers, since that inversion is exactly the kind of thing a naive reader
//! (or a future refactor) could get backwards.

use foliage_proper::{EcsExtension, Elevation, Entity, Foliage, Leaf, Location, ResolvedElevation, Sprout};

fn resolved_of(foliage: &mut Foliage, entity: Entity) -> ResolvedElevation {
    *foliage.world.get::<ResolvedElevation>(entity).unwrap()
}

#[test]
fn a_child_at_up_one_resolves_in_front_of_its_parent() {
    let mut foliage = Foliage::new();
    let parent = foliage
        .world
        .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::abs(0)));
    let child = foliage.world.branch(
        parent,
        Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    let (parent_elev, child_elev) = (resolved_of(&mut foliage, parent), resolved_of(&mut foliage, child));
    assert!(
        child_elev > parent_elev,
        "up(1) should compare as in front of its parent, via ResolvedElevation's own \
         (inverted-from-raw-f32) PartialOrd -- got parent {parent_elev:?}, child {child_elev:?}"
    );
}

#[test]
fn a_higher_up_amount_resolves_further_in_front_than_a_lower_one() {
    let mut foliage = Foliage::new();
    let parent = foliage
        .world
        .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::abs(0)));
    let near = foliage.world.branch(
        parent,
        Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
    );
    let far = foliage.world.branch(
        parent,
        Leaf::sprout().at(Location::new()).elevate(Elevation::up(2)),
    );
    foliage.world.flush();

    let (near_elev, far_elev) = (resolved_of(&mut foliage, near), resolved_of(&mut foliage, far));
    assert!(
        far_elev > near_elev,
        "up(2) should compare as further in front than up(1) -- got up(1) {near_elev:?}, up(2) {far_elev:?}"
    );
}

#[test]
fn elevation_up_propagates_relative_to_the_parents_resolved_value_not_a_fixed_baseline() {
    let mut foliage = Foliage::new();
    // two roots at different absolute elevations -- each child's `up(1)` should resolve
    // relative to *its own* parent, not collapse to the same value regardless of context.
    let low_root = foliage
        .world
        .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::abs(0)));
    let high_root = foliage
        .world
        .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::abs(10)));
    let low_child = foliage.world.branch(
        low_root,
        Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
    );
    let high_child = foliage.world.branch(
        high_root,
        Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    assert_ne!(
        resolved_of(&mut foliage, low_child).value(),
        resolved_of(&mut foliage, high_child).value(),
        "both children are up(1), but their parents sit at different absolute elevations, \
         so the resolved values must differ"
    );
    assert!(
        resolved_of(&mut foliage, high_child) > resolved_of(&mut foliage, low_child),
        "the child of the more-in-front root (abs(10)) should itself resolve more in front"
    );
}
