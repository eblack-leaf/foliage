use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::differential::{Differentiable, Differential, DifferentialBundle};
use crate::elm::config::{ElmConfiguration, ExternalSet};
use crate::elm::leaf::Leaf;
use crate::elm::Elm;
use crate::texture::factors::MipsLevel;
#[allow(unused)]
use crate::{coordinate, differential_enable};
use bevy_ecs::component::Component;
#[allow(unused)]
use bevy_ecs::prelude::{Bundle, IntoSystemConfigs};
use bevy_ecs::prelude::{Query, SystemSet};
use bevy_ecs::query::Changed;
use bundled_cov::BundledIcon;
use serde::{Deserialize, Serialize};

pub mod bundled_cov;
mod proc_gen;
mod renderer;
mod vertex;

#[derive(Bundle)]
pub struct Icon {
    scale: IconScale,
    icon_id: DifferentialBundle<IconId>,
    color: DifferentialBundle<Color>,
    differentiable: Differentiable,
}
impl Icon {
    pub fn new(icon_id: IconId, scale: IconScale, color: Color) -> Self {
        Self {
            icon_id: DifferentialBundle::new(icon_id),
            color: DifferentialBundle::new(color),
            scale,
            differentiable: Differentiable::new::<Self>(
                Position::default(),
                Area::default(),
                Layer::default(),
            ),
        }
    }
}
#[derive(SystemSet, Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum SetDescriptor {
    Area,
}
impl Leaf for Icon {
    type SetDescriptor = SetDescriptor;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, SetDescriptor::Area);
    }

    fn attach(elm: &mut Elm) {
        differential_enable!(elm, CReprPosition, CReprArea, Color, IconId, MipsLevel);
        elm.job.main().add_systems((
            scale_change.in_set(SetDescriptor::Area),
            id_changed.in_set(SetDescriptor::Area),
        ));
    }
}
#[derive(Component, Hash, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct IconId(pub u32);
impl IconId {
    pub fn new(bundled_icon: BundledIcon) -> Self {
        Self(bundled_icon as u32)
    }
}
fn scale_change(
    mut query: Query<
        (&mut IconScale, &mut Area<InterfaceContext>),
        Changed<Area<InterfaceContext>>,
    >,
) {
    tracing::trace!("updating-icons");
    for (mut scale, mut area) in query.iter_mut() {
        *scale = IconScale::from_dim(area.width);
        let initial_px = scale.px();
        area.width = initial_px;
        area.height = initial_px;
    }
}
fn id_changed(
    mut icons: Query<
        (
            &mut Differential<Layer>,
            &mut Differential<CReprPosition>,
            &mut Differential<CReprArea>,
            &mut Differential<Color>,
        ),
        Changed<IconId>,
    >,
) {
    for (mut layer, mut pos, mut area, mut color) in icons.iter_mut() {
        layer.push_cached();
        pos.push_cached();
        area.push_cached();
        color.push_cached();
    }
}
#[derive(Component, Copy, Clone, Serialize, Deserialize, Debug)]
pub struct IconScale(pub(crate) CoordinateUnit);
impl IconScale {
    pub(crate) const UPPER_BOUND: u32 = 100;
    pub(crate) const LOWER_BOUND: u32 = 20;
    pub(crate) const INTERVAL: u32 = 4;
    pub fn px(self) -> CoordinateUnit {
        self.0
    }
    pub fn from_dim(r: CoordinateUnit) -> Self {
        let r = r - r % Self::INTERVAL as CoordinateUnit;
        Self(
            r.min(Self::UPPER_BOUND as f32)
                .max(Self::LOWER_BOUND as f32)
                .floor(),
        )
    }
}