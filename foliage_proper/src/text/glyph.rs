use crate::EcsExtension;
use crate::ash::differential::RenderQueue;
use crate::coordinate::Physical;
use crate::coordinate::section::Section;
use crate::text::Text;
use crate::{Color, Component, Differential, ResolvedVisibility, Resolve};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::query::{Changed, With};
use bevy_ecs::system::{ParamSet, Query, ResMut};
use bevy_ecs::world::DeferredWorld;
use fontdue::layout::CoordinateSystem::PositiveYDown;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
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
impl Display for Glyph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "k: {} s: {} p: {} o: {}",
            self.key.glyph_index, self.section, self.parent, self.offset
        ))
    }
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
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world.trigger_targets(Resolve::<Self>::new(), this);
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

/// `ResolvedGlyphs`' own queuing system -- every other differential channel goes through
/// the shared, generic `cached_differential` (`ash/differential.rs`), which keeps only the
/// *latest* value per entity in its `RenderQueue`. That's correct for every other component
/// here (`Section`, `Color`, `ResolvedColors`, ...): each one is a complete snapshot, so an
/// unflushed value being replaced by a newer one loses nothing. `ResolvedGlyphs` is the one
/// exception -- its value is an *incremental* diff against whatever `resolve_glyphs` saw
/// last tick (see `text/mod.rs`), not a full picture of current state. If two relayouts of
/// the same entity land before a render flush ever drains the queue (a `Section` rewrite
/// during an ancestor's animation is enough, even with unchanged text content -- the second
/// relayout just diffs against the first's already-updated snapshot and comes out empty),
/// the generic "latest wins" queuing silently drops the first diff's real `add()`s. Any
/// other differential channel referencing those same glyph offsets afterward (`ResolvedColors`,
/// most commonly) then points at offsets the render-side coordinator never registered,
/// which is what panics in `InstanceCoordinator::order`. This merges successive diffs by
/// offset instead of replacing one with the next, so nothing pending ever gets lost purely
/// because a later, unrelated relayout happened to land on the same entity first.
pub(crate) fn glyph_differential(
    mut values: ParamSet<(
        Query<
            (Entity, &ResolvedGlyphs),
            (
                Changed<ResolvedGlyphs>,
                With<Differential<Text, ResolvedGlyphs>>,
            ),
        >,
        Query<&ResolvedGlyphs>,
    )>,
    mut caches: Query<&mut Differential<Text, ResolvedGlyphs>>,
    mut visibility: ParamSet<(
        Query<&ResolvedVisibility>,
        Query<
            Entity,
            (
                Changed<ResolvedVisibility>,
                With<Differential<Text, ResolvedGlyphs>>,
            ),
        >,
    )>,
    mut queue: ResMut<RenderQueue<Text, ResolvedGlyphs>>,
) {
    // visibility-restore: a fresh full resend when going hidden -> visible, not a delta
    // relative to anything already queued, so no merge is needed here -- same shape as
    // `cached_differential`'s own visibility-restore half.
    let changed = visibility.p1().iter().collect::<Vec<_>>();
    for c in changed {
        let Ok(visible) = visibility.p0().get(c).map(|v| v.visible()) else {
            continue;
        };
        if visible {
            let Ok(v) = values.p1().get(c).map(|v| v.clone()) else {
                continue;
            };
            let Ok(mut cache) = caches.get_mut(c) else {
                continue;
            };
            cache.cache.replace(v.clone());
            queue.queue.insert(c, v);
        }
    }
    // changed: merge into whatever's already unflushed instead of overwriting it.
    for (e, v) in values.p0().iter() {
        let Ok(visible) = visibility.p0().get(e).map(|v| v.visible()) else {
            continue;
        };
        if visible {
            let Ok(mut cache) = caches.get_mut(e) else {
                continue;
            };
            if cache.different(v.clone()) {
                let existing = queue.queue.remove(&e);
                queue
                    .queue
                    .insert(e, merge_resolved_glyphs(existing, v.clone()));
            }
        }
    }
}

enum GlyphDisposition {
    Updated(Glyph),
    Removed(Glyph),
}
fn merge_resolved_glyphs(
    existing: Option<ResolvedGlyphs>,
    incoming: ResolvedGlyphs,
) -> ResolvedGlyphs {
    let Some(existing) = existing else {
        return incoming;
    };
    let mut by_offset: HashMap<GlyphOffset, GlyphDisposition> = HashMap::new();
    for g in existing.updated {
        by_offset.insert(g.offset, GlyphDisposition::Updated(g));
    }
    for g in existing.removed {
        by_offset.insert(g.offset, GlyphDisposition::Removed(g));
    }
    // incoming is strictly newer -- it overwrites whatever the older diff said per offset.
    for g in incoming.updated {
        by_offset.insert(g.offset, GlyphDisposition::Updated(g));
    }
    for g in incoming.removed {
        by_offset.insert(g.offset, GlyphDisposition::Removed(g));
    }
    let mut merged = ResolvedGlyphs::new();
    for (_, disposition) in by_offset {
        match disposition {
            GlyphDisposition::Updated(g) => merged.updated.push(g),
            GlyphDisposition::Removed(g) => merged.removed.push(g),
        }
    }
    merged
}
