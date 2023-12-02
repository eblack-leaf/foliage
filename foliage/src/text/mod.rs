mod font;
mod renderer;

use crate::color::Color;
use crate::coordinate::area::CReprArea;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::InterfaceContext;
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::{Elm, Leaf};
use bevy_ecs::component::Component;
use bytemuck::{Pod, Zeroable};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};

pub struct Text {
    position: Position<InterfaceContext>,
    font_size: DifferentialBundle<FontSize>,
    c_pos: DifferentialBundle<CReprPosition>,
    c_area: DifferentialBundle<CReprArea>,
    color: DifferentialBundle<Color>,
    text_value: DifferentialBundle<TextValue>,
    differentiable: Differentiable,
}
impl Text {
    pub fn new(
        position: Position<InterfaceContext>,
        layer: Layer,
        font_size: FontSize,
        text_value: TextValue,
        color: Color,
    ) -> Self {
        Self {
            position,
            font_size: DifferentialBundle::new(font_size),
            c_pos: DifferentialBundle::new(CReprPosition::default()),
            c_area: DifferentialBundle::new(CReprArea::default()),
            color: DifferentialBundle::new(color),
            text_value: DifferentialBundle::new(text_value),
            differentiable: Differentiable::new::<Self>(layer),
        }
    }
}
impl Leaf for Text {
    fn attach(elm: &mut Elm) {
        differential_enable!(elm, CReprPosition, CReprArea, Color, TextValue, FontSize);
    }
}
#[derive(Component, Copy, Clone, Default, Serialize, Deserialize)]
pub struct FontSize(pub u32);
#[derive(Component, Clone, Default, Serialize, Deserialize)]
pub struct TextValue(pub CompactString);
