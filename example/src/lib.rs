use foliage::bevy_ecs;
use foliage::bevy_ecs::bundle::Bundle;
use foliage::bevy_ecs::change_detection::Res;
use foliage::bevy_ecs::prelude::{Commands, Entity, IntoSystemConfigs, Resource};
use foliage::bevy_ecs::system::Local;
use foliage::button::{Button, ButtonArgs, ButtonStyle};
use foliage::color::Color;
use foliage::coordinate::area::Area;
use foliage::coordinate::position::Position;
use foliage::coordinate::section::Section;
use foliage::coordinate::{Coordinate, InterfaceContext};
use foliage::differential::Despawn;
use foliage::elm::config::{CoreSet, ElmConfiguration};
use foliage::elm::leaf::{DefaultSystemHook, Leaf};
use foliage::elm::Elm;
use foliage::icon::bundled_cov::BundledIcon;
use foliage::icon::IconId;
use foliage::scene::align::{SceneAligner, SceneAnchor};
use foliage::scene::bind::{SceneBinder, SceneRoot};
use foliage::scene::{Scene, SceneSpawn};
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
#[derive(Resource)]
struct ToDespawn(Entity);
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
            Position::<InterfaceContext>::new(85.0, 250.0),
            Area::new(240.0, 75.0),
        ),
        4,
    );
    let coordinate_three = Coordinate::new(
        Section::new(
            Position::<InterfaceContext>::new(140.0, 375.0),
            Area::new(135.0, 50.0),
        ),
        4,
    );
    let coordinate_four = Coordinate::new(
        Section::new(
            Position::<InterfaceContext>::new(35.0, 700.0),
            Area::new(340.0, 50.0),
        ),
        4,
    );
    let _entity = cmd.spawn_scene::<Button>(
        SceneAnchor(coordinate_one),
        &ButtonArgs::new(
            ButtonStyle::Ring,
            TextValue::new("Afternoon"),
            MaxCharacters(9),
            IconId::new(BundledIcon::Umbrella),
            Color::RED.into(),
            Color::OFF_BLACK.into(),
            &font,
            &scale_factor,
        ),
        SceneRoot::default(),
    );
    let _entity = cmd.spawn_scene::<Button>(
        SceneAnchor(coordinate_two),
        &ButtonArgs::new(
            ButtonStyle::Ring,
            TextValue::new("Fore-"),
            MaxCharacters(5),
            IconId::new(BundledIcon::Droplet),
            Color::GREEN.into(),
            Color::OFF_BLACK.into(),
            &font,
            &scale_factor,
        ),
        SceneRoot::default(),
    );
    let _entity = cmd.spawn_scene::<Button>(
        SceneAnchor(coordinate_three),
        &ButtonArgs::new(
            ButtonStyle::Ring,
            TextValue::new("CAST!"),
            MaxCharacters(5),
            IconId::new(BundledIcon::Cast),
            Color::BLUE.into(),
            Color::OFF_BLACK.into(),
            &font,
            &scale_factor,
        ),
        SceneRoot::default(),
    );
    let _e = cmd.spawn_scene::<DualButton>(
        coordinate_four.into(),
        &ButtonArgs::new(
            ButtonStyle::Ring,
            TextValue::new("Rainy-Day"),
            MaxCharacters(9),
            IconId::new(BundledIcon::CloudDrizzle),
            Color::RED_ORANGE_DARK.into(),
            Color::RED_ORANGE.into(),
            &font,
            &scale_factor,
        ),
        SceneRoot::default(),
    );
    cmd.insert_resource(ToDespawn(_e));
}
fn despawn_button(to_despawn: Res<ToDespawn>, mut cmd: Commands, mut local: Local<bool>) {
    if !*local {
        cmd.entity(to_despawn.0).insert(Despawn::signal_despawn());
        *local = true;
    }
}
impl Leaf for Tester {
    type SetDescriptor = DefaultSystemHook;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.startup().add_systems((spawn_button_tree,));
        elm.main()
            .add_systems((despawn_button.in_set(CoreSet::Spawn),));
    }
}
#[derive(Bundle)]
struct DualButton {}
impl Scene for DualButton {
    type Args<'a> = <Button as Scene>::Args<'a>;

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: SceneAnchor,
        args: &Self::Args<'_>,
        binder: &mut SceneBinder,
    ) -> Self {
        binder.bind_scene::<Button>(
            0.into(),
            ((-5).near(), 0.near(), 0).into(),
            anchor.0.section.area / (2, 1).into(),
            args,
            cmd,
        );
        binder.bind_scene::<Button>(
            1.into(),
            ((-5).far(), 0.near(), 0).into(),
            anchor.0.section.area / (2, 1).into(),
            args,
            cmd,
        );
        Self {}
    }
}
