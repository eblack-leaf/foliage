use foliage::circle::{Circle, CircleMipLevel, CircleStyle, Diameter, Progress};
use foliage::color::Color;
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
        .with_leaf::<Tester>()
        .with_android_interface(android_interface)
        .run();
}

struct Tester;

impl Leaf for Tester {
    fn attach(elm: &mut Elm) {
        elm.job.container.spawn(Panel::new(
            PanelStyle::flat(),
            (52, 10).into(),
            (50, 25).into(),
            1.into(),
            Color::BLUE.into(),
        ));
        elm.job.container.spawn(Panel::new(
            PanelStyle::flat(),
            (112, 10).into(),
            (100, 50).into(),
            4.into(),
            Color::BLUE.into(),
        ));
        elm.job.container.spawn(Panel::new(
            PanelStyle::flat(),
            (222, 10).into(),
            (200, 100).into(),
            4.into(),
            Color::BLUE.into(),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (10, 10).into(),
            Diameter::from_mip_level(CircleMipLevel::Five),
            3.into(),
            Color::BLUE.into(),
            Progress::new(0.2),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (10, 52).into(),
            Diameter::from_mip_level(CircleMipLevel::Four),
            3.into(),
            Color::BLUE.into(),
            Progress::new(0.4),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (10, 126).into(),
            Diameter::from_mip_level(CircleMipLevel::Three),
            3.into(),
            Color::BLUE.into(),
            Progress::new(0.6),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (10, 264).into(),
            Diameter::from_mip_level(CircleMipLevel::Two),
            3.into(),
            Color::BLUE.into(),
            Progress::new(0.8),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (10, 530).into(),
            Diameter::from_mip_level(CircleMipLevel::One),
            3.into(),
            Color::BLUE.into(),
            Progress::new(1.0),
        ));
    }
}
