use crate::coordinate::section::Section;
use crate::coordinate::DeviceContext;
use crate::{Color, Component, Update};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::DeferredWorld;
use fontdue::layout::CoordinateSystem::PositiveYDown;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub(crate) offset: GlyphOffset,
}
pub type GlyphOffset = usize;
#[derive(Component)]
pub(crate) struct Glyphs {
    pub(crate) layout: fontdue::layout::Layout,
    pub(crate) glyphs: Vec<Glyph>,
    pub(crate) last: Vec<Glyph>,
}
impl Glyphs {
    pub(crate) fn new() -> Self {
        Self {
            layout: fontdue::layout::Layout::new(PositiveYDown),
            glyphs: vec![],
            last: vec![],
        }
    }
}
impl Default for Glyphs {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Component, Clone, PartialEq)]
pub(crate) struct ResolvedGlyphs {
    pub(crate) added: Vec<Glyph>,
    pub(crate) removed: Vec<Glyph>,
}
impl ResolvedGlyphs {
    pub(crate) fn new() -> Self {
        Self {
            added: vec![],
            removed: vec![],
        }
    }
}
impl Default for ResolvedGlyphs {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Component, Default)]
pub struct GlyphColors {
    pub exceptions: HashMap<GlyphOffset, Color>,
}
impl GlyphColors {
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.trigger_targets(Update::<Self>::new(), this);
    }
}
#[derive(Component, Default, PartialEq, Clone)]
pub(crate) struct GlyphColor(pub(crate) Color);
#[derive(Component, Default, PartialEq, Clone)]
pub struct ResolvedColors {
    pub colors: Vec<GlyphColor>,
}
