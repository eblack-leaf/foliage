use crate::coordinate::{
    CoordinateContext, CoordinateUnit, DeviceContext, InterfaceContext, NumericalContext,
};
use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::ops::{Add, Mul, Sub};
#[derive(Serialize, Deserialize, Copy, Clone, Component, PartialOrd, PartialEq, Default)]
pub struct Area<Context: CoordinateContext> {
    pub width: CoordinateUnit,
    pub height: CoordinateUnit,
    _phantom: PhantomData<Context>,
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
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub struct CReprArea {
    width: CoordinateUnit,
    height: CoordinateUnit,
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
