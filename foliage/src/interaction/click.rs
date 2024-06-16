use bevy_ecs::event::Event;

use crate::coordinate::position::Position;
use crate::coordinate::LogicalContext;

#[derive(Event)]
pub struct ClickInteraction {
    pub start: Position<LogicalContext>,
    pub current: Position<LogicalContext>,
    pub end: Option<Position<LogicalContext>>,
}
impl ClickInteraction {
    pub fn new(start: Position<LogicalContext>) -> Self {
        Self {
            start,
            current: start,
            end: None,
        }
    }
}
