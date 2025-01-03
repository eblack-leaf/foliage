use crate::coordinate::section::Section;
use crate::coordinate::Physical;
use crate::{Color, Component, Update};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::DeferredWorld;
use fontdue::layout::CoordinateSystem::PositiveYDown;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Range;

#[derive(Serialize, Deserialize, Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub(crate) struct GlyphKey {
    pub(crate) glyph_index: u16,
    pub(crate) px: u32,
    pub(crate) font_hash: usize,
}
#[derive(PartialEq, Clone, Debug)]
pub(crate) struct Glyph {
    pub(crate) key: GlyphKey,
    pub(crate) section: Section<Physical>,
    pub(crate) parent: char,
    pub(crate) offset: GlyphOffset,
}
pub type GlyphOffset = usize;
#[derive(Component)]
pub(crate) struct Glyphs {
    pub(crate) layout: fontdue::layout::Layout,
    pub(crate) glyphs: Vec<Glyph>,
}
impl Glyphs {
    pub(crate) fn new() -> Self {
        Self {
            layout: fontdue::layout::Layout::new(PositiveYDown),
            glyphs: vec![],
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
    pub(crate) updated: Vec<Glyph>,
    pub(crate) removed: Vec<Glyph>,
}
impl ResolvedGlyphs {
    pub(crate) fn new() -> Self {
        Self {
            updated: vec![],
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
#[component(on_insert = Self::on_insert)]
pub struct GlyphColors {
    pub exceptions: HashMap<GlyphOffset, Color>,
}
impl GlyphColors {
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.trigger_targets(Update::<Self>::new(), this);
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add(mut self, offsets: Range<GlyphOffset>, color: Color) -> Self {
        for o in offsets {
            self.exceptions.insert(o, color);
        }
        self
    }
}
#[derive(Component, Default, PartialEq, Clone)]
pub(crate) struct GlyphColor {
    pub(crate) color: Color,
    pub(crate) offset: GlyphOffset,
}
#[derive(Component, Default, PartialEq, Clone)]
pub struct ResolvedColors {
    pub colors: Vec<GlyphColor>,
}
