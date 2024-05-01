use crate::coordinate::{
    CoordinateContext, Coordinates, DeviceContext, LogicalContext, NumericalContext,
};
use crate::CoordinateUnit;
use std::marker::PhantomData;
use winit::dpi::{LogicalSize, PhysicalSize, Size};
#[derive(Copy, Clone, Default)]
pub struct Area<Context: CoordinateContext> {
    pub coordinates: Coordinates<2>,
    _phantom: PhantomData<Context>,
}
impl Area<NumericalContext> {
    pub fn logical<C: Into<Coordinates<2>>>(c: C) -> Area<LogicalContext> {
        Area::<LogicalContext>::new(c)
    }
    pub fn device<C: Into<Coordinates<2>>>(c: C) -> Area<DeviceContext> {
        Area::<DeviceContext>::new(c)
    }
}
impl<Context: CoordinateContext> Area<Context> {
    pub fn new<C: Into<Coordinates<2>>>(c: C) -> Self {
        Self {
            coordinates: c.into(),
            _phantom: PhantomData,
        }
    }
    pub fn width(&self) -> CoordinateUnit {
        self.coordinates.0[0]
    }
    pub fn height(&self) -> CoordinateUnit {
        self.coordinates.0[1]
    }
    pub fn min(&self, o: Self) -> Self {
        Self::new((self.width().min(o.width()), self.height().min(o.height())))
    }
    pub fn max(&self, o: Self) -> Self {
        Self::new((self.width().max(o.width()), self.height().max(o.height())))
    }
}
impl From<Area<LogicalContext>> for Size {
    fn from(value: Area<LogicalContext>) -> Self {
        Self::new(LogicalSize::new(value.width(), value.height()))
    }
}
impl From<Area<DeviceContext>> for Size {
    fn from(value: Area<DeviceContext>) -> Self {
        Self::new(PhysicalSize::new(value.width(), value.height()))
    }
}
