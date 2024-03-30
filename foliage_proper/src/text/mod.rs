use std::collections::{HashMap, HashSet};

use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::Res;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Changed, IntoSystemConfigs, Or, Query, SystemSet};
use compact_str::{CompactString, ToCompactString};
use serde::{Deserialize, Serialize};

use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
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

pub mod font;
mod renderer;
mod vertex;

#[derive(Bundle, Clone)]
pub struct Text {
    pub value: TextValue,
    pub max_chars: MaxCharacters,
    pub lines: TextLineStructure,
    pub color: DifferentialBundle<Color>,
    exceptions: TextColorExceptions,
    tool: TextPlacementTool,
    placement: TextPlacement,
    cached: CachedTextPlacement,
    dims: CharacterDimension,
    color_changes: DifferentialBundle<TextColorChanges>,
    glyph_changes: DifferentialBundle<TextGlyphChanges>,
    font_size: DifferentialBundle<FontSize>,
    unique: DifferentialBundle<TextValueUniqueCharacters>,
    line_structure: TextLinePlacement,
    offset: TextOffset,
    differentiable: Differentiable,
}
impl Text {
    pub const MONOSPACED_ASPECT: f32 = 0.45;
    pub fn aspect_ratio_for<MC: Into<MaxCharacters>>(mc: MC) -> AspectRatio {
        mc.into().mono_aspect()
    }
    pub const DEFAULT_OPT_SCALE: u32 = 40;
    pub fn new<S: Into<TextValue>, TLS: Into<TextLineStructure>, C: Into<Color>>(
        s: S,
        tls: TLS,
        c: C,
    ) -> Self {
        let lines = tls.into();
        Self {
            value: s.into(),
            max_chars: lines.max_chars(),
            lines,
            color: DifferentialBundle::new(c.into()),
            exceptions: TextColorExceptions::blank(),
            tool: TextPlacementTool::default(),
            placement: TextPlacement::default(),
            cached: CachedTextPlacement::default(),
            dims: CharacterDimension(Area::default()),
            color_changes: DifferentialBundle::new(TextColorChanges::default()),
            glyph_changes: DifferentialBundle::new(TextGlyphChanges::default()),
            font_size: DifferentialBundle::new(FontSize::default()),
            unique: DifferentialBundle::new(TextValueUniqueCharacters::default()),
            line_structure: TextLinePlacement::default(),
            offset: TextOffset(Position::default()),
            differentiable: Differentiable::new::<Self>(),
        }
    }
}
#[derive(Component, Copy, Clone, Default)]
pub struct TextLineStructure {
    pub lines: u32,
    pub per_line: u32,
}
#[cfg(test)]
#[test]
fn text_line_structure_test() {
    let tls = TextLineStructure::new(5, 1);
    let next = tls.next_location(TextLineLocation::raw(0, 0), 1);
    assert_eq!(next, TextLineLocation::raw(1, 0));
    let next = tls.next_location(TextLineLocation::raw(1, 0), 1);
    assert_eq!(next, TextLineLocation::raw(2, 0));
    let next = tls.next_location(TextLineLocation::raw(2, 0), 1);
    assert_eq!(next, TextLineLocation::raw(3, 0));
    let next = tls.next_location(TextLineLocation::raw(3, 0), 1);
    assert_eq!(next, TextLineLocation::raw(4, 0));
    let next = tls.next_location(TextLineLocation::raw(4, 0), 1);
    assert_eq!(next, TextLineLocation::raw(4, 0));
    let tls = TextLineStructure::new(15, 3);
    let next = tls.next_location(TextLineLocation::raw(0, 0), -1);
    assert_eq!(next, TextLineLocation::raw(0, 0));
    let next = tls.next_location(TextLineLocation::raw(12, 0), 5);
    assert_eq!(next, TextLineLocation::raw(2, 1));
    let next = tls.next_location(TextLineLocation::raw(12, 0), 25);
    assert_eq!(next, TextLineLocation::raw(7, 2));
    let next = tls.next_location(TextLineLocation::raw(12, 0), 36);
    assert_eq!(next, TextLineLocation::raw(14, 2));
    let next = tls.next_location(TextLineLocation::raw(0, 1), -1);
    assert_eq!(next, TextLineLocation::raw(14, 0));
    let next = tls.next_location(TextLineLocation::raw(0, 1), -5);
    assert_eq!(next, TextLineLocation::raw(10, 0));
    let next = tls.next_location(TextLineLocation::raw(10, 2), -26);
    assert_eq!(next, TextLineLocation::raw(14, 0));
}
impl TextLineStructure {
    pub fn new(per_line: u32, lines: u32) -> Self {
        Self { lines, per_line }
    }
    pub fn per_line_index(&self) -> u32 {
        self.per_line.checked_sub(1).unwrap_or_default()
    }
    pub fn lines_index(&self) -> u32 {
        self.lines.checked_sub(1).unwrap_or_default()
    }
    pub fn last(&self) -> TextLineLocation {
        TextLineLocation::raw(self.per_line_index(), self.lines_index())
    }
    pub fn next_location(&self, location: TextLineLocation, skip_amount: i32) -> TextLineLocation {
        let projected = location.0 as i32 + skip_amount;
        return if projected.is_negative() {
            let overflow = projected.abs();
            let line_skip = overflow / self.per_line as i32;
            let horizontal = (projected + self.per_line as i32) + self.per_line as i32 * line_skip;
            let vertical = location.1 as i32 - line_skip - 1;
            if vertical.is_negative() {
                return TextLineLocation::raw(0, 0);
            }
            TextLineLocation::raw(horizontal as u32, (vertical as u32).min(self.lines_index()))
        } else {
            if projected >= self.per_line as i32 {
                let overflow = projected - self.per_line as i32;
                let line_skip = overflow / self.per_line as i32;
                let horizontal = overflow % self.per_line as i32;
                let vertical = location.1 as i32 + line_skip + 1;
                if vertical as u32 > self.lines_index() {
                    return self.last();
                }
                return TextLineLocation::raw(
                    horizontal as u32,
                    (vertical as u32).min(self.lines_index()),
                );
            }
            TextLineLocation::raw(
                (projected.max(0) as u32).min(self.per_line_index()),
                location.1.min(self.lines_index()),
            )
        };
    }
    pub fn letter(&self, l: TextLineLocation) -> TextKey {
        (self.per_line * l.1 + l.0) as TextKey
    }
    pub fn max_chars(&self) -> MaxCharacters {
        (self.lines * self.per_line).into()
    }
    pub fn with_lines(mut self, l: u32) -> Self {
        self.lines = l;
        self
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Debug, Default)]
pub struct TextLineLocation(pub u32, pub u32);

