//! `Button` (`composite/button.rs`) has no tests at all. The icon-optional invariant is
//! the specific thing worth locking in: this session's `icon-memory non-existent` panic
//! (a real runtime crash found by hand, running the `controls` example) traced to Button
//! unconditionally spawning an icon child even when none was configured -- fixed at the
//! library level (`ButtonHasIcon`, lazy icon spawn), but never regression-locked.

use foliage_proper::{
    Button, Color, EcsExtension, Elevation, Engagement, Entity, Foliage, GridExt, IconValue,
    Location, Sprout, Stem, Text,
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
        .any(|e| foliage.world.get::<IconValue>(*e).is_some())
}

#[test]
fn a_button_spawned_without_an_icon_gets_no_icon_child() {
    let mut foliage = Foliage::new();
    let btn = foliage.world.leaf(
        Button::new()
            .text("Save")
            .at(Location::new().xs(
                0.px().as_left().with(100.px().as_width()),
                0.px().as_top().with(40.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    assert!(
        !has_icon_child(&mut foliage, btn),
        "no .icon(..) was ever called -- no icon child should exist"
    );
}

#[test]
fn a_button_spawned_with_an_icon_gets_exactly_one_icon_child() {
    let mut foliage = Foliage::new();
    let btn = foliage.world.leaf(
        Button::new()
            .icon(0)
            .text("Save")
            .at(Location::new().xs(
                0.px().as_left().with(100.px().as_width()),
                0.px().as_top().with(40.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    let icon_children = children_of(&mut foliage, btn)
        .iter()
        .filter(|e| foliage.world.get::<IconValue>(**e).is_some())
        .count();
    assert_eq!(icon_children, 1);
}

#[test]
fn engaged_and_disengaged_flip_the_engagement_component() {
    let mut foliage = Foliage::new();
    let btn = foliage.world.leaf(
        Button::new()
            .text("Save")
            .at(Location::new().xs(
                0.px().as_left().with(100.px().as_width()),
                0.px().as_top().with(40.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();
    assert!(
        !foliage.world.get::<Engagement>(btn).unwrap().0,
        "sanity: not engaged at rest"
    );

    foliage
        .world
        .trigger_targets(foliage_proper::Engaged::new(), btn);
    foliage.world.flush();
    assert!(foliage.world.get::<Engagement>(btn).unwrap().0);

    foliage
        .world
        .trigger_targets(foliage_proper::Disengaged::new(), btn);
    foliage.world.flush();
    assert!(!foliage.world.get::<Engagement>(btn).unwrap().0);
}

#[test]
fn writing_text_value_updates_the_label_child() {
    let mut foliage = Foliage::new();
    let btn = foliage.world.leaf(
        Button::new()
            .text("Save")
            .at(Location::new().xs(
                0.px().as_left().with(100.px().as_width()),
                0.px().as_top().with(40.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    foliage.write_to(btn, foliage_proper::TextValue("Cancel".to_string()));
    foliage.world.flush();

    let label = children_of(&mut foliage, btn)
        .iter()
        .find_map(|e| foliage.world.get::<Text>(*e).map(|t| t.value.clone()));
    assert_eq!(label, Some("Cancel".to_string()));
}

#[test]
fn spawn_colors_land_on_the_public_style_component() {
    let mut foliage = Foliage::new();
    let fg = Color::gray(900);
    let bg = Color::gray(100);
    let btn = foliage.world.leaf(
        Button::new()
            .text("Save")
            .colors(fg, bg)
            .at(Location::new().xs(
                0.px().as_left().with(100.px().as_width()),
                0.px().as_top().with(40.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    let style = foliage
        .world
        .get::<foliage_proper::ButtonStyle>(btn)
        .unwrap();
    assert_eq!(style.foreground, fg);
    assert_eq!(style.background, bg);
}
