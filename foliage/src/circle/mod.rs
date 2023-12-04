use bevy_ecs::prelude::{Bundle, Component, IntoSystemConfigs, Res, With};
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
use crate::differential_enable;
use crate::elm::{Elm, Leaf, SystemSets};
use crate::texture::factors::{MipsLevel, Progress};
use crate::window::ScaleFactor;

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
        elm.job
            .main()
            .add_systems((mips_adjust.in_set(SystemSets::Resolve),));
    }
}
fn mips_adjust(
    mut query: Query<
        (
            &Diameter,
            &mut MipsLevel,
            &mut Position<InterfaceContext>,
            &mut Area<InterfaceContext>,
        ),
        (Changed<Area<InterfaceContext>>, With<CircleStyle>),
    >,
    scale_factor: Res<ScaleFactor>,
) {
    for (diameter, mut mips, mut pos, mut area) in query.iter_mut() {
        area.width = diameter.0;
        area.height = diameter.0;
        let scaled_px = diameter.0 * scale_factor.factor();
        let clean_scaled_px = diameter.0 * scale_factor.factor().round();
        let scaled_diff = clean_scaled_px - scaled_px;
        let diff = scaled_diff / scale_factor.factor();
        let quarter_diff = diff / 4f32;
        pos.x -= quarter_diff;
        pos.y -= quarter_diff;
        area.width += quarter_diff;
        area.height += quarter_diff;
        *mips = MipsLevel::new(
            (
                Circle::CIRCLE_TEXTURE_DIMENSIONS,
                Circle::CIRCLE_TEXTURE_DIMENSIONS,
            )
                .into(),
            Circle::MIPS,
            (clean_scaled_px, clean_scaled_px).into(),
        );
    }
}
