use bevy_ecs::prelude::{Bundle, Component, Res, SystemSet, With};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::Query;
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::config::{ElmConfiguration, ExternalSet};
use crate::elm::leaf::Leaf;
use crate::elm::Elm;
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
    diameter: Diameter,
    style: DifferentialBundle<CircleStyle>,
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
    pub fn area(&self) -> Area<InterfaceContext> {
        (self.0, self.0).into()
    }
}

impl Circle {
    const CIRCLE_TEXTURE_DIMENSIONS: u32 = 1536;
    #[allow(unused)]
    const MIPS_TARGETS: [u32; Self::MIPS as usize] = [1536, 768, 384, 192, 96, 48, 24, 12];
    const MIPS: u32 = 8;
    pub fn new(style: CircleStyle, diameter: Diameter, color: Color, progress: Progress) -> Self {
        let area = Area::new(diameter.0, diameter.0);
        Self {
            diameter,
            style: DifferentialBundle::new(style),
            color: DifferentialBundle::new(color),
            progress: DifferentialBundle::new(progress),
            mips: DifferentialBundle::new(MipsLevel::default()),
            differentiable: Differentiable::new::<Self>(
                Position::default(),
                area,
                Layer::default(),
            ),
        }
    }
}
#[derive(SystemSet, Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum SetDescriptor {
    Area,
}
impl Leaf for Circle {
    type SetDescriptor = SetDescriptor;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, SetDescriptor::Area);
    }

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
        use bevy_ecs::prelude::IntoSystemConfigs;
        elm.job
            .main()
            .add_systems((mips_adjust.in_set(SetDescriptor::Area),));
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
        (
            Or<(Changed<Area<InterfaceContext>>, Changed<Diameter>)>,
            With<CircleStyle>,
        ),
    >,
    scale_factor: Res<ScaleFactor>,
) {
    for (diameter, mut mips, mut pos, mut area) in query.iter_mut() {
        *area = diameter.area();
        let section = Section::new(*pos, *area);
        let adjusted_section = section.clean_scale(scale_factor.factor());
        *pos = adjusted_section.position;
        *area = adjusted_section.area;
        *mips = MipsLevel::new(
            (
                Circle::CIRCLE_TEXTURE_DIMENSIONS,
                Circle::CIRCLE_TEXTURE_DIMENSIONS,
            )
                .into(),
            Circle::MIPS,
            (adjusted_section.width(), adjusted_section.height()).into(),
        );
    }
}
