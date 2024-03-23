use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::section::Section;
use crate::coordinate::{DeviceContext, InterfaceContext};
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
    pub fn limited<MC: Into<MaxCharacters>>(&self, mc: MC) -> &str {
        let max_chars = mc.into();
        &self.0.as_str()[0..max_chars.0.min(self.0.len() as u32) as usize]
    }
}
impl From<String> for TextValue {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
impl From<&str> for TextValue {
    fn from(value: &str) -> Self {
        TextValue::new(value)
    }
}
pub type TextKey = usize;
pub enum Glyph {
    Control,
    Char(CharGlyph),
}
pub struct CharGlyph {
    pub key: GlyphKey,
    pub section: Section<DeviceContext>,
    pub parent: char,
}
impl CharGlyph {
    pub fn new(key: GlyphKey, section: Section<DeviceContext>, parent: char) -> Self {
        Self {
            key,
            section,
            parent,
        }
    }
}
#[derive(Serialize, Deserialize, Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub(crate) struct GlyphKey {
    pub(crate) glyph_index: u16,
    pub(crate) px: u32,
    pub(crate) font_hash: usize,
}
impl GlyphKey {
    pub(crate) fn new(raster_config: fontdue::layout::GlyphRasterConfig) -> Self {
        Self {
            glyph_index: raster_config.glyph_index,
            px: raster_config.px as u32,
            font_hash: raster_config.font_hash,
        }
    }
}
#[derive(Component)]
pub struct TextPlacement(pub HashMap<TextKey, Glyph>);
impl TextPlacement {}
#[derive(Copy, Clone, Component)]
pub enum TextLineWrap {
    Letter,
    Word,
}
impl TextLineWrap {
    fn to_fontdue(&self) -> fontdue::layout::WrapStyle {
        match self {
            TextLineWrap::Letter => fontdue::layout::WrapStyle::Letter,
            TextLineWrap::Word => fontdue::layout::WrapStyle::Word,
        }
    }
}
#[derive(Component)]
pub struct TextPlacementTool(fontdue::layout::Layout);
impl TextPlacementTool {
    pub fn configure(&mut self, area: Area<InterfaceContext>, wrap: TextLineWrap) {
        self.0.reset(&fontdue::layout::LayoutSettings {
            max_width: Some(area.width),
            max_height: Some(area.height),
            wrap_style: wrap.to_fontdue(),
            ..fontdue::layout::LayoutSettings::default()
        });
    }
    pub fn place(
        &mut self,
        font: &MonospacedFont,
        value: &str,
        size: FontSize,
        scale_factor: &ScaleFactor,
    ) {
        self.0.append(
            &[&font.0],
            &fontdue::layout::TextStyle::new(value, size.px(scale_factor.factor()), 0),
        );
    }
    pub fn placed_glyphs(&self) -> TextPlacement {
        let mut mapping = HashMap::new();
        for g in self.0.glyphs() {
            let glyph = if g.parent.is_ascii_control() {
                Glyph::Control
            } else {
                Glyph::Char(CharGlyph::new(
                    GlyphKey::new(g.key),
                    Section::new((g.x, g.y), (g.width, g.height)),
                    g.parent,
                ))
            };
            mapping.insert(g.byte_offset, glyph);
        }
        TextPlacement(mapping)
    }
}
fn value_to_buffer(
    mut query: Query<
        (
            &TextValue,
            &MaxCharacters,
            &TextLines,
            &TextLineWrap,
            &mut TextPlacement,
            &mut Area<InterfaceContext>,
            &mut FontSize,
            &mut TextValueUniqueCharacters,
            &mut CharacterDimension,
            &mut TextPlacementTool,
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
    for (
        value,
        mc,
        lines,
        wrap,
        mut placement,
        mut area,
        mut font_size,
        mut unique,
        mut dims,
        mut tool,
    ) in query.iter_mut()
    {
        let metrics = font.metrics(mc, lines, *area, &scale_factor);
        *font_size = metrics.font_size;
        *dims = metrics.character_dimensions;
        *unique = TextValueUniqueCharacters::new(value);
        tool.configure(*area, *wrap);
        tool.place(&font, value.limited(*mc), metrics.font_size, &scale_factor);
        *placement = tool.placed_glyphs();
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