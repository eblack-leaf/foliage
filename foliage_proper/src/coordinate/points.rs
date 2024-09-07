use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateContext, LogicalContext};
use bevy_ecs::prelude::Component;

#[derive(Debug, Clone, Default, Component)]
pub struct Points<Context: CoordinateContext> {
    pub data: [Position<Context>; 4],
}
impl<Context: CoordinateContext> Points<Context> {
    pub fn bbox(&self) -> Section<LogicalContext> {
        todo!()
    }
}
