use compact_str::CompactString;

use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::Component;
use foliage_proper::bevy_ecs::system::SystemParamItem;
use foliage_proper::coordinate::{Coordinate, InterfaceContext};
use foliage_proper::elm::Style;
use foliage_proper::panel::Panel;
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle};
use foliage_proper::text::{MaxCharacters, TextValue};

use crate::r_scenes::interactive_text::InteractiveText;
use crate::r_scenes::Colors;

pub struct TextInput {
    pub max_chars: MaxCharacters,
    pub colors: Colors,
    pub text: String,
    pub hint_text: Option<String>,
    pub mode: TextInputMode,
}
impl TextInput {
    pub fn new(
        mode: TextInputMode,
        max_characters: MaxCharacters,
        text: String,
        hint_text: Option<String>,
        colors: Colors,
    ) -> Self {
        Self {
            max_chars: max_characters,
            colors,
            text,
            hint_text,
            mode,
        }
    }
}
#[derive(Component, Copy, Clone)]
pub enum TextInputMode {
    Normal,
    Password,
}
#[derive(Component, Clone, Default)]
pub struct ActualText(pub CompactString);
impl ActualText {
    pub fn to_password(self) -> TextValue {
        let hidden = self.0.chars().map(|_i| "*").collect::<String>();
        TextValue::new(hidden)
    }
}
impl From<String> for ActualText {
    fn from(value: String) -> Self {
        Self(CompactString::new(value))
    }
}
#[derive(Bundle, Clone)]
pub struct TextInputComponents {
    pub actual: ActualText,
    pub display: TextValue,
    pub max_chars: MaxCharacters,
    pub colors: Colors,
    pub mode: TextInputMode,
}
#[derive(InnerSceneBinding)]
pub enum TextInputBindings {
    Panel,
    Text,
}
#[inner_set_descriptor]
pub enum SetDescriptor {
    Update,
}
impl Scene for TextInput {
    type Params = ();
    type Filter = ();
    type Components = TextInputComponents;
    #[allow(unused)]
    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings) {
        // style
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        binder.bind(
            TextInputBindings::Panel,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            )
            .offset_layer(2),
            Panel::new(Style::fill(), self.colors.foreground.0),
        );
        binder.bind_scene(
            TextInputBindings::Text,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                0.9.percent_of(AnchorDim::Width),
                0.9.percent_of(AnchorDim::Height),
            )
            .offset_layer(1),
            InteractiveText::new(self.max_chars, self.text.clone().into(), self.colors),
        );
        let actual: ActualText = self.text.into();
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new().aspect_ratio(self.max_chars.mono_aspect().value() * 1.25),
            TextInputComponents {
                actual: actual.clone(),
                display: match self.mode {
                    TextInputMode::Normal => actual.clone().0.as_str().into(),
                    TextInputMode::Password => actual.clone().to_password(),
                },
                max_chars: self.max_chars,
                colors: self.colors,
                mode: self.mode,
            },
        ))
    }
}