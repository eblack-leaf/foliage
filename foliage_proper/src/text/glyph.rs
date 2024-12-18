use crate::coordinate::section::Section;
use crate::coordinate::DeviceContext;
use crate::Color;
use serde::{Deserialize, Serialize};

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
#[derive(PartialEq, Clone)]
pub(crate) struct Glyph {
    pub(crate) key: GlyphKey,
    pub(crate) section: Section<DeviceContext>,
    pub(crate) parent: char,
    pub(crate) color: Color,
}
pub type GlyphOffset = usize;