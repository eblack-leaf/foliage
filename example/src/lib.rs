use foliage::bevy_ecs;
use foliage::bevy_ecs::bundle::Bundle;
use foliage::bevy_ecs::change_detection::Res;
use foliage::bevy_ecs::prelude::Commands;
use foliage::button::Button;
use foliage::color::Color;
use foliage::coordinate::area::Area;
use foliage::coordinate::position::Position;
use foliage::coordinate::section::Section;
use foliage::coordinate::{Coordinate, InterfaceContext};
use foliage::elm::{Elm, Leaf};
use foliage::icon::bundled_cov::BundledIcon;
use foliage::icon::IconId;
use foliage::scene::{Scene, SceneAligner, SceneAnchor, SceneBinder, SceneRoot, SceneSpawn};
use foliage::text::font::MonospacedFont;
use foliage::text::{MaxCharacters, TextValue};
use foliage::window::{ScaleFactor, WindowDescriptor};
use foliage::{AndroidInterface, Foliage};
pub fn entry(android_interface: AndroidInterface) {
    Foliage::new()
        .with_window_descriptor(
            WindowDescriptor::new()
                .with_title("foliage")
                .with_desktop_dimensions((411, 913)),
        )
        .with_leaf::<Tester>()
        .with_android_interface(android_interface)
        .run();
}

struct Tester;
fn spawn_button_tree(mut cmd: Commands, scale_factor: Res<ScaleFactor>, font: Res<MonospacedFont>) {
    let coordinate_one = Coordinate::new(
        Section::new(
            Position::<InterfaceContext>::new(35.0, 100.0),
            Area::new(340.0, 100.0),
        ),
        4,
    );
    let coordinate_two = Coordinate::new(
        Section::new(
            Position::<InterfaceContext>::new(85.0, 300.0),
            Area::new(240.0, 75.0),
        ),
        4,
    );
    let coordinate_three = Coordinate::new(
        Section::new(
            Position::<InterfaceContext>::new(140.0, 500.0),
            Area::new(135.0, 50.0),
        ),
        4,
    );
    let coordinate_four = Coordinate::new(
        Section::new(
            Position::<InterfaceContext>::new(10.0, 700.0),
            Area::new(390.0, 50.0),
        ),
        4,
    );
    let _entity = cmd.spawn_scene::<Button>(
        SceneAnchor(coordinate_one),
        &(
            TextValue::new("Clock"),
            MaxCharacters(5),
            IconId::new(BundledIcon::Clock),
            Color::RED.into(),
            &font,
            &scale_factor,
        ),
        SceneRoot::default(),
    );
    let _entity = cmd.spawn_scene::<Button>(
        SceneAnchor(coordinate_two),
        &(
            TextValue::new("Point"),
            MaxCharacters(5),
            IconId::new(BundledIcon::Compass),
            Color::GREEN.into(),
            &font,
            &scale_factor,
        ),
        SceneRoot::default(),
    );
    let _entity = cmd.spawn_scene::<Button>(
        SceneAnchor(coordinate_three),
        &(
            TextValue::new("CAST!"),
            MaxCharacters(5),
            IconId::new(BundledIcon::Cast),
            Color::BLUE.into(),
            &font,
            &scale_factor,
        ),
        SceneRoot::default(),
    );
    let _e = cmd.spawn_scene::<DualButton>(
        coordinate_four.into(),
        &(
            TextValue::new("Rainy-Day"),
            MaxCharacters(9),
            IconId::new(BundledIcon::CloudDrizzle),
            Color::BLUE.into(),
            &font,
            &scale_factor,
        ),
        SceneRoot::default(),
    );
}
impl Leaf for Tester {
    fn attach(elm: &mut Elm) {
        elm.job.startup().add_systems((spawn_button_tree,));
    }
}
#[derive(Bundle)]
struct DualButton {}
impl Scene for DualButton {
    type Args<'a> = <Button as Scene>::Args<'a>;

    fn bind_nodes<'a>(
        cmd: &mut Commands,
        anchor: SceneAnchor,
        args: &Self::Args<'a>,
        binder: &mut SceneBinder,
    ) -> Self {
        binder.bind_scene::<Button, _, _, _>(
            0,
            ((-5).near(), 0.near(), 0),
            anchor.0.section.area / (2, 1).into(),
            args,
            cmd,
        );
        binder.bind_scene::<Button, _, _, _>(
            1,
            ((-5).far(), 0.near(), 0),
            anchor.0.section.area / (2, 1).into(),
            args,
            cmd,
        );
        Self {}
    }
}
