use foliage::color::Color;
use foliage::conditional::ConditionalCommand;
use foliage::r_scenes::dropdown::{Dropdown, DropdownOptions, ExpandDirection};
use foliage::r_scenes::Colors;
use foliage::segment::{Justify, MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::view::{Navigate, ViewBuilder, ViewDescriptor, Viewable};

use crate::{BUTTON, ICON, PROGRESS};

pub struct Overlay;
impl Viewable for Overlay {
    const GRID: MacroGrid = MacroGrid::new(8, 5);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        let handle = view_builder.add_scene(
            Dropdown::new(
                DropdownOptions::new([
                    "button", "progress", "icon", "image", "text", "utility", "misc",
                ]),
                ExpandDirection::Down,
                Colors::new(Color::GREY, Color::DARK_GREY),
            ),
            ResponsiveSegment::base(Segment::new(
                3.near().to(6.far()).maximum(250.0),
                1.near().to(1.far()).maximum(50.0),
            ))
            .justify(Justify::Top),
        );
        let branches = &handle.branches().unwrap()[1..];
        view_builder.extend(
            branches.get(0).unwrap().target(),
            ConditionalCommand(Navigate(BUTTON)),
        );
        view_builder.extend(
            branches.get(1).unwrap().target(),
            ConditionalCommand(Navigate(PROGRESS)),
        );
        view_builder.extend(
            branches.get(2).unwrap().target(),
            ConditionalCommand(Navigate(ICON)),
        );
        // ...
        view_builder.finish()
    }
}
