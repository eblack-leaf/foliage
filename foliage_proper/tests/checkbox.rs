//! `Checkbox` (`composite/checkbox.rs`) has no tests at all. Same icon-optionality
//! invariant as `Button` -- this session's other `icon-memory non-existent` fix -- worth
//! locking in the same way.

use bevy_ecs::observer::On;
use bevy_ecs::system::ResMut;
use foliage_proper::{
    Checkbox, CheckboxState, Checked, EcsExtension, Elevation, Entity, Foliage, GridExt, Icon,
    Location, Resource, Sprout, Stem,
};

fn children_of(foliage: &mut Foliage, parent: Entity) -> Vec<Entity> {
    let mut q = foliage.world.query::<(Entity, &Stem)>();
    q.iter(&foliage.world)
        .filter(|(_, stem)| stem.id == Some(parent))
        .map(|(e, _)| e)
        .collect()
}

fn has_icon_child(foliage: &mut Foliage, parent: Entity) -> bool {
    children_of(foliage, parent)
        .iter()
        .any(|e| foliage.world.get::<Icon>(*e).is_some())
}

fn spawn(foliage: &mut Foliage, on: bool) -> Entity {
    foliage.world.leaf(
        Checkbox::new()
            .on(on)
            .at(Location::new().xs(
                0.px().as_left().with(24.px().as_width()),
                0.px().as_top().with(24.px().as_height()),
            ))
            .elevate(foliage_proper::Elevation::up(1)),
    )
}

#[test]
fn spawning_on_true_sets_the_initial_checkbox_state() {
    let mut foliage = Foliage::new();
    let cb = spawn(&mut foliage, true);
    foliage.world.flush();
    assert!(foliage.world.get::<CheckboxState>(cb).unwrap().0);
}

#[test]
fn a_checkbox_spawned_without_a_check_icon_gets_no_icon_child() {
    let mut foliage = Foliage::new();
    let cb = spawn(&mut foliage, false);
    foliage.world.flush();
    assert!(!has_icon_child(&mut foliage, cb));
}

#[test]
fn a_checkbox_spawned_with_a_check_icon_gets_exactly_one_icon_child() {
    let mut foliage = Foliage::new();
    let cb = foliage.world.leaf(
        Checkbox::new()
            .check_icon(0)
            .at(Location::new().xs(
                0.px().as_left().with(24.px().as_width()),
                0.px().as_top().with(24.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    let icon_children = children_of(&mut foliage, cb)
        .iter()
        .filter(|e| foliage.world.get::<Icon>(**e).is_some())
        .count();
    assert_eq!(icon_children, 1);
}

#[test]
fn clicking_toggles_checkbox_state() {
    let mut foliage = Foliage::new();
    let cb = spawn(&mut foliage, false);
    foliage.world.flush();

    foliage
        .world
        .trigger_targets(foliage_proper::OnClick::new(), cb);
    foliage.world.flush();
    assert!(foliage.world.get::<CheckboxState>(cb).unwrap().0);

    foliage
        .world
        .trigger_targets(foliage_proper::OnClick::new(), cb);
    foliage.world.flush();
    assert!(!foliage.world.get::<CheckboxState>(cb).unwrap().0);
}

#[test]
fn writing_checkbox_state_fires_checked_with_the_new_value() {
    #[derive(Resource, Default)]
    struct LastChecked(Option<bool>);

    fn mark(trigger: On<Checked>, mut r: ResMut<LastChecked>) {
        r.0 = Some(trigger.event().on);
    }

    let mut foliage = Foliage::new();
    foliage.world.insert_resource(LastChecked::default());
    foliage.world.add_observer(mark);
    let cb = spawn(&mut foliage, false);
    foliage.world.flush();
    assert_eq!(
        foliage.world.resource::<LastChecked>().0,
        Some(false),
        "react fires once at spawn with the initial value"
    );

    foliage.write_to(cb, CheckboxState(true));
    foliage.world.flush();

    assert_eq!(foliage.world.resource::<LastChecked>().0, Some(true));
}
