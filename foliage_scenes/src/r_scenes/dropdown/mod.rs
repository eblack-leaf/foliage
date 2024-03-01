use crate::r_scenes::dropdown::scene::{Displays, DropdownScene, ExpandDirection};
use crate::r_scenes::UIColor;
use foliage_proper::aesthetic::Aesthetic;
use foliage_proper::elm::ElementStyle;
use foliage_proper::segment::ResponsiveSegment;
use foliage_proper::view::ViewBuilder;

pub mod scene;
pub type DropdownDisplay = String;
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
impl<Value: Clone> Aesthetic for Dropdown<Value> {
    fn pigment(self, view_builder: &mut ViewBuilder) {
        let handle = view_builder.add_scene(
            DropdownScene::new(
                Displays(self.displays),
                self.expand_direction,
                self.element_style,
                self.ui_color,
            ),
            self.rs,
        );
        // base trigger to open conditionals

        // base derived-value for Value from Selection<Value>
        // how unique-ify this Selection<Value> // use component + entity tie?
        // one extra binding?
        for (sb, sn) in handle.bindings().nodes().iter() {
            // so leave one for base
            if sb.0 == 0 {
                // give base a Value<T>
                continue;
            }
            // link displays to nodes entity
            // since conditionals, need target maybe?
            // no, just add conditional<Display> to the branch-root with ExtendTarget::This
            // base-scene adds panel? for view-ability concerns?
            // based on background-color from dropdown-scene
        }
        // add-another on trigger (or group as the inverse reaction) to unset all open dropdown items condition
        {}
        //
        {}
    }
}
