use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::scene::Anchor;

impl<SAP: Into<AlignmentPoint>> From<(SAP, SAP, i32)> for SceneAlignment {
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

impl<SAP: Into<AlignmentPoint>> From<(SAP, SAP, f32)> for SceneAlignment {
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

impl<SAP: Into<AlignmentPoint>> From<(SAP, SAP, u32)> for SceneAlignment {
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
    fn near(self) -> AlignmentPoint;
    fn center(self) -> AlignmentPoint;
    fn far(self) -> AlignmentPoint;
}

impl SceneAligner for CoordinateUnit {
    fn near(self) -> AlignmentPoint {
        AlignmentPoint {
            bias: AlignmentBias::Near,
            offset: self,
        }
    }
    fn center(self) -> AlignmentPoint {
        AlignmentPoint {
            bias: AlignmentBias::Center,
            offset: self,
        }
    }
    fn far(self) -> AlignmentPoint {
        AlignmentPoint {
            bias: AlignmentBias::Far,
            offset: self,
        }
    }
}

impl SceneAligner for i32 {
    fn near(self) -> AlignmentPoint {
        AlignmentPoint {
            bias: AlignmentBias::Near,
            offset: self as CoordinateUnit,
        }
    }
    fn center(self) -> AlignmentPoint {
        AlignmentPoint {
            bias: AlignmentBias::Center,
            offset: self as CoordinateUnit,
        }
    }
    fn far(self) -> AlignmentPoint {
        AlignmentPoint {
            bias: AlignmentBias::Far,
            offset: self as CoordinateUnit,
        }
    }
}
#[derive(Copy, Clone, Default)]
pub struct AlignmentDisable(pub bool);

#[derive(Copy, Clone, Default)]
pub enum AlignmentBias {
    #[default]
    Near,
    Center,
    Far,
}

#[derive(Copy, Clone)]
pub struct AlignmentPoint {
    pub bias: AlignmentBias,
    pub offset: CoordinateUnit,
}

#[derive(Copy, Clone)]
pub struct SceneAlignment {
    pub pos: PositionAlignment,
    pub(crate) layer: LayerAlignment,
}

#[derive(Copy, Clone)]
pub struct PositionAlignment {
    pub horizontal: AlignmentPoint,
    pub vertical: AlignmentPoint,
}

#[derive(Copy, Clone)]
pub struct LayerAlignment(pub CoordinateUnit);

impl LayerAlignment {
    pub fn calc_layer(&self, layer: Layer) -> Layer {
        layer + self.0.into()
    }
}

impl PositionAlignment {
    pub fn calc_pos(
        &self,
        anchor: Anchor,
        node_area: Area<InterfaceContext>,
    ) -> Position<InterfaceContext> {
        let x = match self.horizontal.bias {
            AlignmentBias::Near => anchor.0.section.left() + self.horizontal.offset,
            AlignmentBias::Center => {
                anchor.0.section.center().x - node_area.width / 2f32 + self.horizontal.offset
            }
            AlignmentBias::Far => {
                anchor.0.section.right() - self.horizontal.offset - node_area.width
            }
        };
        let y = match self.vertical.bias {
            AlignmentBias::Near => anchor.0.section.top() + self.vertical.offset,
            AlignmentBias::Center => {
                anchor.0.section.center().y - node_area.height / 2f32 + self.vertical.offset
            }
            AlignmentBias::Far => {
                anchor.0.section.bottom() - self.vertical.offset - node_area.height
            }
        };
        (x, y).into()
    }
}
