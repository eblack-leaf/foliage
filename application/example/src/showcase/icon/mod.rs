use foliage::color::monochromatic::{Asparagus, Monochromatic};
use foliage::color::Color;
use foliage::icon::FeatherIcon;
use foliage::r_scenes::paged::Paged;
use foliage::r_scenes::{Colors, Direction};
use foliage::segment::{MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::view::{ViewBuilder, ViewDescriptor, Viewable};

use crate::showcase::icon::scene::IconDisplay;

pub mod scene;

pub struct IconShowcase;
impl Viewable for IconShowcase {
    const GRID: MacroGrid = MacroGrid::new(8, 5);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        // view_builder.add(
        //     Icon::new(FeatherIcon::Command, Magenta::BASE),
        //     ResponsiveSegment::base(Segment::new(
        //         2.near().to(5.far()).fixed(40.0),
        //         2.near().to(2.far()).fixed(40.0),
        //     ))
        //     .at_layer(6),
        // );
        // view_builder.add(
        //     Text::new(MaxCharacters(4), TextValue::new("icon"), Color::GREY),
        //     ResponsiveSegment::base(Segment::new(
        //         6.near().to(7.far()),
        //         2.near().to(2.far()).maximum(50.0),
        //     ))
        //     .at_layer(5),
        // );
        // view_builder.add_scene(
        //     IconText::new(
        //         FeatherIcon::Command,
        //         Magenta::BASE,
        //         MaxCharacters(6),
        //         TextValue::new("action"),
        //         Magenta::BASE,
        //     ),
        //     ResponsiveSegment::base(Segment::new(
        //         2.near().to(5.far()),
        //         3.near().to(3.far()).maximum(50.0),
        //     ))
        //     .at_layer(5),
        // );
        // view_builder.add(
        //     Text::new(MaxCharacters(4), TextValue::new("text"), Color::GREY),
        //     ResponsiveSegment::base(Segment::new(
        //         6.near().to(7.far()),
        //         3.near().to(3.far()).maximum(50.0),
        //     ))
        //     .at_layer(5),
        // );
        let mut icon_groupings = vec![];
        for i in (0..286).step_by(9) {
            icon_groupings.push(IconDisplay::new(
                (i..i + 9)
                    .map(|a| FeatherIcon::from(a as u32))
                    .collect::<Vec<FeatherIcon>>(),
            ));
        }
        view_builder.apply(Paged::new(
            icon_groupings,
            Colors::new(Asparagus::BASE, Color::DARK_GREY),
            Direction::Horizontal,
            ResponsiveSegment::base(Segment::new(1.near().to(8.far()), 2.near().to(5.far())))
                .at_layer(5),
            FeatherIcon::ChevronLeft,
            FeatherIcon::ChevronRight,
        ));
        view_builder.finish()
    }
}