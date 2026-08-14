mod glyph;
pub(crate) mod monospaced;
mod pipeline;

use crate::AsTree;
use crate::Differential;
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
    Attachment, Layout, LayoutSection, Location, Parent, Physical, Resolve, Resolved,
    ResolvedElevation, ResolvedVisibility, Short, Tree, View, Visibility,
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
        // `update_from_section` sits at `Prepare`, ahead of the glyph work at `Finalize`,
        // and explicitly *after* the scroll pass. Sharing the set is not enough: this reads
        // `Section` under a `Changed` filter and `propagate_offsets` is what mutates it, so
        // without the edge both orders are legal and the losing one leaves `TextBounds` a
        // frame behind the box for as long as a scroll continues -- and `TextBounds` is a
        // scissor the text pipeline intersects with the span clip, so a lag larger than the
        // box's own height leaves the two rects disjoint and the run renders not at all.
        // The `ApplyDeferred` after `Prepare` then lands the `TextBounds` this triggers, in
        // time for `Extract` to ship it.
        //
        // Kept here rather than solved the way `Image::update` solves the same
        // `Changed<Section>` dependency (by sitting in the later `Finalize` set): the
        // `Resolve<Text>` this sends has to reach `Text::update` -- and so write `Glyphs` --
        // before `resolve_glyphs`/`resolve_colors` consume them at `Finalize`.
        foliage.diff.add_systems(
            Text::update_from_section
                .after(crate::grid::view::propagate_offsets)
                .in_set(DiffMarkers::Prepare),
        );
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
#[require(ResolvedGlyphs, ResolvedColors, GlyphColors)]
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
/// [`Section`] and drawn from a per-entity glyph atlas.
///
/// This is the render marker and is spawned through [`Text::new`], not constructed
/// directly. To change the string afterwards, write [`TextValue`] --
/// the public value channel every text-bearing composite shares.
///
/// Layout is a fixed monospace grid: every glyph advances by the same width, taken from
/// one reference character at the current [`FontSize`]. That pitch is also what
/// [`.letters()`](crate::GridExt::letters) resolves against, so a `Location` can be sized
/// in characters rather than pixels. A glyph wider than the reference advance overhangs
/// its cell rather than widening it.
///
/// The entity's `Section` is the layout box: it bounds wrapping and doubles as the render
/// scissor. A [`text_content()`](crate::text_content) width or height inverts that on
/// that axis, sizing the box from the glyphs instead -- and a content width also means no
/// right edge to wrap against, so the run stays on one line.
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
impl crate::Author for TextSprout {
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
/// A text-bearing entity's public value channel: write it to a [`Text`] entity, or to a root
/// that forwards it like [`TextInput`](crate::TextInput), and the content follows.
#[derive(Component, Clone, Default)]
pub struct TextValue(pub String);
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
                tree.write_to(this, Text::new_marker(&value.0));
            }
        }
    }
    /// Starts a [`Text`] entity carrying `value`:
    /// `canopy.branch(parent, Text::new("hello").size(FontSize::new(24)).at(loc))`.
    ///
    /// Chain [`size`](TextSprout::size)/[`color`](TextSprout::color)/
    /// [`glyph_colors`](TextSprout::glyph_colors) here, and the usual
    /// [`Sprout`](crate::Sprout) placement (`.at()`, `.elevate()`, ...) as with any leaf.
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
        let mut tree = world.tree();
        tree.subscribe(this, Remove::push_remove_packet::<Text>);
        tree.subscribe(this, Visibility::push_remove_packet::<Text>);
        tree.subscribe(this, Self::clear_last_on_visibility);
    }
    fn responsive_font_size(
        _trigger: Trigger<Resolved<Layout>>,
        mut font_sizes: Query<(&FontSize, &mut ResolvedFontSize)>,
        layout: Res<Layout>,
        short: Res<Short>,
    ) {
        for (font_size, mut resolved_font_size) in font_sizes.iter_mut() {
            resolved_font_size.value = font_size.resolve(*layout, *short);
        }
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world.tree().send_to(Resolve::<Text>::new(), this);
    }
    /// Driven by change detection rather than by `Resolved<Section<Logical>>`, because that
    /// event only fires on an *insert*: a scroll moves a whole subtree by mutating `Section`
    /// (see `grid::view::propagate_offsets`), and an observer would never hear about it, so
    /// the render scissor would sit where the text used to be. `Changed` covers both, since
    /// an insert marks the component changed too.
    fn update_from_section(
        moved: Query<Entity, (Changed<Section<Logical>>, With<Text>)>,
        mut tree: Tree,
    ) {
        for entity in moved.iter() {
            tree.send_to(Resolve::<Text>::new(), entity);
        }
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
        mut cache: Query<&mut UpdateCache>,
        font: Res<MonospacedFont>,
        font_ids: Query<&FontId>,
        scale_factor: Res<ScaleFactor>,
        locations: Query<&Location>,
        layout: Res<Layout>,
        short: Res<Short>,
        stems: Query<&Parent>,
        views: Query<&View>,
    ) {
        let this = trigger.event_target();
        // Read off the `Location` rather than out of flags beside it: an axis is content-sized
        // exactly when it says `text_content()`, which is also what makes the measure written
        // below survive the next resolve. See `Location::content_axes`.
        let (auto_width, auto_height) = locations
            .get(this)
            .map(|l| l.content_axes(*layout, *short))
            .unwrap_or((false, false));
        let font_id = font_ids.get(this).copied().unwrap_or_default();
        let font_size = ResolvedFontSize::new(
            // `round`, not `as`'s own truncation: this is the px size the atlas bitmap is
            // rasterized at, and truncating drops up to a whole physical pixel off it. The
            // cost is relative, so it lands hardest on the smallest text -- at a scale
            // factor of 1.73 a logical 12 rasterizes at 20 instead of 21, a 5% error, while
            // a logical 32 is under 1%. On an integral scale factor it never triggers.
            (font_sizes.get(this).unwrap().value as f32 * scale_factor.value()).round() as u32,
        );
        let section = sections
            .get(this)
            .unwrap()
            .to_physical(scale_factor.value());
        let horizontal = *horizontal_alignment.get(this).unwrap();
        let vertical = *vertical_alignment.get(this).unwrap();
        let text = texts.get(this).unwrap();
        // Compared against the cache field by field rather than by building a whole
        // `UpdateCache` up front: doing that clones the string on every call, and a scroll
        // calls this once per text entity per frame purely because the box moved.
        //
        // `content_changed` is the distinction consumers that care about "the text *value*
        // changed, glyphs are freshly laid out" (like `TextInput`'s scroll-into-view) need
        // -- `Resolved<Text>` fires for a pure move too and can't make it.
        let cached = cache.get(this).unwrap();
        let content_changed = cached.text.value != text.value;
        // Only the inputs fontdue actually consumes. A pure position shift (a parent's
        // scroll offset moving this box, width and height untouched) changes where the
        // glyphs are drawn, not how they are laid out. The distinction has to hold:
        // relayout marks `Glyphs` changed even when it recomputes an identical layout,
        // and `TextInput::resync_on_glyphs_changed` answers that with another `Section`
        // write -- so relaying out on position would spin that loop every frame it
        // scrolls, with nothing to converge on.
        let layout_dirty = cached.font_size != font_size
            || content_changed
            || cached.horizontal_alignment != horizontal
            || cached.vertical_alignment != vertical
            || cached.section.width() != section.width()
            || cached.section.height() != section.height();
        let moved = cached.section.position != section.position;
        if !layout_dirty && !moved {
            return;
        }
        // A move alone: the glyphs keep their relative places, so the only things that
        // change are where the run is drawn and where it is scissored. Everything below --
        // the character-set count, the line metrics, the auto-size measure -- is a function
        // of the string, the font size and the box's *extent*, none of which a move touches.
        // This is the whole of a scrolling frame's text work.
        if !layout_dirty {
            cache.get_mut(this).unwrap().section = section;
            tree.write_to(this, TextBounds(section));
            tree.send_to(Resolved::<Text>::new(), this);
            return;
        }
        let mut current = UpdateCache {
            font_size,
            text: text.clone(),
            section,
            horizontal_alignment: horizontal,
            vertical_alignment: vertical,
        };
        {
            let mut glyphs = glyph_query.get_mut(this).unwrap();
            glyphs.layout.reset(&fontdue::layout::LayoutSettings {
                horizontal_align: current.horizontal_alignment.into(),
                vertical_align: current.vertical_alignment.into(),
                max_width: if auto_width {
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
            let dims = font.character_block(font_id, current.font_size.value);
            // One box, both axes applied independently. Asking for both is coherent here
            // because a content-sized width means no `max_width`, so the run is a single line
            // and the glyph advance and the laid-out height describe the same box.
            let adjusted = (auto_width || auto_height).then(|| {
                let mut section = current.section;
                if auto_width {
                    section = section.with_width(glyphs.layout.glyphs().len() as f32 * dims.a());
                }
                if auto_height {
                    section = section.with_height(glyphs.layout.height());
                }
                section.to_logical(scale_factor.value())
            });
            let mut insert_adjusted = false;
            if let Some(adjusted) = adjusted {
                let scaled = adjusted.to_physical(scale_factor.value());
                tracing::trace!(
                    entity = ?this,
                    auto_width,
                    auto_height,
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
                max.checked_sub(1).unwrap_or_default() + if auto_width { 1 } else { 0 };
            tree.write_to(
                this,
                (
                    UniqueCharacters::count(&current.text),
                    TextBounds(current.section),
                    line_metrics,
                ),
            );
            // Written in place rather than inserted: nothing observes `UpdateCache`, it is
            // this function's own record of what the last layout was computed from, and an
            // insert would clone the string a second time.
            *cache.get_mut(this).unwrap() = current;
            if let Some(adjusted) = adjusted {
                if insert_adjusted {
                    // `adjusted` came from this entity's own `Section`, so it is screen
                    // space. Writing only that would leave `LayoutSection` holding the
                    // pre-glyph size -- which is both what children resolve against and
                    // what carries the re-resolve cascade, so the caret and highlight
                    // panels inside a text input would keep laying out against a box the
                    // text has already outgrown. State the same box in both spaces.
                    let accumulated = stems
                        .get(this)
                        .ok()
                        .and_then(|s| s.id)
                        .and_then(|p| views.get(p).ok().map(|v| v.accumulated_offset))
                        .unwrap_or_default();
                    let mut in_layout_space = adjusted;
                    in_layout_space.position += accumulated;
                    // `Section` first: `LayoutSection`'s insert is what re-resolves whatever
                    // is anchored to this box, and an anchor value is read out of the
                    // target's `Section`. Inserted the other way round, a dependent resolves
                    // against the box this adjustment is in the middle of replacing.
                    tree.write_to(this, (adjusted, LayoutSection(in_layout_space)));
                }
            }
            tree.send_to(Resolved::<Text>::new(), this);
            if content_changed {
                tree.send_to(TextContentChanged::new(), this);
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
            world.tree().send_to(Resolve::<Text>::new(), this);
        }
        // A `.letters()`-sized `Location` resolves its numbers out of this entity's own
        // `FontSize` (`grid/location.rs`'s own `letter_dims`), so this is the one place
        // that dependency gets honored: the rest of the resolve triggers are structural
        // (`Location`/`Parent`/`Visibility` written, or a parent's `Section` landing) and
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
            world.tree().send_to(Resolve::<Location>::new(), this);
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
    /// Overrides all of the above on a vertically cramped viewport -- see [`short`](Self::short).
    pub short: Option<u32>,
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
            short: None,
        }
    }
    /// Overrides every width breakpoint while the viewport is vertically cramped, exactly
    /// as [`Location::short`](crate::Location::short) does.
    ///
    /// Width alone gets type badly wrong in landscape: a phone on its side is `md`-wide, so
    /// it takes the `md` size -- but has a fraction of the height to put it in, and large
    /// type is what runs out of room first.
    pub fn short(mut self, value: u32) -> Self {
        self.short.replace(value);
        self
    }
    /// The size in force at `layout`, falling back down the breakpoints to `xs`. `short`
    /// wins outright when the viewport is cramped and one was set.
    pub(crate) fn resolve(&self, layout: Layout, short: Short) -> u32 {
        if short == Short::Yes
            && let Some(value) = self.short
        {
            return value;
        }
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
        let short = *world.get_resource::<Short>().unwrap();
        let comp = world.get::<FontSize>(this).unwrap();
        let resolved = comp.resolve(layout, short);
        world.tree().write_to(this, ResolvedFontSize::new(resolved));
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
            short: None,
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
