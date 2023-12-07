use foliage::bevy_ecs::change_detection::Res;
use foliage::bevy_ecs::prelude::{Commands, IntoSystemConfigs};
use foliage::bevy_ecs::system::Local;
use foliage::button::Button;
use foliage::circle::Circle;
use foliage::color::Color;
use foliage::coordinate::area::Area;
use foliage::coordinate::position::Position;
use foliage::coordinate::section::Section;
use foliage::coordinate::{Coordinate, CoordinateLeaf, InterfaceContext};
use foliage::elm::{Elm, Leaf, SystemSets};
use foliage::icon::bundled_cov::BundledIcon;
use foliage::icon::{Icon, IconId, IconScale};
use foliage::panel::Panel;
use foliage::rectangle::Rectangle;
use foliage::scene::{SceneAnchor, SceneRoot, SceneSpawn};
use foliage::text::font::MonospacedFont;
use foliage::text::{MaxCharacters, Text, TextValue};
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
    let _entity = cmd.spawn_scene::<Button>(
        SceneAnchor(coordinate_one),
        &(
            TextValue::new("Clock"),
            MaxCharacters(5),
            IconId::new(BundledIcon::Clock),
            IconScale::Eighty,
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
            IconScale::Forty,
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
            IconScale::Twenty,
            Color::BLUE.into(),
            &font,
            &scale_factor,
        ),
        SceneRoot::default(),
    );
}
impl Leaf for Tester {
    fn attach(elm: &mut Elm) {
        #[cfg(any(not(target_os = "android"), target_arch = "x86_64"))]
        let android_offset = 0;
        #[cfg(all(target_os = "android", target_arch = "aarch64"))]
        let android_offset = 50;
        elm.job.startup().add_systems((spawn_button_tree,));
    }
}
