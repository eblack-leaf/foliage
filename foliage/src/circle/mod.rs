use bevy_ecs::prelude::{Bundle, Component, IntoSystemConfigs, Res};
use bevy_ecs::query::Changed;
use bevy_ecs::system::Query;
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::differential::{Differentiable, DifferentialBundle};
use crate::elm::{Elm, Leaf, SystemSets};
use crate::texture::factors::{MipsLevel, Progress};
use crate::window::ScaleFactor;
use crate::{coordinate, differential_enable};

mod proc_gen;
mod renderer;
mod vertex;

#[repr(C)]
#[derive(Component, Copy, Clone, PartialEq, Default, Pod, Zeroable, Serialize, Deserialize)]
pub struct CircleStyle(pub(crate) f32);

impl CircleStyle {
    pub fn fill() -> Self {
        Self(0.0)
    }
    pub fn ring() -> Self {
        Self(1.0)
    }
}

#[derive(Bundle)]
pub struct Circle {
    position: Position<InterfaceContext>,
    area: Area<InterfaceContext>,
    diameter: Diameter,
    style: DifferentialBundle<CircleStyle>,
    position_c: DifferentialBundle<CReprPosition>,
    area_c: DifferentialBundle<CReprArea>,
    color: DifferentialBundle<Color>,
    progress: DifferentialBundle<Progress>,
    mips: DifferentialBundle<MipsLevel>,
    differentiable: Differentiable,
}
#[derive(Copy, Clone, Component)]
pub struct Diameter(pub CoordinateUnit);
#[derive(Copy, Clone)]
pub enum CircleMipLevel {
    Twelve = 12,
    TwentyFour = 24,
    FortyEight = 48,
    NinetySix = 96,
    OneNinetyTwo = 192,
    ThreeEightyFour = 384,
    SevenSixtyEight = 768,
    Full = 1536,
}
impl Diameter {
    pub const MAX: CoordinateUnit = Circle::CIRCLE_TEXTURE_DIMENSIONS as CoordinateUnit;
    pub fn new(r: CoordinateUnit) -> Self {
        Self(r.min(Self::MAX).max(0f32))
    }
    pub fn from_mip_level(l: CircleMipLevel) -> Self {
        Self::new(l as i32 as CoordinateUnit)
    }
}

impl Circle {
    const CIRCLE_TEXTURE_DIMENSIONS: u32 = 1536;
    #[allow(unused)]
    const MIPS_TARGETS: [u32; Self::MIPS as usize] = [1536, 768, 384, 192, 96, 48, 24, 12];
    const MIPS: u32 = 8;
    pub fn new(
        style: CircleStyle,
        position: Position<InterfaceContext>,
        diameter: Diameter,
        layer: Layer,
        color: Color,
        progress: Progress,
    ) -> Self {
        let area = Area::new(diameter.0, diameter.0);
        Self {
            position,
            area,
            diameter,
            style: DifferentialBundle::new(style),
            position_c: DifferentialBundle::new(CReprPosition::default()),
            area_c: DifferentialBundle::new(CReprArea::default()),
            color: DifferentialBundle::new(color),
            progress: DifferentialBundle::new(progress),
            mips: DifferentialBundle::new(MipsLevel::default()),
            differentiable: Differentiable::new::<Self>(layer),
        }
    }
}

impl Leaf for Circle {
    fn attach(elm: &mut Elm) {
        differential_enable!(
            elm,
            CReprPosition,
            CReprArea,
            Color,
            CircleStyle,
            Progress,
            MipsLevel
        );
        elm.job.main().add_systems((
            mips_adjust.before(SystemSets::Differential),
            diameter_forward.before(coordinate::area_set),
        ));
    }
}
fn diameter_forward(mut query: Query<(&mut Area<InterfaceContext>, &Diameter), Changed<Diameter>>) {
    for (mut area, diameter) in query.iter_mut() {
        area.width = diameter.0;
        area.height = diameter.0;
    }
}
fn mips_adjust(
    mut query: Query<(&mut MipsLevel, &Area<InterfaceContext>)>,
    scale_factor: Res<ScaleFactor>,
) {
    for (mut mips, area) in query.iter_mut() {
        *mips = MipsLevel::new(
            (
                Circle::CIRCLE_TEXTURE_DIMENSIONS,
                Circle::CIRCLE_TEXTURE_DIMENSIONS,
            )
                .into(),
            Circle::MIPS,
            area.to_device(scale_factor.factor()),
        );
    }
}
