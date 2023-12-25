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
use crate::elm::Elm;
use crate::text::font::MonospacedFont;
use crate::text::glyph::{CachedGlyph, Glyph};
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
use std::collections::HashSet;

#[derive(Bundle)]
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
            max_characters,
            font_size: DifferentialBundle::new(font_size),
            color: DifferentialBundle::new(color),
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
        }
    }
    pub const DEFAULT_OPT_SCALE: u32 = 40;
    pub fn area_metrics(
        font_size: FontSize,
        max_characters: MaxCharacters,
        font: &MonospacedFont,
        scale_factor: &ScaleFactor,
    ) -> (Area<InterfaceContext>, CharacterDimension) {
        let dim =
            CharacterDimension(font.character_dimensions(font_size.px(scale_factor.factor())));
        let interface_dim = dim.0.to_interface(scale_factor.factor());
        let width = interface_dim.width * max_characters.0 as f32;
        let area = (width, interface_dim.height).into();
        (area, dim)
    }
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
            changes
                .in_set(ExternalSet::Configure)
                .in_set(Self::SetDescriptor::Area),
            max_character
                .in_set(ExternalSet::Configure)
                .before(changes)
                .in_set(Self::SetDescriptor::Area),
            clear_removes.after(CoreSet::Differential),
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
        let (a, d) = Text::area_metrics(*size, *max, &font, &scale_factor);
        *area = a;
        *dim = d;
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
        _character_dim,
        mut changes,
        mut removes,
        mut cache,
        mut placer,
    ) in query.iter_mut()
    {
        let scaled = area.to_device(scale_factor.factor());
        placer.0.reset(&fontdue::layout::LayoutSettings {
            max_width: None,
            max_height: Some(scaled.height),
            line_height: MonospacedFont::TEXT_HEIGHT_CORRECTION,
            wrap_style: fontdue::layout::WrapStyle::Letter,
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
            let mut change = None;
            let glyph_key = GlyphKey::new(g.key);
            let mut total_update = false;
            let filtered = g.x + g.width as f32 > scaled.width;
            if let Some(cached) = cache.0.get_mut(&g.byte_offset) {
                match cached {
                    CachedGlyph::Present(glyph) => {
                        if filtered {
                            removes.0.push((g.byte_offset, glyph_key));
                            *cached = CachedGlyph::Filtered;
                        } else {
                            if glyph.key != glyph_key {
                                total_update = true;
                            } else {
                                // color change
                            }
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
                        color: *color,
                    }),
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
            cache.0.insert(a, CachedGlyph::Filtered);
            removes.0.push((a, b));
        }
    }
}
pub(crate) fn clear_removes(mut removed: Query<&mut GlyphRemoveQueue, Changed<GlyphRemoveQueue>>) {
    for mut queue in removed.iter_mut() {
        queue.0.clear();
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