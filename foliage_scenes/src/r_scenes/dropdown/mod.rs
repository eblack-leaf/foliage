use crate::r_scenes::dropdown::scene::{Displays, DropdownScene, ExpandDirection, Selection};
use crate::r_scenes::UIColor;
use foliage_proper::aesthetic::Aesthetic;
use foliage_proper::animate::trigger::Trigger;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{
    Changed, Commands, Component, IntoSystemConfigs, Query, World,
};
use foliage_proper::bevy_ecs::system::Command;
use foliage_proper::conditional::ConditionHandle;
use foliage_proper::elm::leaf::Leaf;
use foliage_proper::elm::ElementStyle;
use foliage_proper::segment::ResponsiveSegment;
use foliage_proper::text::TextValue;
use foliage_proper::view::ViewBuilder;
use std::fmt::Display;

pub mod scene;
pub type DropdownDisplay = String;
#[derive(Component, Clone)]
pub struct DropdownValue<V: Clone>(pub V);
pub struct Dropdown<Value: Clone> {
    rs: ResponsiveSegment,
    displays: Vec<DropdownDisplay>,
    values: Vec<Value>,
    expand_direction: ExpandDirection,
    element_style: ElementStyle,
    ui_color: UIColor,
}
impl<Value: Clone> Dropdown<Value> {
    pub fn new<const N: usize>(
        list: [(DropdownDisplay, Value); N],
        responsive_segment: ResponsiveSegment,
        expand_direction: ExpandDirection,
        element_style: ElementStyle,
        ui_color: UIColor,
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
impl<Value: Clone + Display + Send + Sync + 'static> Aesthetic for Dropdown<Value> {
    fn pigment(self, view_builder: &mut ViewBuilder) {
        let max_chars = self
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
        let value = self.values.get(0).expect("must have one value").clone();
        let value_string = self.displays.get(0).expect("must have one display");
        view_builder.place_on(handle.root(), Selection(value.clone()));
        for (sb, sn) in handle.bindings().nodes().iter() {
            // base-cfg
            if sb.0 == 0 {
                // TODO make other handler to give to base
                // will open all conditions
                // view_builder.place_on(
                //     sn.entity(),
                //     OnSelect {
                //         target: handle.bindings().get(0),
                //         display: value_string.clone(),
                //     },
                // );
                // give base a Value<T>
                // also derive from Selection would take care of this
                view_builder.place_on(sn.entity(), TextValue::new(value_string.clone()));
                view_builder.place_on(sn.entity(), DropdownValue(value.clone()));
                view_builder.place_on(
                    sn.entity(),
                    OnExpand {
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
            view_builder.place_on(sn.entity(), TextValue::new(value_string));
            // on_select is only for dependents, so this can uniformly close branches
            view_builder.place_on(
                sn.branch().unwrap().target(),
                OnSelect {
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
    branches: Vec<ConditionHandle>,
}
impl Command for OnExpand {
    fn apply(self, world: &mut World) {
        for branch in self.branches.iter() {
            *world.get_mut::<Trigger>(branch.this()).unwrap() = Trigger::active();
        }
        // change expand state
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
    }
}
fn on_select(query: Query<(&Trigger, &OnSelect), Changed<Trigger>>, mut cmd: Commands) {
    for (trigger, on_select) in query.iter() {
        if trigger.is_active() {
            cmd.add(on_select.clone());
        }
    }
}