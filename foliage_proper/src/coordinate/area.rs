use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Mul, Sub};

use bytemuck::{Pod, Zeroable};
use winit::dpi::{LogicalSize, PhysicalSize, Size};

use crate::coordinate::{
    CoordinateContext, CoordinateUnit, Coordinates, Logical, Numerical, Physical,
};

#[derive(Copy, Clone, Default, PartialEq, PartialOrd)]
/// A size -- the width and height half of a [`Section`](super::section::Section) -- in
/// one coordinate space.
///
/// Distinct from [`Position`](super::position::Position) so a size cannot be passed where
/// a point belongs. Negative components are allowed while a value is being computed;
/// [`abs`](Area::abs) normalizes them.
pub struct Area<Context: CoordinateContext> {
    pub coordinates: Coordinates,
    _phantom: PhantomData<Context>,
}
impl<Context: CoordinateContext> Display for Area<Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.coordinates))
    }
}
// Delegated to `Display`, as on `Coordinates`.
impl<Context: CoordinateContext> std::fmt::Debug for Area<Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default, PartialEq, Debug)]
/// An [`Area`] in GPU-upload layout.
pub struct CReprArea(pub Coordinates);

impl Area<Numerical> {
    /// A size in scale-factor-independent pixels.
    pub fn logical<C: Into<Coordinates>>(c: C) -> Area<Logical> {
        Area::new(c)
    }
    /// A size in device pixels.
    pub fn physical<C: Into<Coordinates>>(c: C) -> Area<Physical> {
        Area::new(c)
    }
    /// A bare number pair, in no particular space.
    pub fn numerical<C: Into<Coordinates>>(c: C) -> Area<Numerical> {
        Area::new(c)
    }
    /// Reinterprets these numbers as logical pixels, without scaling.
    pub fn as_logical(self) -> Area<Logical> {
        Self::logical(self.coordinates)
    }
    /// Reinterprets these numbers as device pixels, without scaling.
    pub fn as_physical(self) -> Area<Physical> {
        Self::physical(self.coordinates)
    }
}

impl<Context: CoordinateContext> Area<Context> {
    /// A size from any `(width, height)` pair.
    pub fn new<C: Into<Coordinates>>(c: C) -> Self {
        Self {
            coordinates: c.into(),
            _phantom: PhantomData,
        }
    }
    /// Rounds both components to the nearest whole unit.
    pub fn rounded(self) -> Self {
        Self::new((self.width().round(), self.height().round()))
    }
    /// Rounds both components down.
    pub fn floored(self) -> Self {
        Self::new((self.width().floor(), self.height().floor()))
    }
    /// Absolute value of both components, normalizing a negative extent.
    pub fn abs(self) -> Self {
        Self::new((self.width().abs(), self.height().abs()))
    }
    /// Horizontal extent.
    pub fn width(&self) -> CoordinateUnit {
        self.coordinates.0[0]
    }
    /// Sets the horizontal extent.
    pub fn set_width(&mut self, w: CoordinateUnit) {
        self.coordinates.set_horizontal(w);
    }
    /// Vertical extent.
    pub fn height(&self) -> CoordinateUnit {
        self.coordinates.0[1]
    }
    /// Sets the vertical extent.
    pub fn set_height(&mut self, h: CoordinateUnit) {
        self.coordinates.set_vertical(h);
    }
    /// This size as a fraction of `c`.
    pub fn normalized<C: Into<Coordinates>>(self, c: C) -> Self {
        let c = c.into();
        Self::new(self.coordinates.normalized(c))
    }
    /// Component-wise minimum -- clamping to a maximum size.
    pub fn min<O: Into<Self>>(&self, o: O) -> Self {
        let o = o.into();
        Self::new((self.width().min(o.width()), self.height().min(o.height())))
    }
    /// Component-wise maximum -- enforcing a minimum size.
    pub fn max<O: Into<Self>>(&self, o: O) -> Self {
        let o = o.into();
        Self::new((self.width().max(o.width()), self.height().max(o.height())))
    }
    /// Drops the space marker, keeping the numbers.
    pub fn to_numerical(self) -> Area<Numerical> {
        Area::numerical((self.width(), self.height()))
    }
}

impl Area<Logical> {
    /// Multiplies by `factor` to get device pixels.
    pub fn to_physical(self, factor: f32) -> Area<Physical> {
        Area::physical((self.width() * factor, self.height() * factor))
    }
}

impl Area<Physical> {
    /// Divides out `factor` to get back to logical pixels.
    pub fn to_logical(self, factor: f32) -> Area<Logical> {
        Area::logical((self.width() / factor, self.height() / factor))
    }
    /// This size in its GPU-upload layout.
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
