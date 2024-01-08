use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::differential::{Differentiable, Differential, DifferentialBundle};
use crate::elm::config::{ElmConfiguration, ExternalSet};
use crate::elm::leaf::Leaf;
use crate::elm::Elm;
use crate::texture::factors::MipsLevel;
use crate::window::ScaleFactor;
#[allow(unused)]
use crate::{coordinate, differential_enable};
use bevy_ecs::component::Component;
#[allow(unused)]
use bevy_ecs::prelude::{Bundle, IntoSystemConfigs};
use bevy_ecs::prelude::{Or, Query, SystemSet};
use bevy_ecs::query::Changed;
use bevy_ecs::system::Res;
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
    mips: DifferentialBundle<MipsLevel>,
    differentiable: Differentiable,
}
impl Icon {
    pub fn new(icon_id: IconId, scale: IconScale, color: Color) -> Self {
        Self {
            icon_id: DifferentialBundle::new(icon_id),
            color: DifferentialBundle::new(color),
            mips: DifferentialBundle::new(scale.initial_mips()),
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
        (
            &IconScale,
            &Position<InterfaceContext>,
            &mut Area<InterfaceContext>,
            &mut MipsLevel,
        ),
        Or<(Changed<IconScale>, Changed<Area<InterfaceContext>>)>,
    >,
    scale_factor: Res<ScaleFactor>,
) {
    tracing::trace!("updating-icons");
    for (scale, pos, mut area, mut mips) in query.iter_mut() {
        let initial_px = scale.px();
        area.width = initial_px;
        area.height = initial_px;
        let section = Section::new(*pos, *area);
        let adjusted_section = section.to_device(scale_factor.factor());
        *mips = MipsLevel::new(
            (Icon::TEXTURE_DIMENSIONS, Icon::TEXTURE_DIMENSIONS).into(),
            Icon::MIPS,
            (adjusted_section.width(), adjusted_section.height()).into(),
        );
    }
}
fn id_changed(
    mut icons: Query<
        (
            &mut Differential<Layer>,
            &mut Differential<CReprPosition>,
            &mut Differential<CReprArea>,
            &mut Differential<MipsLevel>,
            &mut Differential<Color>,
        ),
        Changed<IconId>,
    >,
) {
    for (mut layer, mut pos, mut area, mut mips, mut color) in icons.iter_mut() {
        layer.push_cached();
        pos.push_cached();
        area.push_cached();
        mips.push_cached();
        color.push_cached();
    }
}
#[repr(u32)]
#[derive(Component, Copy, Clone)]
pub enum IconScale {
    Twenty = 16,
    Forty = 32,
    Eighty = 64,
}
impl IconScale {
    pub fn initial_mips(self) -> MipsLevel {
        match self {
            IconScale::Twenty => MipsLevel(2.0),
            IconScale::Forty => MipsLevel(1.0),
            IconScale::Eighty => MipsLevel(0.0),
        }
    }
    pub fn px(self) -> CoordinateUnit {
        self as u32 as CoordinateUnit
    }
    pub fn from_dim(dim: CoordinateUnit) -> Self {
        if dim >= Self::Eighty.px() {
            Self::Eighty
        } else if dim >= Self::Forty.px() {
            Self::Forty
        } else if dim >= Self::Twenty.px() {
            Self::Twenty
        } else {
            Self::Twenty
        }
    }
}
