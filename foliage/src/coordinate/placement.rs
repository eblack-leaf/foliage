use bevy_ecs::bundle::Bundle;

use crate::coordinate::layer::Layer;
use crate::coordinate::section::Section;
use crate::coordinate::CoordinateContext;

#[derive(Bundle)]
pub struct Placement<Context: CoordinateContext> {
    pub section: Section<Context>,
    pub layer: Layer,
}
impl<Context: CoordinateContext> Placement<Context> {
    pub fn new<S: Into<Section<Context>>, L: Into<Layer>>(s: S, l: L) -> Self {
        Self {
            section: s.into(),
            layer: l.into(),
        }
    }
}
