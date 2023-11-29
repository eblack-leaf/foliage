use foliage::circle::{Circle, CircleMipLevel, CircleStyle, Diameter, Progress};
use foliage::color::Color;
use foliage::coordinate::Coordinate;
use foliage::elm::{Elm, Leaf};
use foliage::panel::{Panel, PanelStyle};
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
        .with_leaf::<Coordinate>()
        .with_leaf::<Tester>()
        .with_android_interface(android_interface)
        .run();
}

struct Tester;

impl Leaf for Tester {
    fn attach(elm: &mut Elm) {
        elm.job.container.spawn(Panel::new(
            PanelStyle::fill(),
            (47, 630).into(),
            (140, 50).into(),
            4.into(),
            Color::GREY_MEDIUM.into(),
        ));
        elm.job.container.spawn(Panel::new(
            PanelStyle::fill(),
            (247, 630).into(),
            (140, 50).into(),
            4.into(),
            Color::OFF_WHITE.into(),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (89, 210).into(),
            Diameter::from_mip_level(CircleMipLevel::Two),
            6.into(),
            Color::from(Color::GREY_DARK).with_alpha(1.0),
            Progress::new(0.1, 0.3),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (89, 210).into(),
            Diameter::from_mip_level(CircleMipLevel::Two),
            5.into(),
            Color::GREY_MEDIUM.into(),
            Progress::new(0.3, 0.5),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (89, 210).into(),
            Diameter::from_mip_level(CircleMipLevel::Two),
            4.into(),
            Color::GREY.into(),
            Progress::new(0.5, 0.7),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (89, 210).into(),
            Diameter::from_mip_level(CircleMipLevel::Two),
            3.into(),
            Color::OFF_WHITE.into(),
            Progress::new(0.7, 0.9),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (89, 210).into(),
            Diameter::from_mip_level(CircleMipLevel::Two),
            2.into(),
            Color::WHITE.into(),
            Progress::new(0.9, 1.0),
        ));
    }
}