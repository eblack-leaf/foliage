//! `Disable`/`Enable`'s cascade (`disable.rs`/`enable.rs`) is a separate system from
//! `Visibility` (see `visibility.rs`'s own comment) -- it flips `InteractionListener`'s
//! enablement bits, not `ResolvedVisibility`. A parent's `Disable` cascades to children
//! through a *different* bit (`INHERIT_ENABLED`) than the one it flips on the entity
//! directly targeted (`ENABLED`), which is exactly the distinction that keeps a child's own
//! `Disable` state independent of its parent's -- untested until now, despite being the
//! same "does a flag propagate to already-spawned children" bug class that bit `Grid`
//! three times this session.

use foliage_proper::{
    Disable, EcsExtension, Elevation, Enable, Entity, Foliage, InteractionListener, Leaf, Location,
    Sprout,
};

fn disabled(foliage: &mut Foliage, entity: Entity) -> bool {
    foliage
        .world
        .get::<InteractionListener>(entity)
        .unwrap()
        .disabled()
}

fn spawn_leaf(foliage: &mut Foliage) -> Entity {
    foliage.world.leaf(
        Leaf::sprout()
            .at(Location::new())
            .elevate(Elevation::up(1))
            .with(InteractionListener::new()),
    )
}

fn branch_leaf(foliage: &mut Foliage, parent: Entity) -> Entity {
    foliage.world.branch(
        parent,
        Leaf::sprout()
            .at(Location::new())
            .elevate(Elevation::up(1))
            .with(InteractionListener::new()),
    )
}

#[test]
fn a_bare_leaf_defaults_to_enabled() {
    let mut foliage = Foliage::new();
    let leaf = spawn_leaf(&mut foliage);
    foliage.world.flush();
    assert!(!disabled(&mut foliage, leaf));
}

#[test]
fn disabling_an_entity_flips_its_own_listener_to_disabled() {
    let mut foliage = Foliage::new();
    let leaf = spawn_leaf(&mut foliage);
    foliage.world.flush();

    foliage.world.trigger_targets(Disable::new(), leaf);
    foliage.world.flush();

    assert!(disabled(&mut foliage, leaf));
}

#[test]
fn disabling_a_parent_cascades_to_an_already_spawned_childs_listener() {
    let mut foliage = Foliage::new();
    let parent = spawn_leaf(&mut foliage);
    let child = branch_leaf(&mut foliage, parent);
    foliage.world.flush();
    assert!(
        !disabled(&mut foliage, child),
        "sanity: enabled before any change"
    );

    foliage.world.trigger_targets(Disable::new(), parent);
    foliage.world.flush();

    assert!(
        disabled(&mut foliage, child),
        "the child never had Disable triggered on it directly -- this is purely inherited \
         from the parent cascading down to an already-spawned child"
    );
}

#[test]
fn disabling_a_parent_cascades_through_multiple_levels() {
    let mut foliage = Foliage::new();
    let grandparent = spawn_leaf(&mut foliage);
    let parent = branch_leaf(&mut foliage, grandparent);
    let child = branch_leaf(&mut foliage, parent);
    foliage.world.flush();

    foliage.world.trigger_targets(Disable::new(), grandparent);
    foliage.world.flush();

    assert!(disabled(&mut foliage, parent));
    assert!(
        disabled(&mut foliage, child),
        "cascaded two levels down, not just one"
    );
}

#[test]
fn enabling_a_previously_disabled_parent_restores_the_childs_enabled_state() {
    let mut foliage = Foliage::new();
    let parent = spawn_leaf(&mut foliage);
    let child = branch_leaf(&mut foliage, parent);
    foliage.world.flush();

    foliage.world.trigger_targets(Disable::new(), parent);
    foliage.world.flush();
    assert!(
        disabled(&mut foliage, child),
        "sanity: disabled after the parent's Disable"
    );

    foliage.world.trigger_targets(Enable::new(), parent);
    foliage.world.flush();

    assert!(
        !disabled(&mut foliage, child),
        "Enable should cascade the same way Disable did"
    );
}

#[test]
fn a_childs_own_disable_survives_the_parent_being_re_enabled() {
    // the actual point of using a *different* bit (INHERIT_ENABLED) for inherited state
    // than the one Disable/Enable flip on the entity directly targeted (ENABLED): a child
    // that disabled *itself* shouldn't come back to life just because its parent got
    // re-enabled -- the two states are independent, not last-write-wins.
    let mut foliage = Foliage::new();
    let parent = spawn_leaf(&mut foliage);
    let child = branch_leaf(&mut foliage, parent);
    foliage.world.flush();

    foliage.world.trigger_targets(Disable::new(), child);
    foliage.world.flush();
    assert!(
        disabled(&mut foliage, child),
        "sanity: the child disabled itself directly"
    );

    foliage.world.trigger_targets(Enable::new(), parent);
    foliage.world.flush();

    assert!(
        disabled(&mut foliage, child),
        "the parent's Enable only restores INHERIT_ENABLED -- the child's own ENABLED bit, \
         which it cleared itself, must stay cleared"
    );
}

#[test]
fn disabling_a_child_directly_does_not_affect_its_sibling() {
    let mut foliage = Foliage::new();
    let parent = spawn_leaf(&mut foliage);
    let a = branch_leaf(&mut foliage, parent);
    let b = branch_leaf(&mut foliage, parent);
    foliage.world.flush();

    foliage.world.trigger_targets(Disable::new(), a);
    foliage.world.flush();

    assert!(disabled(&mut foliage, a));
    assert!(
        !disabled(&mut foliage, b),
        "disabling one sibling directly shouldn't touch the other"
    );
}
