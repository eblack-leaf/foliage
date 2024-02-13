use crate::coordinate::{Coordinate, InterfaceContext};
use bevy_ecs::component::Component;
#[derive(Component, Copy, Clone)]
pub struct MicroGrid {}
impl MicroGrid {
    pub fn new() -> Self {
        Self {}
    }
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
impl Alignment {
    pub fn new() -> Self {
        Self {}
    }
}
#[test]
fn example() {
    let grid = MicroGrid::new();
    let coordinate = Coordinate::default();
    let alignment = Alignment::new();
    let determined = grid.determine(coordinate, alignment);
    // assert_eq!(determined, ...);
}
