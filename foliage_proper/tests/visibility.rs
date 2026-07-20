//! `Visibility`'s propagation (`visibility.rs`): a `ResolvedVisibility` is the conjunction
//! of `inherited && current && auto` -- a child is only visible if it, its whole ancestor
//! chain, and its own auto-visibility all agree. Foundational, previously untested. (Not to
//! be confused with `Disable`/`Enable`, a separate system entirely -- that one governs
//! interaction-enablement flags on `InteractionListener`, not `ResolvedVisibility`.)

use foliage_proper::{EcsExtension, Elevation, Entity, Foliage, Leaf, Location, Sprout, Visibility};

fn resolved_visible(foliage: &mut Foliage, entity: Entity) -> bool {
    foliage
        .world
        .get::<foliage_proper::ResolvedVisibility>(entity)
        .unwrap()
        .visible()
}

#[test]
fn a_bare_leaf_defaults_to_visible() {
    let mut foliage = Foliage::new();
    let leaf = foliage
        .world
        .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
    foliage.world.flush();
    assert!(resolved_visible(&mut foliage, leaf));
}

#[test]
fn hiding_a_parent_cascades_to_an_already_spawned_childs_resolved_visibility() {
    let mut foliage = Foliage::new();
    let parent = foliage
        .world
        .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
    let child = foliage.world.branch(
        parent,
        Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
    );
    foliage.world.flush();
    assert!(resolved_visible(&mut foliage, child), "sanity: visible before any change");

    foliage.write_to(parent, Visibility::new(false));
    foliage.world.flush();
    assert!(
        !resolved_visible(&mut foliage, child),
        "the child never had its own Visibility touched -- this is purely inherited from \
         the parent cascading down to an existing child"
    );
}

#[test]
fn a_childs_own_visibility_false_wins_even_with_a_visible_parent() {
    let mut foliage = Foliage::new();
    let parent = foliage
        .world
        .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
    let child = foliage.world.branch(
        parent,
        Leaf::sprout()
            .at(Location::new())
            .elevate(Elevation::up(1))
            .with(Visibility::new(false)),
    );
    foliage.world.flush();

    assert!(resolved_visible(&mut foliage, parent));
    assert!(
        !resolved_visible(&mut foliage, child),
        "inherited(true) && current(false) && auto(true) must resolve to false -- the \
         conjunction, not just whichever value happened to be written last"
    );
}
