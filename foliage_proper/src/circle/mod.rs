use bevy_ecs::prelude::{Bundle, Component, Entity, SystemSet, With};
use bevy_ecs::query::Changed;
use bevy_ecs::system::Query;
use proc_gen::{LOWER_BOUND, STEP, TEXTURE_SIZE, UPPER_BOUND};

use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, InterfaceContext, NumericalContext};
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::config::{ElmConfiguration, ExternalSet};
use crate::elm::leaf::Leaf;
use crate::elm::{Elm, Style};
use crate::texture::factors::{MipsLevel, Progress};

mod proc_gen;
mod renderer;
mod vertex;

#[derive(Bundle, Clone)]
pub struct Circle {
    diameter: Diameter,
    style: DifferentialBundle<Style>,
    color: DifferentialBundle<Color>,
    progress: DifferentialBundle<Progress>,
    differentiable: Differentiable,
}
#[derive(Copy, Clone, Component, Debug)]
pub struct Diameter(pub CoordinateUnit);
const CIRCLE_INTERVAL: CoordinateUnit = 4.0;
impl Diameter {
    pub fn new(r: CoordinateUnit) -> Self {
        let r = r - r % CIRCLE_INTERVAL;
        Self(r.min(UPPER_BOUND as f32).max(LOWER_BOUND as f32).floor())
    }
    pub fn area(&self) -> Area<InterfaceContext> {
        (self.0, self.0).into()
    }
}
#[cfg(test)]
#[test]
fn diameters() {
    let diameter = Diameter::new(36.0);
    assert_eq!(diameter.0, 36.0);
}
impl Circle {
    pub fn new(style: Style, color: Color, progress: Progress) -> Self {
        Self {
            diameter: Diameter::new(20.0),
            style: DifferentialBundle::new(style),
            color: DifferentialBundle::new(color),
            progress: DifferentialBundle::new(progress),
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
    Update,
}
impl Leaf for Circle {
    type SetDescriptor = SetDescriptor;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }

    fn attach(elm: &mut Elm) {
        elm.enable_conditional::<Circle>();
        differential_enable!(
            elm,
            CReprPosition,
            CReprArea,
            Color,
            Style,
            Progress,
            MipsLevel
        );
        use bevy_ecs::prelude::IntoSystemConfigs;
        elm.job
            .main()
            .add_systems((diameter_set.in_set(SetDescriptor::Update),));
    }
}
fn diameter_set(
    mut query: Query<
        (
            Entity,
            &mut Diameter,
            &mut Area<InterfaceContext>,
            &mut Position<InterfaceContext>,
        ),
        (Changed<Area<InterfaceContext>>, With<Style>),
    >,
) {
    tracing::trace!("setting-diameter");
    for (_entity, mut diameter, mut area, mut pos) in query.iter_mut() {
        let initial_dim = area.width.max(area.height);
        *diameter = Diameter::new(initial_dim);
        let val = (initial_dim - diameter.0) / 2f32;
        *pos += Position::new(val, val);
        *area = diameter.area();
    }
}

pub(crate) fn new_placements() -> Vec<(u32, Section<NumericalContext>)> {
    let rects = (LOWER_BOUND..=UPPER_BOUND)
        .step_by(STEP)
        .map(|x| binpack2d::Dimension::with_id(x as isize, x as i32, x as i32, 1))
        .collect::<Vec<binpack2d::Dimension>>();
    let mut bin = binpack2d::bin_new(
        binpack2d::BinType::MaxRects,
        TEXTURE_SIZE as i32,
        TEXTURE_SIZE as i32,
    );
    let (mut inserted, rejected) = bin.insert_list(&rects);
    if !rejected.is_empty() {
        panic!("could not fit all {:?}", rejected)
    }
    let mut r_val = inserted
        .drain(..)
        .map(|i| {
            (
                i.id() as u32,
                Section::new((i.x(), i.y()), (i.width(), i.height())),
            )
        })
        .collect::<Vec<(u32, Section<NumericalContext>)>>();
    r_val.sort_by(|lhs, rhs| lhs.0.partial_cmp(&rhs.0).unwrap());
    r_val
}
#[test]
fn smallest_size() {
    let placements = new_placements();
    for place in placements {
        println!("id: {:?}, rect: {:?}", place.0, place.1);
    }
}
