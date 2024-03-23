use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::InterfaceContext;
use crate::text::font::MonospacedFont;
use crate::text::{CharacterDimension, FontSize, MaxCharacters};
use crate::window::ScaleFactor;
use bevy_ecs::prelude::{Component, Or};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, Res};
use compact_str::{CompactString, ToCompactString};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub struct Text {
    pub value: TextValue,
    pub max_chars: MaxCharacters,
    pub lines: TextLines,
    pub color: Color,
}
pub struct TextMetrics {
    pub font_size: FontSize,
    pub extent: Area<InterfaceContext>,
    pub character_dimensions: CharacterDimension,
}
impl TextMetrics {
    pub fn new(fs: FontSize, fa: Area<InterfaceContext>, d: CharacterDimension) -> Self {
        Self {
            font_size: fs,
            extent: fa,
            character_dimensions: d,
        }
    }
}
impl Text {
    pub fn new<S: AsRef<str>, MC: Into<MaxCharacters>, L: Into<TextLines>, C: Into<Color>>(
        s: S,
        mc: MC,
        l: L,
        c: C,
    ) -> Self {
        Self {
            value: TextValue::new(s),
            max_chars: mc.into(),
            lines: l.into(),
            color: c.into(),
        }
    }
}
#[derive(Copy, Clone, Component)]
pub struct TextLines(pub u32);
impl From<i32> for TextLines {
    fn from(value: i32) -> Self {
        Self(value as u32)
    }
}
impl From<u32> for TextLines {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Component, Clone)]
pub struct TextValue(pub CompactString);
impl TextValue {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self(s.as_ref().to_compact_string())
    }
}
pub type TextKey = usize;
pub enum Glyph {
    Control,
    Char(GlyphKey),
}
#[derive(Serialize, Deserialize, Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub(crate) struct GlyphKey {
    pub(crate) glyph_index: u16,
    pub(crate) px: u32,
    pub(crate) font_hash: usize,
}
#[derive(Component)]
pub struct TextPlacement(pub HashMap<TextKey, Glyph>);
fn config_to_dimensions(query: Query<()>) {}
fn value_to_buffer(
    mut query: Query<
        (
            &TextValue,
            &MaxCharacters,
            &TextLines,
            &mut TextPlacement,
            &mut Area<InterfaceContext>,
            &mut FontSize,
            &mut TextValueUniqueCharacters,
            &mut CharacterDimension,
        ),
        Or<(
            Changed<TextValue>,
            Changed<MaxCharacters>,
            Changed<TextLines>,
        )>,
    >,
    scale_factor: Res<ScaleFactor>,
    font: Res<MonospacedFont>,
) {
    for (value, mc, lines, mut buffer, mut area, mut font_size, mut unique, mut dims) in
        query.iter_mut()
    {
        let metrics = font.metrics(mc, lines, *area, &scale_factor);
        *font_size = metrics.font_size;
        *dims = metrics.character_dimensions;
        *unique = TextValueUniqueCharacters::new(value);
    }
}
#[derive(Component, Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct TextValueUniqueCharacters(pub(crate) u32);
impl TextValueUniqueCharacters {
    pub(crate) fn new(value: &TextValue) -> Self {
        let mut uc = HashSet::new();
        for ch in value.0.chars() {
            uc.insert(ch);
        }
        Self(uc.len() as u32)
    }
}