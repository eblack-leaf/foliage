use crate::r_scenes::dropdown::scene::{
    Displays, DropdownScene, ExpandDirection, ExpandState, Selection,
};
use crate::r_scenes::Colors;
use foliage_proper::animate::trigger::Trigger;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{Changed, Commands, Component, Query, World};
use foliage_proper::bevy_ecs::system::Command;
use foliage_proper::conditional::ConditionHandle;
use foliage_proper::procedure::Procedure;

use foliage_proper::elm::Style;
use foliage_proper::segment::ResponsiveSegment;
use foliage_proper::text::TextValue;
use foliage_proper::view::ViewBuilder;

pub mod scene;
pub type DropdownDisplay = String;
#[derive(Component, Clone)]
pub struct DropdownValue<V: Clone>(pub V);
pub struct Dropdown<Value: Clone> {
    rs: ResponsiveSegment,
    displays: Vec<DropdownDisplay>,
    values: Vec<Value>,
    expand_direction: ExpandDirection,
    element_style: Style,
    ui_color: Colors,
}
impl<Value: Clone> Dropdown<Value> {
    pub fn new<const N: usize>(
        list: [(DropdownDisplay, Value); N],
        responsive_segment: ResponsiveSegment,
        expand_direction: ExpandDirection,
        element_style: Style,
        ui_color: Colors,
    ) -> Self {
        let displays = list.iter().map(|dv| dv.0.clone()).collect();
        let values = list.iter().map(|dv| dv.1.clone()).collect();
        Self {
            rs: responsive_segment,
            displays,
            values,
            expand_direction,
            element_style,
            ui_color,
        }
    }
}
impl<Value: Clone + Send + Sync + 'static> Procedure for Dropdown<Value> {
    fn steps(self, view_builder: &mut ViewBuilder) {
        let _max_chars = self
            .displays
            .iter()
            .map(|d| d.len())
            .max()
            .expect("could not find max-chars");
        let handle = view_builder.add_scene(
            DropdownScene::new(
                Displays(self.displays.clone()),
                self.expand_direction,
                self.element_style,
                self.ui_color,
            ),
            self.rs,
        );
        let value = self.values.first().expect("must have one value").clone();
        let _value_string = self.displays.first().expect("must have one display");
        view_builder.place_on(handle.root(), Selection(value.clone()));
        for (sb, sn) in handle.bindings().nodes().iter() {
            // base-cfg
            if sb.0 == 0 {
                view_builder.place_on(sn.entity(), DropdownValue(value.clone()));
                view_builder.place_on(
                    sn.entity(),
                    OnExpand {
                        target: handle.root(),
                        branches: handle.branches().unwrap().clone(),
                    },
                );
                continue;
            }
            let binding_value = self.values.get(sb.0 as usize).unwrap().clone();
            view_builder.place_on(sn.entity(), DropdownValue(binding_value));
            // can give text value initially, but then when created? will it override?
            // should this be derived as well from Values.get(0)?
            let value_string = self.displays.get(sb.0 as usize).unwrap();
            // view_builder.place_on(sn.entity(), TextValue::new(value_string));
            // on_select is only for dependents, so this can uniformly close branches
            view_builder.place_on(
                sn.branch().unwrap().target(),
                OnSelect {
                    root: handle.root(),
                    target: handle.bindings().get(0),
                    display: value_string.clone(),
                    branches: handle.branches().unwrap().clone(),
                },
            );
        }
    }
}
#[derive(Component, Clone)]
pub struct OnExpand {
    target: Entity,
    branches: Vec<ConditionHandle>,
}
impl Command for OnExpand {
    fn apply(self, world: &mut World) {
        let state = world.get::<ExpandState>(self.target).unwrap().clone();
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
        *world.get_mut::<ExpandState>(self.target).unwrap() = match state {
            ExpandState::Expanded => ExpandState::Collapsed,
            ExpandState::Collapsed => ExpandState::Expanded,
        };
    }
}
fn on_expand(query: Query<(&Trigger, &OnExpand), Changed<Trigger>>, mut cmd: Commands) {
    for (trigger, on_expand) in query.iter() {
        if trigger.is_active() {
            cmd.add(on_expand.clone());
        }
    }
}
#[derive(Component, Clone)]
pub struct OnSelect {
    root: Entity,
    target: Entity,
    display: String,
    branches: Vec<ConditionHandle>,
}
impl Command for OnSelect {
    fn apply(self, world: &mut World) {
        *world.get_mut::<TextValue>(self.target).unwrap() = TextValue::new(self.display);
        // close branches here? for on select
        for branch in self.branches.iter() {
            *world.get_mut::<Trigger>(branch.this()).unwrap() = Trigger::inverse();
        }
        // or another on trigger type?
        *world.get_mut::<ExpandState>(self.root).unwrap() = ExpandState::Collapsed;
    }
}
fn on_select(query: Query<(&Trigger, &OnSelect), Changed<Trigger>>, mut cmd: Commands) {
    for (trigger, on_select) in query.iter() {
        if trigger.is_active() {
            cmd.add(on_select.clone());
        }
    }
}
