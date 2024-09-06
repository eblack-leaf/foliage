use crate::coordinate::position::Position;
use crate::coordinate::CoordinateContext;
use bevy_ecs::prelude::Component;
#[derive(Debug, Clone, Default, Component)]
pub struct Points<Context: CoordinateContext> {
    pub data: Vec<Position<Context>>,
}
impl<Context: CoordinateContext> Points<Context> {
    pub fn new(data: Vec<Position<Context>>) -> Self {
        Self { data }
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
