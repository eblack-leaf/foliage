use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::position::{CReprPosition, Position};
use crate::elm::{Elm, Leaf, SystemSets};
use bevy_ecs::prelude::{IntoSystemConfigs, Query};
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

fn position_set(mut query: Query<(&mut CReprPosition, &Position<InterfaceContext>)>) {
    for (mut c_repr, pos) in query.iter_mut() {
        *c_repr = pos.to_device(1.0).to_c();
    }
}
fn area_set(mut query: Query<(&mut CReprArea, &Area<InterfaceContext>)>) {
    for (mut c_repr, area) in query.iter_mut() {
        *c_repr = area.to_device(1.0).to_c();
    }
}
pub struct Coordinate {}
impl Leaf for Coordinate {
    fn attach(elm: &mut Elm) {
        elm.job.main().add_systems((
            position_set.before(SystemSets::Differential),
            area_set.before(SystemSets::Differential),
        ));
    }
}
