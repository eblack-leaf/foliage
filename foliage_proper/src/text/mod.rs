pub mod font;
mod glyph;
mod renderer;
mod vertex;
use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, DeviceContext, InterfaceContext};
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use crate::elm::leaf::Leaf;
use crate::elm::{Disabled, Elm};
use crate::text::font::MonospacedFont;
use crate::text::glyph::{CachedGlyph, Glyph};
pub use crate::text::renderer::TextKey;
use crate::window::ScaleFactor;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Bundle, IntoSystemConfigs, Or, SystemSet};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, Res};
use compact_str::CompactString;
use glyph::{
    GlyphCache, GlyphChange, GlyphChangeQueue, GlyphKey, GlyphPlacementTool, GlyphRemoveQueue,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Bundle, Clone)]
pub struct Text {
    text_value: TextValue,
    max_characters: MaxCharacters,
    character_dimension: CharacterDimension,
    font_size: DifferentialBundle<FontSize>,
    color: DifferentialBundle<Color>,
    text_value_chars: DifferentialBundle<TextValueUniqueCharacters>,
    glyph_adds: DifferentialBundle<GlyphChangeQueue>,
    glyph_removes: DifferentialBundle<GlyphRemoveQueue>,
    glyph_cache: GlyphCache,
    glyph_placement_tool: GlyphPlacementTool,
    color_changes: GlyphColorChanges,
    differentiable: Differentiable,
}
impl Text {
    pub fn new<C: Into<Color>>(
        max_characters: MaxCharacters,
        text_value: TextValue,
        color: C,
    ) -> Self {
        Self {
            max_characters,
            font_size: DifferentialBundle::new(FontSize(0)),
            color: DifferentialBundle::new(color.into()),
            text_value_chars: DifferentialBundle::new(TextValueUniqueCharacters::new(&text_value)),
            glyph_adds: DifferentialBundle::new(GlyphChangeQueue::default()),
            glyph_removes: DifferentialBundle::new(GlyphRemoveQueue::default()),
            text_value,
            character_dimension: CharacterDimension(Area::default()),
            differentiable: Differentiable::new::<Self>(
                Position::default(),
                Area::default(),
                Layer::default(),
            ),
            glyph_cache: GlyphCache::default(),
            glyph_placement_tool: GlyphPlacementTool(fontdue::layout::Layout::new(
                fontdue::layout::CoordinateSystem::PositiveYDown,
            )),
            color_changes: GlyphColorChanges::default(),
        }
    }
    pub const DEFAULT_OPT_SCALE: u32 = 40;
}
#[derive(Component, Copy, Clone)]
pub struct MaxCharacters(pub u32);
#[derive(SystemSet, Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum SetDescriptor {
    Area,
}
impl Leaf for Text {
    type SetDescriptor = SetDescriptor;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, SetDescriptor::Area);
    }

    fn attach(elm: &mut Elm) {
        differential_enable!(
            elm,
            CReprPosition,
            CReprArea,
            Color,
            TextValue,
            FontSize,
            TextValueUniqueCharacters,
            GlyphChangeQueue,
            GlyphRemoveQueue
        );
        elm.job
            .container
            .insert_resource(MonospacedFont::new(Self::DEFAULT_OPT_SCALE));
        elm.job.main().add_systems((
            changes.in_set(Self::SetDescriptor::Area),
            max_character
                .before(changes)
                .in_set(Self::SetDescriptor::Area),
            clear_removes.after(CoreSet::Differential),
            clear_changes.after(CoreSet::Differential),
        ));
    }
}
#[derive(Component, Copy, Clone)]
pub struct CharacterDimension(pub(crate) Area<DeviceContext>);
impl CharacterDimension {
    pub fn dimensions(&self) -> Area<DeviceContext> {
        self.0
    }
}
pub(crate) fn max_character(
    mut query: Query<
        (
            &MaxCharacters,
            &mut FontSize,
            &mut Area<InterfaceContext>,
            &mut CharacterDimension,
        ),
        Or<(Changed<MaxCharacters>, Changed<Area<InterfaceContext>>)>,
    >,
    scale_factor: Res<ScaleFactor>,
    font: Res<MonospacedFont>,
) {
    for (max, mut size, mut area, mut dim) in query.iter_mut() {
        let (fs, fa) = font.best_fit(*max, *area, &scale_factor);
        *size = fs;
        *area = fa;
        *dim = CharacterDimension(font.character_dimensions(fs.px(scale_factor.factor())));
    }
}
#[derive(Component, Default, Clone)]
pub struct GlyphColorChanges(pub HashMap<TextKey, Color>);
impl GlyphColorChanges {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_change<C: Into<Color>>(mut self, offset: TextKey, c: C) -> Self {
        self.0.insert(offset, c.into());
        self
    }
    pub fn with_range<C: Into<Color>>(mut self, start: TextKey, end: TextKey, c: C) -> Self {
        let color = c.into();
        for i in start..=end {
            self.0.insert(i, color);
        }
        self
    }
}
pub(crate) fn changes(
    mut query: Query<
        (
            &Area<InterfaceContext>,
            &FontSize,
            &Color,
            &TextValue,
            &MaxCharacters,
            &mut TextValueUniqueCharacters,
            &mut GlyphChangeQueue,
            &mut GlyphRemoveQueue,
            &mut GlyphCache,
            &mut GlyphPlacementTool,
            &GlyphColorChanges,
        ),
        Or<(
            Changed<TextValue>,
            Changed<FontSize>,
            Changed<MaxCharacters>,
            Changed<Color>,
            Changed<GlyphColorChanges>,
            Changed<Disabled>,
        )>,
    >,
    font: Res<MonospacedFont>,
    scale_factor: Res<ScaleFactor>,
) {
    for (
        area,
        font_size,
        color,
        value,
        max_chars,
        mut unique,
        mut changes,
        mut removes,
        mut cache,
        mut placer,
        color_changes,
    ) in query.iter_mut()
    {
        tracing::trace!("updating-text @ {:?}", value.0);
        let scaled = area.to_device(scale_factor.factor());
        placer.0.reset(&fontdue::layout::LayoutSettings {
            max_width: None,
            max_height: Some(scaled.height),
            line_height: MonospacedFont::TEXT_HEIGHT_CORRECTION,
            wrap_style: fontdue::layout::WrapStyle::Letter,
            ..fontdue::layout::LayoutSettings::default()
        });
        let max_char_limited_text_value =
            &value.0.as_str()[0..max_chars.0.min(value.0.len() as u32) as usize];
        *unique = TextValueUniqueCharacters::new(value);
        placer.0.append(
            &[&font.0],
            &fontdue::layout::TextStyle::new(
                max_char_limited_text_value,
                font_size.px(scale_factor.factor()),
                0,
            ),
        );
        let glyphs = placer.0.glyphs();
        if glyphs.is_empty() {
            for cached in cache.0.drain() {
                match cached.1 {
                    CachedGlyph::Present(g) => {
                        removes.0.push((cached.0, g.key));
                    }
                    CachedGlyph::Filtered => {}
                }
            }
        }
        for g in glyphs {
            if g.parent.is_ascii_control() {
                continue;
            }
            let mut change = None;
            let glyph_key = GlyphKey::new(g.key);
            let mut total_update = false;
            // let filtered = g.x + g.width as f32 > scaled.width;
            let filtered = false;
            let mut key_change = Some((glyph_key, None));
            if let Some(cached) = cache.0.get_mut(&g.byte_offset) {
                match cached {
                    CachedGlyph::Present(glyph) => {
                        if filtered {
                            removes.0.push((g.byte_offset, glyph.key));
                            *cached = CachedGlyph::Filtered;
                        } else if glyph.key != glyph_key {
                            key_change.as_mut().unwrap().1.replace(glyph.key);
                            total_update = true;
                        } else {
                            // color change cache check
                            total_update = true;
                        }
                    }
                    CachedGlyph::Filtered => {
                        if !filtered {
                            total_update = true;
                        }
                    }
                }
            } else {
                total_update = true;
            }
            if total_update {
                let section = Section::<DeviceContext>::new((g.x, g.y), (g.width, g.height));
                cache.0.insert(
                    g.byte_offset,
                    CachedGlyph::Present(Glyph {
                        key: glyph_key,
                        section,
                        color: *color_changes.0.get(&g.byte_offset).unwrap_or(color),
                    }),
                );
                tracing::trace!(
                    "updating-glyph {:?} using {:?} -------------------------------------------",
                    g.byte_offset,
                    g.parent
                );
                change.replace(GlyphChange {
                    key: key_change,
                    section: Some(section),
                    color: Some(*color_changes.0.get(&g.byte_offset).unwrap_or(color)),
                });
            }
            if let Some(c) = change {
                changes.0.push((g.byte_offset, c));
            }
        }
        let glyphs_len = glyphs.len();
        let mut removals = vec![];
        if glyphs_len < cache.0.len() {
            for (a, b) in cache.0.iter() {
                if a >= &glyphs_len {
                    match b {
                        CachedGlyph::Present(glyph) => {
                            removals.push((*a, glyph.key));
                        }
                        CachedGlyph::Filtered => {}
                    }
                }
            }
        }
        for (a, b) in removals {
            tracing::trace!(
                "removing-glyph {:?} using {:?} ------------------------------------",
                a,
                b.glyph_index
            );
            cache.0.insert(a, CachedGlyph::Filtered);
            removes.0.push((a, b));
        }
    }
}
pub(crate) fn clear_removes(mut removed: Query<&mut GlyphRemoveQueue, Changed<GlyphRemoveQueue>>) {
    for mut queue in removed.iter_mut() {
        tracing::trace!("clearing removed:{:?}", queue.0);
        queue.0.clear();
    }
}
pub(crate) fn clear_changes(mut removed: Query<&mut GlyphChangeQueue, Changed<GlyphChangeQueue>>) {
    for mut queue in removed.iter_mut() {
        tracing::trace!("clearing changes:{:?}", queue.0);
        queue.0.clear();
    }
}
#[derive(Component, Copy, Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct FontSize(pub u32);
impl FontSize {
    pub fn px(&self, scale_factor: CoordinateUnit) -> CoordinateUnit {
        self.0 as CoordinateUnit * scale_factor
    }
}
#[derive(Component, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TextValue(pub CompactString);
impl TextValue {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self(CompactString::new(s))
    }
}
#[derive(Component, Copy, Clone, Serialize, Deserialize, PartialEq)]
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

#[test]
fn unique_characters() {
    use crate::text::{TextValue, TextValueUniqueCharacters};
    let value = TextValue::new("Dither About There");
    let unique_characters = TextValueUniqueCharacters::new(&value);
    assert_eq!(unique_characters.0, 12);
}