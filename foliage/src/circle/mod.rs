use bevy_ecs::prelude::{Bundle, Component, IntoSystemConfigs};
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
use crate::texture::MipsLevel;

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
#[repr(C)]
#[derive(Component, Copy, Clone, PartialEq, Default, Pod, Zeroable, Serialize, Deserialize)]
pub struct Progress(pub f32, pub f32);
impl Progress {
    pub fn full() -> Self {
        Self(0.0, 1.0)
    }
    pub fn empty() -> Self {
        Self(0.0, 0.0)
    }
    pub fn new(start: f32, end: f32) -> Self {
        Self(start, end)
    }
}
#[derive(Bundle)]
pub struct Circle {
    position: Position<InterfaceContext>,
    area: Area<InterfaceContext>,
    style: DifferentialBundle<CircleStyle>,
    position_c: DifferentialBundle<CReprPosition>,
    area_c: DifferentialBundle<CReprArea>,
    color: DifferentialBundle<Color>,
    progress: DifferentialBundle<Progress>,
    mips: DifferentialBundle<MipsLevel>,
    differentiable: Differentiable,
}
#[derive(Copy, Clone)]
pub struct Diameter(pub CoordinateUnit);
#[derive(Copy, Clone)]
pub enum CircleMipLevel {
    Five = 32,
    Four = 64,
    Three = 128,
    Two = 256,
    One = 512,
    Zero = 1024,
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
    const MIPS_TARGETS: [u32; Self::MIPS as usize] = [1024, 512, 256, 128, 64, 32];
    const MIPS: u32 = 6;
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
            .add_systems((mips_adjust.before(SystemSets::Differential),));
    }
}
fn mips_adjust(mut query: Query<(&mut MipsLevel, &Area<InterfaceContext>)>) {
    for (mut mips, area) in query.iter_mut() {
        *mips = MipsLevel::new(
            (
                Circle::CIRCLE_TEXTURE_DIMENSIONS,
                Circle::CIRCLE_TEXTURE_DIMENSIONS,
            )
                .into(),
            Circle::MIPS,
            area.to_device(1.0),
        );
    }
}

#[test]
fn png() {
    use crate::ginkgo::Ginkgo;
    for mip in Circle::MIPS_TARGETS {
        Ginkgo::png_to_cov(
            format!("/home/salt/Desktop/dev/foliage/foliage/src/circle/texture_resources/circle-ring-{}.png", mip),
            format!("/home/salt/Desktop/dev/foliage/foliage/src/circle/texture_resources/circle-ring-texture-{}.cov", mip),
        );
        Ginkgo::png_to_cov(
            format!("/home/salt/Desktop/dev/foliage/foliage/src/circle/texture_resources/circle-{}.png", mip),
            format!("/home/salt/Desktop/dev/foliage/foliage/src/circle/texture_resources/circle-texture-{}.cov", mip),
        );
    }
}
