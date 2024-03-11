use foliage::color::monochromatic::{Asparagus, Monochromatic};
use foliage::color::Color;
use foliage::r_scenes::interactive_text::InteractiveText;
use foliage::r_scenes::Colors;
use foliage::segment::{MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::text::{MaxCharacters, TextValue};
use foliage::view::{ViewBuilder, ViewDescriptor, Viewable};

pub struct TextShowcase;
impl Viewable for TextShowcase {
    const GRID: MacroGrid = MacroGrid::new(8, 5);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        view_builder.add_scene(
            InteractiveText::new(
                MaxCharacters(6),
                TextValue::new("hello"),
                Colors::new(Asparagus::BASE, Color::DARK_GREY),
            ),
            ResponsiveSegment::base(Segment::new(
                2.near().to(6.far()),
                2.near().to(2.far()).maximum(50.0),
            ))
            .at_layer(5),
        );
        // view_builder.add(
        //     Rectangle::new(Asparagus::MINUS_THREE, Progress::full()),
        //     ResponsiveSegment::base(Segment::new(
        //         2.near().to(6.far()),
        //         2.near().to(2.far()).maximum(50.0),
        //     ))
        //     .at_layer(6),
        // );
        view_builder.finish()
    }
}
