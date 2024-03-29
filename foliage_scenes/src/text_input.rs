use compact_str::{CompactString, ToCompactString};

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
use foliage_proper::interaction::{
    FocusedEntity, InputSequence, InteractionListener, KeyboardEvent,
};
use foliage_proper::panel::Panel;
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle, ScenePtr};
use foliage_proper::text::{TextLineLocation, TextLineStructure, TextValue};

use crate::interactive_text::{InteractiveText, InteractiveTextBindings, Selection};
use crate::{AlternateColor, BackgroundColor, Colors, ForegroundColor};

#[derive(Clone)]
pub struct TextInput {
    pub line_structure: TextLineStructure,
    pub colors: Colors,
    pub text: String,
    pub hint_text: Option<String>,
    pub mode: TextInputMode,
}
impl TextInput {
    pub fn new(
        mode: TextInputMode,
        tls: TextLineStructure,
        text: String,
        hint_text: Option<String>,
        colors: Colors,
    ) -> Self {
        Self {
            line_structure: tls,
            colors,
            text,
            hint_text,
            mode,
        }
    }
}
fn input(
    mut keyboards: EventReader<KeyboardEvent>,
    mut text_inputs: Query<(&mut ActualText, &TextLineStructure)>,
    focused_entity: Res<FocusedEntity>,
    mut selections: Query<(&mut Selection, &Bindings, &ScenePtr), With<Tag<InteractiveText>>>,
    listeners: Query<&InteractionListener>,
) {
    for (mut selection, it_bindings, ptr) in selections.iter_mut() {
        let interactive_text = it_bindings.get(InteractiveTextBindings::Text);
        if let Ok((mut actual, tls)) = text_inputs.get_mut(ptr.value()) {
            if listeners.get(interactive_text).unwrap().lost_focus() && actual.0.is_empty() {
                trigger_config(&mut actual);
            }
            if let Some(e) = focused_entity.0 {
                if e == interactive_text {
                    if listeners.get(interactive_text).unwrap().engaged_start()
                        && actual.0.is_empty()
                    {
                        trigger_config(&mut actual);
                        selection.start.replace(TextLineLocation::raw(0, 0));
                    }
                    if selection.start.is_some() {
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
                                        if selection.spans_multiple() {
                                            selection.clear_selection_for(&mut actual.0, *tls);
                                        } else {
                                            // delete preceding char
                                            selection.move_cursor(*tls, -1);
                                            selection.clear_selection_for(&mut actual.0, *tls);
                                        }
                                        selection.move_cursor(*tls, -1);
                                    }
                                }
                                InputSequence::Enter => {
                                    // handle action?
                                }
                                InputSequence::Character(char) => {
                                    if e.state.is_pressed() {
                                        selection.insert_chars(&mut actual.0, &char, tls);
                                    }
                                }
                                InputSequence::ArrowLeft => {
                                    // if shift
                                    if e.state.is_pressed() {
                                        selection.move_cursor(*tls, -1);
                                    }
                                }
                                InputSequence::ArrowLeftShift => {
                                    // highlight left
                                    // move start
                                }
                                InputSequence::ArrowRight => {
                                    // move start
                                    if e.state.is_pressed() {
                                        selection.move_cursor(*tls, 1);
                                    }
                                }
                                InputSequence::ArrowRightShift => {
                                    // highlight right
                                    // move start
                                }
                                InputSequence::Space => {
                                    // insert whitespace
                                    if e.state.is_pressed() {
                                        selection.insert_chars(
                                            &mut actual.0,
                                            &" ".to_compact_string(),
                                            tls,
                                        )
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

fn trigger_config(actual: &mut ActualText) {
    actual.0.clear();
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
    pub max_chars: TextLineStructure,
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
                &'static TextLineStructure,
                &'static ForegroundColor,
                &'static BackgroundColor,
                &'static AlternateColor,
                &'static TextInputMode,
            ),
            With<Tag<TextInput>>,
        >,
        Query<
            'static,
            'static,
            (
                &'static mut TextValue,
                &'static mut TextLineStructure,
                &'static mut ForegroundColor,
                &'static mut BackgroundColor,
            ),
            Without<Tag<TextInput>>,
        >,
        Query<'static, 'static, &'static Bindings, Without<Tag<TextInput>>>,
        Query<'static, 'static, &'static InteractionListener, Without<Tag<TextInput>>>,
    );
    type Filter = Or<(
        Changed<ActualText>,
        Changed<TextLineStructure>,
        Changed<ForegroundColor>,
        Changed<BackgroundColor>,
        Changed<AlternateColor>,
        Changed<HintText>,
    )>;
    type Components = TextInputComponents;
    #[allow(unused)]
    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings) {
        let i_text = bindings.get(TextInputBindings::Text);
        let sub_i_text = ext
            .2
            .get(i_text)
            .unwrap()
            .get(InteractiveTextBindings::Text);
        let started = ext.3.get(sub_i_text).unwrap().engaged_start();
        if let Ok((at, ht, tls, fc, bc, ac, mode)) = ext.0.get(entity) {
            let value = match mode {
                TextInputMode::Normal => TextValue::new(at.0.clone()),
                TextInputMode::Password => at.clone().to_password(),
            };
            let value = if value.0.is_empty() && !started {
                *ext.1.get_mut(i_text).unwrap().2 = ac.0.into();
                TextValue::new(ht.0.clone())
            } else {
                *ext.1.get_mut(i_text).unwrap().2 = *fc;
                value
            };
            *ext.1.get_mut(i_text).unwrap().0 = value;
            *ext.1.get_mut(i_text).unwrap().1 = *tls;
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
                self.line_structure,
                match self.mode {
                    TextInputMode::Normal => actual.clone().0.as_str().into(),
                    TextInputMode::Password => actual.clone().to_password(),
                },
                self.colors,
            ),
        );
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new(),
            TextInputComponents {
                actual,
                max_chars: self.line_structure,
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
        elm.enable_conditional_scene::<TextInput>();
        elm.main().add_systems(
            (input, foliage_proper::scene::config::<TextInput>)
                .chain()
                .in_set(SetDescriptor::Update)
                .before(<InteractiveText as Leaf>::SetDescriptor::Update),
        );
    }
}