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
use crate::text::monospaced::{FontId, MonospacedFont};
use crate::{
    Attachment, Layout, Location, Physical, Resolve, Resolved, ResolvedElevation,
    ResolvedVisibility, Short, Tree, Visibility,
};
use bevy_ecs::bundle::Bundle;
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
        foliage.differential::<Text, FontId>();
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
#[require(FontId, Differential<Text, FontId>)]
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
/// A run of monospaced glyphs, laid out by fontdue into the entity's own
/// [`Section`](crate::Section) and drawn from a per-entity glyph atlas.
///
/// This is the render marker and is spawned through [`Text::new`], not constructed
/// directly. To change the string afterwards, write [`TextValue`](crate::TextValue) --
/// the public value channel every text-bearing composite shares.
///
/// Layout is a fixed monospace grid: every glyph advances by the same width, taken from
/// one reference character at the current [`FontSize`]. That pitch is also what
/// [`.letters()`](crate::GridExt::letters) resolves against, so a `Location` can be sized
/// in characters rather than pixels. A glyph wider than the reference advance overhangs
/// its cell rather than widening it.
///
/// The entity's `Section` is the layout box: it bounds wrapping and doubles as the render
/// scissor. [`TextContentWidth`]/[`TextContentHeight`] invert that, sizing the box from
/// the glyphs instead.
pub struct Text {
    pub value: String,
}
/// Builder for a [`Text`] entity -- see [`Text::new`].
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
    /// Sets the glyph size, per breakpoint if the [`FontSize`] carries one. Defaults to
    /// [`FontSize::DEFAULT_SIZE`].
    pub fn size(mut self, s: FontSize) -> Self {
        self.size = Some(s);
        self
    }
    /// Sets one color for the whole run. For per-glyph colors, see
    /// [`glyph_colors`](Self::glyph_colors), which overrides this at the indices it names.
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
    /// Starts a [`Text`] entity carrying `value`:
    /// `tree.branch(parent, Text::new("hello").size(FontSize::new(24)).at(loc))`.
    ///
    /// Chain [`size`](TextSprout::size)/[`color`](TextSprout::color)/
    /// [`glyph_colors`](TextSprout::glyph_colors) here, and the usual
    /// [`Sprout`](crate::Sprout) placement (`at`, `elevate`, `with`) as with any leaf.
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
        font_ids: Query<&FontId>,
        scale_factor: Res<ScaleFactor>,
        auto_heights: Query<&TextContentHeight>,
        auto_widths: Query<&TextContentWidth>,
    ) {
        let this = trigger.event_target();
        let font_id = font_ids.get(this).copied().unwrap_or_default();
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
            // Only the inputs fontdue actually consumes; a pure position shift (a parent's
            // scroll offset moving this box, width and height untouched) changes where the
            // glyphs are drawn, not how they are laid out. The distinction has to hold:
            // relayout marks `Glyphs` changed even when it recomputes an identical layout,
            // and `TextInput::resync_on_glyphs_changed` answers that with another `Section`
            // write -- so relaying out on position would spin that loop every frame it
            // scrolls, with nothing to converge on.
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
                    &[font.get(font_id).as_ref()],
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
            let dims = font.character_block(font_id, current.font_size.value);
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
                // Sub-pixel tolerance, not `!=`: `scaled` came back through a
                // physical->logical->physical round trip, which is only bit-exact when the
                // scale factor is a power of two (1.0, 2.0). At something like 1.73 it
                // lands an ULP off every time, so an exact compare re-inserted the section
                // on every update -- and each insert re-resolves and relays out, which
                // reads as the text jittering a pixel per keystroke.
                const ADJUST_EPSILON: f32 = 0.01;
                let moved = (current.section.left() - scaled.left()).abs() > ADJUST_EPSILON
                    || (current.section.top() - scaled.top()).abs() > ADJUST_EPSILON
                    || (current.section.width() - scaled.width()).abs() > ADJUST_EPSILON
                    || (current.section.height() - scaled.height()).abs() > ADJUST_EPSILON;
                if moved {
                    insert_adjusted = true;
                    current.section = scaled;
                }
            }
            let mut line_metrics = LineMetrics::default();
            if let Some(lines) = glyphs.layout.lines() {
                for line in lines {
                    // `+ 1` because fontdue's `glyph_end` is inclusive -- the index of the
                    // last glyph, not one past it -- while `lines` holds counts.
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
    /// Diffs the fontdue layout against last frame's glyphs into [`ResolvedGlyphs`], the
    /// per-glyph add/update/remove list the render pipeline consumes.
    ///
    /// Watches `ResolvedVisibility` as well as `Glyphs` because going hidden zeroes the
    /// previous-glyph mirror (`clear_last_on_visibility`) while leaving the layout itself
    /// untouched. Coming back visible therefore changes neither `Glyphs` nor anything
    /// `cached_differential` would re-send, so this is the only thing that rebuilds the
    /// list -- without it a re-shown run stays blank until some unrelated relayout.
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
        _tree: Tree,
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
                    // Position snapped to whole physical pixels; area left exactly as
                    // fontdue reported it, since the atlas bitmap was rasterized at that
                    // size and the blit has to stay 1:1 texel-to-pixel.
                    section: Section::physical((g.x.round(), g.y.round()), (g.width, g.height)),
                    parent: g.parent,
                    offset: i,
                })
                .collect::<Vec<Glyph>>();
            resolved.updated.clear();
            resolved.removed.clear();
            let len_last = glyphs.glyphs.len();
            // Compared per index, not pushed wholesale: this list drives one render-instance
            // write per entry, so a single keystroke in a long document must not re-flag
            // every glyph in it.
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
/// Per-line glyph counts for the current layout, indexed by line. Cursor navigation reads
/// these to answer "where does this line end" without re-walking the fontdue layout --
/// see [`TextInput`](crate::TextInput)'s own `TextInputAction` handling.
#[derive(Component, Clone, Default)]
pub(crate) struct LineMetrics {
    /// Glyph count per line, not an end index: a 5-glyph line stores `5`.
    pub(crate) lines: Vec<u32>,
    /// Highest column index a cursor may occupy, from the box width over the glyph
    /// advance. One past the last glyph when the width is content-driven, since there is
    /// no fixed right edge to stop at.
    pub(crate) max_letter_idx_horizontal: u32,
    /// Index of each line's final glyph, in the whole run's own index space.
    pub(crate) last_offsets: Vec<u32>,
}
/// The scissor the text renders under, in physical pixels -- the entity's own `Section`
/// at the scale factor in force when the glyphs were last laid out. Kept as its own
/// component so the renderer clips against the box the current glyphs were fitted to.
#[derive(Component, Copy, Clone, PartialEq, Debug, Default)]
pub(crate) struct TextBounds(pub(crate) Section<Physical>);
/// Take the entity's height from the laid-out glyphs instead of its `Location`.
///
/// The `Location`'s own height still decides where the box starts and how wrapping is
/// measured; this replaces the resulting height once the glyphs are placed. Pair with a
/// [`text_content()`](crate::text_content) height on a *dependent* entity to have it
/// follow along.
#[derive(Component, Copy, Clone, Default)]
pub struct TextContentHeight(pub bool);
/// Take the entity's width from the laid-out glyphs instead of its `Location`.
///
/// Also disables wrapping: with no fixed right edge there is nothing to wrap against, so
/// the run stays on one line and the box grows to the glyph advance times the glyph count.
#[derive(Component, Copy, Clone, Default)]
pub struct TextContentWidth(pub bool);
/// How many distinct characters the run uses -- the number of atlas cells the renderer
/// has to allocate, since the atlas is keyed per character rather than per glyph
/// occurrence.
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
/// [`FontSize`] collapsed to the one value the current [`Layout`] selects -- the size
/// glyph layout and `.letters()` pitch actually use. Re-derived on every breakpoint change
/// by `Text::responsive_font_size`; its insertion is the "this entity's size just settled"
/// signal everything downstream keys off.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
#[component(on_insert = ResolvedFontSize::on_insert)]
pub(crate) struct ResolvedFontSize {
    pub(crate) value: u32,
}
impl ResolvedFontSize {
    pub(crate) fn new(value: u32) -> Self {
        Self { value }
    }
    /// Re-resolves whatever a settled font size feeds: this entity's glyphs, and its own
    /// `Location` when that `Location` is sized in characters.
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        if world.get::<Text>(this).is_some() {
            world
                .commands()
                .trigger_targets(Resolve::<Text>::new(), this);
        }
        // A `.letters()`-sized `Location` resolves its numbers out of this entity's own
        // `FontSize` (`grid/location.rs`'s own `letter_dims`), so this is the one place
        // that dependency gets honored: the rest of the resolve triggers are structural
        // (`Location`/`Stem`/`Visibility` written, or a parent's `Section` landing) and
        // none of them fire for a plain `FontSize` write.
        //
        // Only this entity needs the trigger. Its own new `Section<Logical>` cascades
        // `Resolve<Location>` to every child (that component's own `on_insert`), and each
        // child's new `Section` in turn re-fires its `Resolve<Text>` via
        // `Resolved<Section<Logical>>`, which re-cuts the `TextBounds` render scissor
        // against the child's new cell. Reaching down to the children directly instead
        // inverts that order -- they would resolve their columns against a parent box that
        // has not moved yet.
        //
        // The gate keeps this narrow: the hook fires for every entity carrying a
        // `FontSize` on every layout change, and `Letters` is the only `LocationValue`
        // whose resolution reads the font size at all.
        let layout = *world.get_resource::<Layout>().unwrap();
        let short = *world.get_resource::<Short>().unwrap();
        if world
            .get::<Location>(this)
            .is_some_and(|l| l.depends_on_own_font_size(layout, short))
        {
            world
                .commands()
                .trigger_targets(Resolve::<Location>::new(), this);
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
/// Glyph size in logical pixels, optionally per breakpoint.
///
/// Only `xs` is required; each larger breakpoint falls back to the nearest smaller one
/// that was set, so `FontSize::new(14).lg(18)` is 14 through `md` and 18 from `lg` up.
///
/// Writing this at runtime re-lays-out the glyphs, and re-resolves the entity's own
/// `Location` when that `Location` is sized in
/// [`.letters()`](crate::GridExt::letters) -- character-sized boxes track the size that
/// defines a character.
///
/// Useful beyond [`Text`]: any entity whose `Grid` or `Location` is expressed in
/// characters needs a `FontSize` to give those characters a width.
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
    /// Size used by anything that never sets one.
    pub const DEFAULT_SIZE: u32 = 16;
    /// One size at every breakpoint. Add exceptions with
    /// [`sm`](Self::sm)/[`md`](Self::md)/[`lg`](Self::lg)/[`xl`](Self::xl).
    pub fn new(value: u32) -> Self {
        Self {
            xs: value,
            sm: None,
            md: None,
            lg: None,
            xl: None,
        }
    }
    /// The size in force at `layout`, falling back down the breakpoints to `xs`.
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
    /// Overrides the size from the `sm` breakpoint up.
    pub fn sm(mut self, value: u32) -> Self {
        self.sm.replace(value);
        self
    }
    /// Overrides the size from the `md` breakpoint up.
    pub fn md(mut self, value: u32) -> Self {
        self.md.replace(value);
        self
    }
    /// Overrides the size from the `lg` breakpoint up.
    pub fn lg(mut self, value: u32) -> Self {
        self.lg.replace(value);
        self
    }
    /// Overrides the size at the `xl` breakpoint.
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
/// The inputs the last glyph layout was computed from. `Text::update` compares against
/// this to decide whether anything needs redoing, and which parts: a change to the box's
/// *position* alone moves already-placed glyphs, while a change to size, string, font size
/// or alignment requires a fresh fontdue pass.
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
// The alignment vocabulary lives in `crate::alignment`, shared with `Icon`; only the
// fontdue conversions belong to text.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;
    use crate::grid::location::GridExt;
    use crate::{EcsExtension, Elevation, Foliage, Leaf, Sprout};

    // Mirrors `application/src/chapters/text.rs` exactly: a `.letters()`-sized `Grid`
    // parent holding one `Text` per column, then a runtime `FontSize` write to the parent
    // *and* every child -- the "grow" step of that chapter.
    const INITIAL: u32 = 40;
    const GROWN: u32 = 64;
    const GAP: i32 = 34;
    const PAD: i32 = 4;
    const COLS: i32 = 3;

    fn cell(i: i32) -> Location {
        let n = i + 1;
        Location::new().xs(
            n.col().as_left().adjust(-PAD).with(n.col().as_right().adjust(PAD)),
            1.row().as_top().with(1.row().as_bottom()),
        )
    }

    fn build(size: u32) -> (Foliage, Entity, Vec<Entity>) {
        let mut foliage = Foliage::new();
        let field = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    40.0.pct()
                        .as_center_x()
                        .with(COLS.letters().as_width().adjust((COLS - 1) * GAP)),
                    42.0.pct().as_top().with(1.letters().as_height()),
                ))
                .elevate(Elevation::up(2))
                .with((Grid::new(1.letters().gap(GAP), 1.letters()), FontSize::new(size))),
        );
        let letters = ['a', 'w', 'g']
            .iter()
            .enumerate()
            .map(|(i, ch)| {
                foliage.world.branch(
                    field,
                    Text::new(ch.to_string())
                        .size(FontSize::new(size))
                        .at(cell(i as i32))
                        .elevate(Elevation::up(2))
                        .with(HorizontalAlignment::Center),
                )
            })
            .collect::<Vec<_>>();
        foliage.world.flush();
        (foliage, field, letters)
    }

    #[test]
    fn growing_font_size_at_runtime_matches_building_at_that_size() {
        let (mut foliage, field, letters) = build(INITIAL);
        let (reference, ref_field, ref_letters) = build(GROWN);

        for l in letters.iter().copied() {
            foliage.world.entity_mut(l).insert(FontSize::new(GROWN));
        }
        foliage.world.entity_mut(field).insert(FontSize::new(GROWN));
        foliage.world.flush();

        let got_field = *foliage.world.get::<Section<Logical>>(field).unwrap();
        let want_field = *reference.world.get::<Section<Logical>>(ref_field).unwrap();
        assert_eq!(got_field, want_field, "field's own section after the grow");

        for (i, (grown, built)) in letters.iter().zip(ref_letters.iter()).enumerate() {
            let got = *foliage.world.get::<Section<Logical>>(*grown).unwrap();
            let want = *reference.world.get::<Section<Logical>>(*built).unwrap();
            assert_eq!(got, want, "letter {i}'s section after the grow");
            let got_bounds = *foliage.world.get::<TextBounds>(*grown).unwrap();
            let want_bounds = *reference.world.get::<TextBounds>(*built).unwrap();
            assert_eq!(got_bounds, want_bounds, "letter {i}'s TextBounds after the grow");
            // the fontdue layout itself, not `Glyphs::glyphs` -- that mirror is filled by
            // `resolve_glyphs` in the diff schedule, which `flush()` alone never runs.
            let got_glyph = foliage.world.get::<Glyphs>(*grown).unwrap().layout.glyphs()[0];
            let want_glyph = reference.world.get::<Glyphs>(*built).unwrap().layout.glyphs()[0];
            assert_eq!(
                (got_glyph.x, got_glyph.y, got_glyph.width, got_glyph.height),
                (want_glyph.x, want_glyph.y, want_glyph.width, want_glyph.height),
                "letter {i}'s glyph placement after the grow"
            );
        }
    }
}
