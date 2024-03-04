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
            [
                (String::from("opt-1"), 1),
                (String::from("opt-2"), 2),
                (String::from("opt-3"), 3),
                (String::from("opt-4"), 4),
                (String::from("opt-5"), 5),
                (String::from("opt-6"), 6),
                (String::from("opt-7"), 7),
            ],
            ResponsiveSegment::base(Segment::new(
                3.near().to(6.far()).maximum(250.0),
                1.near().to(1.far()).maximum(50.0),
            ))
            .justify(Justify::Top),
            ExpandDirection::Down,
            ElementStyle::normal(),
            UIColor::new(Color::GREY, Color::WHITE),
        ));
        view_builder.finish()
    }
}