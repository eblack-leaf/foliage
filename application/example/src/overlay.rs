use crate::IMAGE;
use foliage::color::monochromatic::{Greyscale, Monochromatic};
use foliage::dropdown::{Dropdown, DropdownOptions, ExpandDirection};
use foliage::notifications::NotificationBar;
use foliage::segment::{Justify, MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::view::{Navigate, ViewBuilder, ViewDescriptor, Viewable};
use foliage::Colors;

use crate::{BUTTON, ICON, PROGRESS, TEXT};

pub struct Overlay;
impl Viewable for Overlay {
    const GRID: MacroGrid = MacroGrid::new(8, 5);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        let handle = view_builder.add_scene(
            Dropdown::new(
                DropdownOptions::new([
                    "button", "progress", "icon", "text", "image", // "utility", "misc",
                ]),
                ExpandDirection::Down,
                Colors::new(Greyscale::PLUS_TWO, Greyscale::MINUS_THREE),
            ),
            ResponsiveSegment::base(Segment::new(
                3.near().to(6.far()).maximum(250.0),
                1.near().to(1.far()).maximum(50.0),
            ))
            .justify(Justify::Top),
        );
        let branches = &handle.branches().unwrap()[1..];
        view_builder.add_command_to(branches.get(0).unwrap().target(), Navigate(BUTTON));
        view_builder.add_command_to(branches.get(1).unwrap().target(), Navigate(PROGRESS));
        view_builder.add_command_to(branches.get(2).unwrap().target(), Navigate(ICON));
        view_builder.add_command_to(branches.get(3).unwrap().target(), Navigate(TEXT));
        view_builder.add_command_to(branches.get(4).unwrap().target(), Navigate(IMAGE));
        // ...
        view_builder.apply(NotificationBar::new(Colors::new(
            Greyscale::PLUS_THREE,
            Greyscale::MINUS_THREE,
        )));
        view_builder.finish()
    }
}
