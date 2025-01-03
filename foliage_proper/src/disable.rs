use crate::interaction::listener::InteractionListener;
use crate::{Event, InteractionState};
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::Query;

#[derive(Event, Copy, Clone)]
pub struct Disable {}

impl Disable {
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.entity()) {
            listener.state.remove(InteractionState::ENABLED);
        }
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
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.entity()) {
            listener.state.remove(InteractionState::AUTO_ENABLED);
        }
    }
}
