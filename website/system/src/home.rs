use crate::{Engen, HOME};
use foliage::color::Color;
use foliage::compositor::segment::{ResponsiveSegment, SegmentUnitNumber};
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use foliage::icon::FeatherIcon;
use foliage::icon_fetcher;
use foliage::prebuilt::icon_text::{IconText, IconTextArgs};
use foliage::text::{FontSize, MaxCharacters, Text, TextValue};

pub(crate) struct Home {}
impl Leaf for Home {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        elm.load_remote_icon::<Engen>(
            icon_fetcher!(FeatherIcon::Terminal),
            "/foliage/demo/assets/icons/terminal.gatl",
        );
        elm.add_view_scene_binding::<IconText, ()>(
            HOME,
            IconTextArgs::new(
                FeatherIcon::Terminal.id(),
                MaxCharacters(10),
                TextValue::new("foliage.rs"),
                Color::OFF_WHITE,
                Color::GREY_MEDIUM,
            ),
            ResponsiveSegment::new(0.2.relative(), 0.2.relative(), 0.6.relative(), 50.fixed()),
            (),
        );
        // elm.add_view_binding(
        //     HOME,
        //     Text::new(
        //         MaxCharacters(11),
        //         FontSize(14),
        //         TextValue::new("Next-gen:UI"),
        //         Color::GREY_MEDIUM,
        //     ),
        //     ResponsiveSegment::new(
        //         0.35.relative(),
        //         0.2.relative().offset(145.0),
        //         0.6.relative(),
        //         50.fixed()
        //     ),
        //     (),
        // );
    }
}
