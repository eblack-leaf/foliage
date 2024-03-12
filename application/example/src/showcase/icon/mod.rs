use foliage::color::monochromatic::{FluorescentYellow, Greyscale, Monochromatic};
use foliage::icon::FeatherIcon;
use foliage::paged::Paged;
use foliage::segment::{MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::view::{ViewBuilder, ViewDescriptor, Viewable};
use foliage::{Colors, Direction};

use crate::showcase::icon::scene::IconDisplay;

pub mod scene;

pub struct IconShowcase;
impl Viewable for IconShowcase {
    const GRID: MacroGrid = MacroGrid::new(8, 5);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        let step = 20;
        let mut icon_groupings = vec![];
        for i in (0..286).step_by(step) {
            icon_groupings.push(IconDisplay::new(
                (i..i + step)
                    .map(|a| FeatherIcon::from(a as u32))
                    .collect::<Vec<FeatherIcon>>(),
            ));
        }
        view_builder.apply(Paged::new(
            icon_groupings,
            Colors::new(FluorescentYellow::BASE, Greyscale::MINUS_THREE),
            Direction::Horizontal,
            ResponsiveSegment::base(Segment::new(1.near().to(8.far()), 2.near().to(5.far())))
                .at_layer(5),
            FeatherIcon::Minus,
            FeatherIcon::Plus,
        ));
        view_builder.finish()
    }
}
