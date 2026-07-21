//! `Popover` (`composite/popover.rs`) has no tests at all. Its trigger content and its
//! expanded content follow different lifecycles by design (trigger built once at spawn,
//! content rebuilt fresh every open) -- worth pinning down both halves.

use foliage_proper::{
    EcsExtension, Elevation, Entity, Foliage, GridExt, Leaf, Location, OnClick, Panel, Popover,
    PopoverExpanded, Sprout, Stem,
};

fn children_of(foliage: &mut Foliage, parent: Entity) -> Vec<Entity> {
    let mut q = foliage.world.query::<(Entity, &Stem)>();
    q.iter(&foliage.world)
        .filter(|(_, stem)| stem.id == Some(parent))
        .map(|(e, _)| e)
        .collect()
}

fn panel_count(foliage: &mut Foliage) -> usize {
    let mut q = foliage.world.query::<&Panel>();
    q.iter(&foliage.world).count()
}

fn spawn(foliage: &mut Foliage) -> Entity {
    foliage.world.leaf(
        Popover::new()
            .trigger(|tree, slot| {
                tree.branch(slot, Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)))
            })
            .content(|tree, slot| {
                tree.branch(slot, Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)))
            })
            .extent(100.px())
            .at(Location::new().xs(
                0.px().as_left().with(80.px().as_width()),
                0.px().as_top().with(30.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    )
}

#[test]
fn the_trigger_content_is_built_once_at_spawn() {
    let mut foliage = Foliage::new();
    let popover = spawn(&mut foliage);
    foliage.world.flush();

    // trigger slot is a real Stem-child; the trigger closure branches under it
    let trigger_slot = children_of(&mut foliage, popover);
    assert_eq!(trigger_slot.len(), 1);
    assert!(!children_of(&mut foliage, trigger_slot[0]).is_empty(), "the trigger closure should have built under the trigger slot");
}

#[test]
fn a_freshly_spawned_popover_is_closed_with_no_content_surface() {
    let mut foliage = Foliage::new();
    let popover = spawn(&mut foliage);
    foliage.world.flush();

    assert!(!foliage.world.get::<PopoverExpanded>(popover).unwrap().0);
    // the trigger's own Panel(default()) doesn't exist (trigger is a bare Leaf here) --
    // any Panel in the world at all must be the content surface, so zero means closed.
    assert_eq!(panel_count(&mut foliage), 0);
}

#[test]
fn clicking_the_trigger_opens_it_and_builds_the_content_surface() {
    let mut foliage = Foliage::new();
    let popover = spawn(&mut foliage);
    foliage.world.flush();

    foliage.world.trigger_targets(OnClick::new(), popover);
    foliage.world.flush();

    assert!(foliage.world.get::<PopoverExpanded>(popover).unwrap().0);
    assert_eq!(panel_count(&mut foliage), 1);
}

#[test]
fn clicking_again_closes_it_and_tears_down_the_content_surface() {
    let mut foliage = Foliage::new();
    let popover = spawn(&mut foliage);
    foliage.world.flush();

    foliage.world.trigger_targets(OnClick::new(), popover);
    foliage.world.flush();
    foliage.world.trigger_targets(OnClick::new(), popover);
    foliage.world.flush();

    assert!(!foliage.world.get::<PopoverExpanded>(popover).unwrap().0);
    assert_eq!(panel_count(&mut foliage), 0, "content is a top-level leaf -- closing must tear it down, not just hide it");
}

#[test]
fn reopening_rebuilds_a_fresh_content_surface_each_time() {
    let mut foliage = Foliage::new();
    let popover = spawn(&mut foliage);
    foliage.world.flush();

    foliage.world.trigger_targets(OnClick::new(), popover);
    foliage.world.flush();
    let mut q = foliage.world.query::<(Entity, &Panel)>();
    let first_surface: Entity = q.iter(&foliage.world).next().unwrap().0;

    foliage.world.trigger_targets(OnClick::new(), popover);
    foliage.world.flush();
    foliage.world.trigger_targets(OnClick::new(), popover);
    foliage.world.flush();
    let mut q = foliage.world.query::<(Entity, &Panel)>();
    let second_surface: Entity = q.iter(&foliage.world).next().unwrap().0;

    assert_ne!(first_surface, second_surface, "content has no cheaper patch path -- each open rebuilds fresh, per the module's own doc comment");
}
