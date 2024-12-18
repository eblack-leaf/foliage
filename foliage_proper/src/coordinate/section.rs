use std::fmt::Display;
use std::ops::{Add, AddAssign, Mul, Sub};

use bevy_ecs::component::Component;
use bytemuck::{Pod, Zeroable};

use crate::coordinate::area::{Area, GpuArea};
use crate::coordinate::position::{GpuPosition, Position};
use crate::coordinate::{
    CoordinateContext, CoordinateUnit, Coordinates, DeviceContext, LogicalContext, NumericalContext,
};

#[derive(Copy, Clone, Default, Component, PartialEq, Debug)]
pub struct Section<Context: CoordinateContext> {
    pub position: Position<Context>,
    pub area: Area<Context>,
}
impl<Context: CoordinateContext> Display for Section<Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}|{}", self.position, self.area))
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default, Component, PartialEq, Debug)]
pub struct GpuSection {
    pub pos: GpuPosition,
    pub area: GpuArea,
}
impl GpuSection {
    pub fn new(p: GpuPosition, a: GpuArea) -> Self {
        Self { pos: p, area: a }
    }
    pub fn rounded(self) -> Self {
        Self::new(
            GpuPosition(self.pos.0.rounded()),
            GpuArea(self.area.0.rounded()),
        )
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
    pub fn left(&self) -> CoordinateUnit {
        self.position.x()
    }
    pub fn set_x(&mut self, x: CoordinateUnit) {
        self.position.set_x(x);
    }
    pub fn top(&self) -> CoordinateUnit {
        self.position.y()
    }
    pub fn set_y(&mut self, y: CoordinateUnit) {
        self.position.set_y(y);
    }
    pub fn width(&self) -> CoordinateUnit {
        self.area.width()
    }
    pub fn set_width(&mut self, w: CoordinateUnit) {
        self.area.set_width(w);
    }
    pub fn height(&self) -> CoordinateUnit {
        self.area.height()
    }
    pub fn set_height(&mut self, h: CoordinateUnit) {
        self.area.set_height(h);
    }
    pub fn set_position<P: Into<Position<Context>>>(&mut self, p: P) {
        self.position = p.into();
    }
    pub fn set_area<A: Into<Area<Context>>>(&mut self, a: A) {
        self.area = a.into();
    }
    pub fn right(&self) -> CoordinateUnit {
        self.left() + self.width()
    }
    pub fn bottom(&self) -> CoordinateUnit {
        self.top() + self.height()
    }
    pub fn center(&self) -> Position<Context> {
        Position::new((
            self.left() + self.width() / 2f32,
            self.top() + self.height() / 2f32,
        ))
    }
    pub fn intersection(&self, o: Self) -> Option<Section<Context>> {
        let left = self.left().max(o.left());
        let top = self.top().max(o.top());
        let right = self.right().min(o.right());
        let bottom = self.bottom().min(o.bottom());
        let section = Section::new((left, top), (right - left, bottom - top));
        if right < left || bottom < top {
            return None;
        }
        Some(section)
    }
    pub fn contacts(&self, o: Self) -> bool {
        self.intersection(o).is_some()
    }
    pub fn contains(&self, p: Position<Context>) -> bool {
        p.x() <= self.right()
            && p.x() >= self.left()
            && p.y() <= self.bottom()
            && p.y() >= self.top()
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
    pub fn rounded(self) -> Self {
        Self::new(self.position.rounded(), self.area.rounded())
    }
    pub fn floored(self) -> Self {
        Self::new(self.position.floored(), self.area.floored())
    }
    pub fn abs(self) -> Self {
        Self::new(self.position.abs(), self.area.abs())
    }
}
impl Section<NumericalContext> {
    pub fn as_logical(self) -> Section<LogicalContext> {
        Section::new(self.position.as_logical(), self.area.as_logical())
    }
    pub fn as_device(self) -> Section<DeviceContext> {
        Section::new(self.position.as_device(), self.area.as_device())
    }
}
impl<Context: CoordinateContext, C: Into<Coordinates>, D: Into<Coordinates>> From<(C, D)>
    for Section<Context>
{
    fn from(value: (C, D)) -> Self {
        Self::new(value.0, value.1)
    }
}
impl<Context: CoordinateContext> Add for Section<Context> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.position + rhs.position, self.area + rhs.area)
    }
}
impl<Context: CoordinateContext> AddAssign for Section<Context> {
    fn add_assign(&mut self, rhs: Self) {
        self.position += rhs.position;
        self.area += rhs.area;
    }
}
impl<Context: CoordinateContext> Sub for Section<Context> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.position - rhs.position, self.area - rhs.area)
    }
}

impl<Context: CoordinateContext> Mul<f32> for Section<Context> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.position * rhs, self.area * rhs)
    }
}
