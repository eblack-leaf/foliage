//! `Slider` (`composite/slider.rs`) has no tests at all. Drag/tap-to-seek pixel math reads
//! `CurrentInteraction`'s live click position, whose fields are `pub(crate)` (same reason
//! `interaction/mod.rs`'s own routing tests had to live inline in that module) -- out of
//! reach from an external test with no public constructor for that resource. What's fully
//! testable from here is the composite's own reactive contract: `Progress`'s clamping,
//! `ProgressChanged` firing, and `.interactive(false)` disabling the whole thing --
//! Slider's actual state machine, independent of how a given `Progress` write arrived.

use foliage_proper::{
    Disable, EcsExtension, Elevation, Entity, Foliage, GridExt, InteractionListener, Location,
    Progress, ProgressChanged, Resource, Slider, SliderBehavior, Sprout,
};
use bevy_ecs::observer::On;
use bevy_ecs::system::ResMut;

fn spawn(foliage: &mut Foliage, progress: f32) -> Entity {
    foliage.world.leaf(
        Slider::new()
            .progress(progress)
            .at(Location::new().xs(
                0.px().as_left().with(200.px().as_width()),
                0.px().as_top().with(20.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    )
}

#[test]
fn progress_is_clamped_into_zero_to_one_at_spawn() {
    let mut foliage = Foliage::new();
    let over = spawn(&mut foliage, 1.5);
    let under = spawn(&mut foliage, -0.5);
    foliage.world.flush();

    assert_eq!(foliage.world.get::<Progress>(over).unwrap().0, 1.0);
    assert_eq!(foliage.world.get::<Progress>(under).unwrap().0, 0.0);
}

#[test]
fn writing_progress_fires_progress_changed_with_the_same_value() {
    #[derive(Resource, Default)]
    struct Last(Option<f32>);
    fn mark(trigger: On<ProgressChanged>, mut r: ResMut<Last>) {
        r.0 = Some(trigger.event().progress);
    }

    let mut foliage = Foliage::new();
    foliage.world.insert_resource(Last::default());
    foliage.world.add_observer(mark);
    let slider = spawn(&mut foliage, 0.0);
    foliage.world.flush();
    assert_eq!(foliage.world.resource::<Last>().0, Some(0.0), "react fires once at spawn too");

    foliage.write_to(slider, Progress(0.75));
    foliage.world.flush();

    assert_eq!(foliage.world.resource::<Last>().0, Some(0.75));
}

#[test]
fn a_non_interactive_slider_has_its_listener_disabled() {
    let mut foliage = Foliage::new();
    let slider = foliage.world.leaf(
        Slider::new()
            .interactive(false)
            .at(Location::new().xs(
                0.px().as_left().with(200.px().as_width()),
                0.px().as_top().with(20.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();

    assert!(foliage.world.get::<InteractionListener>(slider).unwrap().disabled());
}

#[test]
fn re_enabling_interactivity_restores_the_listener() {
    let mut foliage = Foliage::new();
    let slider = foliage.world.leaf(
        Slider::new()
            .interactive(false)
            .at(Location::new().xs(
                0.px().as_left().with(200.px().as_width()),
                0.px().as_top().with(20.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    foliage.world.flush();
    assert!(foliage.world.get::<InteractionListener>(slider).unwrap().disabled(), "sanity");

    foliage.write_to(
        slider,
        SliderBehavior { interactive: true },
    );
    foliage.world.flush();

    assert!(!foliage.world.get::<InteractionListener>(slider).unwrap().disabled());
}

#[test]
fn disabling_the_root_directly_disables_its_listener() {
    let mut foliage = Foliage::new();
    let slider = spawn(&mut foliage, 0.0);
    foliage.world.flush();

    foliage.world.trigger_targets(Disable::new(), slider);
    foliage.world.flush();
    assert!(foliage.world.get::<InteractionListener>(slider).unwrap().disabled());
}
