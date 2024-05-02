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
        Area::new(c)
    }
    pub fn device<C: Into<Coordinates<2>>>(c: C) -> Area<DeviceContext> {
        Area::new(c)
    }
    pub fn numerical<C: Into<Coordinates<2>>>(c: C) -> Area<NumericalContext> {
        Area::new(c)
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
    pub fn to_numerical(self) -> Area<NumericalContext> {
        Area::numerical((self.width(), self.height()))
    }
}
impl Area<LogicalContext> {
    pub fn to_device(self, factor: f32) -> Area<DeviceContext> {
        Area::device((self.width() * factor, self.height() * factor))
    }
}
impl Area<DeviceContext> {
    pub fn to_logical(self, factor: f32) -> Area<LogicalContext> {
        Area::logical((self.width() / factor, self.height() / factor))
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
impl From<PhysicalSize<u32>> for Area<DeviceContext> {
    fn from(value: PhysicalSize<u32>) -> Self {
        Self::new((value.width, value.height))
    }
}
