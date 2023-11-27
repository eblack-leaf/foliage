use foliage::{AndroidInterface, Foliage};
use foliage::circle::{Circle, CircleStyle, Diameter};
use foliage::color::Color;
use foliage::elm::{Elm, Leaf};
use foliage::panel::Panel;
use foliage::window::WindowDescriptor;

pub fn entry(android_interface: AndroidInterface) {
    Foliage::new()
        .with_window_descriptor(
            WindowDescriptor::new()
                .with_title("foliage")
                .with_desktop_dimensions((420, 840)),
        )
        .with_renderleaf::<Panel>()
        .with_renderleaf::<Circle>()
        .with_leaf::<Tester>()
        .with_android_interface(android_interface)
        .run();
}

struct Tester;

impl Leaf for Tester {
    fn attach(elm: &mut Elm) {
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (166, 166).into(),
            Diameter::new(256f32),
            5.into(),
            Color::OFF_WHITE.into(),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (10, 10).into(),
            Diameter::new(512f32),
            5.into(),
            Color::OFF_WHITE.into(),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (10, 500).into(),
            Diameter::new(1024f32),
            5.into(),
            Color::OFF_WHITE.into(),
        ));
    }
}