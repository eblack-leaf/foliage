use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::location::Location;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::section::Section;
use crate::elm::{Elm, Leaf, SystemSets};
use crate::window::ScaleFactor;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{IntoSystemConfigs, Query};
use bevy_ecs::system::Res;
use serde::{Deserialize, Serialize};

pub mod area;
pub mod layer;
pub mod location;
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

pub(crate) fn position_set(
    mut query: Query<(&mut CReprPosition, &Position<InterfaceContext>)>,
    scale_factor: Res<ScaleFactor>,
) {
    for (mut c_repr, pos) in query.iter_mut() {
        *c_repr = pos.to_device(scale_factor.factor()).to_c();
        c_repr.x = c_repr.x.round();
        c_repr.y = c_repr.y.round();
    }
}
pub(crate) fn area_set(
    mut query: Query<(&mut CReprArea, &Area<InterfaceContext>)>,
    scale_factor: Res<ScaleFactor>,
) {
    for (mut c_repr, area) in query.iter_mut() {
        *c_repr = area.to_device(scale_factor.factor()).to_c();
        c_repr.width = c_repr.width.round();
        c_repr.height = c_repr.height.round();
    }
}
#[derive(Copy, Clone, Bundle, Default, PartialEq)]
pub struct Coordinate<Context: CoordinateContext> {
    pub section: Section<Context>,
    pub layer: Layer,
}
impl<Context: CoordinateContext> Coordinate<Context> {
    pub fn new<S: Into<Section<Context>>, L: Into<Layer>>(s: S, l: L) -> Self {
        Self {
            section: s.into(),
            layer: l.into(),
        }
    }
    pub fn location(&self) -> Location<Context> {
        Location::new(self.section.position, self.layer)
    }
}
pub struct CoordinateLeaf;
impl Leaf for CoordinateLeaf {
    fn attach(elm: &mut Elm) {
        elm.job.main().add_systems((
            position_set.in_set(SystemSets::FinalizeCoordinate),
            area_set.in_set(SystemSets::FinalizeCoordinate),
        ));
    }
}
