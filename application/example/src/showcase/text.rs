use foliage::color::monochromatic::{Asparagus, Greyscale, Monochromatic};
use foliage::interactive_text::InteractiveText;
use foliage::segment::{MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::text::{TextLineStructure, TextValue};
use foliage::text_input::{TextInput, TextInputMode};
use foliage::view::{ViewBuilder, ViewDescriptor, Viewable};
use foliage::Colors;

pub struct TextShowcase;
impl Viewable for TextShowcase {
    const GRID: MacroGrid = MacroGrid::new(8, 5);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        view_builder.add_scene(
            InteractiveText::new(
                TextLineStructure::new(5, 1),
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
                TextLineStructure::new(10, 1),
                "".to_string(),
                Some("username".into()),
                Colors::new(Asparagus::BASE, Greyscale::MINUS_THREE)
                    .with_alternate(Greyscale::BASE),
            ),
            ResponsiveSegment::base(Segment::new(
                2.near().to(6.far()),
                3.near().to(3.far()).maximum(50.0),
            ))
            .at_layer(5),
        );
        view_builder.add_scene(
            TextInput::new(
                TextInputMode::Password,
                TextLineStructure::new(10, 1),
                "".to_string(),
                Some("password".into()),
                Colors::new(Asparagus::BASE, Greyscale::MINUS_THREE),
            ),
            ResponsiveSegment::base(Segment::new(
                2.near().to(6.far()),
                4.near().to(4.far()).maximum(50.0),
            ))
            .at_layer(5),
        );
        view_builder.finish()
    }
}
