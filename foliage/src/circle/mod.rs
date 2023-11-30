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
use crate::texture::{MipsLevel, Progress};
use crate::window::ScaleFactor;
use crate::{coordinate, differential_enable};

mod progress;
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
    Eight = 8,
    Sixteen = 16,
    ThirtyTwo = 32,
    SixtyFour = 64,
    OneTwentyEight = 128,
    TwoFiftySix = 256,
    FiveTwelve = 512,
    Full = 1024,
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
    const CIRCLE_TEXTURE_DIMENSIONS: u32 = 1024;
    #[allow(unused)]
    const MIPS_TARGETS: [u32; Self::MIPS as usize] = [1024, 512, 256, 128, 64, 32, 16, 8];
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

#[test]
fn coverage_maps() {
    use crate::ginkgo::Ginkgo;
    use std::path::Path;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("circle")
        .join("texture_resources");
    for mip in Circle::MIPS_TARGETS {
        Ginkgo::png_to_cov(
            root.join(format!("circle-ring-{}.png", mip)),
            root.join(format!("circle-ring-texture-{}.cov", mip)),
        );
        Ginkgo::png_to_cov(
            root.join(format!("circle-{}.png", mip)),
            root.join(format!("circle-texture-{}.cov", mip)),
        );
    }
}