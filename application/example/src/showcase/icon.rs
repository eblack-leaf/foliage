use foliage::color::monochromatic::{Magenta, Monochromatic};
use foliage::color::Color;
use foliage::icon::{FeatherIcon, Icon};
use foliage::r_scenes::icon_text::IconText;
use foliage::segment::{MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::text::{MaxCharacters, Text, TextValue};
use foliage::view::{ViewBuilder, ViewDescriptor, Viewable};

pub struct IconShowcase;
impl Viewable for IconShowcase {
    const GRID: MacroGrid = MacroGrid::new(8, 5);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        view_builder.add(
            Icon::new(FeatherIcon::Command, Magenta::BASE),
            ResponsiveSegment::base(Segment::new(
                2.near().to(5.far()).fixed(40.0),
                2.near().to(2.far()).fixed(40.0),
            ))
            .at_layer(6),
        );
        view_builder.add(
            Text::new(MaxCharacters(4), TextValue::new("icon"), Color::GREY),
            ResponsiveSegment::base(Segment::new(
                6.near().to(7.far()),
                2.near().to(2.far()).maximum(50.0),
            ))
            .at_layer(5),
        );
        view_builder.add_scene(
            IconText::new(
                FeatherIcon::Command,
                Magenta::BASE,
                MaxCharacters(6),
                TextValue::new("action"),
                Magenta::BASE,
            ),
            ResponsiveSegment::base(Segment::new(
                2.near().to(5.far()),
                3.near().to(3.far()).maximum(50.0),
            ))
            .at_layer(5),
        );
        view_builder.add(
            Text::new(MaxCharacters(4), TextValue::new("text"), Color::GREY),
            ResponsiveSegment::base(Segment::new(
                6.near().to(7.far()),
                3.near().to(3.far()).maximum(50.0),
            ))
            .at_layer(5),
        );
        view_builder.finish()
    }
}
