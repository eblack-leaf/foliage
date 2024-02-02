use crate::{Engen, HOME};
use foliage::color::Color;
use foliage::compositor::layout::Layout;
use foliage::compositor::segment::{ResponsiveSegment, SegmentUnitNumber};
use foliage::coordinate::area::Area;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use foliage::icon::FeatherIcon;
use foliage::icon_fetcher;
use foliage::prebuilt::icon_text::{IconText, IconTextArgs};
use foliage::rectangle::Rectangle;
use foliage::text::{GlyphColorChanges, MaxCharacters, TextValue};
use foliage::texture::factors::Progress;

pub(crate) struct Home {}
impl Leaf for Home {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        elm.load_remote_icon::<Engen>(
            icon_fetcher!(FeatherIcon::Terminal),
            "/foliage/assets/icons/terminal.gatl",
        );
        elm.load_remote_icon::<Engen>(
            icon_fetcher!(FeatherIcon::Hash),
            "/foliage/assets/icons/hash.gatl",
        );
        elm.load_remote_icon::<Engen>(
            icon_fetcher!(FeatherIcon::ChevronRight),
            "/foliage/assets/icons/chevron-right.gatl",
        );
        elm.load_remote_icon::<Engen>(
            icon_fetcher!(FeatherIcon::ChevronsRight),
            "/foliage/assets/icons/chevrons-right.gatl",
        );
        elm.load_remote_icon::<Engen>(
            icon_fetcher!(FeatherIcon::MoreVertical),
            "/foliage/assets/icons/more-vertical.gatl",
        );
        elm.add_view_scene_binding::<IconText, GlyphColorChanges>(
            HOME,
            IconTextArgs::new(
                FeatherIcon::Terminal.id(),
                MaxCharacters(10),
                TextValue::new("foliage.rs"),
                Color::OFF_WHITE,
                Color::GREY_MEDIUM,
            ),
            ResponsiveSegment::new(0.15.relative(), 0.2.relative(), 0.7.relative(), 50.fixed())
                .at_layer(1),
            {
                let mut changes = GlyphColorChanges::default();
                changes.0.insert(7, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(8, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(9, Color::RED_ORANGE_MEDIUM.into());
                changes
            },
        );
        elm.add_view_binding(
            HOME,
            Rectangle::new(Area::default(), Color::OFF_WHITE, Progress::full()),
            ResponsiveSegment::new(
                0.2.relative(),
                0.2.relative().offset(65.0),
                0.6.relative(),
                3.fixed(),
            )
            .w_exception(Layout::LANDSCAPE, 0.4.relative())
            .x_exception(Layout::LANDSCAPE, 0.3.relative()),
            (),
        );
        elm.add_view_scene_binding::<IconText, GlyphColorChanges>(
            HOME,
            IconTextArgs::new(
                FeatherIcon::ChevronsRight.id(),
                MaxCharacters(20),
                TextValue::new("ls -la BOOK"),
                Color::OFF_WHITE,
                Color::GREY_MEDIUM,
            ),
            ResponsiveSegment::new(
                0.1.relative(),
                0.2.relative().offset(80.0),
                0.8.relative(),
                40.fixed(),
            )
            .at_layer(1)
            .h_exception(Layout::LANDSCAPE, 25.fixed())
            .y_exception(Layout::LANDSCAPE, 0.2.relative().offset(90.0))
            .x_exception(Layout::LANDSCAPE, 0.3.relative())
            .w_exception(Layout::LANDSCAPE, 0.45.relative())
            .x_exception(Layout::PORTRAIT_TABLET, 0.15.relative())
            .y_exception(Layout::PORTRAIT_TABLET, 0.2.relative().offset(90.0))
            .x_exception(
                [Layout::PORTRAIT_DESKTOP, Layout::PORTRAIT_WORKSTATION],
                0.25.relative(),
            )
            .h_exception(
                [
                    Layout::PORTRAIT_TABLET,
                    Layout::PORTRAIT_DESKTOP,
                    Layout::PORTRAIT_WORKSTATION,
                ],
                25.fixed(),
            ),
            {
                let mut changes = GlyphColorChanges::default();
                changes.0.insert(7, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(8, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(9, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(10, Color::RED_ORANGE_MEDIUM.into());
                changes
            },
        );
        elm.add_view_scene_binding::<IconText, GlyphColorChanges>(
            HOME,
            IconTextArgs::new(
                FeatherIcon::ChevronsRight.id(),
                MaxCharacters(20),
                TextValue::new("grep answer | DOCS"),
                Color::OFF_WHITE,
                Color::GREY_MEDIUM,
            ),
            ResponsiveSegment::new(
                0.1.relative(),
                0.2.relative().offset(110.0),
                0.8.relative(),
                40.fixed(),
            )
            .at_layer(1)
            .h_exception(Layout::LANDSCAPE, 25.fixed())
            .y_exception(Layout::LANDSCAPE, 0.2.relative().offset(120.0))
            .x_exception(Layout::LANDSCAPE, 0.3.relative())
            .w_exception(Layout::LANDSCAPE, 0.45.relative())
            .x_exception(Layout::PORTRAIT_TABLET, 0.15.relative())
            .y_exception(Layout::PORTRAIT_TABLET, 0.2.relative().offset(120.0))
            .x_exception(
                [Layout::PORTRAIT_DESKTOP, Layout::PORTRAIT_WORKSTATION],
                0.25.relative(),
            )
            .h_exception(
                [
                    Layout::PORTRAIT_TABLET,
                    Layout::PORTRAIT_DESKTOP,
                    Layout::PORTRAIT_WORKSTATION,
                ],
                25.fixed(),
            ),
            {
                let mut changes = GlyphColorChanges::default();
                changes.0.insert(14, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(15, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(16, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(17, Color::RED_ORANGE_MEDIUM.into());
                changes
            },
        );
        elm.add_view_scene_binding::<IconText, GlyphColorChanges>(
            HOME,
            IconTextArgs::new(
                FeatherIcon::ChevronsRight.id(),
                MaxCharacters(20),
                TextValue::new("chmod+x -wasm DEMO"),
                Color::OFF_WHITE,
                Color::GREY_MEDIUM,
            ),
            ResponsiveSegment::new(
                0.1.relative(),
                0.2.relative().offset(140.0),
                0.8.relative(),
                40.fixed(),
            )
            .at_layer(1)
            .h_exception(Layout::LANDSCAPE, 25.fixed())
            .y_exception(Layout::LANDSCAPE, 0.2.relative().offset(150.0))
            .x_exception(Layout::LANDSCAPE, 0.3.relative())
            .w_exception(Layout::LANDSCAPE, 0.45.relative())
            .y_exception(Layout::PORTRAIT_TABLET, 0.2.relative().offset(150.0))
            .x_exception(
                [Layout::PORTRAIT_DESKTOP, Layout::PORTRAIT_WORKSTATION],
                0.25.relative(),
            )
            .x_exception(Layout::PORTRAIT_TABLET, 0.15.relative())
            .h_exception(
                [
                    Layout::PORTRAIT_TABLET,
                    Layout::PORTRAIT_DESKTOP,
                    Layout::PORTRAIT_WORKSTATION,
                ],
                25.fixed(),
            ),
            {
                let mut changes = GlyphColorChanges::default();
                changes.0.insert(14, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(15, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(16, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(17, Color::RED_ORANGE_MEDIUM.into());
                changes
            },
        );
    }
}