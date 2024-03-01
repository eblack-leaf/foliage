use crate::r_scenes::dropdown::scene::{Displays, DropdownScene, ExpandDirection, Selection};
use crate::r_scenes::text_button::TextButton;
use crate::r_scenes::UIColor;
use foliage_proper::aesthetic::Aesthetic;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::prelude::Component;
use foliage_proper::elm::ElementStyle;
use foliage_proper::segment::ResponsiveSegment;
use foliage_proper::text::{MaxCharacters, TextValue};
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
                Displays(self.displays),
                self.expand_direction,
                self.element_style,
                self.ui_color,
            ),
            self.rs,
        );
        let value = self.values.get(0).expect("must have one value").clone();
        let value_string = value.to_string();
        view_builder.place_on(handle.root(), Selection(value));
        for (sb, sn) in handle.bindings().nodes().iter() {
            // base-cfg
            if sb.0 == 0 {
                // give base a Value<T>
                view_builder.place_on(sn.entity(), TextValue::new(value_string.clone()));
                // also derive from Selection would take care of this
                continue;
            }
            // each other node needs
            // ---- on-trigger => set selection (index not Value) (InteractionTriggers?)
            // ---- func for pulling string from changed selections (Process? but then cant read in process)
            // ---- ^^^ put in process because this is automatic reading which would be something
            // ---- ^^^ you would schedule there
            // ----if I need on-trigger, then I could just set display value there?
            view_builder.place_conditional_scene_on(
                sn.entity(),
                TextButton::new(
                    TextValue::new(
                        self.values
                            .get(sb.0 as usize)
                            .expect("parallel-values & display required")
                            .to_string(),
                    ),
                    MaxCharacters(max_chars as u32),
                    self.element_style,
                    self.ui_color.foreground.0,
                    self.ui_color.background.0,
                ),
            );
        }
    }
}