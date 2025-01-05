use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use bytemuck::{Pod, Zeroable};
use winit::dpi::{LogicalPosition, PhysicalPosition};

use crate::coordinate::{
    CoordinateContext, CoordinateUnit, Coordinates, Logical, Numerical, Physical,
};

#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Debug)]
pub struct Position<Context: CoordinateContext> {
    pub coordinates: Coordinates,
    _phantom: PhantomData<Context>,
}
impl<Context: CoordinateContext> Display for Position<Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.coordinates))
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default, PartialEq, Debug)]
pub struct CReprPosition(pub Coordinates);

impl Position<Numerical> {
    pub fn logical<C: Into<Coordinates>>(c: C) -> Position<Logical> {
        Position::new(c)
    }
    pub fn physical<C: Into<Coordinates>>(c: C) -> Position<Physical> {
        Position::new(c)
    }
    pub fn numerical<C: Into<Coordinates>>(c: C) -> Position<Numerical> {
        Position::new(c)
    }
    pub fn as_logical(self) -> Position<Logical> {
        Position::logical(self.coordinates)
    }
    pub fn as_physical(self) -> Position<Physical> {
        Position::physical(self.coordinates)
    }
}

impl<Context: CoordinateContext> AddAssign for Position<Context> {
    fn add_assign(&mut self, rhs: Self) {
        self.coordinates = (self.left() + rhs.left(), self.top() + rhs.top()).into();
    }
}

impl<Context: CoordinateContext> Position<Context> {
    pub fn new<C: Into<Coordinates>>(c: C) -> Self {
        Self {
            coordinates: c.into(),
            _phantom: PhantomData,
        }
    }
    pub fn rounded(self) -> Self {
        Self::new((self.left().round(), self.top().round()))
    }
    pub fn floored(self) -> Self {
        Self::new((self.left().floor(), self.top().floor()))
    }
    pub fn abs(self) -> Self {
        Self::new((self.left().abs(), self.top().abs()))
    }
    pub fn left(&self) -> CoordinateUnit {
        self.coordinates.0[0]
    }
    pub fn set_left(&mut self, x: CoordinateUnit) {
        self.coordinates.set_horizontal(x);
    }
    pub fn top(&self) -> CoordinateUnit {
        self.coordinates.0[1]
    }
    pub fn set_top(&mut self, y: CoordinateUnit) {
        self.coordinates.set_vertical(y);
    }
    pub fn distance(self, o: Self) -> CoordinateUnit {
        ((self.left() - o.left()).powi(2) + (self.top() - o.top()).powi(2)).sqrt()
    }
    pub fn to_numerical(self) -> Position<Numerical> {
        Position::numerical((self.left(), self.top()))
    }
    pub fn normalized<C: Into<Coordinates>>(self, c: C) -> Self {
        let c = c.into();
        Self::new(self.coordinates.normalized(c))
    }
    pub fn min<O: Into<Self>>(self, o: O) -> Self {
        let o = o.into();
        Self::new((self.left().min(o.left()), self.top().min(o.top())))
    }
    pub fn max<O: Into<Self>>(self, o: O) -> Self {
        let o = o.into();
        Self::new((self.left().max(o.left()), self.top().max(o.top())))
    }
}

impl Position<Logical> {
    pub fn to_physical(self, factor: f32) -> Position<Physical> {
        Position::physical((self.left() * factor, self.top() * factor))
    }
}

impl Position<Physical> {
    pub fn to_logical(self, factor: f32) -> Position<Logical> {
        Position::logical((self.left() / factor, self.top() / factor))
    }
    pub fn c_repr(self) -> CReprPosition {
        CReprPosition(self.coordinates)
    }
}

impl From<LogicalPosition<f32>> for Position<Logical> {
    fn from(value: LogicalPosition<f32>) -> Self {
        Self::new((value.x, value.y))
    }
}

impl From<PhysicalPosition<f32>> for Position<Physical> {
    fn from(value: PhysicalPosition<f32>) -> Self {
        Self::new((value.x, value.y))
    }
}
impl<Context: CoordinateContext, C: Into<Coordinates>> From<C> for Position<Context> {
    fn from(value: C) -> Self {
        Self::new(value)
    }
}

impl<Context: CoordinateContext> Add for Position<Context> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.coordinates + rhs.coordinates)
    }
}

impl<Context: CoordinateContext> Sub for Position<Context> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.coordinates - rhs.coordinates).into()
    }
}
impl<Context: CoordinateContext> SubAssign for Position<Context> {
    fn sub_assign(&mut self, rhs: Self) {
        self.coordinates -= rhs.coordinates;
    }
}
impl<Context: CoordinateContext> Div<f32> for Position<Context> {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        (self.coordinates / rhs).into()
    }
}
impl<Context: CoordinateContext> Mul<f32> for Position<Context> {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self::new((self.left() * rhs, self.top() * rhs))
    }
}
