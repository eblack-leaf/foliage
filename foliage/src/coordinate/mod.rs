mod area;
mod layer;
mod position;
mod section;

pub use area::Area;
use bytemuck::{Pod, Zeroable};
pub use layer::Layer;
pub use position::Position;
pub use section::Section;
use serde::{Deserialize, Serialize};
pub trait CoordinateContext
where
    Self: Send + Sync + 'static + Copy + Clone,
{
}
#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct DeviceContext;
#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct LogicalContext;
#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct NumericalContext;
impl CoordinateContext for DeviceContext {}
impl CoordinateContext for LogicalContext {}
impl CoordinateContext for NumericalContext {}
pub type CoordinateUnit = f32;
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct Coordinates<const N: usize>(pub [CoordinateUnit; N]);
impl<const N: usize> Default for Coordinates<N> {
    fn default() -> Self {
        Self([CoordinateUnit::default(); N])
    }
}
macro_rules! permutation_coordinate_impl {
    ($a:ty, $b:ty) => {
        impl From<($a, $b)> for Coordinates<2> {
            fn from(value: ($a, $b)) -> Self {
                Self([value.0 as CoordinateUnit, value.1 as CoordinateUnit])
            }
        }
        impl From<($b, $a)> for Coordinates<2> {
            fn from(value: ($b, $a)) -> Self {
                Self([value.0 as CoordinateUnit, value.1 as CoordinateUnit])
            }
        }
    };
}
macro_rules! single_coordinate_impl {
    ($a:ty) => {
        impl From<($a, $a)> for Coordinates<2> {
            fn from(value: ($a, $a)) -> Self {
                Self([value.0 as CoordinateUnit, value.1 as CoordinateUnit])
            }
        }
    };
}
single_coordinate_impl!(f32);
single_coordinate_impl!(f64);
permutation_coordinate_impl!(f32, f64);
single_coordinate_impl!(i32);
permutation_coordinate_impl!(f32, i32);
permutation_coordinate_impl!(f64, i32);
single_coordinate_impl!(u32);
permutation_coordinate_impl!(f32, u32);
permutation_coordinate_impl!(i32, u32);
permutation_coordinate_impl!(f64, u32);
single_coordinate_impl!(usize);
permutation_coordinate_impl!(f32, usize);
permutation_coordinate_impl!(i32, usize);
permutation_coordinate_impl!(u32, usize);
permutation_coordinate_impl!(f64, usize);
