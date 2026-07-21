//! `SegmentedControl` (`composite/segmented_control.rs`) has no tests at all -- same
//! structural shape as `RadioGroup` (already covered in `composites.rs`), just via
//! `SegmentedOptions`/`SegmentedSelected` instead of the radio vocabulary.

use bevy_ecs::observer::On;
use bevy_ecs::system::ResMut;
use foliage_proper::{
    Color, EcsExtension, Elevation, Entity, Foliage, GridExt, InteractionListener, Location,
    Logical, Resource, SegmentChanged, SegmentedControl, SegmentedOptions, SegmentedSelected,
    Section, Sprout, Stem,
};

fn children_of(foliage: &mut Foliage, parent: Entity) -> Vec<Entity> {
    let mut q = foliage.world.query::<(Entity, &Stem)>();
    q.iter(&foliage.world)
        .filter(|(_, stem)| stem.id == Some(parent))
        .map(|(e, _)| e)
        .collect()
}

/// Segments (the panels, distinguished from labels by carrying `InteractionListener`),
/// ordered left-to-right by their own resolved `Section` -- the only public way to line a
/// panel back up with "which option index is this," since `SegmentedHandle` is private.
fn active_segment_index(foliage: &mut Foliage, control: Entity, active_color: Color) -> usize {
    let mut panels: Vec<(Entity, f32, Color)> = children_of(foliage, control)
        .iter()
        .filter(|e| foliage.world.get::<InteractionListener>(**e).is_some())
        .map(|e| {
            let left = foliage.world.get::<Section<Logical>>(*e).unwrap().left();
            let color = *foliage.world.get::<Color>(*e).unwrap();
            (*e, left, color)
        })
        .collect();
    panels.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let active: Vec<usize> = panels
        .iter()
        .enumerate()
        .filter(|(_, (_, _, c))| *c == active_color)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(active.len(), 1, "exactly one panel should carry the active color");
    active[0]
}

fn spawn(foliage: &mut Foliage, active: Color, inactive: Color) -> Entity {
    foliage.world.leaf(
        SegmentedControl::new()
            .options(["One", "Two", "Three"])
            .selected(0)
            .colors(active, inactive, Color::default())
            .at(Location::new().xs(
                0.px().as_left().with(300.px().as_width()),
                0.px().as_top().with(40.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    )
}

#[test]
fn only_the_selected_segment_carries_the_active_color() {
    let active = Color::green(500);
    let inactive = Color::gray(600);
    let mut foliage = Foliage::new();
    let control = spawn(&mut foliage, active, inactive);
    foliage.world.flush();

    assert_eq!(active_segment_index(&mut foliage, control, active), 0);
}

#[test]
fn writing_segmented_selected_flips_exclusivity_without_respawning() {
    let active = Color::green(500);
    let inactive = Color::gray(600);
    let mut foliage = Foliage::new();
    let control = spawn(&mut foliage, active, inactive);
    foliage.world.flush();
    let mut before = children_of(&mut foliage, control);
    before.sort();

    foliage.write_to(control, SegmentedSelected(2));
    foliage.world.flush();

    assert_eq!(active_segment_index(&mut foliage, control, active), 2);
    let mut after = children_of(&mut foliage, control);
    after.sort();
    assert_eq!(before, after, "the PATCH reaction recolors stable entities -- it must not respawn anything");
}

#[test]
fn rewriting_options_rebuilds_to_the_new_count() {
    let mut foliage = Foliage::new();
    let control = spawn(&mut foliage, Color::green(500), Color::gray(600));
    foliage.world.flush();

    foliage.write_to(control, SegmentedOptions(vec!["A".into(), "B".into()]));
    foliage.world.flush();

    // 2 options * (panel + label) = 4 children
    assert_eq!(children_of(&mut foliage, control).len(), 4);
}

#[test]
fn selecting_fires_segment_changed_with_the_clamped_index() {
    #[derive(Resource, Default)]
    struct Last(Option<usize>);
    fn mark(trigger: On<SegmentChanged>, mut r: ResMut<Last>) {
        r.0 = Some(trigger.event().index);
    }

    let mut foliage = Foliage::new();
    foliage.world.insert_resource(Last::default());
    foliage.world.add_observer(mark);
    let control = spawn(&mut foliage, Color::green(500), Color::gray(600));
    foliage.world.flush();
    assert_eq!(foliage.world.resource::<Last>().0, Some(0));

    foliage.write_to(control, SegmentedSelected(99));
    foliage.world.flush();

    assert_eq!(
        foliage.world.resource::<Last>().0,
        Some(2),
        "an out-of-range write should clamp onto the last real option, not panic or pass 99 through"
    );
}
