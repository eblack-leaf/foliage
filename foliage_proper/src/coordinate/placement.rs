use bevy_ecs::bundle::Bundle;

use crate::coordinate::elevation::Layer;
use crate::coordinate::section::Section;
use crate::coordinate::CoordinateContext;

#[derive(Bundle, Default, Copy, Clone, Debug)]
pub struct Placement<Context: CoordinateContext> {
    pub section: Section<Context>,
    pub render_layer: Layer,
}

impl<Context: CoordinateContext> Placement<Context> {
    pub fn new<S: Into<Section<Context>>, L: Into<Layer>>(s: S, l: L) -> Self {
        Self {
            section: s.into(),
            render_layer: l.into(),
        }
    }
}
