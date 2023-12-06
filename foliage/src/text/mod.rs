pub mod font;
mod glyph;
mod renderer;
mod vertex;

use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::CReprPosition;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, DeviceContext, InterfaceContext};
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::{Elm, Leaf, SystemSets};
use crate::text::font::MonospacedFont;
use crate::text::glyph::Glyph;
use crate::window::ScaleFactor;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Bundle, IntoSystemConfigs, Or};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, Res};
use compact_str::CompactString;
use glyph::{
    GlyphCache, GlyphChange, GlyphChangeQueue, GlyphKey, GlyphPlacementTool, GlyphRemoveQueue,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Bundle)]
pub struct Text {
    area: Area<InterfaceContext>,
    text_value: TextValue,
    max_characters: MaxCharacters,
    character_dimension: CharacterDimension,
    font_size: DifferentialBundle<FontSize>,
    c_pos: DifferentialBundle<CReprPosition>,
    c_area: DifferentialBundle<CReprArea>,
    color: DifferentialBundle<Color>,
    text_value_chars: DifferentialBundle<TextValueUniqueCharacters>,
    glyph_adds: DifferentialBundle<GlyphChangeQueue>,
    glyph_removes: DifferentialBundle<GlyphRemoveQueue>,
    glyph_cache: GlyphCache,
    glyph_placement_tool: GlyphPlacementTool,
    differentiable: Differentiable,
}
impl Text {
    pub fn new(
        max_characters: MaxCharacters,
        font_size: FontSize,
        text_value: TextValue,
        color: Color,
    ) -> Self {
        Self {
            area: Area::default(),
            max_characters,
            font_size: DifferentialBundle::new(font_size),
            c_pos: DifferentialBundle::new(CReprPosition::default()),
            c_area: DifferentialBundle::new(CReprArea::default()),
            color: DifferentialBundle::new(color),
            text_value_chars: DifferentialBundle::new(TextValueUniqueCharacters::new(&text_value)),
            glyph_adds: DifferentialBundle::new(GlyphChangeQueue::default()),
            glyph_removes: DifferentialBundle::new(GlyphRemoveQueue::default()),
            text_value,
            character_dimension: CharacterDimension(Area::default()),
            differentiable: Differentiable::new::<Self>(Layer::default()),
            glyph_cache: GlyphCache::default(),
            glyph_placement_tool: GlyphPlacementTool(fontdue::layout::Layout::new(
                fontdue::layout::CoordinateSystem::PositiveYDown,
            )),
        }
    }
    pub const DEFAULT_OPT_SCALE: u32 = 40;
}
#[derive(Component, Copy, Clone)]
pub struct MaxCharacters(pub u32);
impl Leaf for Text {
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
            changes.in_set(SystemSets::Resolve),
            max_character.in_set(SystemSets::Resolve).before(changes),
        ));
    }
}
#[derive(Component, Copy, Clone)]
pub(crate) struct CharacterDimension(pub(crate) Area<DeviceContext>);
pub(crate) fn max_character(
    mut query: Query<
        (
            &MaxCharacters,
            &FontSize,
            &mut Area<InterfaceContext>,
            &mut CharacterDimension,
        ),
        Or<(Changed<MaxCharacters>, Changed<FontSize>)>,
    >,
    scale_factor: Res<ScaleFactor>,
    font: Res<MonospacedFont>,
) {
    for (max, size, mut area, mut dim) in query.iter_mut() {
        *dim = CharacterDimension(font.character_dimensions(size.px(scale_factor.factor())));
        let interface_dim = dim.0.to_interface(scale_factor.factor());
        let width = interface_dim.width * max.0 as f32;
        *area = (width, interface_dim.height).into();
    }
}

pub(crate) fn changes(
    mut query: Query<
        (
            &Area<InterfaceContext>,
            &FontSize,
            &Color,
            &TextValue,
            &CharacterDimension,
            &mut GlyphChangeQueue,
            &mut GlyphRemoveQueue,
            &mut GlyphCache,
            &mut GlyphPlacementTool,
        ),
        Or<(Changed<TextValue>, Changed<FontSize>)>,
    >,
    font: Res<MonospacedFont>,
    scale_factor: Res<ScaleFactor>,
) {
    for (
        area,
        font_size,
        color,
        value,
        character_dim,
        mut changes,
        mut removes,
        mut cache,
        mut placer,
    ) in query.iter_mut()
    {
        let scaled = area.to_device(scale_factor.factor());
        placer.0.reset(&fontdue::layout::LayoutSettings {
            max_width: Some(scaled.width),
            max_height: Some(scaled.height),
            line_height: MonospacedFont::TEXT_HEIGHT_CORRECTION,
            ..fontdue::layout::LayoutSettings::default()
        });
        placer.0.append(
            &[&font.0],
            &fontdue::layout::TextStyle::new(
                value.0.as_str(),
                font_size.px(scale_factor.factor()),
                0,
            ),
        );
        let glyphs = placer.0.glyphs();
        for g in glyphs {
            if g.y + g.height as f32 > character_dim.0.height {
                if let Some(old) = cache.0.remove(&g.byte_offset) {
                    removes.0.push((g.byte_offset, old.key));
                }
            } else {
                let mut change = None;
                let glyph_key = GlyphKey::new(g.key);
                let mut total_update = false;
                if let Some(cached) = cache.0.get(&g.byte_offset) {
                    if cached.key != glyph_key {
                        total_update = true;
                    } else {
                        // color change
                    }
                } else {
                    total_update = true;
                }
                if total_update {
                    let section = Section::<DeviceContext>::new((g.x, g.y), (g.width, g.height));
                    cache.0.insert(
                        g.byte_offset,
                        Glyph {
                            key: glyph_key,
                            section,
                            color: *color,
                        },
                    );
                    change.replace(GlyphChange {
                        key: Some((glyph_key, None)),
                        section: Some(section),
                        color: Some(*color),
                    });
                }
                if let Some(c) = change {
                    changes.0.push((g.byte_offset, c));
                }
            }
        }
        let glyphs_len = glyphs.len();
        let mut removals = vec![];
        if glyphs_len < cache.0.len() {
            for (a, b) in cache.0.iter() {
                if a >= &glyphs_len {
                    removals.push((*a, b.key));
                }
            }
        }
        for (a, b) in removals {
            cache.0.remove(&a);
            removes.0.push((a, b));
        }
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
