use crate::HOME;
use foliage::asset::AssetKey;
use foliage::color::Color;
use foliage::compositor::layout::Layout;
use foliage::compositor::segment::{Grid, ResponsiveSegment, SegmentUnitDesc};
use foliage::coordinate::area::Area;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::{BundleExtend, Elm};
use foliage::icon::FeatherIcon;
use foliage::icon_text::IconText;
use foliage::media::HrefLink;
use foliage::rectangle::Rectangle;
use foliage::text::{GlyphColorChanges, MaxCharacters, TextValue};
use foliage::texture::factors::Progress;
#[foliage::assets(crate::Engen, "../assets/", "/foliage/assets/")]
struct Assets {
    #[icon(path = "icons/terminal.gatl", opt = FeatherIcon::Terminal)]
    _terminal: AssetKey,
    #[icon(path = "icons/chevrons-right.gatl", opt = FeatherIcon::ChevronsRight, group = g)]
    _chevrons_right: AssetKey,
}
pub(crate) struct Home {}
impl Leaf for Home {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        let _assets = Assets::proc_gen_load(elm);
        elm.configure_view_grid(HOME, Grid::new(8, 12));
        elm.add_view_scene_binding(
            HOME,
            IconText::new(
                FeatherIcon::Terminal.id(),
                MaxCharacters(10),
                TextValue::new("foliage.rs"),
                Color::OFF_WHITE,
                Color::GREY_MEDIUM,
            ),
            ResponsiveSegment::base(1.near().to_end(8.far()), 2.near().to_end(80.fixed())),
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
            ResponsiveSegment::base(
                2.near().to_end(7.far()),
                2.near().offset(100.0).to_end(4.fixed()),
            ),
            (),
        );
        elm.add_view_scene_binding(
            HOME,
            IconText::new(
                FeatherIcon::ChevronsRight.id(),
                MaxCharacters(20),
                TextValue::new("ls -la BOOK"),
                Color::OFF_WHITE,
                Color::GREY_MEDIUM,
            ),
            ResponsiveSegment::base(1.near().to_end(8.far()), 5.near().to_end(60.fixed()))
                .horizontal_exception(
                    [Layout::PORTRAIT_MOBILE],
                    1.near().offset(-50.0).to_end(8.far().offset(75.0)),
                ),
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
        elm.add_view_scene_binding(
            HOME,
            IconText::new(
                FeatherIcon::ChevronsRight.id(),
                MaxCharacters(20),
                TextValue::new("grep answer | DOCS"),
                Color::OFF_WHITE,
                Color::GREY_MEDIUM,
            ),
            ResponsiveSegment::base(1.near().to_end(8.far()), 7.near().to_end(60.fixed()))
                .horizontal_exception(
                    [Layout::PORTRAIT_MOBILE],
                    1.near().offset(-50.0).to_end(8.far().offset(75.0)),
                ),
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
        elm.add_view_scene_binding(
            HOME,
            IconText::new(
                FeatherIcon::ChevronsRight.id(),
                MaxCharacters(20),
                TextValue::new("chmod+x -wasm DEMO"),
                Color::OFF_WHITE,
                Color::GREY_MEDIUM,
            ),
            ResponsiveSegment::base(1.near().to_end(8.far()), 9.near().to_end(60.fixed()))
                .horizontal_exception(
                    [Layout::PORTRAIT_MOBILE],
                    1.near().offset(-50.0).to_end(8.far().offset(75.0)),
                ),
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
