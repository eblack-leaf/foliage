use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use bytemuck::{Pod, Zeroable};
use winit::dpi::{LogicalPosition, PhysicalPosition};

use crate::coordinate::{
    CoordinateContext, CoordinateUnit, Coordinates, Logical, Numerical, Physical,
};

#[derive(Copy, Clone, Default, PartialEq, PartialOrd)]
/// A point -- the top-left corner of a [`Section`](super::section::Section), a pointer
/// location, a scroll offset -- in one coordinate space.
///
/// Its two components are named `left`/`top` rather than x/y, matching the screen
/// convention the rest of the layout vocabulary uses. Supports the arithmetic positions
/// need: add and subtract to translate, scale by a factor, take a distance.
pub struct Position<Context: CoordinateContext> {
    pub coordinates: Coordinates,
    _phantom: PhantomData<Context>,
}
impl<Context: CoordinateContext> Display for Position<Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.coordinates))
    }
}
// Delegated to `Display`, as on `Coordinates`: the derived form would print the
// `PhantomData` space marker and the struct name around every logged point.
impl<Context: CoordinateContext> std::fmt::Debug for Position<Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default, PartialEq, Debug)]
/// A [`Position`] in GPU-upload layout.
pub struct CReprPosition(pub Coordinates);

impl Position<Numerical> {
    /// A point in scale-factor-independent pixels.
    pub fn logical<C: Into<Coordinates>>(c: C) -> Position<Logical> {
        Position::new(c)
    }
    /// A point in device pixels.
    pub fn physical<C: Into<Coordinates>>(c: C) -> Position<Physical> {
        Position::new(c)
    }
    /// A bare number pair, in no particular space.
    pub fn numerical<C: Into<Coordinates>>(c: C) -> Position<Numerical> {
        Position::new(c)
    }
    /// Reinterprets these numbers as logical pixels, without scaling.
    pub fn as_logical(self) -> Position<Logical> {
        Position::logical(self.coordinates)
    }
    /// Reinterprets these numbers as device pixels, without scaling.
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
    /// A point from any `(left, top)` pair.
    pub fn new<C: Into<Coordinates>>(c: C) -> Self {
        Self {
            coordinates: c.into(),
            _phantom: PhantomData,
        }
    }
    /// Rounds both components to the nearest whole unit.
    pub fn rounded(self) -> Self {
        Self::new((self.left().round(), self.top().round()))
    }
    /// Rounds both components down.
    pub fn floored(self) -> Self {
        Self::new((self.left().floor(), self.top().floor()))
    }
    /// Absolute value of both components.
    pub fn abs(self) -> Self {
        Self::new((self.left().abs(), self.top().abs()))
    }
    /// Horizontal component.
    pub fn left(&self) -> CoordinateUnit {
        self.coordinates.0[0]
    }
    /// Sets the horizontal component.
    pub fn set_left(&mut self, x: CoordinateUnit) {
        self.coordinates.set_horizontal(x);
    }
    /// Vertical component.
    pub fn top(&self) -> CoordinateUnit {
        self.coordinates.0[1]
    }
    /// Sets the vertical component.
    pub fn set_top(&mut self, y: CoordinateUnit) {
        self.coordinates.set_vertical(y);
    }
    /// Straight-line distance to `o` -- what a drag threshold is measured against.
    pub fn distance(self, o: Self) -> CoordinateUnit {
        ((self.left() - o.left()).powi(2) + (self.top() - o.top()).powi(2)).sqrt()
    }
    /// Drops the space marker, keeping the numbers.
    pub fn to_numerical(self) -> Position<Numerical> {
        Position::numerical((self.left(), self.top()))
    }
    /// This point as a fraction of `c`.
    pub fn normalized<C: Into<Coordinates>>(self, c: C) -> Self {
        let c = c.into();
        Self::new(self.coordinates.normalized(c))
    }
    /// Component-wise minimum.
    pub fn min<O: Into<Self>>(self, o: O) -> Self {
        let o = o.into();
        Self::new((self.left().min(o.left()), self.top().min(o.top())))
    }
    /// Component-wise maximum.
    pub fn max<O: Into<Self>>(self, o: O) -> Self {
        let o = o.into();
        Self::new((self.left().max(o.left()), self.top().max(o.top())))
    }
}

impl Position<Logical> {
    /// Multiplies by `factor` to get device pixels.
    pub fn to_physical(self, factor: f32) -> Position<Physical> {
        Position::physical((self.left() * factor, self.top() * factor))
    }
}

impl Position<Physical> {
    /// Divides out `factor` to get back to logical pixels.
    pub fn to_logical(self, factor: f32) -> Position<Logical> {
        Position::logical((self.left() / factor, self.top() / factor))
    }
    /// This point in its GPU-upload layout.
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
