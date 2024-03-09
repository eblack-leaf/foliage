use crate::r_scenes::Colors;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::Component;
use foliage_proper::bevy_ecs::system::SystemParamItem;
use foliage_proper::coordinate::{Coordinate, InterfaceContext};
use foliage_proper::rectangle::Rectangle;
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle};
use foliage_proper::text::{MaxCharacters, Text, TextValue};
use foliage_proper::texture::factors::Progress;

pub struct InteractiveText {
    pub max_chars: MaxCharacters,
    pub text_value: TextValue,
    pub colors: Colors,
}
impl InteractiveText {
    pub fn new(max_characters: MaxCharacters, text_value: TextValue, colors: Colors) -> Self {
        Self {
            max_chars: max_characters,
            text_value,
            colors,
        }
    }
}
#[derive(Component)]
pub struct Selection {
    pub value: String,
    pub start: Option<u32>,
    pub span: Option<i32>,
}
impl Selection {
    pub fn new(value: String, start: Option<u32>, span: Option<i32>) -> Self {
        Self { value, start, span }
    }
}
#[derive(Bundle)]
pub struct InteractiveTextComponents {
    pub selection: Selection,
}
impl Scene for InteractiveText {
    type Params = ();
    type Filter = ();
    type Components = InteractiveTextComponents;

    fn config(
        entity: Entity,
        _coordinate: Coordinate<InterfaceContext>,
        ext: &mut SystemParamItem<Self::Params>,
        bindings: &Bindings,
    ) {
        todo!()
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        binder.bind(
            0,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            Text::new(self.max_chars, self.text_value, self.colors.foreground.0),
        );
        for letter in 0..self.max_chars.0 {
            binder.bind_conditional(
                letter as i32 + 1,
                MicroGridAlignment::unaligned(),
                Rectangle::new(self.colors.foreground.0, Progress::full()),
            );
        }
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new(),
            InteractiveTextComponents {
                selection: Selection::new(String::default(), None, None),
            },
        ))
    }
}