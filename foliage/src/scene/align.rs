use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Changed, Or, Query};

pub(crate) fn calc_alignments(
    mut pos_aligned: Query<
        (
            &SceneAnchor,
            &mut Position<InterfaceContext>,
            &Area<InterfaceContext>,
            &PositionAlignment,
        ),
        Or<(
            Changed<PositionAlignment>,
            Changed<SceneAnchor>,
            Changed<Position<InterfaceContext>>,
            Changed<Area<InterfaceContext>>,
        )>,
    >,
    mut layer_aligned: Query<
        (&SceneAnchor, &mut Layer, &LayerAlignment),
        Or<(
            Changed<LayerAlignment>,
            Changed<Layer>,
            Changed<SceneAnchor>,
        )>,
    >,
) {
    for (anchor, mut pos, area, alignment) in pos_aligned.iter_mut() {
        let position = alignment.calc_pos(*anchor, *area);
        *pos = position;
    }
    for (anchor, mut layer, alignment) in layer_aligned.iter_mut() {
        *layer = alignment.calc_layer(anchor.0.layer);
    }
}

impl<SAP: Into<SceneAlignmentPoint>> From<(SAP, SAP, i32)> for SceneAlignment {
    fn from(value: (SAP, SAP, i32)) -> Self {
        SceneAlignment {
            pos: PositionAlignment {
                horizontal: value.0.into(),
                vertical: value.1.into(),
            },
            layer: LayerAlignment(value.2 as CoordinateUnit),
        }
    }
}

impl<SAP: Into<SceneAlignmentPoint>> From<(SAP, SAP, f32)> for SceneAlignment {
    fn from(value: (SAP, SAP, f32)) -> Self {
        SceneAlignment {
            pos: PositionAlignment {
                horizontal: value.0.into(),
                vertical: value.1.into(),
            },
            layer: LayerAlignment(value.2),
        }
    }
}

impl<SAP: Into<SceneAlignmentPoint>> From<(SAP, SAP, u32)> for SceneAlignment {
    fn from(value: (SAP, SAP, u32)) -> Self {
        SceneAlignment {
            pos: PositionAlignment {
                horizontal: value.0.into(),
                vertical: value.1.into(),
            },
            layer: LayerAlignment(value.2 as CoordinateUnit),
        }
    }
}

pub trait SceneAligner {
    fn near(self) -> SceneAlignmentPoint;
    fn center(self) -> SceneAlignmentPoint;
    fn far(self) -> SceneAlignmentPoint;
}

impl SceneAligner for CoordinateUnit {
    fn near(self) -> SceneAlignmentPoint {
        SceneAlignmentPoint {
            bias: SceneAlignmentBias::Near,
            offset: self,
        }
    }
    fn center(self) -> SceneAlignmentPoint {
        SceneAlignmentPoint {
            bias: SceneAlignmentBias::Center,
            offset: self,
        }
    }
    fn far(self) -> SceneAlignmentPoint {
        SceneAlignmentPoint {
            bias: SceneAlignmentBias::Far,
            offset: self,
        }
    }
}

impl SceneAligner for i32 {
    fn near(self) -> SceneAlignmentPoint {
        SceneAlignmentPoint {
            bias: SceneAlignmentBias::Near,
            offset: self as CoordinateUnit,
        }
    }
    fn center(self) -> SceneAlignmentPoint {
        SceneAlignmentPoint {
            bias: SceneAlignmentBias::Center,
            offset: self as CoordinateUnit,
        }
    }
    fn far(self) -> SceneAlignmentPoint {
        SceneAlignmentPoint {
            bias: SceneAlignmentBias::Far,
            offset: self as CoordinateUnit,
        }
    }
}

#[derive(Copy, Clone, Component)]
pub struct SceneAnchor(pub Coordinate<InterfaceContext>);

impl From<Coordinate<InterfaceContext>> for SceneAnchor {
    fn from(value: Coordinate<InterfaceContext>) -> Self {
        Self(value)
    }
}

#[derive(Copy, Clone)]
pub enum SceneAlignmentBias {
    Near,
    Center,
    Far,
}

#[derive(Copy, Clone)]
pub struct SceneAlignmentPoint {
    pub bias: SceneAlignmentBias,
    pub offset: CoordinateUnit,
}

#[derive(Bundle, Copy, Clone)]
pub struct SceneAlignment {
    pos: PositionAlignment,
    layer: LayerAlignment,
}

#[derive(Component, Copy, Clone)]
pub struct PositionAlignment {
    pub horizontal: SceneAlignmentPoint,
    pub vertical: SceneAlignmentPoint,
}

#[derive(Component, Copy, Clone)]
pub struct LayerAlignment(pub CoordinateUnit);

impl LayerAlignment {
    pub fn calc_layer(&self, layer: Layer) -> Layer {
        layer + self.0.into()
    }
}

impl PositionAlignment {
    pub fn calc_pos(
        &self,
        anchor: SceneAnchor,
        node_area: Area<InterfaceContext>,
    ) -> Position<InterfaceContext> {
        let x = match self.horizontal.bias {
            SceneAlignmentBias::Near => anchor.0.section.left() + self.horizontal.offset,
            SceneAlignmentBias::Center => {
                anchor.0.section.center().x - node_area.width / 2f32 + self.horizontal.offset
            }
            SceneAlignmentBias::Far => {
                anchor.0.section.right() - self.horizontal.offset - node_area.width
            }
        };
        let y = match self.vertical.bias {
            SceneAlignmentBias::Near => anchor.0.section.top() + self.vertical.offset,
            SceneAlignmentBias::Center => {
                anchor.0.section.center().y - node_area.height / 2f32 + self.vertical.offset
            }
            SceneAlignmentBias::Far => {
                anchor.0.section.bottom() - self.vertical.offset - node_area.height
            }
        };
        (x, y).into()
    }
}
