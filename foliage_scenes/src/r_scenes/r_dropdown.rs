use compact_str::CompactString;

use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::animate::trigger::Trigger;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{Component, World};
use foliage_proper::bevy_ecs::system::{Command, SystemParamItem};
use foliage_proper::conditional::ConditionHandle;
use foliage_proper::coordinate::{Coordinate, InterfaceContext};
use foliage_proper::elm::leaf::Leaf;
use foliage_proper::elm::Elm;
use foliage_proper::scene::micro_grid::MicroGrid;
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle};
use foliage_proper::text::{MaxCharacters, TextValue};

use crate::r_scenes::Colors;

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
    pub fn new<const N: usize>(opts: [&'static str; N]) -> Self {
        Self {
            options: opts
                .to_vec()
                .drain(..)
                .map(|s| CompactString::new(s))
                .collect(),
        }
    }
}
#[derive(Component, Copy, Clone)]
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

    fn config(
        entity: Entity,
        coordinate: Coordinate<InterfaceContext>,
        ext: &mut SystemParamItem<Self::Params>,
        bindings: &Bindings,
    ) {
        // style changes here?
    }

    fn create(self, binder: Binder) -> SceneHandle {
        let max = MaxCharacters(self.options.options.iter().map(|o| o.len()).max().unwrap() as u32);
        // bind base

        // bind panel
        // for each option
        for binding in 0..self.options.options.iter().len() as i32 {
            let offset = match self.expand_direction {
                ExpandDirection::Up => -binding,
                ExpandDirection::Down => binding,
            };
            // -- bind conditional minimal text-button (fill)
        }
        // extend base with Expansion
        // iter binder.branches() or save condition_handles
        // -- extend w/ OnSelect
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
        elm.enable_conditional_command::<OnSelect>();
        elm.enable_conditional_command::<Expansion>();
    }
}
#[derive(Clone)]
struct OnSelect {
    root: Entity,
    base: Entity,
    branches: Vec<ConditionHandle>,
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
    branches: Vec<ConditionHandle>,
}
impl Command for Expansion {
    fn apply(self, world: &mut World) {
        let state = world.get::<ExpandState>(self.root).unwrap().clone();
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