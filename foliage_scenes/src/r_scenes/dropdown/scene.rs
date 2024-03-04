use crate::r_scenes::dropdown::{on_expand, on_select};
use crate::r_scenes::text_button::TextButton;
use crate::r_scenes::UIColor;
use foliage_macros::inner_set_descriptor;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{Component, IntoSystemConfigs};
use foliage_proper::bevy_ecs::system::SystemParamItem;
use foliage_proper::coordinate::{Coordinate, InterfaceContext};
use foliage_proper::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::Leaf;
use foliage_proper::elm::{ElementStyle, Elm};
use foliage_proper::panel::Panel;
use foliage_proper::scene::micro_grid::{
    Alignment, AlignmentDesc, AnchorDim, MicroGrid, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle};
use foliage_proper::text::{MaxCharacters, Text, TextValue};

#[derive(Component, Copy, Clone)]
pub enum ExpandDirection {
    Up,
    Down,
}
#[derive(Component, Copy, Clone)]
pub enum ExpandState {
    Expanded,
    Collapsed,
}
#[derive(Component, Clone)]
pub struct Selection<T: Clone>(pub T);
#[derive(Component, Clone)]
pub struct Values<T: Clone>(pub Vec<T>);
#[derive(Component, Clone, Default)]
pub struct Displays(pub Vec<String>);
pub(crate) struct DropdownScene {
    element_style: ElementStyle,
    displays: Displays,
    ui_color: UIColor,
    pub expanded_state: ExpandState,
    pub expand_direction: ExpandDirection,
}
impl DropdownScene {
    pub(crate) fn new(
        displays: Displays,
        expand_direction: ExpandDirection,
        element_style: ElementStyle,
        ui_color: UIColor,
    ) -> Self {
        Self {
            element_style,
            displays,
            ui_color,
            expanded_state: ExpandState::Collapsed,
            expand_direction,
        }
    }
}
#[derive(Component, Copy, Clone)]
pub struct CurrentSelection(pub u32);
#[derive(Bundle)]
pub struct DropdownSceneComponents {
    pub max_characters: MaxCharacters,
    pub style: ElementStyle,
    pub displays: Displays,
    pub ui_color: UIColor,
    pub expanded_state: ExpandState,
    pub expand_direction: ExpandDirection,
}
impl DropdownSceneComponents {
    pub fn new(
        mc: MaxCharacters,
        style: ElementStyle,
        displays: Displays,
        ui_color: UIColor,
        expand_direction: ExpandDirection,
        expand_state: ExpandState,
    ) -> Self {
        Self {
            max_characters: mc,
            style,
            displays,
            ui_color,
            expanded_state: expand_state,
            expand_direction,
        }
    }
}
impl Scene for DropdownScene {
    type Params = ();
    type Filter = ();
    type Components = DropdownSceneComponents;

    fn config(
        _entity: Entity,
        _coordinate: Coordinate<InterfaceContext>,
        _ext: &mut SystemParamItem<Self::Params>,
        _bindings: &Bindings,
    ) {
        // cfg style and display button colors
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        // to have Selection<T> inserted
        // + when change Selection<T> derive base-text value with the .to_string() of T (or From)
        // base node
        let max_chars =
            MaxCharacters(self.displays.0.iter().map(|d| d.len()).max().unwrap() as u32);
        binder.bind_scene(
            0,
            Alignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            TextButton::new(
                TextValue::new(self.displays.0.first().expect("need at least one display")),
                max_chars,
                self.element_style,
                self.ui_color.foreground.0,
                self.ui_color.background.0,
            ),
        );
        for i in 1..self.displays.0.len() {
            let binding = i as i32;
            let offset = match self.expand_direction {
                ExpandDirection::Up => -binding,
                ExpandDirection::Down => binding,
            } as f32
                * 1.2;
            binder.bind_conditional_scene(
                binding,
                Alignment::new(
                    0.percent_from(RelativeMarker::Center),
                    offset.percent_from(RelativeMarker::Top),
                    1.percent_of(AnchorDim::Width),
                    1.percent_of(AnchorDim::Height),
                ),
                TextButton::new(
                    TextValue::new(
                        self.displays
                            .0
                            .get(binding as usize)
                            .expect("need at least one display"),
                    ),
                    max_chars,
                    self.element_style,
                    self.ui_color.foreground.0,
                    self.ui_color.background.0,
                ),
            );
            // bind conditional text-button with display value (parallel) + offset 1 (for base)
            //
        }
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new(),
            DropdownSceneComponents::new(
                max_chars,
                self.element_style,
                self.displays,
                self.ui_color,
                self.expand_direction,
                ExpandState::Collapsed,
            ),
        ))
    }
}
#[inner_set_descriptor]
pub enum SetDescriptor {
    Update,
}
impl Leaf for DropdownScene {
    type SetDescriptor = SetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {
        _elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }

    fn attach(elm: &mut Elm) {
        elm.enable_conditional_scene::<TextButton>();
        elm.main().add_systems((
            foliage_proper::scene::config::<DropdownScene>
                .in_set(SetDescriptor::Update)
                .before(<TextButton as Leaf>::SetDescriptor::Update)
                .before(<Text as Leaf>::SetDescriptor::Update)
                .before(<Panel as Leaf>::SetDescriptor::Update),
            (on_select, on_expand).in_set(CoreSet::ProcessEvent),
        ));
    }
}
