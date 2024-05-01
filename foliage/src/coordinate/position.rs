use crate::coordinate::{CoordinateContext, DeviceContext, LogicalContext, NumericalContext};
use crate::{CoordinateUnit, Coordinates};
use std::marker::PhantomData;
use winit::dpi::{LogicalPosition, PhysicalPosition};

pub struct Position<Context: CoordinateContext> {
    pub coordinates: Coordinates<2>,
    _phantom: PhantomData<Context>,
}
impl Position<NumericalContext> {
    pub fn logical<C: Into<Coordinates<2>>>(c: C) -> Position<LogicalContext> {
        Position::<LogicalContext>::new(c)
    }
    pub fn device<C: Into<Coordinates<2>>>(c: C) -> Position<DeviceContext> {
        Position::<DeviceContext>::new(c)
    }
}
impl<Context: CoordinateContext> Position<Context> {
    pub fn new<C: Into<Coordinates<2>>>(c: C) -> Self {
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
