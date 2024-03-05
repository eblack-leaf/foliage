use crate::ThemeColor;
use foliage::color::monochromatic::Monochromatic;
use foliage::color::Color;
use foliage::elm::Style;
use foliage::r_scenes::dropdown::{Dropdown, DropdownOptions, ExpandDirection};
use foliage::r_scenes::Colors;
use foliage::segment::{Justify, MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::view::{ViewBuilder, ViewDescriptor, Viewable};

pub struct Overlay;
impl Viewable for Overlay {
    const GRID: MacroGrid = MacroGrid::new(8, 8);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        view_builder.add_scene(
            Dropdown::new(
                DropdownOptions::new(["one", "two", "three", "four", "five", "six", "seven"]),
                ExpandDirection::Down,
                Colors::new(ThemeColor::MINUS_ONE, Color::DARK_GREY),
            ),
            ResponsiveSegment::base(Segment::new(
                3.near().to(6.far()).maximum(250.0),
                1.near().to(1.far()).maximum(50.0),
            ))
            .justify(Justify::Top),
        );
        view_builder.finish()
    }
}