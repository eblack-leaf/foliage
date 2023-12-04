use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::differential::{Differentiable, DifferentialBundle};
use crate::elm::{Elm, Leaf, SystemSets};
use crate::texture::factors::MipsLevel;
#[allow(unused)]
use crate::{coordinate, differential_enable};
use bevy_ecs::component::Component;
use bevy_ecs::prelude::Query;
#[allow(unused)]
use bevy_ecs::prelude::{Bundle, IntoSystemConfigs};
use bevy_ecs::query::Changed;
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
    pub fn new(
        icon_id: IconId,
        position: Position<InterfaceContext>,
        scale: IconScale,
        layer: Layer,
        color: Color,
    ) -> Self {
        Self {
            position,
            area: (scale.px(), scale.px()).into(),
            icon_id: DifferentialBundle::new(icon_id),
            c_pos: DifferentialBundle::new(CReprPosition::default()),
            c_area: DifferentialBundle::new(CReprArea::default()),
            color: DifferentialBundle::new(color),
            mips: DifferentialBundle::new(scale.mips()),
            scale,
            differentiable: Differentiable::new::<Self>(layer),
        }
    }
}
impl Leaf for Icon {
    fn attach(elm: &mut Elm) {
        differential_enable!(elm, CReprPosition, CReprArea, Color, IconId, MipsLevel);
        elm.job
            .main()
            .add_systems((scale_change.in_set(SystemSets::Resolve),));
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
    mut query: Query<(&IconScale, &mut Area<InterfaceContext>, &mut MipsLevel), Changed<IconScale>>,
) {
    for (scale, mut area, mut mips) in query.iter_mut() {
        area.width = scale.px();
        area.height = scale.px();
        *mips = scale.mips();
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
    pub fn mips(self) -> MipsLevel {
        match self {
            IconScale::Twenty => MipsLevel(2.0),
            IconScale::Forty => MipsLevel(1.0),
            IconScale::Eighty => MipsLevel(0.0),
        }
    }
    pub fn px(self) -> CoordinateUnit {
        self as u32 as f32
    }
}
