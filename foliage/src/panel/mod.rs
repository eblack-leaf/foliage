use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::InterfaceContext;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::{Elm, Leaf};

mod renderer;
mod vertex;

#[repr(C)]
#[derive(Component, Copy, Clone, PartialEq, Default, Pod, Zeroable, Serialize, Deserialize)]
pub struct PanelStyle(pub(crate) f32);

impl PanelStyle {
    pub fn flat() -> Self {
        Self(0.0)
    }
    pub fn ring() -> Self {
        Self(1.0)
    }
}

#[derive(Bundle)]
pub struct Panel {
    style: DifferentialBundle<PanelStyle>,
    position: DifferentialBundle<Position<InterfaceContext>>,
    area: DifferentialBundle<Area<InterfaceContext>>,
    layer: DifferentialBundle<Layer>,
    color: DifferentialBundle<Color>,
    differentiable: Differentiable,
}

impl Panel {
    pub fn new(
        style: PanelStyle,
        pos: Position<InterfaceContext>,
        area: Area<InterfaceContext>,
        layer: Layer,
        color: Color,
    ) -> Self {
        Self {
            style: DifferentialBundle::new(style),
            position: DifferentialBundle::new(pos),
            area: DifferentialBundle::new(area),
            layer: DifferentialBundle::new(layer),
            color: DifferentialBundle::new(color),
            differentiable: Differentiable::new::<Self>(),
        }
    }
}

impl Leaf for Panel {
    fn attach(elm: &mut Elm) {
        differential_enable!(
            elm,
            Position<InterfaceContext>,
            Area<InterfaceContext>,
            Layer,
            Color,
            PanelStyle
        );
    }
}