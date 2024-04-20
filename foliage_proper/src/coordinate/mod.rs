use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{IntoSystemConfigs, Query};
use bevy_ecs::query::Changed;
use bevy_ecs::system::Res;
use serde::{Deserialize, Serialize};

use crate::animate::{Interpolate, Interpolation, InterpolationExtraction};
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::location::Location;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::section::Section;
use crate::elm::config::CoreSet;
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::Elm;
use crate::window::ScaleFactor;

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
#[derive(Component, Copy, Clone, Default)]
pub struct PositionAdjust(pub Position<InterfaceContext>);
impl Interpolate for PositionAdjust {
    fn interpolations(&self, end: &Self) -> Vec<Interpolation> {
        vec![
            Interpolation::new(self.0.x, end.0.x),
            Interpolation::new(self.0.y, end.0.y),
        ]
    }

    fn apply(&self, extracts: Vec<InterpolationExtraction>) -> Self {
        Self(
            self.0
                + Position::new(
                    extracts.get(0).cloned().unwrap_or_default().0,
                    extracts.get(1).cloned().unwrap_or_default().0,
                ),
        )
    }
}
#[derive(Component, Copy, Clone, Default)]
pub struct AreaAdjust(pub Area<InterfaceContext>);
impl Interpolate for AreaAdjust {
    fn interpolations(&self, end: &Self) -> Vec<Interpolation> {
        vec![
            Interpolation::new(self.0.width, end.0.width),
            Interpolation::new(self.0.height, end.0.height),
        ]
    }

    fn apply(&self, extracts: Vec<InterpolationExtraction>) -> Self {
        let mut copy = *self;
        copy.0.width += extracts.get(0).cloned().unwrap_or_default().0;
        copy.0.height += extracts.get(1).cloned().unwrap_or_default().0;
        copy
    }
}
pub(crate) fn position_set(
    mut query: Query<
        (&mut CReprPosition, &Position<InterfaceContext>),
        Changed<Position<InterfaceContext>>,
    >,
    scale_factor: Res<ScaleFactor>,
) {
    tracing::trace!("setting position");
    for (mut c_repr, pos) in query.iter_mut() {
        *c_repr = (*pos).to_device(scale_factor.factor()).to_c();
        c_repr.x = c_repr.x.round();
        c_repr.y = c_repr.y.round();
    }
}
pub(crate) fn area_set(
    mut query: Query<(&mut CReprArea, &Area<InterfaceContext>)>,
    scale_factor: Res<ScaleFactor>,
) {
    tracing::trace!("setting area");
    for (mut c_repr, area) in query.iter_mut() {
        *c_repr = area.to_device(scale_factor.factor()).to_c();
        c_repr.width = c_repr.width.round();
        c_repr.height = c_repr.height.round();
    }
}
#[derive(Copy, Clone, Bundle, Default, PartialEq, Debug)]
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
    pub fn with_area<A: Into<Area<Context>>>(mut self, a: A) -> Self {
        self.section.area = a.into();
        self
    }
    pub fn with_position<P: Into<Position<Context>>>(mut self, p: P) -> Self {
        self.section.position = p.into();
        self
    }
    pub fn with_layer<L: Into<Layer>>(mut self, l: L) -> Self {
        self.layer = l.into();
        self
    }
    pub fn moved_by<P: Into<Position<Context>>>(mut self, p: P) -> Self {
        self.section.position += p.into();
        self
    }
}
impl Leaf for CoordinateUnit {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        elm.enable_animation::<PositionAdjust>();
        elm.main().add_systems((
            position_set.in_set(CoreSet::CoordinateFinalize),
            area_set.in_set(CoreSet::CoordinateFinalize),
        ));
    }
}
