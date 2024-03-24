use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, DeviceContext, InterfaceContext};
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use crate::elm::leaf::Leaf;
use crate::elm::Elm;
use crate::layout::AspectRatio;
use crate::text::font::MonospacedFont;
use crate::window::ScaleFactor;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Component, IntoSystemConfigs, Or, SystemSet};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, Res};
use compact_str::{CompactString, ToCompactString};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
#[derive(Bundle, Clone)]
pub struct Text {
    pub value: TextValue,
    pub max_chars: MaxCharacters,
    pub lines: TextLines,
    pub color: DifferentialBundle<Color>,
    exceptions: TextColorExceptions,
    tool: TextPlacementTool,
    placement: TextPlacement,
    wrap: TextLineWrap,
    dims: CharacterDimension,
    color_changes: DifferentialBundle<TextColorChanges>,
    glyph_changes: DifferentialBundle<TextGlyphChanges>,
    font_size: DifferentialBundle<FontSize>,
    unique: DifferentialBundle<TextValueUniqueCharacters>,
    differentiable: Differentiable,
}
#[derive(Component, Copy, Clone)]
pub struct CharacterDimension(pub(crate) Area<InterfaceContext>);
impl CharacterDimension {
    pub fn new<A: Into<Area<InterfaceContext>>>(a: A) -> Self {
        Self(a.into())
    }
    pub fn dimensions(&self) -> Area<InterfaceContext> {
        self.0
    }
}
#[derive(SystemSet, Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum SetDescriptor {
    Update,
}
impl Leaf for Text {
    type SetDescriptor = SetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {
        _elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }

    fn attach(elm: &mut Elm) {
        elm.enable_conditional::<Text>();
        differential_enable!(
            elm,
            TextValue,
            FontSize,
            TextValueUniqueCharacters,
            TextGlyphChanges,
            TextColorChanges
        );
        elm.container()
            .insert_resource(MonospacedFont::new(Self::DEFAULT_OPT_SCALE));
        elm.main().add_systems((
            place_text.in_set(SetDescriptor::Update),
            clear_changes.after(CoreSet::Differential),
        ));
    }
}
#[derive(Component, Copy, Clone)]
pub struct MaxCharacters(pub u32);
impl MaxCharacters {
    pub fn mono_aspect(self) -> AspectRatio {
        AspectRatio(self.0 as CoordinateUnit * crate::text::Text::MONOSPACED_ASPECT)
    }
    pub fn new(v: u32) -> Self {
        Self(v)
    }
}
impl From<i32> for MaxCharacters {
    fn from(value: i32) -> Self {
        Self(value as u32)
    }
}
impl From<u32> for MaxCharacters {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
pub struct TextMetrics {
    pub font_size: FontSize,
    pub area: Area<InterfaceContext>,
    pub character_dimensions: CharacterDimension,
}
impl TextMetrics {
    pub fn new(fs: FontSize, fa: Area<InterfaceContext>, d: CharacterDimension) -> Self {
        Self {
            font_size: fs,
            area: fa,
            character_dimensions: d,
        }
    }
}
#[derive(Component, Copy, Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct FontSize(pub u32);
impl FontSize {
    pub fn px(&self, scale_factor: CoordinateUnit) -> CoordinateUnit {
        self.0 as CoordinateUnit * scale_factor
    }
}
impl Text {
    pub const MONOSPACED_ASPECT: f32 = 0.45;
    pub fn aspect_ratio_for<MC: Into<MaxCharacters>>(mc: MC) -> AspectRatio {
        mc.into().mono_aspect()
    }
    pub const DEFAULT_OPT_SCALE: u32 = 40;
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
            color: DifferentialBundle::new(c.into()),
            exceptions: TextColorExceptions::blank(),
            tool: TextPlacementTool::default(),
            placement: TextPlacement::default(),
            wrap: TextLineWrap::Word,
            dims: CharacterDimension(Area::default()),
            color_changes: DifferentialBundle::new(TextColorChanges::default()),
            glyph_changes: DifferentialBundle::new(TextGlyphChanges::default()),
            font_size: DifferentialBundle::new(FontSize::default()),
            unique: DifferentialBundle::new(TextValueUniqueCharacters::default()),
            differentiable: Differentiable::new::<Self>(),
        }
    }
    pub fn with_letter_wrap(mut self) -> Self {
        self.wrap = TextLineWrap::Letter;
        self
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
#[derive(PartialEq, Clone, Debug)]
pub enum Glyph {
    Control,
    Char(CharGlyph),
}
#[derive(PartialEq, Clone)]
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
#[derive(Component, Default)]
pub struct TextPlacement(pub HashMap<TextKey, Glyph>);
pub struct CachedTextPlacement(pub HashMap<TextKey, Glyph>);
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
impl Default for TextPlacementTool {
    fn default() -> Self {
        Self(fontdue::layout::Layout::new(
            fontdue::layout::CoordinateSystem::PositiveYDown,
        ))
    }
}
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
fn place_text(
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
            Changed<Area<InterfaceContext>>,
            Changed<TextLineWrap>,
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
        let metrics = font.line_metrics(mc, lines, *area, &scale_factor);
        *font_size = metrics.font_size;
        *dims = metrics.character_dimensions;
        let aligned_area = metrics.area; // TODO make fit bounds better
        *area = aligned_area;
        tool.configure(*area, *wrap);
        let limited = value.limited(*mc);
        *unique = TextValueUniqueCharacters::new(limited);
        tool.place(&font, limited, metrics.font_size, &scale_factor);
        *placement = tool.placed_glyphs();
    }
}
#[derive(Component, Copy, Clone, Serialize, Deserialize, PartialEq, Debug, Default)]
pub(crate) struct TextValueUniqueCharacters(pub(crate) u32);
impl TextValueUniqueCharacters {
    pub(crate) fn new(value: &str) -> Self {
        let mut uc = HashSet::new();
        for ch in value.chars() {
            uc.insert(ch);
        }
        Self(uc.len() as u32)
    }
}
#[derive(Component)]
pub struct TextColorExceptions {
    pub exceptions: HashMap<TextKey, Color>,
}
impl TextColorExceptions {
    pub fn blank() -> Self {
        Self {
            exceptions: HashMap::new(),
        }
    }
    pub fn with<C: Into<Color>>(mut self, key: TextKey, c: C) -> Self {
        self.exceptions.insert(key, c.into());
        self
    }
    pub fn with_range<C: Into<Color>>(mut self, start: TextKey, end: TextKey, c: C) -> Self {
        let color = c.into();
        for i in start..=end {
            self.exceptions.insert(i, color);
        }
        self
    }
}
#[derive(Component, Clone, Default, PartialEq)]
pub(crate) struct TextColorChanges(pub HashMap<TextKey, Color>);
#[derive(Component, Clone, Default, PartialEq)]
pub(crate) struct TextGlyphChanges {
    pub(crate) added: HashMap<TextKey, Glyph>,
    pub(crate) removed: HashMap<TextKey, Glyph>,
}
impl TextGlyphChanges {
    pub(crate) fn clear(&mut self) {
        self.added.clear();
        self.removed.clear();
    }
}
fn clear_changes(
    mut query: Query<
        (&mut TextGlyphChanges, &mut TextColorChanges),
        Or<(Changed<TextGlyphChanges>, Changed<TextColorChanges>)>,
    >,
) {
    for (mut a, mut b) in query.iter_mut() {
        a.clear();
        b.0.clear();
    }
}
