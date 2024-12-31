use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Mul, Sub};

use bytemuck::{Pod, Zeroable};
use winit::dpi::{LogicalSize, PhysicalSize, Size};

use crate::coordinate::{
    CoordinateContext, CoordinateUnit, Coordinates, Logical, Numerical, Physical,
};

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct Area<Context: CoordinateContext> {
    pub coordinates: Coordinates,
    _phantom: PhantomData<Context>,
}
impl<Context: CoordinateContext> Display for Area<Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.coordinates))
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default, PartialEq, Debug)]
pub struct CReprArea(pub Coordinates);

impl Area<Numerical> {
    pub fn logical<C: Into<Coordinates>>(c: C) -> Area<Logical> {
        Area::new(c)
    }
    pub fn physical<C: Into<Coordinates>>(c: C) -> Area<Physical> {
        Area::new(c)
    }
    pub fn numerical<C: Into<Coordinates>>(c: C) -> Area<Numerical> {
        Area::new(c)
    }
    pub fn as_logical(self) -> Area<Logical> {
        Self::logical(self.coordinates)
    }
    pub fn as_physical(self) -> Area<Physical> {
        Self::physical(self.coordinates)
    }
}

impl<Context: CoordinateContext> Area<Context> {
    pub fn new<C: Into<Coordinates>>(c: C) -> Self {
        Self {
            coordinates: c.into(),
            _phantom: PhantomData,
        }
    }
    pub fn rounded(self) -> Self {
        Self::new((self.width().round(), self.height().round()))
    }
    pub fn floored(self) -> Self {
        Self::new((self.width().floor(), self.height().floor()))
    }
    pub fn abs(self) -> Self {
        Self::new((self.width().abs(), self.height().abs()))
    }
    pub fn width(&self) -> CoordinateUnit {
        self.coordinates.0[0]
    }
    pub fn set_width(&mut self, w: CoordinateUnit) {
        self.coordinates.set_horizontal(w);
    }
    pub fn height(&self) -> CoordinateUnit {
        self.coordinates.0[1]
    }
    pub fn set_height(&mut self, h: CoordinateUnit) {
        self.coordinates.set_vertical(h);
    }
    pub fn normalized<C: Into<Coordinates>>(self, c: C) -> Self {
        let c = c.into();
        Self::new(self.coordinates.normalized(c))
    }
    pub fn min<O: Into<Self>>(&self, o: O) -> Self {
        let o = o.into();
        Self::new((self.width().min(o.width()), self.height().min(o.height())))
    }
    pub fn max<O: Into<Self>>(&self, o: O) -> Self {
        let o = o.into();
        Self::new((self.width().max(o.width()), self.height().max(o.height())))
    }
    pub fn to_numerical(self) -> Area<Numerical> {
        Area::numerical((self.width(), self.height()))
    }
}

impl Area<Logical> {
    pub fn to_physical(self, factor: f32) -> Area<Physical> {
        Area::physical((self.width() * factor, self.height() * factor))
    }
}

impl Area<Physical> {
    pub fn to_logical(self, factor: f32) -> Area<Logical> {
        Area::logical((self.width() / factor, self.height() / factor))
    }
    pub fn c_repr(self) -> CReprArea {
        CReprArea(self.coordinates)
    }
}

impl From<Area<Logical>> for Size {
    fn from(value: Area<Logical>) -> Self {
        Self::new(LogicalSize::new(value.width(), value.height()))
    }
}

impl From<Area<Physical>> for Size {
    fn from(value: Area<Physical>) -> Self {
        Self::new(PhysicalSize::new(value.width(), value.height()))
    }
}

impl From<PhysicalSize<u32>> for Area<Physical> {
    fn from(value: PhysicalSize<u32>) -> Self {
        Self::new((value.width, value.height))
    }
}
impl<Context: CoordinateContext, C: Into<Coordinates>> From<C> for Area<Context> {
    fn from(value: C) -> Self {
        Self::new(value)
    }
}

impl<Context: CoordinateContext> Sub for Area<Context> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.coordinates - rhs.coordinates).into()
    }
}

impl<Context: CoordinateContext> Div for Area<Context> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        (self.coordinates / rhs.coordinates).into()
    }
}

impl<Context: CoordinateContext> Add for Area<Context> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        (self.coordinates + rhs.coordinates).into()
    }
}
impl<Context: CoordinateContext> AddAssign for Area<Context> {
    fn add_assign(&mut self, rhs: Self) {
        self.coordinates += rhs.coordinates;
    }
}
impl<Context: CoordinateContext> Mul<f32> for Area<Context> {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self::new((self.width() * rhs, self.height() * rhs))
    }
}
