mod font;
mod renderer;
mod vertex;

use crate::color::Color;
use crate::coordinate::area::CReprArea;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, DeviceContext, InterfaceContext, NumericalContext};
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::{Elm, Leaf};
use crate::text::font::MonospacedFont;
use crate::text::renderer::TextKey;
use bevy_ecs::component::Component;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct Text {
    position: Position<InterfaceContext>,
    text_value: TextValue,
    font_size: DifferentialBundle<FontSize>,
    c_pos: DifferentialBundle<CReprPosition>,
    c_area: DifferentialBundle<CReprArea>,
    color: DifferentialBundle<Color>,
    text_value_chars: DifferentialBundle<TextValueUniqueCharacters>,
    glyph_adds: DifferentialBundle<GlyphChangeQueue>,
    glyph_removes: DifferentialBundle<GlyphRemoveQueue>,
    glyph_cache: GlyphCache,
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
            text_value_chars: DifferentialBundle::new(TextValueUniqueCharacters::new(&text_value)),
            glyph_adds: DifferentialBundle::new(GlyphChangeQueue::default()),
            glyph_removes: DifferentialBundle::new(GlyphRemoveQueue::default()),
            text_value,
            differentiable: Differentiable::new::<Self>(layer),
            glyph_cache: GlyphCache::default(),
        }
    }
}
impl Leaf for Text {
    fn attach(elm: &mut Elm) {
        differential_enable!(elm, CReprPosition, CReprArea, Color, TextValue, FontSize);
        elm.job.container.insert_resource(MonospacedFont::new(40));
    }
}
#[derive(Component, Copy, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FontSize(pub u32);
impl FontSize {
    pub fn px(&self, scale_factor: CoordinateUnit) -> CoordinateUnit {
        self.0 as CoordinateUnit * scale_factor
    }
}
#[derive(Component, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TextValue(pub CompactString);
#[derive(Component, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct TextValueUniqueCharacters(pub(crate) u32);
impl TextValueUniqueCharacters {
    pub(crate) fn new(value: &TextValue) -> Self {
        Self(value.0.len() as u32)
    }
}
#[derive(Component, Default)]
pub(crate) struct GlyphCache(pub(crate) HashMap<TextKey, Glyph>);
#[derive(Component, Clone, Serialize, Deserialize, PartialEq, Default)]
pub(crate) struct GlyphChangeQueue(pub(crate) Vec<(TextKey, GlyphChange)>);
#[derive(Component, Clone, Serialize, Deserialize, PartialEq, Default)]
pub(crate) struct GlyphRemoveQueue(pub(crate) Vec<TextKey>);
#[derive(Component, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct Glyph {
    pub(crate) character: char,
    pub(crate) section: Section<DeviceContext>,
    pub(crate) color: Color,
}
pub(crate) struct GlyphChange {
    pub(crate) character: Option<char>,
    pub(crate) section: Option<Section<DeviceContext>>,
    pub(crate) color: Option<Color>,
}
