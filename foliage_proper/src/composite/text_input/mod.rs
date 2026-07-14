pub(crate) mod action;
pub(crate) mod keybindings;

use crate::composite::Root;
use crate::coordinate::position::Position;
use crate::foliage::MainMarkers;
use crate::ginkgo::ScaleFactor;
use crate::grid::view::ViewAdjustment;
use crate::interaction::CurrentInteraction;
use crate::text::monospaced::MonospacedFont;
use crate::text::{Glyphs, LineMetrics};
use crate::Trigger;
use crate::{
    auto, Attachment, AutoHeight, AutoWidth, Color, Component, Dragged, EcsExtension, Elevation,
    Engaged, Event, FocusBehavior, Foliage, FontSize, GlyphOffset, Grid, GridExt, InputSequence,
    InteractionListener, InteractionPropagation, Key, Layout, Leaf, LeafSprout, Location, Logical,
    Opacity, OverscrollPropagation, Panel, Section, Sprout, Stem, Text, TextValue, Tree, Unfocused,
    View,
};
use action::{InputAction, TextInputAction};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::prelude::IntoScheduleConfigs;
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, Res, SystemParam};
use keybindings::KeyBindings;
use std::collections::HashMap;
use std::ops::Range;

/// `scale_factor`/`views`/`sections` bundled into one `SystemParam` -- `bevy_ecs`'s
/// `SystemParam` tuple impl only goes up to 16 (`all_tuples!(.., 0, 16, ..)` in
/// `system_param.rs`), and `Input::obs` was already close to that before these three were
/// needed for scroll-into-view. A `#[derive(SystemParam)]` struct counts as one parameter to
/// the outer system no matter how many fields it bundles.
#[derive(SystemParam)]
pub(crate) struct ScrollContext<'w, 's> {
    scale_factor: Res<'w, ScaleFactor>,
    views: Query<'w, 's, &'static View>,
    sections: Query<'w, 's, &'static Section<Logical>>,
}

/// Largest char boundary strictly before `byte` (0 if none). All cursor/selection offsets are
/// byte offsets into the `TextValue` string (matching fontdue's `byte_offset`), so every text
/// mutation must move between boundaries with these instead of `± 1`, or multi-byte characters
/// panic `String::remove`/slicing.
fn prev_boundary(s: &str, byte: usize) -> usize {
    let mut b = byte.min(s.len()).saturating_sub(1);
    while b > 0 && !s.is_char_boundary(b) {
        b -= 1;
    }
    b
}
/// Smallest char boundary strictly after `byte` (`s.len()` if none).
fn next_boundary(s: &str, byte: usize) -> usize {
    let mut b = (byte + 1).min(s.len());
    while b < s.len() && !s.is_char_boundary(b) {
        b += 1;
    }
    b
}
/// Clamps a selection range onto char boundaries so it is always safe to slice with.
fn align_range(s: &str, range: &Range<GlyphOffset>) -> Range<GlyphOffset> {
    let mut start = range.start.min(s.len());
    while start > 0 && !s.is_char_boundary(start) {
        start -= 1;
    }
    let mut end = range.end.min(s.len());
    while end < s.len() && !s.is_char_boundary(end) {
        end += 1;
    }
    start..end
}

