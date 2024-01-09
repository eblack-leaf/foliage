use crate::coordinate::area::Area;
use crate::coordinate::{
    CoordinateContext, CoordinateUnit, DeviceContext, InterfaceContext, NumericalContext,
};
use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Sub};

#[derive(Serialize, Deserialize, Copy, Clone, Component, PartialOrd, PartialEq, Default, Debug)]
pub struct Position<Context: CoordinateContext> {
    pub x: CoordinateUnit,
    pub y: CoordinateUnit,
    _phantom: PhantomData<Context>,
}

impl<Context: CoordinateContext> Position<Context> {
    pub fn new(x: CoordinateUnit, y: CoordinateUnit) -> Self {
        Self {
            x,
            y,
            _phantom: PhantomData,
        }
    }
    /// returns a copy as just a number.
    pub fn to_numerical(self) -> Position<NumericalContext> {
        Position::<NumericalContext>::new(self.x, self.y)
    }
    /// returns a copy as a raw position
    pub fn to_c(self) -> CReprPosition {
        CReprPosition {
            x: self.x,
            y: self.y,
        }
    }
    pub fn normalized(self, area: Area<Context>) -> Position<Context> {
        (self.x / area.width, self.y / area.height).into()
    }
}

impl Position<InterfaceContext> {
    /// useful for converting to a device position accounting for scale factor
    pub fn to_device(self, scale_factor: CoordinateUnit) -> Position<DeviceContext> {
        Position::<DeviceContext>::new(self.x * scale_factor, self.y * scale_factor)
    }
}

impl Position<DeviceContext> {
    /// converts to interface context accounting for scale factor
    pub fn to_interface(self, scale_factor: CoordinateUnit) -> Position<InterfaceContext> {
        Position::<InterfaceContext>::new(self.x / scale_factor, self.y / scale_factor)
    }
}

impl<Context: CoordinateContext> Add for Position<Context> {
    type Output = Position<Context>;
    fn add(self, rhs: Self) -> Self::Output {
        Position::<Context>::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<Context: CoordinateContext> Sub for Position<Context> {
    type Output = Position<Context>;
    fn sub(self, rhs: Self) -> Self::Output {
        Position::<Context>::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<Context: CoordinateContext> Div for Position<Context> {
    type Output = Position<Context>;

    fn div(self, rhs: Self) -> Self::Output {
        Position::<Context>::new(self.x / rhs.x, self.y / rhs.y)
    }
}

/// Raw position for interacting with C
#[repr(C)]
#[derive(
    Pod, Zeroable, Copy, Clone, Default, Serialize, Deserialize, Debug, Component, PartialEq,
)]
pub struct CReprPosition {
    pub(crate) x: CoordinateUnit,
    pub(crate) y: CoordinateUnit,
}

impl CReprPosition {
    pub const fn new(x: CoordinateUnit, y: CoordinateUnit) -> Self {
        Self { x, y }
    }
}

impl<Context: CoordinateContext> From<(CoordinateUnit, CoordinateUnit)> for Position<Context> {
    fn from(value: (CoordinateUnit, CoordinateUnit)) -> Self {
        Position::<Context>::new(value.0, value.1)
    }
}

impl<Context: CoordinateContext> From<(f64, f64)> for Position<Context> {
    fn from(value: (f64, f64)) -> Self {
        Position::<Context>::new(value.0 as CoordinateUnit, value.1 as CoordinateUnit)
    }
}

impl<Context: CoordinateContext> From<(u32, u32)> for Position<Context> {
    fn from(value: (u32, u32)) -> Self {
        Position::<Context>::new(value.0 as CoordinateUnit, value.1 as CoordinateUnit)
    }
}

impl<Context: CoordinateContext> From<(i32, i32)> for Position<Context> {
    fn from(value: (i32, i32)) -> Self {
        Position::<Context>::new(value.0 as CoordinateUnit, value.1 as CoordinateUnit)
    }
}

impl<Context: CoordinateContext> From<(usize, usize)> for Position<Context> {
    fn from(value: (usize, usize)) -> Self {
        Position::<Context>::new(value.0 as CoordinateUnit, value.1 as CoordinateUnit)
    }
}

impl<Context: CoordinateContext> AddAssign for Position<Context> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
