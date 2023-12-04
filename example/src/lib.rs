use foliage::bevy_ecs::change_detection::Res;
use foliage::bevy_ecs::prelude::{Commands, IntoSystemConfigs, ResMut};
use foliage::bevy_ecs::system::{Local, Resource};
use foliage::button::Button;
use foliage::circle::Circle;
use foliage::color::Color;
use foliage::coordinate::area::Area;
use foliage::coordinate::position::Position;
use foliage::coordinate::section::Section;
use foliage::coordinate::{Coordinate, CoordinateLeaf};
use foliage::elm::{Elm, Leaf, SystemSets};
use foliage::icon::bundled_cov::BundledIcon;
use foliage::icon::{Icon, IconId};
use foliage::panel::Panel;
use foliage::rectangle::Rectangle;
use foliage::scene::{AddScene, SceneHandle};
use foliage::text::{FontSize, Text};
use foliage::window::{ScaleFactor, WindowDescriptor};
use foliage::{AndroidInterface, Foliage};

pub fn entry(android_interface: AndroidInterface) {
    Foliage::new()
        .with_window_descriptor(
            WindowDescriptor::new()
                .with_title("foliage")
                .with_desktop_dimensions((411, 913)),
        )
        .with_renderleaf::<Panel>()
        .with_renderleaf::<Circle>()
        .with_renderleaf::<Rectangle>()
        .with_renderleaf::<Icon>()
        .with_renderleaf::<Text>()
        .with_leaf::<CoordinateLeaf>()
        .with_leaf::<Tester>()
        .with_android_interface(android_interface)
        .run();
}

struct Tester;
#[derive(Clone)]
struct TestButton;
fn spawn_button_tree(
    mut first_run: Local<bool>,
    button_scene: Res<SceneHandle<Button<TestButton>>>,
    mut cmd: Commands,
    scale_factor: Res<ScaleFactor>,
) {
    if !*first_run {
        let coordinate = Coordinate::new(
            Section::new(Position::new(10.0, 10.0), Area::new(200.0, 100.0)),
            4,
        );
        let _entities = button_scene.spawn_at(coordinate, &mut cmd, &scale_factor);
        *first_run = true;
    }
}
impl Leaf for Tester {
    fn attach(elm: &mut Elm) {
        #[cfg(any(not(target_os = "android"), target_arch = "x86_64"))]
        let android_offset = 0;
        #[cfg(all(target_os = "android", target_arch = "aarch64"))]
        let android_offset = 50;
        elm.add_scene(Button::<TestButton>::new(
            FontSize(14),
            IconId::new(BundledIcon::AtSign),
            Color::OFF_WHITE.into(),
        ));
        elm.job
            .main()
            .add_systems((spawn_button_tree.in_set(SystemSets::Spawn),));
    }
}