impl Attachment for TextInput {
    fn attach(foliage: &mut Foliage) {
        // Only genuinely multi-origin/reactive/public steps are registered as observers.
        // LocationFromClick/MoveCursor/ExtendRange/ClearSelection/ReselectRange/ForwardText used
        // to each be their own event+observer even though every one of them has exactly one
        // caller-family (an `Input::obs` action arm) and no external subscriber -- that meant a
        // single keystroke fanned out across 4-5 separate *deferred* trigger dispatches, each
        // re-fetching its own Querys from whatever the command queue happened to have applied so
        // far. They're now plain functions called directly, in order, within one system: same
        // sequencing, no re-entrant staleness risk.
        foliage.define(TextInputState::obs);
        foliage.define(PlaceCursor::obs);
        foliage.define(Input::obs);
        foliage.define(Input::forward);
        foliage.define(InsertText::obs);
        foliage.world.insert_resource(KeyBindings::default());
        // Change-detection safety net, not another manually-threaded call site: every place
        // that mutates `Selection` (there were a dozen `clear_selection` call sites alone)
        // would otherwise need to remember to also call `reselect_range` to despawn/sync the
        // per-glyph highlight panels -- `clear_selection` never did, so plain (non-Shift)
        // arrow keys reset the selection data but left the highlight panels on screen
        // forever. This catches every `Selection` write, including future ones, instead of
        // relying on every caller getting it right by hand.
        foliage
            .main
            .add_systems(TextInput::sync_highlights_to_selection.in_set(MainMarkers::Process));
        // Same shape, different staleness: the cursor's `Location` isn't "wherever
        // byte-offset N is" -- it's a snapshot, `(col, row)` numbers looked up in the glyph
        // layout at the time of the last click/keystroke. A parent resize re-triggers
        // `Update<Location>` for the cursor (via `Section::on_insert`'s cascade) and
        // correctly re-resolves that *same stored* `(col, row)` against the new context,
        // but nothing re-derives `(col, row)` from the cursor's actual byte-offset against
        // the *new* (rewrapped) glyph layout -- same offset, different row after a rewrap,
        // and the resize alone never calls `move_cursor` to notice. Highlight panels have
        // the identical exposure (their positions are also glyph-layout lookups baked in at
        // `reselect_range` time) -- `sync_highlights_to_selection` only catches range
        // changes, not glyphs reflowing under an unchanged range. Reacting to `Glyphs`
        // itself changing (typing *and* rewrap-from-resize both touch it) covers both.
        foliage
            .main
            .add_systems(TextInput::resync_on_glyphs_changed.in_set(MainMarkers::Process));
    }
}
#[derive(Component, Copy, Clone)]
#[require(LineConstraint, Cursor, Selection, HintText, HintColor)]
#[require(TextInputStyle, FontSize, TextValue)]
pub struct TextInput {}
/// TextInput's OWN config vocabulary (widgets each own theirs -- nothing is library-blessed),
/// poked as one unit: `tree.write_to(input, TextInputStyle { .. })`.
#[derive(Component, Copy, Clone, Default)]
pub struct TextInputStyle {
    /// text + hint content color
    pub foreground: Color,
    /// field/backdrop panel color
    pub background: Color,
    /// cursor + selection-highlight color
    pub accent: Color,
}
impl TextInput {
    const HIGHLIGHT_SCROLL_THRESHOLD: f32 = 10.0;
    pub fn new() -> TextInputSprout {
        TextInputSprout::default()
    }
    pub(crate) fn new_marker() -> TextInput {
        TextInput {}
    }
}
#[derive(Default)]
pub struct TextInputSprout {
    leaf: LeafSprout,
    text: Option<String>,
    style: TextInputStyle,
    font_size: Option<FontSize>,
    hint_text: Option<String>,
    line_constraint: Option<LineConstraint>,
}
impl Sprout for TextInputSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            TextInput::new_marker(),
            TextValue(self.text.unwrap_or_default()),
            self.style,
            self.font_size.unwrap_or_default(),
            HintText::new(self.hint_text.unwrap_or_default()),
            self.line_constraint.unwrap_or_default(),
            Grid::default(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // static skeleton. One `branch` per child; the parent is a required argument, so a
        // child can't be orphaned. `cursor` (no visual of its own -- a bare interaction
        // hit-area) uses `Leaf::sprout()`.
        let panel = tree.branch(
            this,
            Panel::new()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    0.pct()
                        .as_left()
                        .adjust(4)
                        .with(100.pct().as_right().adjust(-4)),
                    0.pct()
                        .as_top()
                        .adjust(4)
                        .with(100.pct().as_bottom().adjust(-4)),
                ))
                .with((
                    Grid::new(1.letters(), 1.letters()),
                    InteractionListener::new(),
                    Root(this),
                )),
        );
        tree.subscribe(panel, TextInput::unfocused);
        tree.subscribe(panel, PlaceCursor::forward);

        // panel owns cursor/visible/text/hint_text -- branching from it mirrors the
        // real hierarchy.
        let cursor = tree.branch(
            panel,
            Leaf::sprout()
                .elevate(Elevation::up(6))
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    1.col().as_top().with(1.col().as_bottom()),
                ))
                .with((
                    InteractionListener::new(),
                    InteractionPropagation::pass_through(),
                    Root(this),
                )),
        );
        let visible = tree.branch(
            panel,
            Panel::new()
                .elevate(Elevation::up(3))
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    1.col().as_top().with(1.col().as_bottom()),
                ))
                .with((
                    InteractionListener::new(),
                    InteractionPropagation::pass_through(),
                    FocusBehavior::ignore(),
                    Root(this),
                )),
        );
        // no Location / auto flags: LineConstraint-dependent, set by that reaction's
        // first fire below, in the same command batch.
        let text = tree.branch(
            panel,
            Text::new("")
                .elevate(Elevation::up(5))
                .with((InteractionListener::new(), Root(this))),
        );
        let hint_text = tree.branch(
            panel,
            Text::new("").elevate(Elevation::up(4)).with((
                InteractionPropagation::pass_through(),
                FocusBehavior::ignore(),
                Root(this),
            )),
        );
        tree.subscribe(cursor, TextInput::unfocused);
        tree.subscribe(cursor, Cursor::engaged);
        tree.subscribe(cursor, Selection::select);
        tree.subscribe(text, TextInput::unfocused);
        tree.subscribe(text, PlaceCursor::forward);
        tree.subscribe(text, Selection::reselect);
        tree.subscribe(this, TextInput::unfocused);

        // Handle BEFORE the reactions: their first fires look it up.
        tree.write_to(
            this,
            Handle {
                panel,
                text,
                hint_text,
                cursor,
                visible,
                highlights: Default::default(),
            },
        );

        // everything data-dependent, initial state included
        tree.react::<LineConstraint, _>(this, TextInput::update_line_constraint);
        tree.react::<TextValue, _>(this, TextInput::update_text_value);
        tree.react::<TextInputStyle, _>(this, TextInput::update_style);
        tree.react::<FontSize, _>(this, TextInput::update_font_size);
        tree.react::<HintText, _>(this, TextInput::update_hint);
    }
}
impl TextInputSprout {
    pub fn text(mut self, t: impl Into<String>) -> Self {
        self.text = Some(t.into());
        self
    }
    pub fn foreground(mut self, c: Color) -> Self {
        self.style.foreground = c;
        self
    }
    pub fn background(mut self, c: Color) -> Self {
        self.style.background = c;
        self
    }
    pub fn accent(mut self, c: Color) -> Self {
        self.style.accent = c;
        self
    }
    pub fn font_size(mut self, f: FontSize) -> Self {
        self.font_size = Some(f);
        self
    }
    pub fn hint_text(mut self, t: impl Into<String>) -> Self {
        self.hint_text = Some(t.into());
        self
    }
    pub fn line_constraint(mut self, l: LineConstraint) -> Self {
        self.line_constraint = Some(l);
        self
    }
}
impl TextInput {
    pub(crate) fn unfocused(
        trigger: Trigger<Unfocused>,
        mut tree: Tree,
        roots: Query<&Root>,
        handles: Query<&Handle>,
        current_interaction: Res<CurrentInteraction>,
        mut selections: Query<&mut Selection>,
    ) {
        let main = Root::resolve(trigger.event_target(), &roots);
        let handle = handles.get(main).unwrap();
        if let Some(f) = current_interaction.focused {
            if f == main || f == handle.panel || f == handle.text || f == handle.cursor {
                return;
            }
        }
        Self::clear_selection(main, &mut selections);
        tree.trigger_targets(TextInputState::Inactive, main);
    }
    /// The `LineConstraint` reaction: single-line inputs auto-size width, multi-line
    /// inputs auto-size height. First fire places text/hint (they spawn without a
    /// `Location`); later writes re-place them.
    fn update_line_constraint(
        trigger: Trigger<Insert, LineConstraint>,
        mut tree: Tree,
        constraints: Query<&LineConstraint>,
        handles: Query<&Handle>,
    ) {
        let this = trigger.event_target();
        let line_constraint = *constraints.get(this).unwrap();
        let handle = handles.get(this).unwrap();
        let text_location = Location::new().xs(
            match line_constraint {
                LineConstraint::Single => 0.pct().as_left().with(auto().as_width()),
                LineConstraint::Multiple => 0.pct().as_left().with(100.pct().as_right()),
            },
            match line_constraint {
                LineConstraint::Single => 0.pct().as_top().with(100.pct().as_bottom()),
                LineConstraint::Multiple => 0.pct().as_top().with(auto().as_height()),
            },
        );
        let auto_width = match line_constraint {
            LineConstraint::Single => AutoWidth(true),
            LineConstraint::Multiple => AutoWidth(false),
        };
        let auto_height = match line_constraint {
            LineConstraint::Single => AutoHeight(false),
            LineConstraint::Multiple => AutoHeight(true),
        };
        tree.entity(handle.text)
            .insert((text_location, auto_width, auto_height));
        tree.entity(handle.hint_text)
            .insert((text_location, auto_width, auto_height));
    }
    fn update_text_value(
        trigger: Trigger<Insert, TextValue>,
        mut tree: Tree,
        values: Query<&TextValue>,
        handles: Query<&Handle>,
        mut selections: Query<&mut Selection>,
    ) {
        let this = trigger.event_target();
        Self::forward_text(this, &mut tree, &values, &handles);
        Self::clear_selection(this, &mut selections);
        tree.trigger_targets(TextInputState::Inactive, this);
    }
    fn update_font_size(
        trigger: Trigger<Insert, FontSize>,
        mut tree: Tree,
        font_sizes: Query<&FontSize>,
        handles: Query<&Handle>,
    ) {
        let handle = handles.get(trigger.event_target()).unwrap();
        tree.entity(handle.text)
            .insert(font_sizes.get(trigger.event_target()).unwrap().clone());
        tree.entity(handle.hint_text)
            .insert(font_sizes.get(trigger.event_target()).unwrap().clone());
        tree.entity(handle.panel)
            .insert(font_sizes.get(trigger.event_target()).unwrap().clone());
    }
    fn update_style(
        trigger: Trigger<Insert, TextInputStyle>,
        mut tree: Tree,
        handles: Query<&Handle>,
        styles: Query<&TextInputStyle>,
    ) {
        let handle = handles.get(trigger.event_target()).unwrap();
        let style = *styles.get(trigger.event_target()).unwrap();
        tree.entity(handle.text).insert(style.foreground);
        tree.entity(handle.hint_text).insert(style.foreground);
        tree.entity(handle.panel).insert(style.background);
        tree.entity(handle.visible).insert(style.accent);
        for (_, (e, _, _)) in handle.highlights.iter() {
            tree.entity(*e).insert(style.accent);
        }
    }
    // TODO: hint text sometimes renders clipped on initial spawn (e.g. "multiline" instead
    // of the full "multiline input..."). Specifically only self-corrects on a *resize*, not
    // on any other action (click/keystroke do NOT fix it) -- points at the hint text's
    // `Section` being stuck at an initial/wrong width until something forces a general
    // grid-resolve pass, since resize is the one thing that touches width broadly and clicks
    // /keystrokes don't touch the hint text's `Section` at all. Not root-caused yet -- that's
    // still an inference from the symptom, not a diagnosis; confirmed NOT an
    // entrance-animation settling artifact (there isn't one here; ruled out when raised).
    // Needs an actual repro trace before touching anything.
    fn update_hint(
        trigger: Trigger<Insert, HintText>,
        mut tree: Tree,
        handles: Query<&Handle>,
        hints: Query<&HintText>,
        hint_colors: Query<&HintColor>,
        values: Query<&TextValue>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let hint = hints.get(this).unwrap();
        tree.entity(handle.hint_text)
            .insert(Text::new_marker(&hint.0))
            .insert(crate::Visibility::new(
                values.get(this).unwrap().0.is_empty(),
            ));
        if let Ok(hint_color) = hint_colors.get(this) {
            tree.entity(handle.hint_text).insert(hint_color.0);
        }
    }

