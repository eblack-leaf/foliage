use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Pod, Zeroable, Component)]
pub struct Layer(pub f32);

impl Layer {
    pub fn new(l: f32) -> Self {
        Self(l)
    }
}
macro_rules! layer_implementation {
    ($i:ty) => {
        impl From<$i> for Layer {
            fn from(value: $i) -> Self {
                Self::new(value as f32)
            }
        }
    };
}
layer_implementation!(f32);
layer_implementation!(i32);
layer_implementation!(u32);
layer_implementation!(usize);
layer_implementation!(isize);
layer_implementation!(f64);
layer_implementation!(i64);
layer_implementation!(u64);
