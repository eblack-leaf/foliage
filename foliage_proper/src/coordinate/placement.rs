use bevy_ecs::bundle::Bundle;

use crate::coordinate::section::Section;
use crate::coordinate::CoordinateContext;
use crate::Elevation;

#[derive(Bundle, Default, Copy, Clone, Debug)]
pub struct Placement<Context: CoordinateContext> {
    pub section: Section<Context>,
    pub elevation: Elevation,
}

impl<Context: CoordinateContext> Placement<Context> {
    pub fn new<S: Into<Section<Context>>, L: Into<Elevation>>(s: S, l: L) -> Self {
        Self {
            section: s.into(),
            elevation: l.into(),
        }
    }
}
