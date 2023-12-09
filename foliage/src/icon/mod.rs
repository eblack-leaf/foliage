use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::differential::{Differentiable, DifferentialBundle};
use crate::elm::leaf::Leaf;
use crate::elm::set_category::{CoreSet, ElmConfiguration, ExternalSet};
use crate::elm::Elm;
use crate::texture::factors::MipsLevel;
use crate::window::ScaleFactor;
#[allow(unused)]
use crate::{coordinate, differential_enable};
use bevy_ecs::component::Component;
#[allow(unused)]
use bevy_ecs::prelude::{Bundle, IntoSystemConfigs};
use bevy_ecs::prelude::{Query, SystemSet};
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
    position: Position<InterfaceContext>,
    area: Area<InterfaceContext>,
    scale: IconScale,
    icon_id: DifferentialBundle<IconId>,
    c_pos: DifferentialBundle<CReprPosition>,
    c_area: DifferentialBundle<CReprArea>,
    color: DifferentialBundle<Color>,
    mips: DifferentialBundle<MipsLevel>,
    differentiable: Differentiable,
}
impl Icon {
    pub fn new(icon_id: IconId, scale: IconScale, color: Color) -> Self {
        Self {
            position: Default::default(),
            area: Area::default(),
            icon_id: DifferentialBundle::new(icon_id),
            c_pos: DifferentialBundle::new(CReprPosition::default()),
            c_area: DifferentialBundle::new(CReprArea::default()),
            color: DifferentialBundle::new(color),
            mips: DifferentialBundle::new(scale.initial_mips()),
            scale,
            differentiable: Differentiable::new::<Self>(Layer::default()),
        }
    }
}
#[derive(SystemSet, Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum SystemHook {
    Area,
}
impl Leaf for Icon {
    type SystemHook = SystemHook;

    fn config(elm_configuration: &mut ElmConfiguration) {
        use bevy_ecs::prelude::IntoSystemSetConfigs;
        elm_configuration.configure_hook(SystemHook::Area.in_set(ExternalSet::Resolve));
    }

    fn attach(elm: &mut Elm) {
        differential_enable!(elm, CReprPosition, CReprArea, Color, IconId, MipsLevel);
        elm.job.main().add_systems((scale_change
            .in_set(ExternalSet::Resolve)
            .in_set(SystemHook::Area),));
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
            &mut Position<InterfaceContext>,
            &mut Area<InterfaceContext>,
            &mut MipsLevel,
        ),
        Changed<IconScale>,
    >,
    scale_factor: Res<ScaleFactor>,
) {
    for (scale, mut pos, mut area, mut mips) in query.iter_mut() {
        let initial_px = scale.px();
        area.width = initial_px;
        area.height = initial_px;
        // TODO resolve if use clean scale or not
        let section = Section::new(*pos, *area);
        let adjusted_section = section; //.clean_scale(scale_factor.factor());
        *pos = adjusted_section.position;
        *area = adjusted_section.area;
        // still set mips either way but from scaled
        *mips = MipsLevel::new(
            (Icon::TEXTURE_DIMENSIONS, Icon::TEXTURE_DIMENSIONS).into(),
            Icon::MIPS,
            (adjusted_section.width(), adjusted_section.height()).into(),
        );
    }
}
#[repr(u32)]
#[derive(Component, Copy, Clone)]
pub enum IconScale {
    Twenty = 20,
    Forty = 40,
    Eighty = 80,
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
