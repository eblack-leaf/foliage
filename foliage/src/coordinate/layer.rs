use std::ops::{Add, Sub};

use crate::CoordinateUnit;
use bevy_ecs::component::Component;
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// Layer represents what plane this entity resides on. Used to differentiate z coords in
/// rendering.
#[repr(C)]
#[derive(
    Component, Copy, Clone, PartialOrd, PartialEq, Default, Pod, Zeroable, Serialize, Deserialize,
)]
pub struct Layer {
    pub z: CoordinateUnit,
}

impl Layer {
    pub fn new(z: CoordinateUnit) -> Self {
        Self { z }
    }
}
impl From<u32> for Layer {
    fn from(value: u32) -> Self {
        Self::new(value as CoordinateUnit)
    }
}
impl From<i32> for Layer {
    fn from(value: i32) -> Self {
        Layer::new(value as CoordinateUnit)
    }
}

impl Add for Layer {
    type Output = Layer;
    fn add(self, rhs: Self) -> Self::Output {
        Layer::new(self.z + rhs.z)
    }
}

impl Sub for Layer {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Layer::new(self.z - rhs.z)
    }
}
