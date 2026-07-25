//! `Dropdown` (`composite/dropdown.rs`) has no tests at all.

use bevy_ecs::observer::On;
use bevy_ecs::system::ResMut;
use foliage_proper::{
    Dropdown, DropdownOptions, EcsExtension, Elevation, Entity, Expanded, Foliage, GridExt, List,
    Location, OnClick, Resource, Selected, SelectionChanged, Sprout, Stem, TextValue,
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
        Dropdown::new()
            .options(["One", "Two", "Three"])
            .selected(0)
            .at(Location::new().xs(
                0.px().as_left().with(200.px().as_width()),
                0.px().as_top().with(40.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    )
}

/// The option surface is a top-level `List` (see the module doc comment on why: `pct()` in
/// it needs to resolve against the viewport, not the dropdown's own small box) -- it's
/// never a `Stem`-child of the dropdown, so it has to be found by scanning for a `List`
/// entity entirely, not through `children_of`.
fn option_surface_exists(foliage: &mut Foliage) -> bool {
    let mut q = foliage.world.query::<&List>();
    q.iter(&foliage.world).next().is_some()
}

#[test]
fn a_freshly_spawned_dropdown_is_collapsed_with_no_option_surface() {
    let mut foliage = Foliage::new();
    spawn(&mut foliage);
    foliage.world.flush();

    assert!(!option_surface_exists(&mut foliage));
}

#[test]
fn clicking_the_trigger_expands_it_and_spawns_the_option_surface() {
    let mut foliage = Foliage::new();
    let dd = spawn(&mut foliage);
    foliage.world.flush();

    foliage.world.trigger_targets(OnClick::new(), dd);
    foliage.world.flush();

    assert!(foliage.world.get::<Expanded>(dd).unwrap().0);
    assert!(option_surface_exists(&mut foliage));
}

#[test]
fn clicking_the_trigger_twice_collapses_it_again_and_removes_the_surface() {
    let mut foliage = Foliage::new();
    let dd = spawn(&mut foliage);
    foliage.world.flush();

    foliage.world.trigger_targets(OnClick::new(), dd);
    foliage.world.flush();
    foliage.world.trigger_targets(OnClick::new(), dd);
    foliage.world.flush();

    assert!(!foliage.world.get::<Expanded>(dd).unwrap().0);
    assert!(!option_surface_exists(&mut foliage));
}

#[test]
fn writing_selected_updates_the_trigger_text_and_fires_selection_changed() {
    #[derive(Resource, Default)]
    struct Last(Option<usize>);
    fn mark(trigger: On<SelectionChanged>, mut r: ResMut<Last>) {
        r.0 = Some(trigger.event().index);
    }

    let mut foliage = Foliage::new();
    foliage.world.insert_resource(Last::default());
    foliage.world.add_observer(mark);
    let dd = spawn(&mut foliage);
    foliage.world.flush();
    assert_eq!(foliage.world.resource::<Last>().0, Some(0));

    foliage.write_to(dd, Selected(2));
    foliage.world.flush();

    assert_eq!(foliage.world.resource::<Last>().0, Some(2));
    let trigger_text = children_of(&mut foliage, dd)
        .iter()
        .find_map(|e| foliage.world.get::<TextValue>(*e).map(|t| t.0.clone()));
    assert_eq!(trigger_text, Some("Three".to_string()));
}

#[test]
fn rewriting_options_while_collapsed_does_not_spawn_a_surface() {
    let mut foliage = Foliage::new();
    let dd = spawn(&mut foliage);
    foliage.world.flush();

    foliage.write_to(dd, DropdownOptions(vec!["X".into(), "Y".into()]));
    foliage.world.flush();

    assert!(!option_surface_exists(&mut foliage));
}
