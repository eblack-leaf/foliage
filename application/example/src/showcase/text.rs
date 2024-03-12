use foliage::color::monochromatic::{Asparagus, Greyscale, Monochromatic};
use foliage::interactive_text::InteractiveText;
use foliage::segment::{MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::text::{MaxCharacters, TextValue};
use foliage::text_input::{TextInput, TextInputMode};
use foliage::view::{ViewBuilder, ViewDescriptor, Viewable};
use foliage::Colors;

pub struct TextShowcase;
impl Viewable for TextShowcase {
    const GRID: MacroGrid = MacroGrid::new(8, 5);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        view_builder.add_scene(
            InteractiveText::new(
                MaxCharacters(6),
                TextValue::new("hello"),
                Colors::new(Asparagus::BASE, Greyscale::MINUS_THREE),
            ),
            ResponsiveSegment::base(Segment::new(
                2.near().to(6.far()),
                2.near().to(2.far()).maximum(50.0),
            ))
            .at_layer(5),
        );
        view_builder.add_scene(
            TextInput::new(
                TextInputMode::Normal,
                MaxCharacters(10),
                "a".to_string(),
                None,
                Colors::new(Asparagus::BASE, Greyscale::MINUS_THREE),
            ),
            ResponsiveSegment::base(Segment::new(
                2.near().to(6.far()),
                3.near().to(3.far()).maximum(50.0),
            ))
            .at_layer(5),
        );
        view_builder.finish()
    }
}