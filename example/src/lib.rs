use foliage::circle::{Circle, CircleStyle, Radius};
use foliage::color::Color;
use foliage::elm::{Elm, Leaf};
use foliage::panel::Panel;
use foliage::window::WindowDescriptor;
use foliage::{AndroidInterface, Foliage};

pub fn entry(android_interface: AndroidInterface) {
    Foliage::new()
        .with_window_descriptor(
            WindowDescriptor::new()
                .with_title("foliage")
                .with_desktop_dimensions((420, 820)),
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
            CircleStyle::flat(),
            (10, 10).into(),
            Radius::new(100f32),
            5.into(),
            Color::OFF_WHITE.into(),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::flat(),
            (10, 420).into(),
            Radius::new(800f32),
            5.into(),
            Color::OFF_WHITE.into(),
        ));
    }
}
