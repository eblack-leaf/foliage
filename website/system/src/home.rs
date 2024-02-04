use crate::{Engen, HOME};
use foliage::asset::AssetKey;
use foliage::color::Color;
use foliage::compositor::layout::Layout;
use foliage::compositor::segment::{ResponsiveSegment, SegmentUnitNumber};
use foliage::coordinate::area::Area;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::{BundleExtend, Elm};
use foliage::icon::FeatherIcon;
use foliage::media::HrefLink;
use foliage::prebuilt::icon_text::{IconText, IconTextArgs};
use foliage::rectangle::Rectangle;
use foliage::text::{GlyphColorChanges, MaxCharacters, TextValue};
use foliage::texture::factors::Progress;
#[foliage::assets(crate::Engen, "../assets/", "/foliage/assets/")]
struct Assets {
    #[icon(path = "icons/terminal.gatl", Terminal)]
    _terminal: AssetKey,
    #[icon(path = "icons/chevrons-right.gatl", ChevronsRight)]
    _chevrons_right: AssetKey,
}
pub(crate) struct Home {}
impl Leaf for Home {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        let _assets = Assets::proc_gen_load(elm);
        elm.add_view_scene_binding::<IconText, _>(
            HOME,
            IconTextArgs::new(
                FeatherIcon::Terminal.id(),
                MaxCharacters(10),
                TextValue::new("foliage.rs"),
                Color::OFF_WHITE,
                Color::GREY_MEDIUM,
            ),
            ResponsiveSegment::new(0.15.relative(), 0.2.relative(), 0.7.relative(), 70.fixed())
                .y_exception(Layout::LANDSCAPE, 0.05.relative())
                .h_exception(Layout::LANDSCAPE_MOBILE, 50.fixed())
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
                0.2.relative().offset(85.0),
                0.6.relative(),
                3.fixed(),
            )
            .x_exception(Layout::LANDSCAPE, 0.3.relative())
            .y_exception(Layout::LANDSCAPE, 0.05.relative().offset(85.0))
            .y_exception(Layout::LANDSCAPE_MOBILE, 0.05.relative().offset(55.0))
            .w_exception(Layout::LANDSCAPE, 0.4.relative())
            .at_layer(0),
            (),
        );
        elm.add_view_scene_binding::<IconText, _>(
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
                0.2.relative().offset(100.0),
                0.8.relative(),
                40.fixed(),
            )
            .x_exception(Layout::PORTRAIT_TABLET, 0.15.relative())
            .x_exception(Layout::LANDSCAPE, 0.3.relative())
            .x_exception(
                [Layout::PORTRAIT_DESKTOP, Layout::PORTRAIT_WORKSTATION],
                0.25.relative(),
            )
            .y_exception(Layout::LANDSCAPE, 0.05.relative().offset(110.0))
            .y_exception(Layout::LANDSCAPE_MOBILE, 0.05.relative().offset(75.0))
            .y_exception(Layout::PORTRAIT_TABLET, 0.2.relative().offset(110.0))
            .w_exception(Layout::LANDSCAPE, 0.45.relative())
            .h_exception(
                [
                    Layout::PORTRAIT_TABLET,
                    Layout::PORTRAIT_DESKTOP,
                    Layout::PORTRAIT_WORKSTATION,
                ],
                25.fixed(),
            )
            .at_layer(1),
            {
                let mut changes = GlyphColorChanges::default();
                changes.0.insert(7, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(8, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(9, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(10, Color::RED_ORANGE_MEDIUM.into());
                changes
            }
            .extend(HrefLink::new("/foliage/book/index.html")),
        );
        elm.add_view_scene_binding::<IconText, _>(
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
                0.2.relative().offset(200.0),
                0.8.relative(),
                40.fixed(),
            )
            .x_exception(Layout::LANDSCAPE, 0.3.relative())
            .x_exception(Layout::PORTRAIT_TABLET, 0.15.relative())
            .x_exception(
                [Layout::PORTRAIT_DESKTOP, Layout::PORTRAIT_WORKSTATION],
                0.25.relative(),
            )
            .y_exception(Layout::LANDSCAPE, 0.05.relative().offset(210.0))
            .y_exception(Layout::LANDSCAPE_MOBILE, 0.05.relative().offset(130.0))
            .y_exception(Layout::PORTRAIT_TABLET, 0.2.relative().offset(210.0))
            .w_exception(Layout::LANDSCAPE, 0.45.relative())
            .h_exception(
                [
                    Layout::PORTRAIT_TABLET,
                    Layout::PORTRAIT_DESKTOP,
                    Layout::PORTRAIT_WORKSTATION,
                ],
                25.fixed(),
            )
            .at_layer(1),
            {
                let mut changes = GlyphColorChanges::default();
                changes.0.insert(14, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(15, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(16, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(17, Color::RED_ORANGE_MEDIUM.into());
                changes
            }
            .extend(HrefLink::new("/foliage/documentation/foliage/index.html")),
        );
        elm.add_view_scene_binding::<IconText, _>(
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
                0.2.relative().offset(300.0),
                0.8.relative(),
                40.fixed(),
            )
            .x_exception(Layout::LANDSCAPE, 0.3.relative())
            .x_exception(
                [Layout::PORTRAIT_DESKTOP, Layout::PORTRAIT_WORKSTATION],
                0.25.relative(),
            )
            .x_exception(Layout::PORTRAIT_TABLET, 0.15.relative())
            .y_exception(Layout::LANDSCAPE, 0.05.relative().offset(310.0))
            .y_exception(Layout::LANDSCAPE_MOBILE, 0.05.relative().offset(190.0))
            .y_exception(Layout::PORTRAIT_TABLET, 0.2.relative().offset(310.0))
            .w_exception(Layout::LANDSCAPE, 0.45.relative())
            .h_exception(
                [
                    Layout::PORTRAIT_TABLET,
                    Layout::PORTRAIT_DESKTOP,
                    Layout::PORTRAIT_WORKSTATION,
                ],
                25.fixed(),
            )
            .at_layer(1),
            {
                let mut changes = GlyphColorChanges::default();
                changes.0.insert(14, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(15, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(16, Color::RED_ORANGE_MEDIUM.into());
                changes.0.insert(17, Color::RED_ORANGE_MEDIUM.into());
                changes
            }
            .extend(HrefLink::new("/foliage/demo/index.html")),
        );
    }
}