use foliage::circle_button::CircleButton;
use foliage::color::monochromatic::{
    FluorescentYellow, Greyscale, Magenta, Monochromatic, Orange, StrongCyan,
};
use foliage::color::Color;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::{BundleExtend, Elm, Style};
use foliage::icon::FeatherIcon;
use foliage::icon_text::{IconText, IconTextBindings};
use foliage::media::{Href, HrefLink};
use foliage::rectangle::Rectangle;
use foliage::segment::{MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::text::{TextColorExceptions, TextLineStructure, TextValue};
use foliage::texture::factors::Progress;
use foliage::view::{ViewBuilder, ViewDescriptor, ViewHandle, Viewable};
use foliage::Colors;

#[foliage::assets(crate::Engen, "../assets/", "/foliage/assets/")]
struct Assets {
    #[icon(path = "icons/terminal.icon", opt = FeatherIcon::Terminal)]
    _terminal: AssetKey,
    #[icon(path = "icons/chevrons-right.icon", opt = FeatherIcon::ChevronsRight, group = g)]
    _chevrons_right: AssetKey,
    #[icon(path = "icons/github.icon", opt = FeatherIcon::Github, group = g)]
    _chevrons_right: AssetKey,
}
pub(crate) struct Home {}
impl Leaf for Home {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        let _assets = Assets::proc_gen_load(elm);
        elm.add_view::<Home>(ViewHandle(0));
        elm.navigate_to(ViewHandle(0));
    }
}
impl Viewable for Home {
    const GRID: MacroGrid = MacroGrid::new(6, 6);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        let a = view_builder.add_scene(
            CircleButton::new(
                FeatherIcon::Github,
                Style::fill(),
                Colors::new(Greyscale::MINUS_THREE, Orange::BASE),
            ),
            ResponsiveSegment::base(Segment::new(
                6.near().to(6.far()).fixed(40.0),
                1.near().to(1.far()).fixed(40.0),
            )),
        );
        view_builder.extend(
            a.bindings().get(2),
            Href::new("https://github.com/eblack-leaf/foliage", true),
        );
        let b = view_builder.add_scene(
            IconText::new(
                FeatherIcon::Terminal.id(),
                Color::WHITE,
                TextLineStructure::new(10, 1),
                TextValue::new("foliage.rs"),
                Greyscale::BASE,
            ),
            ResponsiveSegment::base(Segment::new(2.near().to(5.far()), 1.near().to(2.near()))),
        );
        view_builder.extend(
            b.bindings().get(IconTextBindings::Text),
            TextColorExceptions::blank().with_range(7, 9, Orange::MINUS_ONE),
        );
        view_builder.add(
            Rectangle::new(Color::WHITE, Progress::full()),
            ResponsiveSegment::base(Segment::new(
                2.near().to(5.far()),
                2.near().offset(15.0).to(4.absolute()),
            )),
        );
        let c = view_builder.add_scene(
            IconText::new(
                FeatherIcon::ChevronsRight.id(),
                Color::WHITE,
                TextLineStructure::new(20, 1),
                TextValue::new("ls -la BOOK [arch]"),
                Greyscale::BASE,
            ),
            ResponsiveSegment::base(Segment::new(2.near().to(5.far()), 3.near().to(3.far()))),
        );
        view_builder.extend(
            c.bindings().get(IconTextBindings::Text),
            TextColorExceptions::blank()
                .with_range(7, 10, StrongCyan::MINUS_ONE)
                .extend(HrefLink::relative("/foliage/book/index.html")),
        );
        let d = view_builder.add_scene(
            IconText::new(
                FeatherIcon::ChevronsRight.id(),
                Color::WHITE,
                TextLineStructure::new(20, 1),
                TextValue::new("grep answer | DOCS"),
                Greyscale::BASE,
            ),
            ResponsiveSegment::base(Segment::new(2.near().to(5.far()), 4.near().to(4.far()))),
        );
        view_builder.extend(
            d.bindings().get(IconTextBindings::Text),
            TextColorExceptions::blank()
                .with_range(14, 17, Magenta::MINUS_ONE)
                .extend(HrefLink::relative(
                    "/foliage/documentation/foliage/index.html",
                )),
        );
        let e = view_builder.add_scene(
            IconText::new(
                FeatherIcon::ChevronsRight.id(),
                Color::WHITE,
                TextLineStructure::new(20, 1),
                TextValue::new("chmod+x -wasm DEMO"),
                Greyscale::BASE,
            ),
            ResponsiveSegment::base(Segment::new(2.near().to(5.far()), 5.near().to(5.far()))),
        );
        view_builder.extend(
            e.bindings().get(IconTextBindings::Text),
            TextColorExceptions::blank()
                .with_range(14, 17, FluorescentYellow::MINUS_ONE)
                .extend(HrefLink::relative("/foliage/demo/index.html")),
        );
        view_builder.finish()
    }
}