    /// Was `Input::forward_to_text`/`ForwardText::obs`: sync the root's `TextValue` onto the
    /// displayed text + hint visibility, and broadcast `TextChanged` for enclosing composites.
    fn forward_text(
        this: Entity,
        tree: &mut Tree,
        values: &Query<&TextValue>,
        handles: &Query<&Handle>,
    ) {
        let handle = handles.get(this).unwrap();
        let value = values.get(this).unwrap();
        tree.write_to(handle.text, Text::new_marker(&value.0));
        tree.write_to(handle.hint_text, crate::Visibility::new(value.0.is_empty()));
        tree.trigger_targets(TextChanged::new(), this);
    }
    /// Was `ClearSelection::obs`.
    fn clear_selection(this: Entity, selections: &mut Query<&mut Selection>) {
        if let Ok(mut selection) = selections.get_mut(this) {
            selection.range = Range::default();
        }
    }
}
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum TextInputStage {
    Inactive,
    Highlighting,
    AwaitingInput,
}
/// bevy 0.19 targeted events must be structs (the target entity lives inside the event), so the
/// old `TextInputState` enum became this struct + `TextInputStage`. The associated consts keep
/// every `TextInputState::AwaitingInput`-style trigger site reading exactly as before.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub(crate) struct TextInputState {
    pub(crate) stage: TextInputStage,
}
#[allow(non_upper_case_globals)]
impl TextInputState {
    pub(crate) const Inactive: Self = Self {
        entity: Entity::PLACEHOLDER,
        stage: TextInputStage::Inactive,
    };
    pub(crate) const Highlighting: Self = Self {
        entity: Entity::PLACEHOLDER,
        stage: TextInputStage::Highlighting,
    };
    pub(crate) const AwaitingInput: Self = Self {
        entity: Entity::PLACEHOLDER,
        stage: TextInputStage::AwaitingInput,
    };
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        handles: Query<&Handle>,
        styles: Query<&TextInputStyle>,
        virtual_keyboard: Res<crate::virtual_keyboard::VirtualKeyboardAdapter>,
    ) {
        let value = trigger.event();
        let handle = handles.get(trigger.event_target()).unwrap();
        match value.stage {
            TextInputStage::Inactive => {
                virtual_keyboard.close();
                tree.write_to(trigger.event_target(), OverscrollPropagation(true));
                tree.write_to(handle.visible, Opacity::new(0.0));
                tree.write_to(handle.cursor, InteractionPropagation::pass_through());
                tree.disable(handle.cursor);
            }
            TextInputStage::Highlighting => {
                tree.write_to(trigger.event_target(), OverscrollPropagation(false));
                tree.write_to(
                    handle.visible,
                    (
                        Opacity::new(0.75),
                        styles.get(trigger.event_target()).unwrap().foreground,
                    ),
                )
            }
            TextInputStage::AwaitingInput => {
                virtual_keyboard.open(crate::virtual_keyboard::VirtualKeyboardType::Keyboard);
                tree.write_to(trigger.event_target(), OverscrollPropagation(true));
                tree.write_to(handle.cursor, InteractionPropagation::grab().disable_drag());
                tree.write_to(
                    handle.visible,
                    (
                        Opacity::new(0.25),
                        styles.get(trigger.event_target()).unwrap().accent,
                    ),
                );
                tree.enable(handle.cursor);
            }
        }
    }
}
#[derive(Component, Copy, Clone, Default)]
pub enum LineConstraint {
    #[default]
    Single,
    Multiple,
}
#[derive(Component, Copy, Clone, Default)]
pub(crate) struct Cursor {
    pub(crate) location: GlyphOffset,
    pub(crate) column: u32,
    pub(crate) row: u32,
}
impl Cursor {
    pub(crate) fn new() -> Self {
        Self {
            location: 0,
            column: 0,
            row: 0,
        }
    }
    // we clicked explicitly on cursor, start drag behavior
    pub(crate) fn engaged(trigger: Trigger<Engaged>, mut tree: Tree, roots: Query<&Root>) {
        tree.trigger_targets(
            TextInputState::Highlighting,
            Root::resolve(trigger.event_target(), &roots),
        );
    }
}
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub(crate) struct PlaceCursor {}
impl PlaceCursor {
    pub(crate) fn forward(trigger: Trigger<Engaged>, mut tree: Tree, roots: Query<&Root>) {
        tree.trigger_targets(
            PlaceCursor::new(),
            Root::resolve(trigger.event_target(), &roots),
        );
    }
    pub(crate) fn obs(
        trigger: Trigger<PlaceCursor>,
        mut tree: Tree,
        current_interaction: Res<CurrentInteraction>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        handles: Query<&Handle>,
        mut cursor: Query<&mut Cursor>,
        glyphs: Query<&Glyphs>,
        line_metrics: Query<&LineMetrics>,
        mut selections: Query<&mut Selection>,
        scroll: ScrollContext,
    ) {
        let this = trigger.event_target();
        // TODO: every click unconditionally restarts the selection here, even Shift+Click --
        // some editors instead extend the existing selection to the click point when Shift
        // is held. Not implemented; scoping only, not clear this is wanted yet.
        TextInput::clear_selection(this, &mut selections);
        tree.trigger_targets(TextInputState::AwaitingInput, this);
        let (col, row) = TextInput::location_from_click(
            this,
            true,
            &current_interaction,
            &font,
            &font_sizes,
            *layout,
            &scroll.sections,
            &scroll.views,
            &handles,
            &line_metrics,
        );
        TextInput::move_cursor(
            this,
            &mut tree,
            RequestedLocation::ColRow((col, row)),
            &glyphs,
            &font,
            &font_sizes,
            *layout,
            &handles,
            &mut cursor,
            &line_metrics,
            &scroll,
            true,
            true,
        );
    }
}
/// Ephemeral hand-off value: was a `Component` (`RequestedLocation`) written by one system and
/// immediately consumed by `MoveCursor`'s observer -- nothing else ever read it as persistent
/// state, so it's now just an argument passed directly into `TextInput::move_cursor`.
#[derive(Copy, Clone, Debug)]
pub(crate) enum RequestedLocation {
    Offset(GlyphOffset),
    ColRow((u32, u32)),
}
impl TextInput {
    /// Was `LocationFromClick::obs`: click position -> (column, row) in the text's glyph grid.
    fn location_from_click(
        this: Entity,
        can_go_past_end: bool,
        current_interaction: &CurrentInteraction,
        font: &MonospacedFont,
        font_sizes: &Query<&FontSize>,
        layout: Layout,
        sections: &Query<&Section<Logical>>,
        views: &Query<&View>,
        handles: &Query<&Handle>,
        line_metrics: &Query<&LineMetrics>,
    ) -> (u32, u32) {
        let lfc = u32::from(can_go_past_end);
        let click = current_interaction.click.current;
        let fsv = font_sizes.get(this).unwrap().resolve(layout).value;
        let dims = font.character_block(fsv);
        let section = sections.get(this).unwrap();
        let handle = handles.get(this).unwrap();
        let relative =
            click - section.position - (4, 4).into() + views.get(handle.panel).unwrap().offset;
        let (x, y) = (
            (relative.left().max(0.0) / dims.a()) as u32,
            (relative.top().max(0.0) / dims.b()) as u32,
        );
        let metrics = line_metrics.get(handle.text).unwrap();
        let row = y.min(metrics.lines.len().checked_sub(1).unwrap_or_default() as u32);
        let column = x
            .min(
                metrics
                    .lines
                    .get(row as usize)
                    .and_then(|l| Some(l + lfc))
                    .unwrap_or_default(),
            )
            .min(metrics.max_letter_idx_horizontal);
        (column, row)
    }
    // TODO: scroll up away from the cursor (cursor sitting at/below the bottom of view), then
    // resize the box bigger -- the resulting rewrap drops `view.offset` back toward the
    // cursor instead of leaving the user's manual scroll position alone. Two candidate
    // causes, not yet distinguished by an actual trace: this function's `allow_scroll` gate
    // not covering every path a resize takes through here, or `extent_check`'s regrow/clamp
    // (`grid/view.rs`) recomputing a "valid" offset against the *new*, bigger extent that
    // happens to land back near the cursor as a side effect, independent of any explicit
    // follow logic. Not root-caused or fixed -- resolve which one it actually is before
    // touching anything.

