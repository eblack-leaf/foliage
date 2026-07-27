mod glyph;
pub(crate) mod monospaced;
mod pipeline;

use crate::Differential;
use crate::EcsExtension;
use crate::Trigger;
use crate::alignment::{HorizontalAlignment, VerticalAlignment};
use crate::ash::clip::ClipContext;
use crate::color::Color;
use crate::coordinate::Logical;
use crate::coordinate::section::Section;
use crate::foliage::{DiffMarkers, Foliage};
use crate::ginkgo::ScaleFactor;
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::text::glyph::{Glyph, GlyphColor, GlyphKey, ResolvedColors};
use crate::text::monospaced::MonospacedFont;
use crate::{
    Attachment, Branch, Layout, Location, Physical, Resolve, Resolved, ResolvedElevation,
    ResolvedVisibility, Tree, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::prelude::{Component, IntoScheduleConfigs, Res};
use bevy_ecs::query::{Changed, Or, With};
use bevy_ecs::system::{ParamSet, Query};
use bevy_ecs::world::DeferredWorld;
pub use glyph::GlyphColors;
pub use glyph::GlyphOffset;
pub(crate) use glyph::{Glyphs, ResolvedGlyphs};
use std::collections::HashSet;

impl Attachment for Text {
    fn attach(foliage: &mut Foliage) {
        foliage
            .world
            .insert_resource(MonospacedFont::new(Text::OPT_SCALE));
        foliage.define(Text::update);
        foliage.define(Text::apply_text_value);
        foliage.define(Text::responsive_font_size);
        foliage.diff.add_systems(
            (Text::resolve_glyphs, Text::resolve_colors)
                .chain()
                .in_set(DiffMarkers::Finalize),
        );
        foliage.remove_queue::<Text>();
        foliage.differential::<Text, ResolvedFontSize>();
        foliage.differential::<Text, BlendedOpacity>();
        foliage.differential::<Text, Section<Logical>>();
        foliage.differential::<Text, ResolvedElevation>();
        foliage.differential::<Text, ClipContext>();
        // ResolvedGlyphs gets its own queuing system, not the generic `differential()` --
        // see `glyph::glyph_differential`'s own doc comment for why.
        foliage
            .world
            .insert_resource(crate::ash::differential::RenderQueue::<Text, ResolvedGlyphs>::new());
        foliage
            .diff
            .add_systems(glyph::glyph_differential.in_set(DiffMarkers::Extract));
        foliage.differential::<Text, ResolvedColors>();
        foliage.differential::<Text, UniqueCharacters>();
        foliage.differential::<Text, TextBounds>();
    }
}
#[derive(Component, Clone, PartialEq, Default, Debug)]
#[require(Color, FontSize, ResolvedFontSize, UpdateCache)]
#[require(HorizontalAlignment, VerticalAlignment, Glyphs)]
#[require(
    ResolvedGlyphs,
    ResolvedColors,
    GlyphColors,
    TextContentHeight,
    TextContentWidth
)]
#[require(UniqueCharacters, Differential<Text, UniqueCharacters>)]
#[require(Differential<Text, ResolvedFontSize>)]
#[require(Differential<Text, BlendedOpacity>)]
#[require(Differential<Text, Section<Logical>>)]
#[require(Differential<Text, ResolvedElevation>)]
#[require(Differential<Text, ClipContext>)]
#[require(Differential<Text, ResolvedGlyphs>)]
#[require(Differential<Text, ResolvedColors>)]
#[require(TextBounds, Differential<Text, TextBounds>)]
#[component(on_add = Text::on_add)]
#[component(on_insert = Text::on_insert)]
pub struct Text {
    pub value: String,
}
#[derive(Default)]
pub struct TextSprout {
    leaf: crate::LeafSprout,
    value: String,
    size: Option<FontSize>,
    color: Option<Color>,
    glyph_colors: Option<GlyphColors>,
}
impl crate::Sprout for TextSprout {
    fn seed(&mut self) -> &mut crate::LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Text::new_marker(self.value),
            self.size.unwrap_or_default(),
            self.color.unwrap_or_default(),
            self.glyph_colors.unwrap_or_default(),
        )
    }
}
impl TextSprout {
    pub fn size(mut self, s: FontSize) -> Self {
        self.size = Some(s);
        self
    }
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    /// Colors each glyph individually: `f(i)` is called once per char index (0-based --
    /// the same index space `GlyphOffset` already uses) over `self.value` as it stands
    /// right now, so call this after `Text::new(..)` has the final string. Builds the
    /// `GlyphColors` component for the caller instead of them constructing
    /// `GlyphColors::new().add(range, color)` by hand.
    pub fn glyph_colors<F: Fn(usize) -> Color>(mut self, f: F) -> Self {
        let mut colors = GlyphColors::new();
        for i in 0..self.value.chars().count() {
            colors = colors.add(i..i + 1, f(i));
        }
        self.glyph_colors = Some(colors);
        self
    }
}
impl Text {
    pub(crate) const OPT_SCALE: u32 = 20;
    /// A text's public value channel: write `TextValue` to a text entity and the glyphs
    /// follow -- the render marker stays private. Entities carrying `TextValue` as mere
    /// config (a Button or TextInput root) are skipped by the `With<Text>` filter.
    fn apply_text_value(
        trigger: Trigger<bevy_ecs::lifecycle::Insert, crate::TextValue>,
        values: Query<&crate::TextValue>,
        texts: Query<(), With<Text>>,
        mut tree: Tree,
    ) {
        let this = trigger.event_target();
        if texts.contains(this) {
            if let Ok(value) = values.get(this) {
                tree.entity(this).insert(Text::new_marker(&value.0));
            }
        }
    }
    pub fn new<S: AsRef<str>>(value: S) -> TextSprout {
        TextSprout {
            value: value.as_ref().to_string(),
            ..Default::default()
        }
    }
    pub(crate) fn new_marker<S: AsRef<str>>(value: S) -> Self {
        Self {
            value: value.as_ref().to_string(),
        }
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world
            .commands()
            .entity(this)
            .observe(Remove::push_remove_packet::<Text>);
        world
            .commands()
            .entity(this)
            .observe(Visibility::push_remove_packet::<Text>);
        world
            .commands()
            .entity(this)
            .observe(Self::update_from_section);
        world
            .commands()
            .entity(this)
            .observe(Self::clear_last_on_visibility);
    }
    fn responsive_font_size(
        _trigger: Trigger<Resolved<Layout>>,
        mut font_sizes: Query<(&FontSize, &mut ResolvedFontSize)>,
        layout: Res<Layout>,
    ) {
        for (font_size, mut resolved_font_size) in font_sizes.iter_mut() {
            resolved_font_size.value = font_size.resolve(*layout);
        }
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world
            .commands()
            .trigger_targets(Resolve::<Text>::new(), this);
    }
    fn update_from_section(trigger: Trigger<Resolved<Section<Logical>>>, mut tree: Tree) {
        tree.trigger_targets(Resolve::<Text>::new(), trigger.event_target());
    }
    fn resolve_colors(
        mut glyph_colors: ParamSet<(Query<&GlyphColors>, Query<Entity, Changed<GlyphColors>>)>,
        mut colors: ParamSet<(Query<&Color>, Query<Entity, (Changed<Color>, With<Text>)>)>,
        mut glyphs: ParamSet<(Query<&Glyphs>, Query<Entity, Changed<Glyphs>>)>,
        mut resolved: Query<&mut ResolvedColors>,
    ) {
        let mut changed = glyph_colors.p1().iter().collect::<Vec<_>>();
        changed.extend(colors.p1().iter().collect::<Vec<_>>());
        changed.extend(glyphs.p1().iter().collect::<Vec<_>>());
        for e in changed {
            let resolve_start = web_time::Instant::now();
            let mut res = ResolvedColors::default();
            let color = *colors.p0().get(e).unwrap();
            let exceptions = glyph_colors.p0().get(e).unwrap().exceptions.clone();
            let glyph_count = glyphs.p0().get(e).unwrap().glyphs.len();
            for g in glyphs.p0().get(e).unwrap().glyphs.iter() {
                let c = if let Some(gc) = exceptions.get(&g.offset) {
                    *gc
                } else {
                    color
                };
                res.colors.push(GlyphColor {
                    color: c,
                    offset: g.offset,
                });
            }
            *resolved.get_mut(e).unwrap() = res;
            tracing::trace!(
                entity = ?e,
                glyph_count,
                elapsed = ?resolve_start.elapsed(),
                "text: resolve_colors"
            );
        }
    }
    fn update(
        trigger: Trigger<Resolve<Text>>,
        mut tree: Tree,
        texts: Query<&Text>,
        font_sizes: Query<&ResolvedFontSize>,
        mut glyph_query: Query<&mut Glyphs>,
        horizontal_alignment: Query<&HorizontalAlignment>,
        vertical_alignment: Query<&VerticalAlignment>,
        sections: Query<&mut Section<Logical>>,
        cache: Query<&mut UpdateCache>,
        font: Res<MonospacedFont>,
        scale_factor: Res<ScaleFactor>,
        auto_heights: Query<&TextContentHeight>,
        auto_widths: Query<&TextContentWidth>,
    ) {
        let this = trigger.event_target();
        let mut current = UpdateCache {
            font_size: ResolvedFontSize::new(
                (font_sizes.get(this).unwrap().value as f32 * scale_factor.value()) as u32,
            ),
            text: texts.get(this).unwrap().clone(),
            section: sections
                .get(this)
                .unwrap()
                .to_physical(scale_factor.value()),
            horizontal_alignment: *horizontal_alignment.get(this).unwrap(),
            vertical_alignment: *vertical_alignment.get(this).unwrap(),
        };
        // Captured before the cache comparison/overwrite below -- `UpdateCache` also
        // changes on a pure `Section` shift (e.g. the entity repositioning as a side
        // effect of a parent's scroll offset), not just a genuine content edit. Consumers
        // that care specifically about "the text *value* changed, glyphs are freshly
        // laid out" (like `TextInput`'s scroll-into-view) need that distinction --
        // `Resolved<Text>` below fires for both cases and can't make it.
        let content_changed = cache.get(this).unwrap().text.value != current.text.value;
        if cache.get(this).unwrap() != &current {
            let old = cache.get(this).unwrap().clone();
            // A pure position shift (this entity repositioned by a parent's scroll offset, its
            // own `Section.position` changing with `width`/`height` untouched) doesn't affect
            // wrapping or glyph shapes at all -- only where the already-laid-out glyphs get
            // drawn. Re-running `layout.reset`/`.append` on *every* `UpdateCache` difference
            // (position included) unconditionally re-marked `Glyphs` Changed even when the
            // recomputed layout was byte-identical to before. During any scroll -- or the
            // `Location` write `TextInput::move_cursor` does when following the cursor --
            // that fired every single frame, and `TextInput::resync_on_glyphs_changed`
            // reacting to `Changed<Glyphs>` fed straight back into another `Section` write,
            // never settling (a real hang -- wayland eventually kills an unresponsive
            // client). Gating the relayout on the fields that actually affect it breaks the
            // loop at its source instead of trying to dampen it downstream.
            let layout_dirty = old.font_size != current.font_size
                || old.text != current.text
                || old.horizontal_alignment != current.horizontal_alignment
                || old.vertical_alignment != current.vertical_alignment
                || old.section.width() != current.section.width()
                || old.section.height() != current.section.height();
            let mut glyphs = glyph_query.get_mut(this).unwrap();
            let auto_width = auto_widths.get(this).unwrap();
            let auto_height = auto_heights.get(this).unwrap();
            if layout_dirty {
                let relayout_start = web_time::Instant::now();
                glyphs.layout.reset(&fontdue::layout::LayoutSettings {
                    horizontal_align: current.horizontal_alignment.into(),
                    vertical_align: current.vertical_alignment.into(),
                    max_width: if auto_width.0 {
                        None
                    } else {
                        Some(current.section.width())
                    },
                    max_height: Some(current.section.height()),
                    ..fontdue::layout::LayoutSettings::default()
                });
                glyphs.layout.append(
                    &[&font.0],
                    &fontdue::layout::TextStyle::new(
                        current.text.value.as_str(),
                        current.font_size.value as f32,
                        0,
                    ),
                );
                tracing::trace!(
                    entity = ?this,
                    text_len = current.text.value.len(),
                    elapsed = ?relayout_start.elapsed(),
                    "text: fontdue relayout"
                );
            }
            let dims = font.character_block(current.font_size.value);
            let adjusted = if auto_height.0 {
                Some(
                    current
                        .section
                        .with_height(glyphs.layout.height())
                        .to_logical(scale_factor.value()),
                )
            } else if auto_width.0 {
                Some(
                    current
                        .section
                        .with_width(glyphs.layout.glyphs().len() as f32 * dims.a())
                        .to_logical(scale_factor.value()),
                )
            } else {
                None
            };
            let mut insert_adjusted = false;
            if let Some(adjusted) = adjusted {
                let scaled = adjusted.to_physical(scale_factor.value());
                tracing::trace!(
                    entity = ?this,
                    auto_width = auto_width.0,
                    auto_height = auto_height.0,
                    pre_adjust_section = ?current.section,
                    post_adjust_section = ?scaled,
                    glyph_layout_height = glyphs.layout.height(),
                    "text: auto width/height adjusted section"
                );
                if current.section != scaled {
                    insert_adjusted = true;
                    current.section = scaled;
                }
            }
            let mut line_metrics = LineMetrics::default();
            if let Some(lines) = glyphs.layout.lines() {
                for line in lines {
                    // fontdue's `glyph_end` is the index of the *last* glyph in the line
                    // (inclusive), not one-past-the-end -- `lines` here is meant to be a
                    // glyph *count*, so the index distance alone undercounts by 1 (a 5-glyph
                    // line has glyph_start=0/glyph_end=4, a distance of 4, not 5). That made
                    // `TextInputAction::End` land one column short of the true end of line.
                    line_metrics.lines.push(
                        (line
                            .glyph_end
                            .checked_sub(line.glyph_start)
                            .unwrap_or_default()
                            + 1) as u32,
                    );
                    line_metrics.last_offsets.push(line.glyph_end as u32);
                }
            }
            let max = (current.section.width() / dims.a()).floor() as u32;
            line_metrics.max_letter_idx_horizontal =
                max.checked_sub(1).unwrap_or_default() + if auto_width.0 { 1 } else { 0 };
            tree.entity(this)
                .insert(UniqueCharacters::count(&current.text))
                .insert(TextBounds(current.section))
                .insert(line_metrics)
                .insert(current.clone());
            if let Some(adjusted) = adjusted {
                if insert_adjusted {
                    tree.entity(this).insert(adjusted);
                }
            }
            tree.trigger_targets(Resolved::<Text>::new(), this);
            if content_changed {
                tree.trigger_targets(TextContentChanged::new(), this);
            }
        }
    }
    fn clear_last_on_visibility(
        trigger: Trigger<Resolved<Visibility>>,
        mut glyphs: Query<&mut Glyphs>,
        vis: Query<&ResolvedVisibility>,
    ) {
        let value = vis.get(trigger.event_target()).unwrap();
        if !value.visible() {
            glyphs
                .get_mut(trigger.event_target())
                .unwrap()
                .glyphs
                .clear();
        }
    }
    // Re-diffs on `Changed<ResolvedVisibility>` too, not just `Changed<Glyphs>` -- the bail
    // below (`!vis.visible()`) skips the actual diff while hidden, but `clear_last_on_visibility`
    // already zeroed `glyphs.glyphs` at that point. Going hidden->visible later touches only
    // `ResolvedVisibility`, not `Glyphs` (the fontdue layout itself never changed), so without
    // this the loop below never re-ran and `resolved`/`glyphs.glyphs` stayed at their
    // last-computed-while-visible values forever -- `cached_differential`'s own
    // visibility-restore path (ash/differential.rs) just re-sends whatever stale `ResolvedGlyphs`
    // is already sitting there, it doesn't force a fresh diff. A `TextInput`'s hint text hitting
    // this exact hidden->visible transition (typed text cleared back to empty) was the visible
    // symptom: it wouldn't reappear until something else (a resize) forced a genuine relayout
    // and therefore a real `Changed<Glyphs>`.
    fn resolve_glyphs(
        mut glyph_query: Query<
            (
                Entity,
                &mut Glyphs,
                &ResolvedVisibility,
                &mut ResolvedGlyphs,
            ),
            Or<(Changed<Glyphs>, Changed<ResolvedVisibility>)>,
        >,
        mut tree: Tree,
    ) {
        for (entity, mut glyphs, vis, mut resolved) in glyph_query.iter_mut() {
            if !vis.visible() {
                continue;
            }
            let resolve_start = web_time::Instant::now();
            let new = glyphs
                .layout
                .glyphs()
                .iter()
                .enumerate()
                .map(|(i, g)| Glyph {
                    key: GlyphKey {
                        glyph_index: g.key.glyph_index,
                        px: g.key.px as u32,
                        font_hash: g.key.font_hash,
                    },
                    // rounded to whole physical pixels, same as the text element's own
                    // container position (`text/pipeline.rs`'s own `.rounded()` on
                    // `Section<Logical>::to_physical`) -- fontdue's own `(g.x, g.y)` are raw
                    // sub-pixel floats (character advance widths at a given physical font
                    // size are almost never whole numbers), and drawing a glyph's already-
                    // rasterized bitmap at a sub-pixel offset resamples/blurs it slightly,
                    // by a *different* fractional amount per glyph. At scale factors near a
                    // round number (1.0, 2.0, 1.5) that remainder tends to be small; at an
                    // odd one (e.g. 1.73) it drifts unpredictably character to character,
                    // which is what actually read as "glitchy" rather than uniformly soft.
                    section: Section::physical((g.x, g.y), (g.width, g.height)).rounded(),
                    parent: g.parent,
                    offset: i,
                })
                .collect::<Vec<Glyph>>();
            resolved.updated.clear();
            resolved.removed.clear();
            let len_last = glyphs.glyphs.len();
            // Only glyphs that actually differ from last frame belong in `updated` -- this
            // used to push *every* pre-existing glyph unconditionally on every relayout
            // (never compared `n` against `g`), so appending a single character to an
            // already-large document re-flagged the entire, mostly-unchanged glyph list as
            // updated every time. That differential feeds the render pipeline's per-glyph
            // instance sync (`text/pipeline.rs`), so it was re-touching every render
            // instance for the whole document on every keystroke -- the real driver of the
            // multi-second pastes into an already-large `TextInput` (relayout itself is only
            // a fraction of that cost).
            for (i, g) in glyphs.glyphs.iter().enumerate() {
                if let Some(n) = new.get(i) {
                    if n != g {
                        resolved.updated.push(n.clone());
                    }
                } else {
                    resolved.removed.push(g.clone());
                }
            }
            let len_new = new.len();
            if len_new > len_last {
                for glyph in new.iter().take(len_new).skip(len_last) {
                    resolved.updated.push(glyph.clone());
                }
            }
            glyphs.glyphs = new;
            tracing::trace!(
                entity = ?entity,
                len_last,
                len_new,
                updated_count = resolved.updated.len(),
                removed_count = resolved.removed.len(),
                elapsed = ?resolve_start.elapsed(),
                "text: resolve_glyphs"
            );
        }
    }
}
#[derive(Component, Clone, Default)]
pub(crate) struct LineMetrics {
    pub(crate) lines: Vec<u32>,
    pub(crate) max_letter_idx_horizontal: u32,
    pub(crate) last_offsets: Vec<u32>,
}
#[derive(Component, Copy, Clone, PartialEq, Debug, Default)]
pub(crate) struct TextBounds(pub(crate) Section<Physical>);
#[derive(Component, Copy, Clone, Default)]
pub struct TextContentHeight(pub bool);
#[derive(Component, Copy, Clone, Default)]
pub struct TextContentWidth(pub bool);
#[derive(Copy, Clone, Component, Default, PartialEq)]
pub(crate) struct UniqueCharacters(pub(crate) u32);
impl UniqueCharacters {
    pub(crate) fn count(text: &Text) -> Self {
        let mut set = HashSet::new();
        for ch in text.value.chars() {
            set.insert(ch);
        }
        Self(set.len() as u32)
    }
}
#[derive(Component, Clone, Copy, PartialEq, Debug)]
#[component(on_insert = ResolvedFontSize::on_insert)]
pub(crate) struct ResolvedFontSize {
    pub(crate) value: u32,
}
impl ResolvedFontSize {
    pub(crate) fn new(value: u32) -> Self {
        Self { value }
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        if world.get::<Text>(this).is_some() {
            world
                .commands()
                .trigger_targets(Resolve::<Text>::new(), this);
        }
        // a `Grid`'s own `.letters()`-based column/row pitch depends on this entity's
        // FontSize (see `grid/location.rs`'s own `letter_dims`/`stem_letter_dims`), so a
        // change here can shift how every child's own `Location` ought to resolve -- but
        // nothing else re-triggers `Resolve<Location>` for them in response to a plain
        // `FontSize` write (only a child's own `Location`/`Stem`/`Visibility` being
        // written does). Without this, a runtime `FontSize` change left every child's own
        // `Section` (and so its `TextBounds`-driven render scissor, for a `Text` child
        // specifically) stale until something unrelated forced a fresh resolve elsewhere
        // (a window resize, say) -- a `Text` child would keep rendering clipped to its
        // old, pre-change cell size.
        if let Some(branch) = world.get::<Branch>(this) {
            let children: Vec<Entity> = branch.ids.iter().copied().collect();
            if !children.is_empty() {
                world
                    .commands()
                    .trigger_targets(Resolve::<Location>::new(), children);
            }
        }
    }
}
impl Default for ResolvedFontSize {
    fn default() -> Self {
        Self {
            value: FontSize::DEFAULT_SIZE,
        }
    }
}
#[derive(Component, Clone, Copy, PartialEq)]
#[component(on_insert = FontSize::on_insert)]
pub struct FontSize {
    pub xs: u32,
    pub sm: Option<u32>,
    pub md: Option<u32>,
    pub lg: Option<u32>,
    pub xl: Option<u32>,
}
impl FontSize {
    pub const DEFAULT_SIZE: u32 = 16;
    pub fn new(value: u32) -> Self {
        Self {
            xs: value,
            sm: None,
            md: None,
            lg: None,
            xl: None,
        }
    }
    pub(crate) fn resolve(&self, layout: Layout) -> u32 {
        match layout {
            Layout::Xs => self.xs,
            Layout::Sm => self.sm.unwrap_or(self.xs),
            Layout::Md => self.md.or(self.sm).unwrap_or(self.xs),
            Layout::Lg => self.lg.or(self.md).or(self.sm).unwrap_or(self.xs),
            Layout::Xl => self
                .xl
                .or(self.lg)
                .or(self.md)
                .or(self.sm)
                .unwrap_or(self.xs),
        }
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let layout = *world.get_resource::<Layout>().unwrap();
        let comp = world.get::<FontSize>(this).unwrap();
        let resolved = comp.resolve(layout);
        world.commands().entity(this).insert(ResolvedFontSize::new(resolved));
    }
    pub fn sm(mut self, value: u32) -> Self {
        self.sm.replace(value);
        self
    }
    pub fn md(mut self, value: u32) -> Self {
        self.md.replace(value);
        self
    }
    pub fn lg(mut self, value: u32) -> Self {
        self.lg.replace(value);
        self
    }
    pub fn xl(mut self, value: u32) -> Self {
        self.xl.replace(value);
        self
    }
}
impl Default for FontSize {
    fn default() -> Self {
        Self {
            xs: FontSize::DEFAULT_SIZE,
            sm: None,
            md: None,
            lg: None,
            xl: None,
        }
    }
}
#[derive(Component, Clone, PartialEq, Default, Debug)]
pub(crate) struct UpdateCache {
    pub(crate) font_size: ResolvedFontSize,
    pub(crate) text: Text,
    pub(crate) section: Section<Physical>,
    pub(crate) horizontal_alignment: HorizontalAlignment,
    pub(crate) vertical_alignment: VerticalAlignment,
}
/// Fires only when the text *value* actually changed and glyphs/`LineMetrics` have just been
/// freshly recomputed -- unlike `Resolved<Text>`, which also fires for a pure `Section` shift
/// (e.g. scrolling a parent view repositions this entity with no content change at all).
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub(crate) struct TextContentChanged {}
// The alignment vocabulary itself lives in `crate::alignment` (shared with `Icon`); only
// the fontdue conversions are text's own concern.
impl From<HorizontalAlignment> for fontdue::layout::HorizontalAlign {
    fn from(value: HorizontalAlignment) -> Self {
        match value {
            HorizontalAlignment::Left => fontdue::layout::HorizontalAlign::Left,
            HorizontalAlignment::Center => fontdue::layout::HorizontalAlign::Center,
            HorizontalAlignment::Right => fontdue::layout::HorizontalAlign::Right,
        }
    }
}
impl From<VerticalAlignment> for fontdue::layout::VerticalAlign {
    fn from(value: VerticalAlignment) -> Self {
        match value {
            VerticalAlignment::Top => fontdue::layout::VerticalAlign::Top,
            VerticalAlignment::Middle => fontdue::layout::VerticalAlign::Middle,
            VerticalAlignment::Bottom => fontdue::layout::VerticalAlign::Bottom,
        }
    }
}
