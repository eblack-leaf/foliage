use crate::r_scenes::dropdown::scene::DropdownScene;
use foliage_proper::aesthetic::Aesthetic;
use foliage_proper::segment::ResponsiveSegment;
use foliage_proper::view::ViewBuilder;

mod scene;

pub struct Dropdown<Display, ValueSetter> {
    rs: ResponsiveSegment,
    displays: Vec<Display>,
    // value setters (on-trigger of the interactive anonymous binding slots?)
    //
}

impl<Display, DerivedValue> Aesthetic for Dropdown<Display, DerivedValue> {
    fn pigment(self, view_builder: &mut ViewBuilder) {
        let handle = view_builder.add_scene(
            DropdownScene::new(
                // num slots
                self.displays.len(),
            ),
            self.rs,
        );
        // base trigger to open conditionals

        // base derived-value for Value from Selection<Value>
        // how unique-ify this Selection<Value> // use component + entity tie?
        // one extra binding?
        for (sb, sn) in handle.bindings().nodes().iter() {
            // so leave one for Selection
            if sb.0 == 0 {
                continue;
            }
            // and one for base
            if sb.0 == 1 {
                // first slot is not conditional
                // add specific stuff for initial triggers here
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
