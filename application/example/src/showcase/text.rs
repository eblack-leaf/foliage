use foliage::bevy_ecs;
use foliage::bevy_ecs::prelude::Resource;
use foliage::color::monochromatic::{Asparagus, Greyscale, Monochromatic};
use foliage::derivation::ResourceDerivedValue;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use foliage::interactive_text::InteractiveText;
use foliage::segment::{MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::text::{TextLineStructure, TextValue};
use foliage::text_input::{TextInput, TextInputBindings, TextInputMode};
use foliage::view::{ViewBuilder, ViewDescriptor, Viewable};
use foliage::Colors;

pub struct TextShowcase;
impl Viewable for TextShowcase {
    const GRID: MacroGrid = MacroGrid::new(8, 5);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        // view_builder.add_scene(
        //     InteractiveText::new(
        //         TextLineStructure::new(30, 2),
        //         TextValue::new("Click to start selection.     Drag to extend section."),
        //         Colors::new(Asparagus::BASE, Greyscale::MINUS_THREE),
        //     ),
        //     ResponsiveSegment::base(Segment::new(2.near().to(7.far()), 2.near().to(2.far())))
        //         .at_layer(5),
        // );
        let input = view_builder.add_scene(
            TextInput::new(
                TextInputMode::Normal,
                TextLineStructure::new(20, 20),
                "".to_string(),
                Some("".into()),
                Colors::new(Asparagus::BASE, Greyscale::MINUS_THREE)
                    .with_alternate(Greyscale::BASE),
            ),
            ResponsiveSegment::base(Segment::new(1.near().to(8.far()), 2.near().to(5.far())))
                .at_layer(5),
        );
        view_builder.extend(
            input.bindings().get(TextInputBindings::Text),
            ResourceDerivedValue::<TextValueResource, TextValue>::new(),
        );
        // view_builder.add_scene(
        //     TextInput::new(
        //         TextInputMode::Password,
        //         TextLineStructure::new(15, 1),
        //         "".to_string(),
        //         Some("password".into()),
        //         Colors::new(Asparagus::BASE, Greyscale::MINUS_THREE),
        //     ),
        //     ResponsiveSegment::base(Segment::new(
        //         2.near().to(6.far()),
        //         5.near().to(5.far()).maximum(45.0),
        //     ))
        //     .at_layer(5),
        // );
        view_builder.finish()
    }
}
#[derive(Resource, Clone)]
pub(crate) struct TextValueResource(pub String);
impl From<TextValueResource> for TextValue {
    fn from(value: TextValueResource) -> Self {
        Self::new(value.0)
    }
}
impl Leaf for TextValueResource {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        elm.enable_resource_derivation::<TextValueResource, TextValue>();
        elm.container()
            .insert_resource(TextValueResource(String::from(
                "type here to test input...",
            )));
    }
}
