use bevy_ecs::bundle::Bundle;
use bytemuck::{Pod, Zeroable};

use crate::coordinate::area::{Area, GpuArea};
use crate::coordinate::position::{GpuPosition, Position};
use crate::coordinate::{
    CoordinateContext, CoordinateUnit, Coordinates, DeviceContext, LogicalContext, NumericalContext,
};

#[derive(Copy, Clone, Default, Bundle)]
pub struct Section<Context: CoordinateContext> {
    pub position: Position<Context>,
    pub area: Area<Context>,
}
impl Section<NumericalContext> {
    pub fn device<C: Into<Coordinates>>(p: C, a: C) -> Section<DeviceContext> {
        Section::new(p, a)
    }
    pub fn logical<C: Into<Coordinates>>(p: C, a: C) -> Section<LogicalContext> {
        Section::new(p, a)
    }
    pub fn numerical<C: Into<Coordinates>>(p: C, a: C) -> Section<NumericalContext> {
        Section::new(p, a)
    }
}

impl<Context: CoordinateContext> Section<Context> {
    pub fn new<C: Into<Coordinates>>(p: C, a: C) -> Self {
        Self {
            position: Position::new(p),
            area: Area::new(a),
        }
    }
    pub fn x(&self) -> CoordinateUnit {
        self.position.x()
    }
    pub fn y(&self) -> CoordinateUnit {
        self.position.y()
    }
    pub fn width(&self) -> CoordinateUnit {
        self.area.width()
    }
    pub fn height(&self) -> CoordinateUnit {
        self.area.height()
    }
    pub fn right(&self) -> CoordinateUnit {
        self.x() + self.width()
    }
    pub fn bottom(&self) -> CoordinateUnit {
        self.y() + self.height()
    }
    pub fn center(&self) -> Coordinates {
        todo!()
    }
    pub fn intersection(&self) -> Option<Section<Context>> {
        todo!()
    }
    pub fn contacts(&self, o: Self) -> bool {
        todo!()
    }
    pub fn min(self, o: Self) -> Self {
        Self::new(
            self.position.min(o.position).coordinates,
            self.area.min(o.area).coordinates,
        )
    }
    pub fn max(self, o: Self) -> Self {
        Self::new(
            self.position.max(o.position).coordinates,
            self.area.max(o.area).coordinates,
        )
    }
}
