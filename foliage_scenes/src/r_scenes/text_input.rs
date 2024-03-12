use compact_str::{CompactString, ToCompactString};
use std::ops::Add;

use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::event::EventReader;
use foliage_proper::bevy_ecs::prelude::{Component, IntoSystemConfigs};
use foliage_proper::bevy_ecs::query::{Changed, Or, With, Without};
use foliage_proper::bevy_ecs::system::{Query, Res, SystemParamItem};
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::{Elm, Style};
use foliage_proper::interaction::{FocusedEntity, InputSequence, KeyboardEvent};
use foliage_proper::panel::Panel;
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle, ScenePtr};
use foliage_proper::text::{MaxCharacters, TextValue};

use crate::r_scenes::interactive_text::{InteractiveText, InteractiveTextBindings, Selection};
use crate::r_scenes::{BackgroundColor, Colors, ForegroundColor};

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
fn input(
    mut keyboards: EventReader<KeyboardEvent>,
    mut text_inputs: Query<(&mut ActualText, &MaxCharacters)>,
    focused_entity: Res<FocusedEntity>,
    mut selections: Query<(&mut Selection, &Bindings, &ScenePtr), With<Tag<InteractiveText>>>,
) {
    if let Some(e) = focused_entity.0 {
        for (mut sel, it_bindings, ptr) in selections.iter_mut() {
            if e == it_bindings.get(InteractiveTextBindings::Text) {
                if let Ok((mut actual, mc)) = text_inputs.get_mut(ptr.value()) {
                    if let Some(start) = sel.start {
                        for e in keyboards.read() {
                            match e.sequence() {
                                InputSequence::CtrlX => {
                                    // remove selection + copy to clipboard
                                }
                                InputSequence::CtrlC => {
                                    // copy to clipboard
                                }
                                InputSequence::CtrlA => {
                                    // select all
                                }
                                InputSequence::CtrlZ => {
                                    // last?
                                }
                                InputSequence::Backspace => {
                                    if e.state.is_pressed() {
                                        if sel.range().is_some() {
                                            clear_selection(&mut sel, &mut actual);
                                        } else {
                                            // delete start - 1 if possible
                                            if start != 0 {
                                                actual.0.remove(start as usize - 1);
                                                sel.start.replace(start - 1);
                                            }
                                        }
                                    }
                                }
                                InputSequence::Enter => {
                                    // handle action?
                                }
                                InputSequence::Character(char) => {
                                    if e.state.is_pressed() {
                                        clear_selection(&mut sel, &mut actual);
                                        if actual.0.len() + char.len() <= mc.0 as usize {
                                            actual.0.insert_str(
                                                sel.start.unwrap() as usize,
                                                char.as_str(),
                                            );
                                            let updated_start =
                                                sel.start.unwrap() + char.len() as i32;
                                            sel.start.replace(
                                                updated_start
                                                    .max(0)
                                                    .min(actual.0.len() as i32)
                                                    .min(mc.0.checked_sub(1).unwrap_or_default()
                                                        as i32),
                                            );
                                        }
                                    }
                                }
                                InputSequence::ArrowLeft => {
                                    // if shift
                                    if e.state.is_pressed() {
                                        let new_start = start - 1;
                                        sel.start.replace(new_start.max(0));
                                    }
                                }
                                InputSequence::ArrowLeftShift => {
                                    // highlight left
                                    // move start
                                }
                                InputSequence::ArrowRight => {
                                    // move start
                                    if e.state.is_pressed() {
                                        let new_start = start + 1;
                                        sel.start.replace(
                                            new_start
                                                .min(actual.0.len() as i32)
                                                .min(mc.0.checked_sub(1).unwrap_or_default() as i32),
                                        );
                                    }
                                }
                                InputSequence::ArrowRightShift => {
                                    // highlight right
                                    // move start
                                }
                                InputSequence::Space => {
                                    // insert whitespace
                                    if e.state.is_pressed() {
                                        clear_selection(&mut sel, &mut actual);
                                        if actual.0.len() + 1 <= mc.0 as usize {
                                            actual.0.insert_str(sel.start.unwrap() as usize, " ");
                                            let updated_start = sel.start.unwrap() + 1;
                                            sel.start.replace(
                                                updated_start
                                                    .max(0)
                                                    .min(actual.0.len() as i32)
                                                    .min(mc.0.checked_sub(1).unwrap_or_default()
                                                        as i32),
                                            );
                                        }
                                    }
                                }
                                InputSequence::Delete => {
                                    // delete current space
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}
fn clear_selection(sel: &mut Selection, actual: &mut ActualText) {
    if let Some(r) = sel.range() {
        actual.0.replace_range(
            *r.start() as usize
                ..=(*r.end() as usize).min(actual.0.len().checked_sub(1).unwrap_or_default()),
            "",
        );
        for _i in r {
            *sel.start.as_mut().unwrap() -= 1;
        }
        *sel.span.as_mut().unwrap() = 0;
        *sel.start.as_mut().unwrap() = sel.start.unwrap().add(1).max(0);
    }
}
#[derive(Component, Clone)]
pub struct HintText(pub CompactString);
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
    pub max_chars: MaxCharacters,
    pub colors: Colors,
    pub mode: TextInputMode,
    pub hint_text: HintText,
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
    type Params = (
        Query<
            'static,
            'static,
            (
                &'static ActualText,
                &'static HintText,
                &'static MaxCharacters,
                &'static ForegroundColor,
                &'static BackgroundColor,
                &'static TextInputMode,
            ),
            With<Tag<TextInput>>,
        >,
        Query<
            'static,
            'static,
            (
                &'static mut TextValue,
                &'static mut MaxCharacters,
                &'static mut ForegroundColor,
                &'static mut BackgroundColor,
            ),
            Without<Tag<TextInput>>,
        >,
    );
    type Filter = Or<(
        Changed<ActualText>,
        Changed<MaxCharacters>,
        Changed<ForegroundColor>,
        Changed<BackgroundColor>,
        Changed<HintText>,
    )>;
    type Components = TextInputComponents;
    #[allow(unused)]
    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings) {
        let i_text = bindings.get(TextInputBindings::Text);
        if let Ok((at, ht, mc, fc, bc, mode)) = ext.0.get(entity) {
            let value = match mode {
                TextInputMode::Normal => TextValue::new(at.0.clone()),
                TextInputMode::Password => at.clone().to_password(),
            };
            let value = if value.0.is_empty() {
                TextValue::new(ht.0.clone())
            } else {
                value
            };
            *ext.1.get_mut(i_text).unwrap().0 = value;
            *ext.1.get_mut(i_text).unwrap().1 = *mc;
            *ext.1.get_mut(i_text).unwrap().2 = *fc;
            *ext.1.get_mut(i_text).unwrap().3 = *bc;
        }
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
            .offset_layer(3),
            Panel::new(Style::fill(), self.colors.background.0),
        );
        let actual: ActualText = self.text.into();
        binder.bind_scene(
            TextInputBindings::Text,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                0.9.percent_of(AnchorDim::Width),
                0.9.percent_of(AnchorDim::Height),
            )
            .offset_layer(1),
            InteractiveText::new(
                self.max_chars,
                match self.mode {
                    TextInputMode::Normal => actual.clone().0.as_str().into(),
                    TextInputMode::Password => actual.clone().to_password(),
                },
                self.colors,
            ),
        );
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new().aspect_ratio(self.max_chars.mono_aspect().value() * 1.25),
            TextInputComponents {
                actual,
                max_chars: self.max_chars,
                colors: self.colors,
                mode: self.mode,
                hint_text: HintText(self.hint_text.unwrap_or_default().to_compact_string()),
            },
        ))
    }
}
impl Leaf for TextInput {
    type SetDescriptor = SetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {
        _elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems(
            (input, foliage_proper::scene::config::<TextInput>)
                .chain()
                .in_set(SetDescriptor::Update)
                .before(<InteractiveText as Leaf>::SetDescriptor::Update),
        );
    }
}