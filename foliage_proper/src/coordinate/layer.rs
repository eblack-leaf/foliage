use std::ops::Add;

use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Pod, Zeroable, Component, Debug)]
pub struct Layer(pub f32);
impl Add for Layer {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0 + rhs.0)
    }
}
impl Layer {
    pub fn new(l: f32) -> Self {
        Self(l)
    }
}
macro_rules! layer_conversion_implementation {
    ($i:ty) => {
        impl From<$i> for Layer {
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
