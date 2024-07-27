use bevy_ecs::bundle::Bundle;

use crate::coordinate::elevation::RenderLayer;
use crate::coordinate::section::Section;
use crate::coordinate::CoordinateContext;

#[derive(Bundle, Default, Copy, Clone, Debug)]
pub struct Placement<Context: CoordinateContext> {
    pub section: Section<Context>,
    pub render_layer: RenderLayer,
}

impl<Context: CoordinateContext> Placement<Context> {
    pub fn new<S: Into<Section<Context>>, L: Into<RenderLayer>>(s: S, l: L) -> Self {
        Self {
            section: s.into(),
            render_layer: l.into(),
        }
    }
}
