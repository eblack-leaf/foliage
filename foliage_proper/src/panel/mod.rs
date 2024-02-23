use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Component, IntoSystemConfigs, Query, SystemSet};
use bevy_ecs::query::{Changed, With};
use serde::{Deserialize, Serialize};

use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::InterfaceContext;
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::config::{ElmConfiguration, ExternalSet};
use crate::elm::leaf::{Leaf, Tag};
use crate::elm::{ElementStyle, Elm};

mod proc_gen;
mod renderer;
mod vertex;

#[derive(Bundle, Clone)]
pub struct Panel {
    tag: Tag<Self>,
    style: DifferentialBundle<ElementStyle>,
    color: DifferentialBundle<Color>,
    differentiable: Differentiable,
}
#[derive(Component, Copy, Clone, Serialize, Deserialize)]
pub struct PanelContentArea(pub Area<InterfaceContext>);
impl Panel {
    pub fn new(style: ElementStyle, color: Color) -> Self {
        Self {
            tag: Tag::new(),
            style: DifferentialBundle::new(style),
            color: DifferentialBundle::new(color),
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
impl Leaf for Panel {
    type SetDescriptor = SetDescriptor;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }

    fn attach(elm: &mut Elm) {
        differential_enable!(elm, Color, ElementStyle);
        elm.job
            .main()
            .add_systems((reduce_area.in_set(SetDescriptor::Update),));
    }
}
fn reduce_area(
    mut query: Query<
        &mut Area<InterfaceContext>,
        (Changed<Area<InterfaceContext>>, With<Tag<Panel>>),
    >,
) {
    tracing::trace!("updating-panels");
    for mut area in query.iter_mut() {
        area.width = (area.width - Panel::BASE_CORNER_DEPTH * 2f32).max(0f32);
        area.height = (area.height - Panel::BASE_CORNER_DEPTH * 2f32).max(0f32);
    }
}
