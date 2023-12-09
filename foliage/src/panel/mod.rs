use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Component, IntoSystemConfigs, Query, SystemSet};
use bevy_ecs::query::Changed;
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::InterfaceContext;
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::config::{ElmConfiguration, ExternalSet};
use crate::elm::leaf::Leaf;
use crate::elm::Elm;

mod proc_gen;
mod renderer;
mod vertex;

#[repr(C)]
#[derive(Component, Copy, Clone, PartialEq, Default, Pod, Zeroable, Serialize, Deserialize)]
pub struct PanelStyle(pub(crate) f32);

impl PanelStyle {
    pub fn fill() -> Self {
        Self(0.0)
    }
    pub fn ring() -> Self {
        Self(1.0)
    }
}

#[derive(Bundle)]
pub struct Panel {
    position: Position<InterfaceContext>,
    area: Area<InterfaceContext>,
    content: PanelContentArea,
    style: DifferentialBundle<PanelStyle>,
    position_c: DifferentialBundle<CReprPosition>,
    area_c: DifferentialBundle<CReprArea>,
    color: DifferentialBundle<Color>,
    differentiable: Differentiable,
}
#[derive(Component, Copy, Clone, Serialize, Deserialize)]
pub struct PanelContentArea(pub Area<InterfaceContext>);
impl Panel {
    pub fn new(style: PanelStyle, area: Area<InterfaceContext>, color: Color) -> Self {
        Self {
            position: Default::default(),
            area,
            content: PanelContentArea(area),
            style: DifferentialBundle::new(style),
            position_c: DifferentialBundle::new(CReprPosition::default()),
            area_c: DifferentialBundle::new(CReprArea::default()),
            color: DifferentialBundle::new(color),
            differentiable: Differentiable::new::<Self>(Layer::default()),
        }
    }
}
#[derive(SystemSet, Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum SetDescriptor {
    Area,
}
impl Leaf for Panel {
    type SetDescriptor = SetDescriptor;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Resolve, SetDescriptor::Area);
    }

    fn attach(elm: &mut Elm) {
        differential_enable!(elm, CReprPosition, CReprArea, Color, PanelStyle);
        elm.job.main().add_systems((reduce_area
            .in_set(ExternalSet::Resolve)
            .in_set(SetDescriptor::Area),));
    }
}
fn reduce_area(
    mut query: Query<(&mut Area<InterfaceContext>, &PanelContentArea), Changed<PanelContentArea>>,
) {
    for (mut area, content) in query.iter_mut() {
        area.width = (content.0.width - Panel::BASE_CORNER_DEPTH * 2f32).max(0f32);
        area.height = (content.0.height - Panel::BASE_CORNER_DEPTH * 2f32).max(0f32);
    }
}
