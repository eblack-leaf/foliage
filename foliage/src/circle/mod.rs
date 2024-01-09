use bevy_ecs::prelude::{Bundle, Component, SystemSet, With};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::Query;
use bytemuck::{Pod, Zeroable};
use proc_gen::{LOWER_BOUND, STEP, TEXTURE_SIZE, UPPER_BOUND};
use rectangle_pack::{pack_rects, GroupedRectsToPlace, RectToInsert, RectanglePackOk, TargetBin};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::config::{ElmConfiguration, ExternalSet};
use crate::elm::leaf::Leaf;
use crate::elm::Elm;
use crate::texture::factors::{MipsLevel, Progress};

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
    differentiable: Differentiable,
}
#[derive(Copy, Clone, Component)]
pub struct Diameter(pub CoordinateUnit);
impl Diameter {
    pub fn new(r: CoordinateUnit) -> Self {
        // TODO align to nearest 4 value
        Self(r.min(UPPER_BOUND as f32).max(LOWER_BOUND as f32))
    }
    pub fn area(&self) -> Area<InterfaceContext> {
        (self.0, self.0).into()
    }
}

impl Circle {
    pub fn new(style: CircleStyle, diameter: Diameter, color: Color, progress: Progress) -> Self {
        let area = Area::new(diameter.0, diameter.0);
        Self {
            diameter,
            style: DifferentialBundle::new(style),
            color: DifferentialBundle::new(color),
            progress: DifferentialBundle::new(progress),
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
            .add_systems((diameter_set.in_set(SetDescriptor::Area),));
    }
}
fn diameter_set(
    mut query: Query<
        (&mut Diameter, &mut Area<InterfaceContext>),
        (
            Or<(Changed<Area<InterfaceContext>>, Changed<Diameter>)>,
            With<CircleStyle>,
        ),
    >,
) {
    for (mut diameter, mut area) in query.iter_mut() {
        *diameter = Diameter::new(area.width);
        *area = diameter.area();
    }
}

pub(crate) fn placements() -> RectanglePackOk<u32, i32> {
    let mut rects = GroupedRectsToPlace::new();
    for x in (LOWER_BOUND..=UPPER_BOUND).step_by(STEP) {
        rects.push_rect(x, Some(vec!["one"]), RectToInsert::new(x, x, 1));
    }
    let mut bins = BTreeMap::new();
    bins.insert(0, TargetBin::new(TEXTURE_SIZE, TEXTURE_SIZE, 255));
    let placements = pack_rects(
        &rects,
        &mut bins,
        &rectangle_pack::volume_heuristic,
        &rectangle_pack::contains_smallest_box,
    )
    .unwrap();
    placements
}
