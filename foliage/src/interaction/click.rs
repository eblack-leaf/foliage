use bevy_ecs::event::Event;

use crate::coordinate::LogicalContext;
use crate::coordinate::position::Position;

#[derive(Event)]
pub struct ClickInteraction {
    pub start: Position<LogicalContext>,
    pub current: Position<LogicalContext>,
    pub end: Option<Position<LogicalContext>>
}