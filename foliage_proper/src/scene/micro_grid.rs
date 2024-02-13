use crate::coordinate::{Coordinate, InterfaceContext};
use bevy_ecs::component::Component;
#[derive(Component, Copy, Clone)]
pub struct MicroGrid {}
impl MicroGrid {
    pub fn determine(
        &self,
        anchor: Coordinate<InterfaceContext>,
        alignment: Alignment,
    ) -> Coordinate<InterfaceContext> {
        todo!()
    }
}

#[derive(Component, Copy, Clone)]
pub struct Alignment {}