    /// Was `MoveCursor::obs`: resolve a requested (column, row) or byte offset against the
    /// text's actual glyph layout, and place the cursor + its visible indicator there.
    #[allow(clippy::too_many_arguments)]
    fn move_cursor(
        this: Entity,
        tree: &mut Tree,
        req: RequestedLocation,
        glyphs: &Query<&Glyphs>,
        font: &MonospacedFont,
        font_sizes: &Query<&FontSize>,
        layout: Layout,
        handles: &Query<&Handle>,
        cursor: &mut Query<&mut Cursor>,
        line_metrics: &Query<&LineMetrics>,
        scroll: &ScrollContext,
        // Gates writing `Location` onto the cursor/visible entities (the visual position).
        // `false` only for `after_edit`'s own synchronous call, where col/row can still be
        // stale-glyph-derived (it runs before `Text::update` has settled) -- writing a stale
        // visual position there raced against the guaranteed-fresh reactive follow-up
        // (`Selection::reselect`, via `TextContentChanged`) and sometimes landed *after* it,
        // clobbering the correct value. `cursor.location`/`column`/`row` (logical state,
        // always trustworthy) update regardless of `follow`. Everywhere else this is `true`.
        follow: bool,
        // Gates the scroll-into-view check specifically -- separate from `follow` because
        // they answer different questions ("should the visual position update" vs "did the
        // *cursor* actually move, such that following it into view makes sense"). A passive
        // resync after a rewrap (`resync_on_glyphs_changed`) needs `follow = true` (the
        // cursor's row genuinely changed and must redraw there) but `allow_scroll = false`:
        // forcing a scroll there isn't a real cursor move, and doing it anyway closed a loop
        // -- the `ViewAdjustment` it inserted cascaded through `Section::on_insert` back to
        // the panel's children, re-triggering `Text::update`'s `Write<Section<Logical>>`
        // listener, which mutates `Glyphs` again, re-triggering this same resync forever,
        // full glyph-layout rescans every frame. The natural reclamp already in
        // `extent_check_v2` is what should settle the offset once the extent itself is
        // correct, not an explicit force from here.
        allow_scroll: bool,
    ) {
        tracing::trace!(entity = ?this, req = ?req, "text_input: move_cursor invoked");
        // `character_block` is fed a raw pixel size and returns raw pixel metrics -- no
        // inherent Logical/Physical concept, it's whatever space the input size is in.
        // `text_glyphs` below (`Glyphs.layout`) was laid out using the *scaled* font size
        // (`text/mod.rs`'s `Text::update`), i.e. Physical space, so `dims` has to be computed
        // the same way here or every ratio against a real glyph position is off by exactly
        // `scale_factor` -- invisible at scale_factor 1.0 (native desktop), a straight 2x on
        // HiDPI/mobile. `location_from_click` is a different, unrelated computation (compares
        // against `Section<Logical>`-relative click position, not raw glyph data) and must
        // stay unscaled -- don't "fix" it to match this.
        let fsv = font_sizes.get(this).unwrap().resolve(layout).value;
        let dims = font.character_block((fsv as f32 * scroll.scale_factor.value()) as u32);
        // Grid's own `.col()`/`.row()` resolution (`grid/location.rs`'s `calc`, the
        // `LocationValue::Column`/`Row`/`Letters` arms) sizes cells from `stem_letters`,
        // computed via `character_block` at the *unscaled* font size -- deliberately
        // different from `dims` above. Reusing `dims / scale_factor` here to reconstruct a
        // pixel position doesn't reliably round-trip to the same number `stem_letters`
        // gives directly: fontdue's hinting/kerning isn't a linear function of size, so
        // scale-then-unscale drifts from computing at the unscaled size outright. Needed to
        // match Grid's actual pixel math bit-for-bit, or the scroll-into-view check below
        // disagrees with where the cursor really renders.
        let letter_block = font.character_block(fsv);
        let handle = handles.get(this).unwrap();
        let metrics = line_metrics.get(handle.text).unwrap();
        let mut cursor = cursor.get_mut(this).unwrap();
        let text_glyphs = glyphs.get(handle.text).unwrap().layout.glyphs();
        let (location, col, row) = match req {
            RequestedLocation::Offset(offset) => {
                // `text_glyphs` is built by fontdue processing the text strictly
                // left-to-right (`fontdue::layout::Layout::append` walks the string forward,
                // pushing each glyph's `byte_offset` in increasing order), so it's sorted --
                // a binary search finds the same result as the old two-tier linear scan (an
                // exact-match `.find()`, then on miss a backward walk that reran a *full*
                // linear scan for every position stepped back -- worst-case O(n^2) on a
                // large document with runs of skipped/whitespace positions between real
                // glyphs). `perf` showed that scan alone as 96%+ of total CPU time pasting
                // into an already-large `TextInput`.
                match text_glyphs.binary_search_by_key(&offset, |g| g.byte_offset) {
                    Ok(idx) => {
                        let found = &text_glyphs[idx];
                        let col = (found.x / dims.a()) as u32;
                        let row = (found.y / dims.b()) as u32;
                        (found.byte_offset, col, row)
                    }
                    Err(insert_idx) => {
                        // `location` is always the requested offset -- it's the authoritative
                        // value `after_edit` just computed from the real (post-write) text
                        // length, whereas `text_glyphs` here can still be lagging one edit
                        // behind (glyph layout hasn't caught up to a just-applied, still-
                        // deferred text write). Letting the nearest-glyph interpolation below
                        // override `location` made every keystroke land the cursor one
                        // character behind where it was just typed. Only `col`/`row` (visual
                        // placement) are worth best-effort interpolating from a stale glyph;
                        // they can be a frame late without corrupting where the next
                        // character actually gets inserted.
                        //
                        // `insert_idx` is where `offset` would land to keep the list sorted,
                        // so `text_glyphs[insert_idx - 1]` (if any) is the nearest earlier
                        // settled glyph -- the same one the old backward scan would have
                        // walked to, just found directly instead of by stepping through
                        // every position in between.
                        let (col, row) = if insert_idx > 0 {
                            let found = &text_glyphs[insert_idx - 1];
                            // however many positions back that glyph is -- stale `Glyphs` can
                            // be missing more than just the one just-typed character (e.g.
                            // typing outpaces the glyph-recompute cycle), so the column has
                            // to advance by the actual gap, not a hard-coded 1, or the cursor
                            // undershoots by however many positions were skipped.
                            let distance = (offset - found.byte_offset) as u32;
                            let col = ((found.x / dims.a()) as u32 + distance)
                                .min(metrics.max_letter_idx_horizontal);
                            let row = (found.y / dims.b()) as u32;
                            (col, row)
                        } else {
                            // no settled glyph at all to derive a visual column from (e.g. the
                            // very first keystroke into an empty field) -- approximate
                            // directly from the byte offset; self-corrects next tick once
                            // glyphs catch up.
                            (
                                offset.min(metrics.max_letter_idx_horizontal as GlyphOffset) as u32,
                                0,
                            )
                        };
                        (offset, col, row)
                    }
                }
            }
            RequestedLocation::ColRow((c, r)) => {
                // Same shape of bug as the `Offset` arm above, just triggered by scanning
                // columns instead of byte offsets: the old code searched the *entire*
                // document's glyphs for an exact (col, row) match, and on a miss, repeated
                // that full scan once per column stepped back. `LineMetrics.last_offsets`
                // (the byte offset of each row's last glyph) pins down exactly which
                // contiguous slice of the (byte-offset-sorted) `text_glyphs` belongs to row
                // `r`, via two binary searches -- bounding every lookup to that one row's
                // glyphs (tens of characters) instead of the whole document (hundreds of
                // thousands), regardless of how far into the document row `r` is.
                let row_start_offset = if r == 0 {
                    0
                } else {
                    metrics
                        .last_offsets
                        .get(r as usize - 1)
                        .map(|o| *o as GlyphOffset + 1)
                        .unwrap_or(0)
                };
                let row_start_idx = text_glyphs
                    .partition_point(|g| g.byte_offset < row_start_offset)
                    .min(text_glyphs.len());
                let row_end_idx = metrics
                    .last_offsets
                    .get(r as usize)
                    .map(|&last| {
                        text_glyphs.partition_point(|g| g.byte_offset <= last as GlyphOffset)
                    })
                    .unwrap_or(text_glyphs.len())
                    .min(text_glyphs.len());
                let row_glyphs = &text_glyphs[row_start_idx..row_end_idx.max(row_start_idx)];
                match row_glyphs.binary_search_by_key(&c, |g| (g.x / dims.a()) as u32) {
                    Ok(idx) => {
                        let found = &row_glyphs[idx];
                        let col = (found.x / dims.a()) as u32;
                        let row = (found.y / dims.b()) as u32;
                        (found.byte_offset, col, row)
                    }
                    Err(insert_idx) => {
                        if insert_idx > 0 {
                            // nearest earlier column in this row -- the same glyph the old
                            // backward scan would have stopped at first.
                            let found = &row_glyphs[insert_idx - 1];
                            let sc = (found.x / dims.a()) as u32;
                            let col = (sc + 1).min(metrics.max_letter_idx_horizontal);
                            let row = (found.y / dims.b()) as u32;
                            let location = found.byte_offset + 1;
                            (location, col, row)
                        } else {
                            // no glyph in this row at any column before `c` -- same terminal
                            // fallback as the old loop's `sc == 0` case.
                            let col = 0;
                            let row = r;
                            let location = if row == 0 {
                                0
                            } else {
                                *metrics.last_offsets.get(row as usize - 1).unwrap()
                                    as GlyphOffset
                                    + 1
                            };
                            (location, col, row)
                        }
                    }
                }
            }
        };
        // `cursor.location`/`column`/`row` (logical state) are always updated -- `location`
        // (byte offset) is authoritative regardless of glyph staleness (see the comment
        // above), and something needs the target offset available immediately for
        // subsequent typing to insert at the right spot.
        cursor.location = location;
        cursor.column = col;
        cursor.row = row;

        // The *visual* placement (writing `Location` onto the cursor/visible entities, and
        // the scroll-into-view check) is gated on `follow` and skipped entirely when col/row
        // might still be stale-glyph-derived (`after_edit`'s own synchronous call, which
        // runs before `Text::update` has settled). Writing it there anyway created a real
        // race: that write is deferred into the *outer* system's command buffer, while the
        // guaranteed-fresh reactive follow-up (`Selection::reselect`, via
        // `TextContentChanged`) runs its own nested trigger cascade and applies its Commands
        // essentially immediately -- so the outer, stale write was landing *after* the
        // fresher one and clobbering it. Only ever writing the visual position from the
        // fresh call removes the race instead of trying to win it.
        //
        // This used to also skip the write when col/row hadn't moved, to stop
        // `resync_on_glyphs_changed`'s per-frame calls from cascading back into another
        // `Section` write -- but the comparison was against `cursor.column`/`row`, which
        // *every* call updates regardless of `follow` (including `after_edit`'s stale one),
        // so the following genuine `follow=true` call frequently saw "no change" against a
        // value `after_edit` had just quietly set, and skipped the real visual write
        // entirely (cursor never rendered, or froze after the first edit). The actual
        // feedback loop is fixed at its source now: `Text::update` no longer marks `Glyphs`
        // Changed on a pure position shift (see its own comment), so this insert cascading
        // into a `Section` write on the text entity no longer re-triggers a relayout. No
        // gate needed here.
        if follow {
            let visual_location = Location::new().xs(
                (col + 1).col().as_left().with((col + 1).col().as_right()),
                (row + 1).row().as_top().with((row + 1).row().as_bottom()),
            );
            tree.entity(handle.cursor).insert(visual_location);
            tree.entity(handle.visible).insert(visual_location);
        }

        if allow_scroll
            && let (Ok(view), Ok(section)) = (
            scroll.views.get(handle.panel),
            scroll.sections.get(handle.panel),
        ) {
            let cursor_content: Position<Logical> = (
                col as f32 * letter_block.a(),
                row as f32 * letter_block.b(),
            )
                .into();
            let window_relative = cursor_content - view.offset;
            let mut delta = Position::<Logical>::default();
            // "fully in view" means the *whole cell* (col/row extended by one letter's
            // width/height), not just its near edge -- checking only
            // `window_relative.top() > section.height()` missed the case where the row
            // starts exactly at (or is straddling) the bottom edge: a row starting exactly
            // at `section.height()` has zero visible pixels, but `176 > 176` is false, so
            // no correction fired. That's what let the cursor sit rendered off-screen.
            if window_relative.left() < 0.0 {
                delta.set_left(window_relative.left());
            } else if window_relative.left() + letter_block.a() > section.width() {
                delta.set_left(window_relative.left() + letter_block.a() - section.width());
            }
            if window_relative.top() < 0.0 {
                delta.set_top(window_relative.top());
            } else if window_relative.top() + letter_block.b() > section.height() {
                delta.set_top(window_relative.top() + letter_block.b() - section.height());
            }
            tracing::trace!(
                entity = ?this,
                col,
                row,
                letter_block = ?letter_block,
                cursor_content = ?cursor_content,
                view_offset = ?view.offset,
                view_extent = ?view.extent,
                section = ?*section,
                window_relative = ?window_relative,
                delta = ?delta,
                "text_input: scroll-into-view check"
            );
            if delta.left() != 0.0 || delta.top() != 0.0 {
                tree.entity(handle.panel).insert(ViewAdjustment(delta));
            }
        }
    }
}
#[derive(Component, Clone, Default)]
pub(crate) struct Selection {
    pub(crate) range: Range<GlyphOffset>,
    pub(crate) inverted: bool,
}
impl Selection {
    pub(crate) fn reselect(
        trigger: Trigger<crate::text::TextContentChanged>,
        mut tree: Tree,
        roots: Query<&Root>,
        glyphs: Query<&Glyphs>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        handles: Query<&mut Handle>,
        // single mutable Query -- see the comment on `Input::obs`'s `handles` param.
        mut cursor: Query<&mut Cursor>,
        line_metrics: Query<&LineMetrics>,
        scroll: ScrollContext,
    ) {
        let root = Root::resolve(trigger.event_target(), &roots);
        let offset = cursor.get(root).unwrap().location;
        TextInput::move_cursor(
            root,
            &mut tree,
            RequestedLocation::Offset(offset),
            &glyphs,
            &font,
            &font_sizes,
            *layout,
            &handles.as_readonly(),
            &mut cursor,
            &line_metrics,
            &scroll,
            // Now gated at the source (`Text::update` only fires `TextContentChanged` for a
            // genuine value change, not a pure scroll-caused `Section` shift), so this is
            // always a real edit settling with fresh glyph/line data -- safe to follow.
            true,
            // A genuine edit (typing) should scroll the cursor into view.
            true,
        );
        // No direct `reselect_range` call -- a genuine content change always relays out
        // (`Text::update`'s `layout_dirty` includes the text field), so `Changed<Glyphs>`
        // is guaranteed this same frame and `resync_on_glyphs_changed` already does this
        // exact call. `move_cursor` above is still needed on its own: unlike that system's
        // (deliberately non-scrolling) call, this one is a genuine edit and should scroll
        // the cursor into view.
    }
    // TODO: no auto-scroll while dragging near the box's edges -- drag-selection is only
    // ever in view of whatever's currently visible. Calls `extend_range` directly rather
    // than through `move_cursor`/`extend_and_reselect`, so it never touches the
    // scroll-into-view logic keyboard navigation has. Reusing that math but driven
    // continuously while the pointer sits near the boundary (not just once per keystroke)
    // is the likely shape of the fix, not built yet -- scoping only.
    pub(crate) fn select(
        trigger: Trigger<Dragged>,
        roots: Query<&Root>,
        current_interaction: Res<CurrentInteraction>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        sections: Query<&Section<Logical>>,
        views: Query<&View>,
        // single mutable Query -- see the comment on `Input::obs`'s `handles` param.
        handles: Query<&mut Handle>,
        line_metrics: Query<&LineMetrics>,
        cursors: Query<&Cursor>,
        mut selections: Query<&mut Selection>,
        glyphs: Query<&Glyphs>,
    ) {
        let root = Root::resolve(trigger.event_target(), &roots);
        let (col, row) = TextInput::location_from_click(
            root,
            false,
            &current_interaction,
            &font,
            &font_sizes,
            *layout,
            &sections,
            &views,
            &handles.as_readonly(),
            &line_metrics,
        );
        let req = RequestedLocation::ColRow((col, row));
        TextInput::extend_range(
            root,
            req,
            &cursors,
            &mut selections,
            &glyphs,
            &handles.as_readonly(),
            &font,
            &font_sizes,
            *layout,
        );
        // No direct `reselect_range` call -- `sync_highlights_to_selection` (a
        // `Changed<Selection>` reactive safety net, see its comment) picks this up
        // automatically every frame while dragging. Calling it here too just doubled the
        // cost of every glyph the drag passed over.
    }
}
impl TextInput {
    /// Was `ExtendRange::obs`: grow/shrink the active selection to include a newly requested
    /// (column, row).
    fn extend_range(
        this: Entity,
        req: RequestedLocation,
        cursors: &Query<&Cursor>,
        selections: &mut Query<&mut Selection>,
        glyphs: &Query<&Glyphs>,
        handles: &Query<&Handle>,
        font: &MonospacedFont,
        font_sizes: &Query<&FontSize>,
        layout: Layout,
    ) {
        let handle = handles.get(this).unwrap();
        let fsv = font_sizes.get(this).unwrap().resolve(layout).value;
        let dims = font.character_block(fsv);
        let cursor = cursors.get(this).unwrap();
        let mut selection = selections.get_mut(this).unwrap();
        if let RequestedLocation::ColRow((c, r)) = req {
            let text_glyphs = glyphs.get(handle.text).unwrap().layout.glyphs();
            // Exact (col, row) match if one exists; otherwise the row's last glyph. Up/Down
            // landing on a shorter row (crossing into a blank/short line) previously found
            // no exact match at all and silently did nothing -- Shift+Up/Down could get
            // stuck, unable to move past a short line.
            let target = text_glyphs
                .iter()
                .find(|g| (g.x / dims.a()) as u32 == c && (g.y / dims.b()) as u32 == r)
                .or_else(|| {
                    text_glyphs
                        .iter()
                        .filter(|g| (g.y / dims.b()) as u32 == r)
                        .max_by_key(|g| g.byte_offset)
                });
            if let Some(glyph) = target {
                // The anchor is the *fixed* end of an already-in-progress selection --
                // whichever end isn't the one that's been moving -- falling back to the
                // cursor's own position only when starting a fresh selection (empty range).
                // Using `cursor.location` unconditionally here relied on it never changing
                // across repeated Shift+Arrow presses; now that `extend_and_reselect`
                // correctly moves the cursor to the selection's moving edge after every
                // press (fixing "stops after one"), treating that moving position as the
                // anchor again made every press replace the selection with a sliver near
                // wherever the cursor currently was, instead of extending from where it
                // started.
                let anchor = if selection.range.is_empty() {
                    cursor.location
                } else if selection.inverted {
                    selection.range.end
                } else {
                    selection.range.start
                };
                // The target glyph's byte offset is a cursor-equivalent *boundary* (same as
                // what `move_cursor` treats it as), not "a character to include" -- wrapping
                // it in `next_boundary` used to pad the range by one extra character on
                // whichever side was just touched (e.g. one Shift+Right from a fresh
                // selection selected two characters, not one), compounding on every
                // repeated press at a boundary the target couldn't move past (e.g.
                // Shift+Down at the bottom row kept growing the range by one more character
                // each press instead of no-op'ing). The range is just the plain span
                // between anchor and target.
                if anchor < glyph.byte_offset {
                    selection.inverted = false;
                    selection.range = anchor..glyph.byte_offset;
                } else {
                    selection.inverted = true;
                    selection.range = glyph.byte_offset..anchor;
                }
            }
        }
    }
    /// Runs whenever `Selection` changes for *any* reason (typing, clicking, Shift+Arrow,
    /// `clear_selection`) and syncs the highlight-panel-per-glyph set to match -- see the
    /// comment on its registration in `attach` for why this is change-detection-driven
    /// instead of yet another call site callers have to remember.
    fn sync_highlights_to_selection(
        changed: Query<Entity, Changed<Selection>>,
        mut tree: Tree,
        mut handles: Query<&mut Handle>,
        glyphs: Query<&Glyphs>,
        selections: Query<&Selection>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        styles: Query<&TextInputStyle>,
    ) {
        for this in changed.iter() {
            TextInput::reselect_range(
                this,
                &mut tree,
                &mut handles,
                &glyphs,
                &selections,
                &font,
                &font_sizes,
                *layout,
                &styles,
            );
        }
    }
    /// Runs whenever `Glyphs` changes for *any* text entity (typing, or a rewrap from a
    /// parent resize) and re-derives the cursor's `(col, row)` from its actual byte-offset
    /// against the current layout, plus resyncs the highlight panels -- see the comment on
    /// its registration in `attach` for why. `Root::resolve` + the `cursor.get` guard is
    /// what scopes this to text entities that actually belong to a `TextInput` (`hint_text`
    /// resolves to the same root as `text` and just does harmless duplicate work; a
    /// standalone `Text` outside any `TextInput` has no `Cursor` to look up and is skipped).
    fn resync_on_glyphs_changed(
        changed: Query<Entity, Changed<Glyphs>>,
        roots: Query<&Root>,
        mut tree: Tree,
        mut cursor: Query<&mut Cursor>,
        mut handles: Query<&mut Handle>,
        glyphs: Query<&Glyphs>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        line_metrics: Query<&LineMetrics>,
        scroll: ScrollContext,
        selections: Query<&Selection>,
        styles: Query<&TextInputStyle>,
    ) {
        for text_entity in changed.iter() {
            let this = Root::resolve(text_entity, &roots);
            let offset = if let Ok(cursor_val) = cursor.get(this) {
                cursor_val.location
            } else {
                continue;
            };
            TextInput::move_cursor(
                this,
                &mut tree,
                RequestedLocation::Offset(offset),
                &glyphs,
                &font,
                &font_sizes,
                *layout,
                &handles.as_readonly(),
                &mut cursor,
                &line_metrics,
                &scroll,
                true,
                // No scroll-into-view here: this is a passive resync after a rewrap, not a
                // genuine cursor move. Forcing it anyway is what closed the feedback loop --
                // see `move_cursor`'s `allow_scroll` comment.
                false,
            );
            TextInput::reselect_range(
                this,
                &mut tree,
                &mut handles,
                &glyphs,
                &selections,
                &font,
                &font_sizes,
                *layout,
                &styles,
            );
        }
    }
    /// Was `ReselectRange::obs`: sync the highlight-panel-per-*row* set to the current
    /// `Selection.range`.
    ///
    /// One panel per selected *glyph* (the original design) doesn't scale: selecting a large
    /// pasted block spawned/repositioned thousands of individual entities, each a `Location`
    /// insert cascading through `Section`/view-extent resolution -- multi-second freezes (and
    /// eventually a wayland-killing hang) on Ctrl+A, deselect, or especially a resize while a
    /// large selection was active, since a rewrap moves almost every selected glyph's (col,
    /// row) at once. Selected columns within a single row are always contiguous (`Selection`
    /// is one contiguous byte range), so one rectangle per row -- spanning from that row's
    /// first to last selected column -- renders identically with `O(selected_lines)` entities
    /// instead of `O(selected_glyphs)`.
    fn reselect_range(
        this: Entity,
        tree: &mut Tree,
        handles: &mut Query<&mut Handle>,
        glyphs: &Query<&Glyphs>,
        selections: &Query<&Selection>,
        font: &MonospacedFont,
        font_sizes: &Query<&FontSize>,
        layout: Layout,
        styles: &Query<&TextInputStyle>,
    ) {
        let mut handle = handles.get_mut(this).unwrap();
        let selection = selections.get(this).unwrap();
        let glyph = glyphs.get(handle.text).unwrap();
        let fsv = font_sizes.get(this).unwrap().resolve(layout).value;
        let dims = font.character_block(fsv);
        let mut spans: HashMap<u32, (u32, u32)> = HashMap::new();
        for g in glyph
            .layout
            .glyphs()
            .iter()
            .filter(|g| selection.range.contains(&g.byte_offset))
        {
            let col = (g.x / dims.a()) as u32;
            let row = (g.y / dims.b()) as u32;
            spans
                .entry(row)
                .and_modify(|(start, end)| {
                    *start = (*start).min(col);
                    *end = (*end).max(col);
                })
                .or_insert((col, col));
        }
        let stale = handle
            .highlights
            .iter()
            .filter(|(row, _)| !spans.contains_key(row))
            .map(|(row, (e, ..))| (*row, *e))
            .collect::<Vec<_>>();
        let stale_count = stale.len();
        let stale_start = std::time::Instant::now();
        for (row, e) in stale {
            handle.highlights.remove(&row);
            tree.remove(e);
        }
        tracing::trace!(
            entity = ?this,
            stale_count,
            elapsed = ?stale_start.elapsed(),
            "text_input: reselect_range stale-highlight removal"
        );
        let panel = handle.panel;
        let color = styles.get(this).unwrap().accent;
        for (row, (start_col, end_col)) in spans {
            if let Some((existing, prev_start, prev_end)) = handle.highlights.get(&row).copied() {
                // Already highlighted and the span hasn't moved -- skip the re-`insert`
                // entirely (see the comment on `Handle::highlights`).
                if start_col != prev_start || end_col != prev_end {
                    let location = Location::new().xs(
                        (start_col + 1)
                            .col()
                            .as_left()
                            .with((end_col + 1).col().as_right()),
                        (row + 1).row().as_top().with((row + 1).row().as_bottom()),
                    );
                    tree.entity(existing).insert(location);
                    handle.highlights.insert(row, (existing, start_col, end_col));
                }
            } else {
                let location = Location::new().xs(
                    (start_col + 1)
                        .col()
                        .as_left()
                        .with((end_col + 1).col().as_right()),
                    (row + 1).row().as_top().with((row + 1).row().as_bottom()),
                );
                let h = tree.branch(
                    panel,
                    Panel::new()
                        .color(color)
                        .elevate(Elevation::up(2))
                        .at(location)
                        .with((
                            Opacity::new(1.0),
                            InteractionPropagation::pass_through(),
                            FocusBehavior::ignore(),
                        )),
                );
                handle.highlights.insert(row, (h, start_col, end_col));
            }
        }
    }
}

