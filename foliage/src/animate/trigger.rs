use crate::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::Elm;
use bevy_ecs::prelude::IntoSystemConfigs;
use bevy_ecs::query::Changed;
use bevy_ecs::system::Query;

#[derive(Default, Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Trigger(pub(crate) bool);
impl Trigger {
    pub fn triggered(&self) -> bool {
        self.0
    }
}
fn clear_triggered(mut triggers: Query<&mut Trigger, Changed<Trigger>>) {
    for mut trigger in triggers.iter_mut() {
        if trigger.0 {
            trigger.0 = false;
        }
    }
}
impl Leaf for Trigger {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.main()
            .add_systems(clear_triggered.after(CoreSet::Differential));
    }
}
