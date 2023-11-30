use foliage::circle::{Circle, CircleMipLevel, CircleStyle, Diameter};
use foliage::color::Color;
use foliage::coordinate::Coordinate;
use foliage::elm::{Elm, Leaf};
use foliage::panel::{Panel, PanelStyle};
use foliage::rectangle::{Rectangle, RectangleStyle};
use foliage::texture::Progress;
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
        .with_renderleaf::<Rectangle>()
        .with_leaf::<Coordinate>()
        .with_leaf::<Tester>()
        .with_android_interface(android_interface)
        .run();
}

struct Tester;

impl Leaf for Tester {
    fn attach(elm: &mut Elm) {
        #[cfg(not(target_os = "android"))]
        let android_offset = 0;
        #[cfg(target_os = "android")]
        let android_offset = 50;
        elm.job.container.spawn(Rectangle::new(
            RectangleStyle::fill(),
            (158, 512 + android_offset).into(),
            (200, 10).into(),
            6.into(),
            Color::GREY_DARK.into(),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Rectangle::new(
            RectangleStyle::fill(),
            (158, 512 + android_offset).into(),
            (200, 10).into(),
            5.into(),
            Color::BLUE.into(),
            Progress::new(0.0, 0.85),
        ));
        elm.job.container.spawn(Rectangle::new(
            RectangleStyle::fill(),
            (158, 536 + android_offset).into(),
            (200, 10).into(),
            6.into(),
            Color::GREY_DARK.into(),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Rectangle::new(
            RectangleStyle::fill(),
            (158, 536 + android_offset).into(),
            (200, 10).into(),
            5.into(),
            Color::from_rgb(0.6, 0.4, 0.2).into(),
            Progress::new(0.0, 0.65),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (29, 210 + android_offset).into(),
            Diameter::from_mip_level(CircleMipLevel::SixtyFour),
            6.into(),
            Color::from(Color::GREY_DARK).with_alpha(1.0),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (29, 210 + android_offset).into(),
            Diameter::from_mip_level(CircleMipLevel::SixtyFour),
            5.into(),
            Color::from(Color::from_rgb(0.6, 0.4, 0.2)).with_alpha(1.0),
            Progress::new(0.0, 0.3),
        ));
        elm.job.container.spawn(Rectangle::new(
            RectangleStyle::fill(),
            (113, 178 + android_offset).into(),
            (2, 128).into(),
            6.into(),
            Color::GREY_DARK.into(),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (29, 500 + android_offset).into(),
            Diameter::from_mip_level(CircleMipLevel::SixtyFour),
            6.into(),
            Color::from(Color::GREY_DARK).with_alpha(1.0),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (29, 500 + android_offset).into(),
            Diameter::from_mip_level(CircleMipLevel::SixtyFour),
            5.into(),
            Color::from(Color::CYAN).with_alpha(1.0),
            Progress::new(0.0, 0.85),
        ));
        elm.job.container.spawn(Rectangle::new(
            RectangleStyle::fill(),
            (113, 468 + android_offset).into(),
            (2, 128).into(),
            6.into(),
            Color::GREY_DARK.into(),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::fill(),
            (138, 512 + android_offset).into(),
            Diameter::from_mip_level(CircleMipLevel::Eight),
            5.into(),
            Color::from(Color::BLUE_MEDIUM).with_alpha(1.0),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::fill(),
            (138, 536 + android_offset).into(),
            Diameter::from_mip_level(CircleMipLevel::Eight),
            5.into(),
            Color::from(Color::from_rgb(0.6, 0.4, 0.2)).with_alpha(1.0),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Panel::new(
            PanelStyle::fill(),
            (138, 210 + android_offset).into(),
            (140, 50).into(),
            4.into(),
            Color::GREEN.into(),
        ));
        elm.job.container.spawn(Rectangle::new(
            RectangleStyle::fill(),
            (148, 280 + android_offset).into(),
            (120, 8).into(),
            7.into(),
            Color::GREY_DARK.into(),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Rectangle::new(
            RectangleStyle::fill(),
            (148, 280 + android_offset).into(),
            (120, 8).into(),
            6.into(),
            Color::GREEN_DARK.into(),
            Progress::new(0.0, 0.25),
        ));
        elm.job.container.spawn(Rectangle::new(
            RectangleStyle::fill(),
            (178, 276 + android_offset).into(),
            (16, 16).into(),
            5.into(),
            Color::from(Color::GREEN).with_alpha(1.0),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Panel::new(
            PanelStyle::fill(),
            (10, 10 + android_offset).into(),
            (80, 12).into(),
            3.into(),
            Color::OFF_WHITE.into(),
            // Progress::new(0.0, 0.65),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::fill(),
            (100, 12 + android_offset).into(),
            Diameter::from_mip_level(CircleMipLevel::Eight),
            4.into(),
            Color::GREY_DARK.into(),
            Progress::new(0.0, 1.0),
        ));
    }
}