#[foliage_macros::targeted_event]
pub(crate) struct Input {
    pub(crate) sequence: InputSequence,
}
impl Input {
    pub(crate) fn forward(
        trigger: Trigger<InputSequence>,
        mut tree: Tree,
        roots: Query<&Root>,
        current_interaction: Res<CurrentInteraction>,
        handles: Query<&Handle>,
    ) {
        if let Some(f) = current_interaction.focused {
            let main = Root::resolve(f, &roots);
            let Ok(handle) = handles.get(main) else {
                return;
            };
            if f != main && f != handle.panel && f != handle.text && f != handle.cursor {
                return;
            }
            tree.trigger_targets(
                Input {
                    entity: Entity::PLACEHOLDER,
                    sequence: trigger.event().clone(),
                },
                main,
            );
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        mut values: Query<&mut TextValue>,
        mut cursor: Query<&mut Cursor>,
        line_constraints: Query<&LineConstraint>,
        line_metrics: Query<&LineMetrics>,
        mut selections: Query<&mut Selection>,
        mut clipboard: bevy_ecs::system::ResMut<crate::Clipboard>,
        glyphs: Query<&Glyphs>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        // single mutable Query -- a second, immutable `Query<&Handle>` alongside this one is
        // rejected by bevy at `foliage.define(...)` time as conflicting access to the same
        // component. `.as_readonly()` covers every call site that only needs read access.
        mut handles: Query<&mut Handle>,
        styles: Query<&TextInputStyle>,
        key_bindings: Res<KeyBindings>,
        scroll: ScrollContext,
    ) {
        let this = trigger.event_target();
        let cursor_val = *cursor.get(this).unwrap();
        let lc = *line_constraints.get(this).unwrap();
        let handle = handles.get(this).unwrap().clone();
        let metrics = line_metrics.get(handle.text).unwrap();
        if let Some(action) = key_bindings.action(&trigger.event().sequence) {
            // every action is also broadcast at the root so enclosing composites (Prompt) can
            // route Up/Down/Tab/Escape/Enter without re-implementing key matching
            tree.trigger_targets(InputAction::new(action), this);
            match action {
                TextInputAction::Enter => match lc {
                    LineConstraint::Single => {}
                    LineConstraint::Multiple => {
                        let end = TextInput::insert_text(
                            this,
                            "\n",
                            cursor_val.location,
                            &mut values,
                            &selections,
                        );
                        TextInput::after_edit(
                            this,
                            end,
                            &mut tree,
                            &values.as_readonly(),
                            &handles.as_readonly(),
                            &mut cursor,
                            &glyphs,
                            &font,
                            &font_sizes,
                            *layout,
                            &line_metrics,
                            &mut selections,
                            &scroll,
                        );
                    }
                },
                TextInputAction::Backspace => {
                    let selection = selections.get(this).unwrap();
                    if !selection.range.is_empty() {
                        let end = TextInput::insert_text(
                            this,
                            "",
                            cursor_val.location,
                            &mut values,
                            &selections,
                        );
                        TextInput::after_edit(
                            this,
                            end,
                            &mut tree,
                            &values.as_readonly(),
                            &handles.as_readonly(),
                            &mut cursor,
                            &glyphs,
                            &font,
                            &font_sizes,
                            *layout,
                            &line_metrics,
                            &mut selections,
                            &scroll,
                        );
                    } else if cursor_val.location > 0 {
                        let mut value = values.get_mut(this).unwrap();
                        if !value.0.is_empty() {
                            let idx = prev_boundary(&value.0, cursor_val.location);
                            value.0.remove(idx);
                            drop(value);
                            TextInput::forward_text(
                                this,
                                &mut tree,
                                &values.as_readonly(),
                                &handles.as_readonly(),
                            );
                            TextInput::move_cursor(
                                this,
                                &mut tree,
                                RequestedLocation::Offset(idx),
                                &glyphs,
                                &font,
                                &font_sizes,
                                *layout,
                                &handles.as_readonly(),
                                &mut cursor,
                                &line_metrics,
                                &scroll,
                                true,
                                true,
                            );
                            tree.trigger_targets(TextInputState::AwaitingInput, this);
                            TextInput::clear_selection(this, &mut selections);
                        }
                    }
                }
                TextInputAction::Delete => {
                    let selection = selections.get(this).unwrap();
                    if !selection.range.is_empty() {
                        let end = TextInput::insert_text(
                            this,
                            "",
                            cursor_val.location,
                            &mut values,
                            &selections,
                        );
                        TextInput::after_edit(
                            this,
                            end,
                            &mut tree,
                            &values.as_readonly(),
                            &handles.as_readonly(),
                            &mut cursor,
                            &glyphs,
                            &font,
                            &font_sizes,
                            *layout,
                            &line_metrics,
                            &mut selections,
                            &scroll,
                        );
                    } else if cursor_val.location < values.get(this).unwrap().0.len() {
                        let mut value = values.get_mut(this).unwrap();
                        value.0.remove(cursor_val.location);
                        drop(value);
                        TextInput::forward_text(
                            this,
                            &mut tree,
                            &values.as_readonly(),
                            &handles.as_readonly(),
                        );
                        TextInput::move_cursor(
                            this,
                            &mut tree,
                            RequestedLocation::Offset(cursor_val.location),
                            &glyphs,
                            &font,
                            &font_sizes,
                            *layout,
                            &handles.as_readonly(),
                            &mut cursor,
                            &line_metrics,
                            &scroll,
                            true,
                            true,
                        );
                        tree.trigger_targets(TextInputState::AwaitingInput, this);
                        TextInput::clear_selection(this, &mut selections);
                    }
                }
                TextInputAction::End => {
                    let col = metrics
                        .lines
                        .get(cursor_val.row as usize)
                        .copied()
                        .unwrap_or_default();
                    TextInput::move_cursor(
                        this,
                        &mut tree,
                        RequestedLocation::ColRow((col, cursor_val.row)),
                        &glyphs,
                        &font,
                        &font_sizes,
                        *layout,
                        &handles.as_readonly(),
                        &mut cursor,
                        &line_metrics,
                        &scroll,
                        true,
                        true,
                    );
                    tree.trigger_targets(TextInputState::AwaitingInput, this);
                    TextInput::clear_selection(this, &mut selections);
                }
                TextInputAction::Home => {
                    TextInput::move_cursor(
                        this,
                        &mut tree,
                        RequestedLocation::ColRow((0, cursor_val.row)),
                        &glyphs,
                        &font,
                        &font_sizes,
                        *layout,
                        &handles.as_readonly(),
                        &mut cursor,
                        &line_metrics,
                        &scroll,
                        true,
                        true,
                    );
                    tree.trigger_targets(TextInputState::AwaitingInput, this);
                    TextInput::clear_selection(this, &mut selections);
                }
                TextInputAction::Copy => {
                    let selection = selections.get(this).unwrap();
                    if !selection.range.is_empty() {
                        let value = values.get(this).unwrap();
                        let range = align_range(&value.0, &selection.range);
                        clipboard.write(value.0[range].to_string());
                        // selection stays visible, matching every native text field
                    }
                }
                TextInputAction::Paste => {
                    let mut text = clipboard.read();
                    if matches!(lc, LineConstraint::Single) {
                        text = text.replace('\n', "");
                    }
                    if !text.is_empty() {
                        // InsertText already replaces any active selection
                        let end = TextInput::insert_text(
                            this,
                            &text,
                            cursor_val.location,
                            &mut values,
                            &selections,
                        );
                        TextInput::after_edit(
                            this,
                            end,
                            &mut tree,
                            &values.as_readonly(),
                            &handles.as_readonly(),
                            &mut cursor,
                            &glyphs,
                            &font,
                            &font_sizes,
                            *layout,
                            &line_metrics,
                            &mut selections,
                            &scroll,
                        );
                    }
                }
                TextInputAction::SelectAll => {
                    let len = values.get(this).unwrap().0.len();
                    if len > 0 {
                        let mut selection = selections.get_mut(this).unwrap();
                        selection.range = 0..len;
                        selection.inverted = false;
                        drop(selection);
                        TextInput::move_cursor(
                            this,
                            &mut tree,
                            RequestedLocation::Offset(len),
                            &glyphs,
                            &font,
                            &font_sizes,
                            *layout,
                            &handles.as_readonly(),
                            &mut cursor,
                            &line_metrics,
                            &scroll,
                            true,
                            true,
                        );
                        // No direct `reselect_range` call -- `sync_highlights_to_selection`
                        // (a `Changed<Selection>` reactive safety net, see its comment) picks
                        // this up automatically. Calling it here too just doubled the cost of
                        // every selecting action for no benefit.
                        tree.trigger_targets(TextInputState::Highlighting, this);
                    }
                }
                TextInputAction::ExtendLeft => {
                    TextInput::extend_and_reselect(
                        this,
                        &mut tree,
                        RequestedLocation::ColRow((
                            cursor_val.column.saturating_sub(1),
                            cursor_val.row,
                        )),
                        &mut cursor,
                        &mut selections,
                        &glyphs,
                        &mut handles,
                        &font,
                        &font_sizes,
                        *layout,
                        &line_metrics,
                        &scroll,
                    );
                    tree.trigger_targets(TextInputState::Highlighting, this);
                }
                TextInputAction::ExtendRight => {
                    TextInput::extend_and_reselect(
                        this,
                        &mut tree,
                        RequestedLocation::ColRow((
                            (cursor_val.column + 1).min(metrics.max_letter_idx_horizontal),
                            cursor_val.row,
                        )),
                        &mut cursor,
                        &mut selections,
                        &glyphs,
                        &mut handles,
                        &font,
                        &font_sizes,
                        *layout,
                        &line_metrics,
                        &scroll,
                    );
                    tree.trigger_targets(TextInputState::Highlighting, this);
                }
                TextInputAction::ExtendUp => {
                    TextInput::extend_and_reselect(
                        this,
                        &mut tree,
                        RequestedLocation::ColRow((
                            cursor_val.column,
                            cursor_val.row.saturating_sub(1),
                        )),
                        &mut cursor,
                        &mut selections,
                        &glyphs,
                        &mut handles,
                        &font,
                        &font_sizes,
                        *layout,
                        &line_metrics,
                        &scroll,
                    );
                    tree.trigger_targets(TextInputState::Highlighting, this);
                }
                TextInputAction::ExtendDown => {
                    let target_row = (cursor_val.row + 1)
                        .min(metrics.lines.len().checked_sub(1).unwrap_or_default() as u32);
                    TextInput::extend_and_reselect(
                        this,
                        &mut tree,
                        RequestedLocation::ColRow((cursor_val.column, target_row)),
                        &mut cursor,
                        &mut selections,
                        &glyphs,
                        &mut handles,
                        &font,
                        &font_sizes,
                        *layout,
                        &line_metrics,
                        &scroll,
                    );
                    tree.trigger_targets(TextInputState::Highlighting, this);
                }
                TextInputAction::Up => {
                    TextInput::move_cursor(
                        this,
                        &mut tree,
                        RequestedLocation::ColRow((
                            cursor_val.column,
                            cursor_val.row.checked_sub(1).unwrap_or_default(),
                        )),
                        &glyphs,
                        &font,
                        &font_sizes,
                        *layout,
                        &handles.as_readonly(),
                        &mut cursor,
                        &line_metrics,
                        &scroll,
                        true,
                        true,
                    );
                    tree.trigger_targets(TextInputState::AwaitingInput, this);
                    TextInput::clear_selection(this, &mut selections);
                }
                TextInputAction::Down => {
                    let target_row = (cursor_val.row + 1)
                        .min(metrics.lines.len().checked_sub(1).unwrap_or_default() as u32);
                    TextInput::move_cursor(
                        this,
                        &mut tree,
                        RequestedLocation::ColRow((cursor_val.column, target_row)),
                        &glyphs,
                        &font,
                        &font_sizes,
                        *layout,
                        &handles.as_readonly(),
                        &mut cursor,
                        &line_metrics,
                        &scroll,
                        true,
                        true,
                    );
                    tree.trigger_targets(TextInputState::AwaitingInput, this);
                    TextInput::clear_selection(this, &mut selections);
                }
                TextInputAction::Left => {
                    let offset = if cursor_val.location > 0 {
                        prev_boundary(&values.get(this).unwrap().0, cursor_val.location)
                    } else {
                        0
                    };
                    TextInput::move_cursor(
                        this,
                        &mut tree,
                        RequestedLocation::Offset(offset),
                        &glyphs,
                        &font,
                        &font_sizes,
                        *layout,
                        &handles.as_readonly(),
                        &mut cursor,
                        &line_metrics,
                        &scroll,
                        true,
                        true,
                    );
                    tree.trigger_targets(TextInputState::AwaitingInput, this);
                    TextInput::clear_selection(this, &mut selections);
                }
                TextInputAction::Right => {
                    tree.trigger_targets(TextInputState::AwaitingInput, this);
                    TextInput::clear_selection(this, &mut selections);
                    let offset = next_boundary(&values.get(this).unwrap().0, cursor_val.location);
                    TextInput::move_cursor(
                        this,
                        &mut tree,
                        RequestedLocation::Offset(offset),
                        &glyphs,
                        &font,
                        &font_sizes,
                        *layout,
                        &handles.as_readonly(),
                        &mut cursor,
                        &line_metrics,
                        &scroll,
                        true,
                        true,
                    );
                }
                TextInputAction::Space => {
                    let end = TextInput::insert_text(
                        this,
                        " ",
                        cursor_val.location,
                        &mut values,
                        &selections,
                    );
                    TextInput::after_edit(
                        this,
                        end,
                        &mut tree,
                        &values.as_readonly(),
                        &handles.as_readonly(),
                        &mut cursor,
                        &glyphs,
                        &font,
                        &font_sizes,
                        *layout,
                        &line_metrics,
                        &mut selections,
                        &scroll,
                    );
                }
                // no text mutation; enclosing composites react via the InputAction broadcast
                TextInputAction::Tab => {}
                TextInputAction::Escape => {}
            }
        } else {
            if let Key::Character(text) = &trigger.sequence.key {
                let text = text.to_string();
                let end =
                    TextInput::insert_text(this, &text, cursor_val.location, &mut values, &selections);
                TextInput::after_edit(
                    this,
                    end,
                    &mut tree,
                    &values.as_readonly(),
                    &handles.as_readonly(),
                    &mut cursor,
                    &glyphs,
                    &font,
                    &font_sizes,
                    *layout,
                    &line_metrics,
                    &mut selections,
                    &scroll,
                );
            }
        }
    }
}
impl TextInput {
    /// Was `InsertText::obs`'s text-mutation half: typing/paste append, or selection replacement
    /// (cursor lands after the inserted text, at the start of the removed range). Cursor
    /// placement/forwarding is the caller's job via `after_edit`, which needs the returned
    /// offset -- the *whole* text's length (`after_edit`'s old assumption) is only correct
    /// when inserting at the true end; typing after arrow-key repositioning inserts in the
    /// middle, and the cursor must land right after just what was inserted, not at the end
    /// of whatever text happens to follow it. `cursor_location` is where the insert starts
    /// when there's no active selection to replace instead.
    fn insert_text(
        this: Entity,
        text: &str,
        cursor_location: GlyphOffset,
        values: &mut Query<&mut TextValue>,
        selections: &Query<&mut Selection>,
    ) -> GlyphOffset {
        let selection = selections.get(this).unwrap();
        let mut value = values.get_mut(this).unwrap();
        let mut new_location = cursor_location.min(value.0.len());
        if !selection.range.is_empty() {
            let range = align_range(&value.0, &selection.range);
            new_location = range.start;
            value.0.replace_range(range, "");
        }
        while new_location > 0 && !value.0.is_char_boundary(new_location) {
            new_location -= 1;
        }
        value.0.insert_str(new_location, text);
        new_location + text.len()
    }
    /// Common tail after any text mutation: forward the new text, place the cursor at
    /// `new_location` (the caller-supplied offset right after what `insert_text` just
    /// inserted), clear the (now-consumed) selection, and mark the input as actively being
    /// typed into.
    #[allow(clippy::too_many_arguments)]
    fn after_edit(
        this: Entity,
        new_location: GlyphOffset,
        tree: &mut Tree,
        values: &Query<&TextValue>,
        handles: &Query<&Handle>,
        cursor: &mut Query<&mut Cursor>,
        glyphs: &Query<&Glyphs>,
        font: &MonospacedFont,
        font_sizes: &Query<&FontSize>,
        layout: Layout,
        line_metrics: &Query<&LineMetrics>,
        selections: &mut Query<&mut Selection>,
        scroll: &ScrollContext,
    ) {
        Self::forward_text(this, tree, values, handles);
        TextInput::move_cursor(
            this,
            tree,
            RequestedLocation::Offset(new_location),
            glyphs,
            font,
            font_sizes,
            layout,
            handles,
            cursor,
            line_metrics,
            scroll,
            // `false`: glyphs/LineMetrics may still reflect pre-edit state here (this runs
            // synchronously, before `Text::update` settles) -- the guaranteed-fresh reactive
            // follow-up (`Selection::reselect`, via `TextContentChanged`) does the actual
            // visual placement instead. See `move_cursor`'s comment on `follow`.
            false,
            // Same reasoning as above applies to the scroll-into-view check.
            false,
        );
        tree.trigger_targets(TextInputState::AwaitingInput, this);
        Self::clear_selection(this, selections);
    }
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    fn extend_and_reselect(
        this: Entity,
        tree: &mut Tree,
        req: RequestedLocation,
        cursor: &mut Query<&mut Cursor>,
        selections: &mut Query<&mut Selection>,
        glyphs: &Query<&Glyphs>,
        // single mutable Query -- splitting this into a read step then a write step internally
        // (rather than taking both an `&Query<&Handle>` and an `&mut Query<&mut Handle>` as two
        // parameters) is what lets the caller pass one `Query<&mut Handle>` instead of two
        // conflicting borrows of it in the same call.
        handles: &mut Query<&mut Handle>,
        font: &MonospacedFont,
        font_sizes: &Query<&FontSize>,
        layout: Layout,
        line_metrics: &Query<&LineMetrics>,
        scroll: &ScrollContext,
    ) {
        {
            let cursor_ro = cursor.as_readonly();
            TextInput::extend_range(
                this,
                req,
                &cursor_ro,
                selections,
                glyphs,
                &handles.as_readonly(),
                font,
                font_sizes,
                layout,
            );
        }
        // `extend_range` only ever updates `Selection.range` -- `Cursor` was left frozen at
        // its pre-extend position, so every subsequent Shift+Arrow recomputed the exact same
        // target instead of advancing (it stopped after one step, every direction after that
        // a no-op). The blinking cursor has to track the selection's moving edge -- the end
        // opposite whichever side is now anchored -- both logically (so the *next* keystroke
        // extends from here) and visually (placement + scroll-into-view).
        let edge = {
            let selection = selections.get(this).unwrap();
            if selection.inverted {
                selection.range.start
            } else {
                selection.range.end
            }
        };
        TextInput::move_cursor(
            this,
            tree,
            RequestedLocation::Offset(edge),
            glyphs,
            font,
            font_sizes,
            layout,
            &handles.as_readonly(),
            cursor,
            line_metrics,
            scroll,
            true,
            true,
        );
        // No direct `reselect_range` call -- `sync_highlights_to_selection` (a
        // `Changed<Selection>` reactive safety net, see its comment) picks this up
        // automatically. Calling it here too just doubled the cost of every Shift+Arrow
        // keystroke, and the slowdown scaled with how much was already selected.
    }
}
/// Fired at the `TextInput` root whenever its text content changes (typing, deletion, paste,
/// programmatic `TextValue` writes). Subscribe with `tree.subscribe(input, ...)`.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct TextChanged {}
/// Programmatically inserts text at the cursor (or replaces the active selection). Public: any
/// composite/consumer can trigger this directly, not just this file's own key-handling arms.
#[foliage_macros::targeted_event]
pub struct InsertText {
    pub text: String,
}
impl InsertText {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        mut values: Query<&mut TextValue>,
        mut selections: Query<&mut Selection>,
        handles: Query<&Handle>,
        mut cursor: Query<&mut Cursor>,
        glyphs: Query<&Glyphs>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        line_metrics: Query<&LineMetrics>,
        scroll: ScrollContext,
    ) {
        let this = trigger.event_target();
        let cursor_location = cursor.get(this).unwrap().location;
        let end = TextInput::insert_text(
            this,
            &trigger.text,
            cursor_location,
            &mut values,
            &selections,
        );
        TextInput::after_edit(
            this,
            end,
            &mut tree,
            &values.as_readonly(),
            &handles,
            &mut cursor,
            &glyphs,
            &font,
            &font_sizes,
            *layout,
            &line_metrics,
            &mut selections,
            &scroll,
        );
    }
}
// No teardown hook: every child (highlights included -- reselect_range spawns them via
// `tree.branch(panel, ..)`) is `Stem`-parented, so `Remove`'s cascade reaches them all.
#[derive(Component, Clone, Debug)]
pub struct Handle {
    pub panel: Entity,
    pub text: Entity,
    pub hint_text: Entity,
    pub cursor: Entity,
    pub visible: Entity,
    // Row -> `(Entity, start_col, end_col)` -- one highlight panel per selected *row*, not
    // per selected glyph (selected columns within a row are always contiguous, since
    // `Selection` is one contiguous byte range, so a single rectangle per row renders
    // identically). `start_col`/`end_col` are the span the panel's `Location` was last
    // actually written with, so `reselect_range` can skip a redundant re-`insert` when a
    // row's span hasn't moved. `insert` always marks the component (and everything
    // downstream via `on_insert` cascades) Changed regardless of whether the value differs --
    // per-glyph, that meant every keystroke/rewrap re-cascaded `Location` -> `Section` ->
    // view-extent regrow for every selected character individually. On a large pasted block
    // with everything selected, that was thousands of redundant cascades per call -- the
    // multi-second freezes/crashes on resize-while-selected, Ctrl+A, and deselect.
    pub highlights: HashMap<u32, (Entity, u32, u32)>,
}
#[derive(Component, Clone, Default)]
pub struct HintText(pub(crate) String);
impl HintText {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
}
#[derive(Component, Clone, Default)]
pub struct HintColor(pub Color);
