use std::marker::PhantomData;
use std::ops::{Add, Div, DivAssign, Mul, MulAssign, Sub};

use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::coordinate::{
    CoordinateContext, CoordinateUnit, DeviceContext, InterfaceContext, NumericalContext,
};

#[derive(Serialize, Deserialize, Copy, Clone, Component, PartialOrd, PartialEq, Default, Debug)]
pub struct Area<Context: CoordinateContext> {
    pub width: CoordinateUnit,
    pub height: CoordinateUnit,
    _phantom: PhantomData<Context>,
}

impl<Context: CoordinateContext> Area<Context> {
    pub fn min_bound<A: Into<Self>>(&self, bounds: A) -> Self {
        let b = bounds.into();
        (self.width.max(b.width), self.height.max(b.height)).into()
    }
    pub fn max_bound<A: Into<Self>>(&self, bounds: A) -> Self {
        let b = bounds.into();
        (self.width.min(b.width), self.height.min(b.height)).into()
    }
}

impl<Context: CoordinateContext> Area<Context> {
    pub fn new(width: CoordinateUnit, height: CoordinateUnit) -> Self {
        Self {
            width,
            height,
            _phantom: PhantomData,
        }
    }
    pub fn to_numerical(self) -> Area<NumericalContext> {
        Area::<NumericalContext>::new(self.width, self.height)
    }
    /// return a copy as raw struct for gpu interactions
    pub fn to_c(self) -> CReprArea {
        CReprArea {
            width: self.width,
            height: self.height,
        }
    }
}

impl Area<InterfaceContext> {
    /// accounts for scale factor to convert this to device area
    pub fn to_device(self, scale_factor: CoordinateUnit) -> Area<DeviceContext> {
        Area::<DeviceContext>::new(self.width * scale_factor, self.height * scale_factor)
    }
}

impl Area<DeviceContext> {
    /// accounts for scale factor to convert this to interface area
    pub fn to_interface(self, scale_factor: CoordinateUnit) -> Area<InterfaceContext> {
        Area::<InterfaceContext>::new(self.width / scale_factor, self.height / scale_factor)
    }
}
impl Area<NumericalContext> {
    pub fn as_interface(self) -> Area<InterfaceContext> { (self.width, self.height).into() }
    pub fn as_device(self) -> Area<DeviceContext> { (self.width, self.height).into() }
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default, Component, Serialize, Deserialize, PartialEq)]
pub struct CReprArea {
    pub width: CoordinateUnit,
    pub height: CoordinateUnit,
}

impl CReprArea {
    pub fn new(width: CoordinateUnit, height: CoordinateUnit) -> Self {
        Self { width, height }
    }
}

impl<Context: CoordinateContext> From<(usize, usize)> for Area<Context> {
    fn from(value: (usize, usize)) -> Self {
        Self::new(value.0 as CoordinateUnit, value.1 as CoordinateUnit)
    }
}

impl<Context: CoordinateContext> From<(i32, i32)> for Area<Context> {
    fn from(value: (i32, i32)) -> Self {
        Self::new(value.0 as CoordinateUnit, value.1 as CoordinateUnit)
    }
}

impl<Context: CoordinateContext> From<(i32, f32)> for Area<Context> {
    fn from(value: (i32, f32)) -> Self {
        Self::new(value.0 as CoordinateUnit, value.1 as CoordinateUnit)
    }
}

impl<Context: CoordinateContext> From<(CoordinateUnit, CoordinateUnit)> for Area<Context> {
    fn from(value: (CoordinateUnit, CoordinateUnit)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl<Context: CoordinateContext> From<(u32, u32)> for Area<Context> {
    fn from(value: (u32, u32)) -> Self {
        Self::new(value.0 as CoordinateUnit, value.1 as CoordinateUnit)
    }
}

impl<Context: CoordinateContext> Mul for Area<Context> {
    type Output = Area<Context>;
    fn mul(self, rhs: Self) -> Self::Output {
        Area::<Context>::new(self.width * rhs.width, self.height * rhs.height)
    }
}

impl<Context: CoordinateContext> Add for Area<Context> {
    type Output = Area<Context>;
    fn add(self, rhs: Self) -> Self::Output {
        Area::<Context>::new(self.width + rhs.width, self.height + rhs.height)
    }
}

impl<Context: CoordinateContext> Sub for Area<Context> {
    type Output = Area<Context>;
    fn sub(self, rhs: Self) -> Self::Output {
        Area::<Context>::new(self.width - rhs.width, self.height - rhs.height)
    }
}

impl<Context: CoordinateContext> Div for Area<Context> {
    type Output = Area<Context>;
    fn div(self, rhs: Self) -> Self::Output {
        Area::<Context>::new(self.width / rhs.width, self.height / rhs.height)
    }
}

impl<Context: CoordinateContext> MulAssign for Area<Context> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<Context: CoordinateContext> DivAssign for Area<Context> {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}