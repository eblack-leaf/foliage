use crate::{Event, InteractionListener, InteractionState};
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::Query;

#[derive(Event, Copy, Clone)]
pub struct Enable {}
impl Enable {
    pub fn new() -> Enable {
        Enable {}
    }
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.entity()) {
            listener.state.insert(InteractionState::ENABLED);
        }
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct AutoEnable {}
impl AutoEnable {
    pub(crate) fn new() -> AutoEnable {
        AutoEnable {}
    }
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.entity()) {
            listener.state.insert(InteractionState::AUTO_ENABLED);
        }
    }
}
