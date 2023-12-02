mod font;
mod renderer;
mod vertex;

use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, DeviceContext, InterfaceContext, NumericalContext};
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::{Elm, Leaf, SystemSets};
use crate::text::font::MonospacedFont;
use crate::text::renderer::TextKey;
use crate::window::ScaleFactor;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Bundle, IntoSystemConfigs, Or};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, Res};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Bundle)]
pub struct Text {
    position: Position<InterfaceContext>,
    area: Area<InterfaceContext>,
    text_value: TextValue,
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
        position: Position<InterfaceContext>,
        area: Area<InterfaceContext>,
        layer: Layer,
        font_size: FontSize,
        text_value: TextValue,
        color: Color,
    ) -> Self {
        Self {
            position,
            area,
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
            glyph_placement_tool: GlyphPlacementTool(fontdue::layout::Layout::new(
                fontdue::layout::CoordinateSystem::PositiveYDown,
            )),
        }
    }
    pub const DEFAULT_OPT_SCALE: u32 = 40;
}
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
        elm.job
            .main()
            .add_systems((changes.before(SystemSets::Differential),));
    }
}
#[derive(Serialize, Deserialize, Copy, Clone, Hash, Eq, PartialEq)]
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
pub(crate) fn changes(
    mut query: Query<
        (
            &Area<InterfaceContext>,
            &FontSize,
            &Color,
            &TextValue,
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
    for (area, font_size, color, value, mut changes, mut removes, mut cache, mut placer) in
        query.iter_mut()
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
        for g in placer.0.glyphs() {
            changes.0.push((
                g.byte_offset,
                GlyphChange {
                    character: Some(g.parent),
                    key: Some((GlyphKey::new(g.key), None)),
                    section: Some(Section::<DeviceContext>::new(
                        (g.x, g.y),
                        (g.width, g.height),
                    )),
                    color: Some(*color),
                },
            ))
        }
    }
}
#[derive(Component)]
pub(crate) struct GlyphPlacementTool(pub(crate) fontdue::layout::Layout);
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
        Self(value.0.len() as u32)
    }
}
#[derive(Component, Default)]
pub(crate) struct GlyphCache(pub(crate) HashMap<TextKey, Glyph>);
#[derive(Component, Clone, Serialize, Deserialize, PartialEq, Default)]
pub(crate) struct GlyphChangeQueue(pub(crate) Vec<(TextKey, GlyphChange)>);
#[derive(Component, Clone, Serialize, Deserialize, PartialEq, Default)]
pub(crate) struct GlyphRemoveQueue(pub(crate) Vec<(TextKey, GlyphKey)>);
#[derive(Component, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct Glyph {
    pub(crate) character: char,
    pub(crate) key: GlyphKey,
    pub(crate) section: Section<DeviceContext>,
    pub(crate) color: Color,
}
#[derive(Component, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct GlyphChange {
    pub(crate) character: Option<char>,
    pub(crate) key: Option<(GlyphKey, Option<GlyphKey>)>,
    pub(crate) section: Option<Section<DeviceContext>>,
    pub(crate) color: Option<Color>,
}
