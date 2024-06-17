use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::LogicalContext;
use crate::interaction::click::ClickInteraction;
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::{Component, Event, Resource};
use bevy_ecs::system::Query;

pub mod click;
pub mod keyboard;
pub mod mouse;
pub enum InteractionPhase {
    Start,
    Move,
    End,
}
#[derive(Component)]
pub struct ClickInteractionListener {
    engaged_start: bool,
    engaged_end: bool,
    engaged: bool,
    focused: bool,
    active: bool,
    interaction: Option<ClickInteraction>,
}

pub(crate) fn listen_for_click_interactions(
    mut listeners: Query<(
        &mut ClickInteractionListener,
        &Position<LogicalContext>,
        &Area<LogicalContext>,
        &Layer,
    )>,
    mut events: EventReader<ClickInteraction>,
) {
    for event in events.read() {}
}
