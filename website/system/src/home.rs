use crate::HOME;
use foliage::asset::AssetKey;
use foliage::color::Color;
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
    #[icon(path = "icons/terminal.icon", opt = FeatherIcon::Terminal)]
    _terminal: AssetKey,
    #[icon(path = "icons/chevrons-right.icon", opt = FeatherIcon::ChevronsRight, group = g)]
    _chevrons_right: AssetKey,
}
pub(crate) struct Home {}
impl Leaf for Home {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        let _assets = Assets::proc_gen_load(elm);
        elm.configure_view_grid(HOME, Grid::new(6, 6));
        elm.add_view_scene_binding(
            HOME,
            IconText::new(
                FeatherIcon::Terminal.id(),
                MaxCharacters(10),
                TextValue::new("foliage.rs"),
                Color::OFF_WHITE,
                Color::GREY_MEDIUM,
            ),
            ResponsiveSegment::base(
                2.near().to(5.far()).minimum(200.0).maximum(500.0),
                1.near().to(2.near()).minimum(50.0).maximum(80.0),
            ),
            GlyphColorChanges::new().with_range(7, 9, Color::RED_ORANGE_MEDIUM),
        );
        elm.add_view_binding(
            HOME,
            Rectangle::new(Area::default(), Color::OFF_WHITE, Progress::full()),
            ResponsiveSegment::base(2.near().to(5.far()), 2.near().offset(25.0).to(4.fixed())),
            (),
        );
        elm.add_view_scene_binding(
            HOME,
            IconText::new(
                FeatherIcon::ChevronsRight.id(),
                MaxCharacters(20),
                TextValue::new("ls -la BOOK [arch]"),
                Color::OFF_WHITE,
                Color::GREY_MEDIUM,
            ),
            ResponsiveSegment::base(
                2.near().to(5.far()).minimum(300.0).maximum(600.0),
                3.near().to(3.far()).minimum(40.0).maximum(60.0),
            ),
            GlyphColorChanges::new()
                .with_range(7, 10, Color::RED_ORANGE_MEDIUM)
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
            ResponsiveSegment::base(
                2.near().to(5.far()).minimum(300.0).maximum(600.0),
                4.near().to(4.far()).minimum(40.0).maximum(60.0),
            ),
            GlyphColorChanges::new()
                .with_range(14, 17, Color::RED_ORANGE_MEDIUM)
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
            ResponsiveSegment::base(
                2.near().to(5.far()).minimum(300.0).maximum(600.0),
                5.near().to(5.far()).minimum(40.0).maximum(60.0),
            ),
            GlyphColorChanges::new()
                .with_range(14, 17, Color::RED_ORANGE_MEDIUM)
                .extend(HrefLink::new("/foliage/demo/index.html")),
        );
    }
}
