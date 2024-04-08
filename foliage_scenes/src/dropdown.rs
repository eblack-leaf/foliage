use compact_str::{CompactString, ToCompactString};

use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::animate::trigger::Trigger;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{Component, World};
use foliage_proper::bevy_ecs::system::{Command, SystemParamItem};
use foliage_proper::conditional::ConditionalCommand;

use foliage_proper::elm::leaf::Leaf;
use foliage_proper::elm::{Elm, Style};
use foliage_proper::panel::Panel;
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle};
use foliage_proper::text::{MaxCharacters, TextLineStructure, TextValue};
use foliage_proper::view::BranchPool;

use crate::text_button::TextButton;
use crate::Colors;
#[derive(Clone)]
pub struct Dropdown {
    pub options: DropdownOptions,
    pub expand_direction: ExpandDirection,
    pub colors: Colors,
}
impl Dropdown {
    pub fn new(
        options: DropdownOptions,
        expand_direction: ExpandDirection,
        colors: Colors,
    ) -> Self {
        Self {
            options,
            expand_direction,
            colors,
        }
    }
}
#[derive(Component, Clone)]
pub struct DropdownOptions {
    pub options: Vec<CompactString>,
}
impl DropdownOptions {
    pub fn new<V: AsRef<[S]>, S: AsRef<str>>(opts: V) -> Self {
        Self {
            options: opts
                .as_ref()
                .iter()
                .map(|s| s.as_ref().to_compact_string())
                .collect(),
        }
    }
}
#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum ExpandDirection {
    Up,
    Down,
}
#[derive(Copy, Clone, Component)]
pub enum ExpandState {
    Expanded,
    Collapsed,
}
#[derive(Component, Copy, Clone)]
pub struct CurrentSelection(pub u32);
#[derive(Bundle, Clone)]
pub struct DropdownComponents {
    pub options: DropdownOptions,
    pub expand_state: ExpandState,
    pub current_selection: CurrentSelection,
}
#[derive(InnerSceneBinding)]
pub enum DropdownBindings {
    Base,
}
impl Scene for Dropdown {
    type Params = ();
    type Filter = ();
    type Components = DropdownComponents;

    fn config(_entity: Entity, _ext: &mut SystemParamItem<Self::Params>, _bindings: &Bindings) {
        // style changes here?
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        let max = MaxCharacters(self.options.options.iter().map(|o| o.len()).max().unwrap() as u32);
        // bind base
        let structure = TextLineStructure::new(max.0, 1);
        let base = binder.bind_scene(
            DropdownBindings::Base,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            TextButton::new(
                TextValue::new(self.options.options.first().unwrap()),
                structure,
                Style::fill(),
                Colors::new(self.colors.foreground.0, self.colors.background.0),
            ),
        );
        // bind panel
        let num_options = self.options.options.len() as i32;
        let option_offset: f32 = if self.expand_direction == ExpandDirection::Down {
            12.0
        } else {
            -12.0
        };
        binder.bind_conditional(
            1,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                if self.expand_direction == ExpandDirection::Down {
                    1
                } else {
                    -num_options
                }
                .percent_from(RelativeMarker::Top)
                .adjust(
                    option_offset
                        - if option_offset.is_sign_positive() {
                            4.0
                        } else {
                            -4.0
                        },
                ),
                1.percent_of(AnchorDim::Width),
                num_options.percent_of(AnchorDim::Height).adjust(4.0),
            )
            .offset_layer(1),
            Panel::new(Style::fill(), self.colors.background.0),
        );
        // for each option
        for (i, binding) in (2..self.options.options.iter().len() as i32 + 2).enumerate() {
            let offset_index = i as i32 + 1;
            let offset = match self.expand_direction {
                ExpandDirection::Up => -offset_index,
                ExpandDirection::Down => offset_index,
            };
            // -- bind conditional minimal text-button (fill)
            binder.bind_conditional_scene(
                binding,
                MicroGridAlignment::new(
                    0.percent_from(RelativeMarker::Center),
                    offset
                        .percent_from(RelativeMarker::Top)
                        .adjust(option_offset),
                    0.95.percent_of(AnchorDim::Width),
                    0.9.percent_of(AnchorDim::Height),
                ),
                TextButton::new(
                    TextValue::new(self.options.options.get(i).unwrap()),
                    structure,
                    Style::fill(),
                    Colors::new(self.colors.foreground.0, self.colors.background.0),
                ),
            );
        }
        // extend base with Expansion
        binder.extend(
            base.root(),
            ConditionalCommand(Expansion {
                root: binder.root(),
                branches: binder.branches().clone(),
            }),
        );
        for (i, branch) in binder.branches().clone()[1..].iter().enumerate() {
            binder.extend(
                branch.target(),
                ConditionalCommand(OnSelect {
                    root: binder.root(),
                    base: base.root(),
                    branches: binder.branches().clone(),
                    selection: i as u32,
                }),
            );
        }
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new(),
            DropdownComponents {
                options: self.options,
                expand_state: ExpandState::Collapsed,
                current_selection: CurrentSelection(0),
            },
        ))
    }
}
#[inner_set_descriptor]
pub enum SetDescriptor {
    Update,
}
impl Leaf for Dropdown {
    type SetDescriptor = SetDescriptor;

    fn attach(elm: &mut Elm) {
        elm.enable_conditional_scene::<Dropdown>();
        elm.enable_conditional_scene::<TextButton>();
        elm.enable_conditional::<Panel>();
        elm.enable_conditional_command::<OnSelect>();
        elm.enable_conditional_command::<Expansion>();
    }
}
#[derive(Clone)]
struct OnSelect {
    root: Entity,
    base: Entity,
    branches: BranchPool,
    selection: u32,
}
impl Command for OnSelect {
    fn apply(self, world: &mut World) {
        let value = world
            .get::<DropdownOptions>(self.root)
            .unwrap()
            .options
            .get(self.selection as usize)
            .unwrap()
            .clone();
        world.get_mut::<CurrentSelection>(self.root).unwrap().0 = self.selection;
        *world.get_mut::<TextValue>(self.base).unwrap() = TextValue::new(value);
        for branch in self.branches.iter() {
            *world.get_mut::<Trigger>(branch.this()).unwrap() = Trigger::inverse();
        }
        *world.get_mut::<ExpandState>(self.root).unwrap() = ExpandState::Collapsed;
    }
}
#[derive(Clone)]
struct Expansion {
    root: Entity,
    branches: BranchPool,
}
impl Command for Expansion {
    fn apply(self, world: &mut World) {
        let state = *world.get::<ExpandState>(self.root).unwrap();
        let trigger_state = match state {
            ExpandState::Expanded => {
                // collapse it
                Trigger::inverse()
            }
            ExpandState::Collapsed => {
                // expand it
                Trigger::active()
            }
        };
        for branch in self.branches.iter() {
            *world.get_mut::<Trigger>(branch.this()).unwrap() = trigger_state;
        }
        *world.get_mut::<ExpandState>(self.root).unwrap() = match state {
            ExpandState::Expanded => ExpandState::Collapsed,
            ExpandState::Collapsed => ExpandState::Expanded,
        };
    }
}