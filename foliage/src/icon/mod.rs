use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::differential::{Differentiable, DifferentialBundle};
use crate::elm::{Elm, Leaf, SystemSets};
use crate::texture::factors::MipsLevel;
use crate::window::ScaleFactor;
#[allow(unused)]
use crate::{coordinate, differential_enable};
use bevy_ecs::component::Component;
use bevy_ecs::prelude::Query;
#[allow(unused)]
use bevy_ecs::prelude::{Bundle, IntoSystemConfigs};
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
    pub fn new(
        icon_id: IconId,
        position: Position<InterfaceContext>,
        scale: IconScale,
        layer: Layer,
        color: Color,
    ) -> Self {
        Self {
            position,
            area: Area::default(),
            icon_id: DifferentialBundle::new(icon_id),
            c_pos: DifferentialBundle::new(CReprPosition::default()),
            c_area: DifferentialBundle::new(CReprArea::default()),
            color: DifferentialBundle::new(color),
            mips: DifferentialBundle::new(scale.initial_mips()),
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
        *mips = scale.initial_mips();
        // let scaled_px = initial_px * scale_factor.factor();
        // let clean_scaled_px = initial_px * scale_factor.factor().round();
        // let scaled_diff = clean_scaled_px - scaled_px;
        // let diff = scaled_diff / scale_factor.factor();
        // let half_diff = diff / 2f32;
        // if diff.is_sign_negative() {
        //     pos.x -= half_diff;
        //     pos.y -= half_diff;
        //     area.width += half_diff;
        //     area.height += half_diff;
        // } else {
        //     pos.x += half_diff;
        //     pos.y += half_diff;
        //     area.width -= half_diff;
        //     area.height -= half_diff;
        // }
        // *mips = MipsLevel::new(
        //     (Icon::TEXTURE_DIMENSIONS, Icon::TEXTURE_DIMENSIONS).into(),
        //     Icon::MIPS,
        //     (clean_scaled_px, clean_scaled_px).into(),
        // );
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
}