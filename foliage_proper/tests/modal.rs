//! `Modal` (`composite/modal.rs`) opens and closes instantly -- no animation of its own.
//! `CloseModal` fires `Closed` and removes the whole modal subtree in the same command
//! batch.

use bevy_ecs::observer::On;
use bevy_ecs::system::ResMut;
use foliage_proper::{
    CloseModal, Closed, EcsExtension, Elevation, Entity, Foliage, Leaf, Location, Modal, Resource,
    Sprout, Stem,
};

fn children_of(foliage: &mut Foliage, parent: Entity) -> Vec<Entity> {
    let mut q = foliage.world.query::<(Entity, &Stem)>();
    q.iter(&foliage.world)
        .filter(|(_, stem)| stem.id == Some(parent))
        .map(|(e, _)| e)
        .collect()
}

fn spawn(foliage: &mut Foliage) -> Entity {
    foliage.world.leaf(
        Modal::new()
            .content(|tree, slot| {
                tree.branch(
                    slot,
                    Leaf::sprout()
                        .at(Location::new())
                        .elevate(Elevation::up(1)),
                )
            })
            .elevate(Elevation::up(50)),
    )
}

/// The content-holding slot -- a real `Stem`-child of the modal root (unlike its actual
/// author content, which is branched under the slot, not the modal itself).
fn slot_of(foliage: &mut Foliage, modal: Entity) -> Entity {
    let children = children_of(foliage, modal);
    assert_eq!(children.len(), 1, "the modal root should have exactly one direct child: the content slot");
    children[0]
}

#[test]
fn a_freshly_spawned_modal_has_content_under_its_slot() {
    let mut foliage = Foliage::new();
    let modal = spawn(&mut foliage);
    foliage.world.flush();

    let slot = slot_of(&mut foliage, modal);
    assert!(!children_of(&mut foliage, slot).is_empty(), "the author's .content(..) closure should have branched under the slot");
}

#[test]
fn close_modal_fires_closed_immediately_and_removes_content_right_away() {
    let mut foliage = Foliage::new();
    let modal = spawn(&mut foliage);
    foliage.world.flush();
    let slot = slot_of(&mut foliage, modal);

    #[derive(Resource, Default)]
    struct Fired(bool);
    fn mark(_trigger: On<Closed>, mut r: ResMut<Fired>) {
        r.0 = true;
    }
    foliage.world.insert_resource(Fired::default());
    foliage.world.add_observer(mark);

    foliage.world.trigger_targets(CloseModal::new(), modal);
    foliage.world.flush();

    assert!(foliage.world.resource::<Fired>().0, "Closed should fire when CloseModal is triggered");
    assert!(
        children_of(&mut foliage, slot).is_empty(),
        "content is removed as part of closing"
    );
}

#[test]
fn closing_removes_the_modal_root_itself_immediately() {
    let mut foliage = Foliage::new();
    let modal = spawn(&mut foliage);
    foliage.world.flush();

    foliage.world.trigger_targets(CloseModal::new(), modal);
    foliage.world.flush();

    assert!(
        foliage.world.get_entity(modal).is_err(),
        "closing is instant -- the root entity should be gone in the same command batch, not deferred"
    );
}