impl TextLineLocation {
    pub fn raw(x: u32, y: u32) -> TextLineLocation {
        Self(x, y)
    }
}

impl TextLineLocation {
    pub fn new(c: Position<InterfaceContext>, b: Area<InterfaceContext>) -> Self {
        let a = c / Position::new(b.width, b.height);
        let horizontal = a.x.floor().max(0.0) as u32;
        let vertical = a.y.floor().max(0.0) as u32;
        TextLineLocation(horizontal, vertical)
    }
}
#[derive(Component, Copy, Clone)]
pub struct TextOffset(pub Position<InterfaceContext>);
#[derive(Component, Clone, Default)]
pub struct TextLinePlacement(pub HashMap<TextLineLocation, TextKey>);
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
            (place_text, distill_changes, color_diff)
                .chain()
                .in_set(SetDescriptor::Update),
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
    pub max_chars: MaxCharacters,
}
impl TextMetrics {
    pub fn new(
        fs: FontSize,
        fa: Area<InterfaceContext>,
        d: CharacterDimension,
        mc: MaxCharacters,
    ) -> Self {
        Self {
            font_size: fs,
            area: fa,
            character_dimensions: d,
            max_chars: mc,
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
#[derive(Component, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextValue(pub CompactString);

impl TextValue {
    pub fn is_letter(&self, i: usize) -> bool {
        let exists = self.0.get(i..i + 1).is_some();
        if exists {
            self.0
                .get(i..i + 1)
                .unwrap()
                .chars()
                .map(|c| c.is_control() as i32)
                .sum::<i32>()
                == 0
        } else {
            false
        }
    }
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self(
            s.as_ref()
                .chars()
                .filter(|c| !c.is_control())
                .collect::<String>()
                .to_compact_string(),
        )
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

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub(crate) enum Glyph {
    Control,
    Char(CharGlyph),
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct CharGlyph {
    pub(crate) key: GlyphKey,
    pub(crate) section: Section<DeviceContext>,
    pub(crate) parent: char,
}

impl CharGlyph {
    pub(crate) fn new(key: GlyphKey, section: Section<DeviceContext>, parent: char) -> Self {
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

#[derive(Component, Default, Clone)]
pub(crate) struct TextPlacement(pub HashMap<TextKey, Glyph>);
#[derive(Component, Default, Clone)]
pub(crate) struct CachedTextPlacement(pub HashMap<TextKey, Glyph>);

impl TextPlacement {
    pub(crate) fn glyphs(&self) -> &HashMap<TextKey, Glyph> {
        &self.0
    }
}

#[derive(Component)]
pub(crate) struct TextPlacementTool(fontdue::layout::Layout);
impl Clone for TextPlacementTool {
    fn clone(&self) -> Self {
        Self::default()
    }
}
impl Default for TextPlacementTool {
    fn default() -> Self {
        Self(fontdue::layout::Layout::new(
            fontdue::layout::CoordinateSystem::PositiveYDown,
        ))
    }
}

impl TextPlacementTool {
    pub(crate) fn configure(&mut self, area: Area<InterfaceContext>) {
        self.0.reset(&fontdue::layout::LayoutSettings {
            max_width: Some(area.width),
            max_height: Some(area.height),
            wrap_style: fontdue::layout::WrapStyle::Letter,
            ..fontdue::layout::LayoutSettings::default()
        });
    }
    pub(crate) fn place(
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
    pub(crate) fn placed_glyphs(&self) -> TextPlacement {
        let mut mapping = HashMap::new();
        for g in self.0.glyphs() {
            let glyph = if g.parent.is_ascii_control() {
                continue;
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
            &TextLineStructure,
            &mut MaxCharacters,
            &mut TextPlacement,
            &mut Area<InterfaceContext>,
            &mut FontSize,
            &mut TextValueUniqueCharacters,
            &mut CharacterDimension,
            &mut TextPlacementTool,
            &mut TextLinePlacement,
            &mut Position<InterfaceContext>,
            &mut TextOffset,
        ),
        Or<(
            Changed<TextValue>,
            Changed<MaxCharacters>,
            Changed<TextLineStructure>,
            Changed<Area<InterfaceContext>>,
        )>,
    >,
    scale_factor: Res<ScaleFactor>,
    font: Res<MonospacedFont>,
) {
    for (
        value,
        structure,
        mut mc,
        mut placement,
        mut area,
        mut font_size,
        mut unique,
        mut dims,
        mut tool,
        mut line_placement,
        mut pos,
        mut offset,
    ) in query.iter_mut()
    {
        let metrics = font.line_metrics(structure, *area, &scale_factor);
        *mc = metrics.max_chars;
        *font_size = metrics.font_size;
        *dims = metrics.character_dimensions;
        let aligned_area = metrics.area; // TODO make fit bounds better
        if aligned_area < *area {
            let diff = (*area - aligned_area) / Area::new(2.0, 2.0);
            let o = Position::new(diff.width, diff.height);
            offset.0 = o;
            *pos = *pos + o;
        }
        *area = aligned_area;
        tool.configure(*area);
        let limited = value.limited(*mc);
        *unique = TextValueUniqueCharacters::new(limited);
        tool.place(&font, limited, metrics.font_size, &scale_factor);
        *placement = tool.placed_glyphs();
        for g in placement.0.iter() {
            match g.1 {
                Glyph::Control => {}
                Glyph::Char(ch) => {
                    let line_location = TextLineLocation::new(
                        ch.section.position.to_interface(1.0),
                        dims.dimensions(),
                    );
                    line_placement.0.insert(line_location, *g.0);
                }
            }
        }
    }
}
fn distill_changes(
    mut query: Query<
        (
            &TextPlacement,
            &mut CachedTextPlacement,
            &mut TextGlyphChanges,
        ),
        Changed<TextPlacement>,
    >,
) {
    for (placement, mut cached, mut changes) in query.iter_mut() {
        for (tk, g) in placement.glyphs().iter() {
            if let Some(old) = cached.0.remove(tk) {
                if &old != g {
                    changes.removed.insert(*tk, old.clone());
                    changes.added.insert(*tk, g.clone());
                }
            } else {
                changes.added.insert(*tk, g.clone());
            }
        }
        for (tk, g) in cached.0.drain() {
            changes.removed.insert(tk, g);
        }
        cached.0 = placement.0.clone();
    }
}
fn color_diff(
    mut query: Query<
        (
            &Color,
            &TextColorExceptions,
            &mut TextColorChanges,
            &TextPlacement,
        ),
        Or<(
            Changed<TextColorExceptions>,
            Changed<Color>,
            Changed<TextPlacement>,
        )>,
    >,
) {
    for (color, excepts, mut changes, placement) in query.iter_mut() {
        for (tk, _) in placement.glyphs().iter() {
            changes.0.insert(*tk, *color);
        }
        for (tk, c) in excepts.exceptions.iter() {
            changes.0.insert(*tk, *c);
        }
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

#[derive(Component, Clone, Default)]
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

#[derive(Component, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct TextColorChanges(pub HashMap<TextKey, Color>);

#[derive(Component, Clone, Default, PartialEq, Serialize, Deserialize)]
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
