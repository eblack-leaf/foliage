use std::ops::{Add, Sub};

use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Pod, Zeroable, Component, Debug)]
pub(crate) struct RenderLayer(pub f32);
#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Pod, Zeroable, Component, Debug)]
pub struct Elevation(pub f32);
impl Elevation {
    pub fn new(e: i32) -> Self {
        Self(e as f32)
    }
}
macro_rules! elevation_conversion_implementation {
    ($i:ty) => {
        impl From<$i> for Elevation {
            fn from(value: $i) -> Self {
                Self::new(value as i32)
            }
        }
    };
}
elevation_conversion_implementation!(f32);
elevation_conversion_implementation!(i32);
elevation_conversion_implementation!(u32);
elevation_conversion_implementation!(usize);
elevation_conversion_implementation!(isize);
elevation_conversion_implementation!(f64);
elevation_conversion_implementation!(i64);
elevation_conversion_implementation!(u64);
impl Add for RenderLayer {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0 + rhs.0)
    }
}
impl Sub for RenderLayer {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.0 - rhs.0)
    }
}
impl RenderLayer {
    pub(crate) fn new(l: f32) -> Self {
        Self(l)
    }
}
macro_rules! layer_conversion_implementation {
    ($i:ty) => {
        impl From<$i> for RenderLayer {
            fn from(value: $i) -> Self {
                Self::new(value as f32)
            }
        }
    };
}
layer_conversion_implementation!(f32);
layer_conversion_implementation!(i32);
layer_conversion_implementation!(u32);
layer_conversion_implementation!(usize);
layer_conversion_implementation!(isize);
layer_conversion_implementation!(f64);
layer_conversion_implementation!(i64);
layer_conversion_implementation!(u64);