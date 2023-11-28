use foliage::circle::{Circle, CircleMipLevel, CircleStyle, Diameter, Progress};
use foliage::color::Color;
use foliage::elm::{Elm, Leaf};
use foliage::panel::{Panel};
use foliage::window::WindowDescriptor;
use foliage::{AndroidInterface, Foliage};

pub fn entry(android_interface: AndroidInterface) {
    Foliage::new()
        .with_window_descriptor(
            WindowDescriptor::new()
                .with_title("foliage")
                .with_desktop_dimensions((434, 840)),
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
            (79, 292).into(),
            Diameter::from_mip_level(CircleMipLevel::Two),
            6.into(),
            Color::BLUE.into(),
            Progress::new(0.1, 0.3),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (79, 292).into(),
            Diameter::from_mip_level(CircleMipLevel::Two),
            5.into(),
            Color::GREEN.into(),
            Progress::new(0.3, 0.5),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (79, 292).into(),
            Diameter::from_mip_level(CircleMipLevel::Two),
            4.into(),
            Color::RED.into(),
            Progress::new(0.5, 0.7),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (79, 292).into(),
            Diameter::from_mip_level(CircleMipLevel::Two),
            3.into(),
            Color::GREY.into(),
            Progress::new(0.7, 0.9),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (79, 292).into(),
            Diameter::from_mip_level(CircleMipLevel::Two),
            2.into(),
            Color::OFF_WHITE.into(),
            Progress::new(0.9, 1.0),
        ));
    }
}