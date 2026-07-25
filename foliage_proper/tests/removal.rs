//! `Remove` (`remove.rs`): removing an entity cascades to every `Stem`-descendant
//! recursively, not just the one entity directly targeted -- documented in several
//! composites' own doc comments ("Teardown is nothing: `Remove` walks every `Stem`-child
//! recursively") but never actually asserted anywhere until now.

use foliage_proper::{EcsExtension, Elevation, Foliage, Leaf, Location, Sprout};

#[test]
fn removing_a_parent_despawns_its_whole_descendant_chain() {
    let mut foliage = Foliage::new();
    let grandparent = foliage
        .world
        .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
    let parent = foliage.world.branch(
        grandparent,
        Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
    );
    let child = foliage.world.branch(
        parent,
        Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
    );
    let grandchild = foliage.world.branch(
        child,
        Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
    );
    foliage.world.flush();
    assert!(
        foliage.world.get_entity(child).is_ok(),
        "sanity: everything exists before removal"
    );

    foliage.world.remove(parent);
    foliage.world.flush();

    assert!(
        foliage.world.get_entity(grandparent).is_ok(),
        "the grandparent wasn't removed -- only its descendant was"
    );
    assert!(
        foliage.world.get_entity(parent).is_err(),
        "the directly-removed entity is gone"
    );
    assert!(
        foliage.world.get_entity(child).is_err(),
        "cascaded one level down"
    );
    assert!(
        foliage.world.get_entity(grandchild).is_err(),
        "cascaded two levels down -- this is the actual point of the test, not just \
         the immediate child"
    );
}

#[test]
fn removing_a_leaf_child_does_not_affect_its_siblings() {
    let mut foliage = Foliage::new();
    let parent = foliage
        .world
        .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
    let a = foliage.world.branch(
        parent,
        Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
    );
    let b = foliage.world.branch(
        parent,
        Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    foliage.world.remove(a);
    foliage.world.flush();

    assert!(foliage.world.get_entity(a).is_err());
    assert!(
        foliage.world.get_entity(b).is_ok(),
        "removing one sibling shouldn't touch the other"
    );
    assert!(foliage.world.get_entity(parent).is_ok());
}
