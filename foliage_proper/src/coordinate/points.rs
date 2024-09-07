use crate::coordinate::position::Position;
use crate::coordinate::CoordinateContext;
use bevy_ecs::prelude::Component;
#[derive(Debug, Clone, Default, Component)]
pub struct Points<Context: CoordinateContext> {
    pub data: [Position<Context>; 4],
}
impl<Context: CoordinateContext> Points<Context> {}
