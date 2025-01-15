use crate::interaction::listener::InteractionListener;
use crate::{Attachment, Event, Foliage, InteractionState, Tree, Write};
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::Query;

#[derive(Event, Copy, Clone)]
pub struct Disable {}
impl Attachment for Disable {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Disable::interactions);
        foliage.define(Disable::user_signal);
        foliage.define(AutoDisable::interactions);
        foliage.define(AutoDisable::user_signal);
    }
}
impl Disable {
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.entity()) {
            listener.state.remove(InteractionState::ENABLED);
        }
    }
    fn user_signal(trigger: Trigger<Self>, mut tree: Tree) {
        tree.trigger_targets(Write::<Disable>::new(), trigger.entity());
    }
    pub fn new() -> Disable {
        Disable {}
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct AutoDisable {}
impl AutoDisable {
    pub(crate) fn new() -> Self {
        Self {}
    }
    fn user_signal(trigger: Trigger<Self>, mut tree: Tree) {
        tree.trigger_targets(Write::<Disable>::new(), trigger.entity());
    }
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.entity()) {
            listener.state.remove(InteractionState::AUTO_ENABLED);
        }
    }
}
