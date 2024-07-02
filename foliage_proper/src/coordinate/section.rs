use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bytemuck::{Pod, Zeroable};

use crate::coordinate::area::{Area, GpuArea};
use crate::coordinate::position::{GpuPosition, Position};
use crate::coordinate::{
    CoordinateContext, CoordinateUnit, Coordinates, DeviceContext, LogicalContext, NumericalContext,
};

#[derive(Copy, Clone, Default, Bundle, PartialEq)]
pub struct Section<Context: CoordinateContext> {
    pub position: Position<Context>,
    pub area: Area<Context>,
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default, Component, PartialEq, Debug)]
pub struct GpuSection {
    pub pos: GpuPosition,
    pub area: GpuArea,
}
impl GpuSection {
    pub fn new(p: GpuPosition, a: GpuArea) -> Self {
        Self {
            pos: p.into(),
            area: a.into(),
        }
    }
}
impl Section<NumericalContext> {
    pub fn device<P: Into<Position<DeviceContext>>, A: Into<Area<DeviceContext>>>(
        p: P,
        a: A,
    ) -> Section<DeviceContext> {
        Section::new(p, a)
    }
    pub fn logical<P: Into<Position<LogicalContext>>, A: Into<Area<LogicalContext>>>(
        p: P,
        a: A,
    ) -> Section<LogicalContext> {
        Section::new(p, a)
    }
    pub fn numerical<P: Into<Position<NumericalContext>>, A: Into<Area<NumericalContext>>>(
        p: P,
        a: A,
    ) -> Section<NumericalContext> {
        Section::new(p, a)
    }
}
impl Section<DeviceContext> {
    pub fn to_gpu(self) -> GpuSection {
        GpuSection::new(self.position.to_gpu(), self.area.to_gpu())
    }
    pub fn to_logical(self, scale_factor: f32) -> Section<LogicalContext> {
        Section::new(
            self.position.to_logical(scale_factor),
            self.area.to_logical(scale_factor),
        )
    }
}
impl Section<LogicalContext> {
    pub fn to_device(self, factor: f32) -> Section<DeviceContext> {
        Section::new(self.position.to_device(factor), self.area.to_device(factor))
    }
}
impl<Context: CoordinateContext> Section<Context> {
    pub fn new<P: Into<Position<Context>>, A: Into<Area<Context>>>(p: P, a: A) -> Self {
        Self {
            position: p.into(),
            area: a.into(),
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
    pub fn center(&self) -> Position<Context> {
        Position::new((
            self.x() + self.width() / 2f32,
            self.y() + self.height() / 2f32,
        ))
    }
    pub fn intersection(&self, o: Self) -> Option<Section<Context>> {
        todo!()
    }
    pub fn contacts(&self, o: Self) -> bool {
        todo!()
    }
    pub fn contains(&self, p: Position<Context>) -> bool {
        p.x() <= self.right() && p.x() >= self.x() && p.y() <= self.bottom() && p.y() >= self.y()
    }
    pub fn normalized<C: Into<Coordinates>>(&self, c: C) -> Self {
        let c = c.into();
        Self::new(
            self.position.coordinates.normalized(c),
            self.area.coordinates.normalized(c),
        )
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
    pub fn to_numerical(self) -> Section<NumericalContext> {
        Section::new(self.position.to_numerical(), self.area.to_numerical())
    }
}
impl<Context: CoordinateContext, C: Into<Coordinates>, D: Into<Coordinates>> From<(C, D)>
    for Section<Context>
{
    fn from(value: (C, D)) -> Self {
        Self::new(value.0, value.1)
    }
}
