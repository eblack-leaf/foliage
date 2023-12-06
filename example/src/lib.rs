use foliage::bevy_ecs::change_detection::Res;
use foliage::bevy_ecs::prelude::{Commands, IntoSystemConfigs};
use foliage::bevy_ecs::system::Local;
use foliage::button::Button;
use foliage::circle::Circle;
use foliage::coordinate::area::Area;
use foliage::coordinate::position::Position;
use foliage::coordinate::section::Section;
use foliage::coordinate::{Coordinate, CoordinateLeaf, InterfaceContext};
use foliage::elm::{Elm, Leaf, SystemSets};
use foliage::icon::Icon;
use foliage::panel::Panel;
use foliage::rectangle::Rectangle;
use foliage::text::font::MonospacedFont;
use foliage::text::Text;
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
fn spawn_button_tree(
    mut first_run: Local<bool>,
    mut cmd: Commands,
    scale_factor: Res<ScaleFactor>,
    font: Res<MonospacedFont>,
) {
    if !*first_run {
        let coordinate = Coordinate::new(
            Section::new(
                Position::<InterfaceContext>::new(10.0, 10.0),
                Area::new(140.0, 50.0),
            ),
            4,
        );
        *first_run = true;
    }
}
impl Leaf for Tester {
    fn attach(elm: &mut Elm) {
        #[cfg(any(not(target_os = "android"), target_arch = "x86_64"))]
        let android_offset = 0;
        #[cfg(all(target_os = "android", target_arch = "aarch64"))]
        let android_offset = 50;
        elm.job
            .main()
            .add_systems((spawn_button_tree.in_set(SystemSets::Spawn),));
    }
}
