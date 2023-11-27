use foliage::circle::{Circle, CircleMipLevel, CircleStyle, Diameter};
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
        elm.job.container.spawn(Panel::new(
            PanelStyle::flat(),
            (125, 100).into(),
            (200, 100).into(),
            1.into(),
            Color::RED_ORANGE_MEDIUM.into(),
        ));
        elm.job.container.spawn(Panel::new(
            PanelStyle::ring(),
            (50, 300).into(),
            (200, 100).into(),
            4.into(),
            Color::RED_ORANGE.into(),
        ));
        elm.job.container.spawn(Panel::new(
            PanelStyle::ring(),
            (175, 500).into(),
            (200, 100).into(),
            4.into(),
            Color::RED_ORANGE_MEDIUM.into(),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (10, 10).into(),
            Diameter::from_mip_level(CircleMipLevel::Four),
            3.into(),
            Color::GREEN_MEDIUM.into(),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (10, 10).into(),
            Diameter::from_mip_level(CircleMipLevel::Three),
            3.into(),
            Color::BLUE_MEDIUM.into(),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (10, 500).into(),
            Diameter::from_mip_level(CircleMipLevel::Two),
            3.into(),
            Color::BLUE.into(),
        ));
    }
}
