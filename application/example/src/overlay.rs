use crate::ThemeColor;
use foliage::color::monochromatic::Monochromatic;
use foliage::color::Color;
use foliage::elm::ElementStyle;
use foliage::r_scenes::dropdown::scene::ExpandDirection;
use foliage::r_scenes::dropdown::Dropdown;
use foliage::r_scenes::UIColor;
use foliage::segment::{Justify, MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::view::{ViewBuilder, ViewDescriptor, Viewable};

pub struct Overlay;
impl Viewable for Overlay {
    const GRID: MacroGrid = MacroGrid::new(8, 8);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        view_builder.apply_aesthetic(Dropdown::<i32>::new(
            [(String::from("opt-1"), 1), (String::from("opt-2"), 2)],
            ResponsiveSegment::base(Segment::new(
                3.near().to(6.far()),
                1.near().to(1.far()).maximum(50.0),
            ))
            .justify(Justify::Top),
            ExpandDirection::Down,
            ElementStyle::fill(),
            UIColor::new(ThemeColor::BASE, Color::BLACK),
        ));
        view_builder.finish()
    }
}
