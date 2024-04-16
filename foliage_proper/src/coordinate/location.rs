use bevy_ecs::prelude::Bundle;
use serde::{Deserialize, Serialize};

use crate::coordinate::CoordinateContext;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;

#[derive(Bundle, Copy, Clone, Default, Serialize, Deserialize, PartialEq, PartialOrd, Debug)]
pub struct Location<Context: CoordinateContext> {
    pub position: Position<Context>,
    pub layer: Layer,
}
impl<Context: CoordinateContext> Location<Context> {
    pub fn new<P: Into<Position<Context>>, L: Into<Layer>>(position: P, layer: L) -> Self {
        Self {
            position: position.into(),
            layer: layer.into(),
        }
    }
}