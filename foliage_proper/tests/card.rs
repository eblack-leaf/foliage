//! `Card` (`composite/card.rs`) opens and closes instantly -- no animation of its own.
//! `CloseCard` fires `Closed` and removes the whole card subtree in the same command
//! batch. `main` occupies the top two-thirds; `header`/`desc` (both optional) share the
//! bottom third as their own two-row grouping, header above desc.
//!
//! Geometry (main's actual height ratio, header-above-desc ordering) is verified in
//! `composite/card.rs`'s own internal `#[cfg(test)]` module instead of here: a root-level
//! `Card` positions itself with `.pct()` against the viewport when unanchored, and
//! `ViewportHandle` is crate-internal (`ginkgo` is a private module, not re-exported) --
//! unreachable from this external integration-test crate, so there's no way to set up a
//! resolvable viewport from out here. This file only checks structure and lifecycle.

use bevy_ecs::observer::On;
use bevy_ecs::system::ResMut;
use foliage_proper::{
    Card, CloseCard, Closed, EcsExtension, Elevation, Entity, Foliage, GridExt, Leaf, Location,
    Resource, Sprout, Stem,
};

fn children_of(foliage: &mut Foliage, parent: Entity) -> Vec<Entity> {
    let mut q = foliage.world.query::<(Entity, &Stem)>();
    q.iter(&foliage.world)
        .filter(|(_, stem)| stem.id == Some(parent))
        .map(|(e, _)| e)
        .collect()
}

fn leaf_child(tree: &mut foliage_proper::Tree, slot: Entity) -> Entity {
    tree.branch(
        slot,
        Leaf::sprout()
            .at(Location::new())
            .elevate(Elevation::up(1)),
    )
}

fn spawn_main_only(foliage: &mut Foliage) -> Entity {
    foliage.world.leaf(
        Card::new()
            .main(leaf_child)
            .at(Location::new().xs(
                0.px().as_left().with(300.px().as_width()),
                0.px().as_top().with(300.px().as_height()),
            ))
            .elevate(Elevation::up(50)),
    )
}

fn spawn_with_header_and_desc(foliage: &mut Foliage) -> Entity {
    foliage.world.leaf(
        Card::new()
            .main(leaf_child)
            .header(leaf_child)
            .desc(leaf_child)
            .at(Location::new().xs(
                0.px().as_left().with(300.px().as_width()),
                0.px().as_top().with(300.px().as_height()),
            ))
            .elevate(Elevation::up(50)),
    )
}

#[test]
fn a_freshly_spawned_card_has_content_under_its_main_slot() {
    let mut foliage = Foliage::new();
    let card = spawn_main_only(&mut foliage);
    foliage.world.flush();

    let children = children_of(&mut foliage, card);
    assert_eq!(
        children.len(),
        1,
        "main only -- exactly one child slot (no header/desc configured)"
    );
    assert!(
        !children_of(&mut foliage, children[0]).is_empty(),
        "the author's .main(..) closure should have branched under the main slot"
    );
}

#[test]
fn header_and_desc_are_not_spawned_when_not_configured() {
    let mut foliage = Foliage::new();
    let card = spawn_main_only(&mut foliage);
    foliage.world.flush();

    assert_eq!(
        children_of(&mut foliage, card).len(),
        1,
        "no header/desc configured -- no bottom-third container should exist at all"
    );
}

#[test]
fn header_and_desc_together_add_exactly_one_bottom_third_container() {
    let mut foliage = Foliage::new();
    let card = spawn_with_header_and_desc(&mut foliage);
    foliage.world.flush();

    let children = children_of(&mut foliage, card);
    assert_eq!(
        children.len(),
        2,
        "main slot + one shared bottom-third container, not one container per slot"
    );
    let bottom_third = children
        .into_iter()
        .find(|e| children_of(&mut foliage, *e).len() == 2)
        .expect("a bottom-third container holding exactly header + desc should exist");
    assert_eq!(children_of(&mut foliage, bottom_third).len(), 2);
}

#[test]
fn close_card_fires_closed_and_removes_the_whole_subtree_immediately() {
    let mut foliage = Foliage::new();
    let card = spawn_main_only(&mut foliage);
    foliage.world.flush();

    #[derive(Resource, Default)]
    struct Fired(bool);
    fn mark(_trigger: On<Closed>, mut r: ResMut<Fired>) {
        r.0 = true;
    }
    foliage.world.insert_resource(Fired::default());
    foliage.world.add_observer(mark);

    foliage.world.trigger_targets(CloseCard::new(), card);
    foliage.world.flush();

    assert!(
        foliage.world.resource::<Fired>().0,
        "Closed should fire when CloseCard is triggered"
    );
    assert!(
        foliage.world.get_entity(card).is_err(),
        "closing is instant -- the root entity should be gone in the same command batch, not deferred"
    );
}
