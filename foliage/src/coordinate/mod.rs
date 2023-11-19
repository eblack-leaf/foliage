use serde::{Deserialize, Serialize};
pub mod area;
pub mod layer;
pub mod position;
pub mod section;

pub type CoordinateUnit = f32;
pub trait CoordinateContext
where
    Self: Send + Sync + 'static + Copy + Clone,
{
}
#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct DeviceContext;
#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct InterfaceContext;
#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct NumericalContext;
impl CoordinateContext for DeviceContext {}
impl CoordinateContext for InterfaceContext {}
impl CoordinateContext for NumericalContext {}
