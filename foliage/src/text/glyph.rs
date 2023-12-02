use crate::color::Color;
use crate::coordinate::section::Section;
use crate::coordinate::DeviceContext;
use crate::text::renderer::TextKey;
use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Component, Clone, Serialize, Deserialize, PartialEq, Default)]
pub(crate) struct GlyphChange {
    pub(crate) character: Option<char>,
    pub(crate) key: Option<(GlyphKey, Option<GlyphKey>)>,
    pub(crate) section: Option<Section<DeviceContext>>,
    pub(crate) color: Option<Color>,
}

#[derive(Component)]
pub(crate) struct GlyphPlacementTool(pub(crate) fontdue::layout::Layout);
