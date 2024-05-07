use std::marker::PhantomData;
use std::ops::AddAssign;

use winit::dpi::{LogicalPosition, PhysicalPosition};

use crate::coordinate::{CoordinateContext, DeviceContext, LogicalContext, NumericalContext};
use crate::{CoordinateUnit, Coordinates};

#[derive(Copy, Clone, Default)]
pub struct Position<Context: CoordinateContext> {
    pub coordinates: Coordinates,
    _phantom: PhantomData<Context>,
}

impl Position<NumericalContext> {
    pub fn logical<C: Into<Coordinates>>(c: C) -> Position<LogicalContext> {
        Position::new(c)
    }
    pub fn device<C: Into<Coordinates>>(c: C) -> Position<DeviceContext> {
        Position::new(c)
    }
    pub fn numerical<C: Into<Coordinates>>(c: C) -> Position<NumericalContext> {
        Position::new(c)
    }
    pub fn as_logical(self) -> Position<LogicalContext> {
        Position::logical(self.coordinates)
    }
    pub fn as_device(self) -> Position<DeviceContext> {
        Position::device(self.coordinates)
    }
}

impl<Context: CoordinateContext> AddAssign for Position<Context> {
    fn add_assign(&mut self, rhs: Self) {
        self.coordinates = (self.x() + rhs.x(), self.y() + rhs.y()).into();
    }
}

impl<Context: CoordinateContext> Position<Context> {
    pub fn new<C: Into<Coordinates>>(c: C) -> Self {
        Self {
            coordinates: c.into(),
            _phantom: PhantomData,
        }
    }
    pub fn x(&self) -> CoordinateUnit {
        self.coordinates.0[0]
    }
    pub fn y(&self) -> CoordinateUnit {
        self.coordinates.0[0]
    }
    pub fn to_numerical(self) -> Position<NumericalContext> {
        Position::numerical((self.x(), self.y()))
    }
    pub fn min(self, o: Self) -> Self {
        Self::new((self.x().min(o.x()), self.y().min(o.y())))
    }
    pub fn max(self, o: Self) -> Self {
        Self::new((self.x().max(o.x()), self.y().max(o.y())))
    }
}

impl Position<LogicalContext> {
    pub fn to_device(self, factor: f32) -> Position<DeviceContext> {
        Position::device((self.x() * factor, self.y() * factor))
    }
}

impl Position<DeviceContext> {
    pub fn to_logical(self, factor: f32) -> Position<LogicalContext> {
        Position::logical((self.x() / factor, self.y() / factor))
    }
}

impl From<LogicalPosition<f32>> for Position<LogicalContext> {
    fn from(value: LogicalPosition<f32>) -> Self {
        Self::new((value.x, value.y))
    }
}

impl From<PhysicalPosition<f32>> for Position<DeviceContext> {
    fn from(value: PhysicalPosition<f32>) -> Self {
        Self::new((value.x, value.y))
    }
}
