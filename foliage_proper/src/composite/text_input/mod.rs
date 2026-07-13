pub(crate) mod action;
pub(crate) mod keybindings;

use crate::composite::Root;
use crate::interaction::CurrentInteraction;
use crate::text::monospaced::MonospacedFont;
use crate::text::{Glyphs, LineMetrics};
use crate::Trigger;
use crate::{
    auto, Attachment, AutoHeight, AutoWidth, Color, Component, Dragged, EcsExtension, Elevation,
    Engaged, Event, FocusBehavior, Foliage, FontSize, GlyphOffset, Grid, GridExt, InputSequence,
    InteractionListener, InteractionPropagation, Key, Layout, Leaf, LeafSprout, Location, Logical,
    Opacity, OverscrollPropagation, Panel, Section, Sprout, Stem, Text, TextValue, Tree, Unfocused,
    View, Write,
};
use action::{InputAction, TextInputAction};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::{Query, Res};
use keybindings::KeyBindings;
use std::collections::HashMap;
use std::ops::Range;

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
        for (_, e) in handle.highlights.iter() {
            tree.entity(*e).insert(style.accent);
        }
    }
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
        sections: Query<&Section<Logical>>,
        views: Query<&View>,
        handles: Query<&Handle>,
        mut cursor: Query<&mut Cursor>,
        glyphs: Query<&Glyphs>,
        line_metrics: Query<&LineMetrics>,
        mut selections: Query<&mut Selection>,
    ) {
        let this = trigger.event_target();
        TextInput::clear_selection(this, &mut selections);
        tree.trigger_targets(TextInputState::AwaitingInput, this);
        let (col, row) = TextInput::location_from_click(
            this,
            true,
            &current_interaction,
            &font,
            &font_sizes,
            *layout,
            &sections,
            &views,
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
        );
    }
}
/// Ephemeral hand-off value: was a `Component` (`RequestedLocation`) written by one system and
/// immediately consumed by `MoveCursor`'s observer -- nothing else ever read it as persistent
/// state, so it's now just an argument passed directly into `TextInput::move_cursor`.
#[derive(Copy, Clone)]
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
    /// Was `MoveCursor::obs`: resolve a requested (column, row) or byte offset against the
    /// text's actual glyph layout, and place the cursor + its visible indicator there.
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
    ) {
        let fsv = font_sizes.get(this).unwrap().resolve(layout).value;
        let dims = font.character_block(fsv);
        let handle = handles.get(this).unwrap();
        let metrics = line_metrics.get(handle.text).unwrap();
        let mut cursor = cursor.get_mut(this).unwrap();
        let text_glyphs = glyphs.get(handle.text).unwrap().layout.glyphs();
        let (location, col, row) = if let Some(found) = text_glyphs.iter().find(|glyph| match req {
            RequestedLocation::ColRow((column, row)) => {
                (glyph.x / dims.a()) as u32 == column && (glyph.y / dims.b()) as u32 == row
            }
            RequestedLocation::Offset(offset) => glyph.byte_offset == offset,
        }) {
            let col = (found.x / dims.a()) as u32;
            let row = (found.y / dims.b()) as u32;
            (found.byte_offset, col, row)
        } else {
            let mut col = 0;
            let mut row = 0;
            let mut location = 0;
            match req {
                RequestedLocation::Offset(offset) => {
                    let mut scan = offset;
                    while let Some(s) = scan.checked_sub(1) {
                        if let Some(found) = text_glyphs.iter().find(|g| g.byte_offset == s) {
                            col = (found.x / dims.a()) as u32;
                            col = (col + 1).min(metrics.max_letter_idx_horizontal);
                            row = (found.y / dims.b()) as u32;
                            location = found.byte_offset + 1;
                            break;
                        } else {
                            if s == 0 {
                                // no glyph at all to anchor to (glyph layout hasn't caught up
                                // to a just-applied text write yet, e.g. the very first
                                // keystroke) -- trust the requested offset itself rather than
                                // snapping to 0, or the next keystroke inserts before this one.
                                col = 0;
                                row = 0;
                                location = offset;
                                break;
                            }
                        }
                        scan = s;
                    }
                }
                RequestedLocation::ColRow((c, r)) => {
                    let mut scan = c;
                    while let Some(sc) = scan.checked_sub(1) {
                        if let Some(found) = text_glyphs
                            .iter()
                            .find(|g| (g.x / dims.a()) as u32 == sc && (g.y / dims.b()) as u32 == r)
                        {
                            col = (sc + 1).min(metrics.max_letter_idx_horizontal);
                            row = r;
                            location = found.byte_offset + 1;
                            break;
                        } else {
                            if sc == 0 {
                                col = 0;
                                row = r;
                                if row == 0 {
                                    location = 0;
                                } else {
                                    location = *metrics.last_offsets.get(row as usize - 1).unwrap()
                                        as GlyphOffset
                                        + 1;
                                }
                                break;
                            }
                        }
                        scan = sc;
                    }
                }
            }
            (location, col, row)
        };
        cursor.location = location;
        cursor.column = col;
        cursor.row = row;
        let location = Location::new().xs(
            (col + 1).col().as_left().with((col + 1).col().as_right()),
            (row + 1).row().as_top().with((row + 1).row().as_bottom()),
        );
        tree.entity(handle.cursor).insert(location);
        tree.entity(handle.visible).insert(location);
    }
}
#[derive(Component, Clone, Default)]
pub(crate) struct Selection {
    pub(crate) range: Range<GlyphOffset>,
    pub(crate) inverted: bool,
}
impl Selection {
    pub(crate) fn reselect(
        trigger: Trigger<Write<Text>>,
        mut tree: Tree,
        roots: Query<&Root>,
        glyphs: Query<&Glyphs>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        mut handles: Query<&mut Handle>,
        selections: Query<&Selection>,
        styles: Query<&TextInputStyle>,
        // single mutable Query -- see the comment on `Input::obs`'s `handles` param.
        mut cursor: Query<&mut Cursor>,
        line_metrics: Query<&LineMetrics>,
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
        );
        TextInput::reselect_range(
            root,
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
    pub(crate) fn select(
        trigger: Trigger<Dragged>,
        mut tree: Tree,
        roots: Query<&Root>,
        current_interaction: Res<CurrentInteraction>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        sections: Query<&Section<Logical>>,
        views: Query<&View>,
        // single mutable Query -- see the comment on `Input::obs`'s `handles` param.
        mut handles: Query<&mut Handle>,
        line_metrics: Query<&LineMetrics>,
        cursors: Query<&Cursor>,
        mut selections: Query<&mut Selection>,
        glyphs: Query<&Glyphs>,
        values: Query<&TextValue>,
        styles: Query<&TextInputStyle>,
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
            &values,
        );
        let selections_ro = selections.as_readonly();
        TextInput::reselect_range(
            root,
            &mut tree,
            &mut handles,
            &glyphs,
            &selections_ro,
            &font,
            &font_sizes,
            *layout,
            &styles,
        );
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
        values: &Query<&TextValue>,
    ) {
        let value = values.get(this).unwrap();
        let handle = handles.get(this).unwrap();
        let fsv = font_sizes.get(this).unwrap().resolve(layout).value;
        let dims = font.character_block(fsv);
        let cursor = cursors.get(this).unwrap();
        let mut selection = selections.get_mut(this).unwrap();
        if let RequestedLocation::ColRow((c, r)) = req {
            for glyph in glyphs.get(handle.text).unwrap().layout.glyphs() {
                if (glyph.x / dims.a()) as u32 == c && (glyph.y / dims.b()) as u32 == r {
                    if cursor.location < glyph.byte_offset {
                        selection.inverted = false;
                        selection.range =
                            cursor.location..next_boundary(&value.0, glyph.byte_offset);
                    } else {
                        selection.inverted = true;
                        selection.range =
                            glyph.byte_offset..next_boundary(&value.0, cursor.location);
                    }
                }
            }
        }
    }
    /// Was `ReselectRange::obs`: sync the highlight-panel-per-glyph set to the current
    /// `Selection.range`.
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
        let stale = handle
            .highlights
            .iter()
            .filter(|(o, _)| !selection.range.contains(*o))
            .map(|(o, e)| (*o, *e))
            .collect::<Vec<_>>();
        for (o, e) in stale {
            handle.highlights.remove(&o);
            tree.remove(e);
        }
        // one child per selected glyph -- new glyphs entering the range branch from
        // `handle.panel`, same as the composite's own construction.
        let panel = handle.panel;
        let existing: Vec<GlyphOffset> = glyph
            .layout
            .glyphs()
            .iter()
            .filter(|g| {
                selection.range.contains(&g.byte_offset)
                    && handle.highlights.contains_key(&g.byte_offset)
            })
            .map(|g| g.byte_offset)
            .collect();
        for o in existing {
            let (col, row) = glyph
                .layout
                .glyphs()
                .iter()
                .find(|g| g.byte_offset == o)
                .map(|g| ((g.x / dims.a()) as u32, (g.y / dims.b()) as u32))
                .unwrap();
            let location = Location::new().xs(
                (col + 1).col().as_left().with((col + 1).col().as_right()),
                (row + 1).row().as_top().with((row + 1).row().as_bottom()),
            );
            let existing = *handle.highlights.get(&o).unwrap();
            tree.entity(existing)
                .insert(Opacity::new(1.0))
                .insert(location);
        }
        let new_glyphs: Vec<(GlyphOffset, u32, u32)> = glyph
            .layout
            .glyphs()
            .iter()
            .filter(|g| {
                selection.range.contains(&g.byte_offset)
                    && !handle.highlights.contains_key(&g.byte_offset)
            })
            .map(|g| {
                (
                    g.byte_offset,
                    (g.x / dims.a()) as u32,
                    (g.y / dims.b()) as u32,
                )
            })
            .collect();
        let color = styles.get(this).unwrap().accent;
        for (offset, col, row) in new_glyphs {
            let location = Location::new().xs(
                (col + 1).col().as_left().with((col + 1).col().as_right()),
                (row + 1).row().as_top().with((row + 1).row().as_bottom()),
            );
            let h = tree.branch(
                panel,
                Panel::new().elevate(Elevation::up(2)).at(location).with((
                    Opacity::new(1.0),
                    color,
                    InteractionPropagation::pass_through(),
                    FocusBehavior::ignore(),
                )),
            );
            handle.highlights.insert(offset, h);
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
                        TextInput::insert_text(
                            this,
                            "\n",
                            cursor_val.location,
                            &mut values,
                            &selections,
                        );
                        TextInput::after_edit(
                            this,
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
                        );
                    }
                },
                TextInputAction::Backspace => {
                    let selection = selections.get(this).unwrap();
                    if !selection.range.is_empty() {
                        TextInput::insert_text(
                            this,
                            "",
                            cursor_val.location,
                            &mut values,
                            &selections,
                        );
                        TextInput::after_edit(
                            this,
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
                            );
                            tree.trigger_targets(TextInputState::AwaitingInput, this);
                            TextInput::clear_selection(this, &mut selections);
                        }
                    }
                }
                TextInputAction::Delete => {
                    let selection = selections.get(this).unwrap();
                    if !selection.range.is_empty() {
                        TextInput::insert_text(
                            this,
                            "",
                            cursor_val.location,
                            &mut values,
                            &selections,
                        );
                        TextInput::after_edit(
                            this,
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
                        TextInput::insert_text(
                            this,
                            &text,
                            cursor_val.location,
                            &mut values,
                            &selections,
                        );
                        TextInput::after_edit(
                            this,
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
                        );
                        let selections_ro = selections.as_readonly();
                        TextInput::reselect_range(
                            this,
                            &mut tree,
                            &mut handles,
                            &glyphs,
                            &selections_ro,
                            &font,
                            &font_sizes,
                            *layout,
                            &styles,
                        );
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
                        &cursor,
                        &mut selections,
                        &glyphs,
                        &mut handles,
                        &font,
                        &font_sizes,
                        *layout,
                        &values.as_readonly(),
                        &styles,
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
                        &cursor,
                        &mut selections,
                        &glyphs,
                        &mut handles,
                        &font,
                        &font_sizes,
                        *layout,
                        &values.as_readonly(),
                        &styles,
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
                        &cursor,
                        &mut selections,
                        &glyphs,
                        &mut handles,
                        &font,
                        &font_sizes,
                        *layout,
                        &values.as_readonly(),
                        &styles,
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
                        &cursor,
                        &mut selections,
                        &glyphs,
                        &mut handles,
                        &font,
                        &font_sizes,
                        *layout,
                        &values.as_readonly(),
                        &styles,
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
                    );
                }
                TextInputAction::Space => {
                    TextInput::insert_text(
                        this,
                        " ",
                        cursor_val.location,
                        &mut values,
                        &selections,
                    );
                    TextInput::after_edit(
                        this,
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
                    );
                }
                // no text mutation; enclosing composites react via the InputAction broadcast
                TextInputAction::Tab => {}
                TextInputAction::Escape => {}
            }
        } else {
            if let Key::Character(text) = &trigger.sequence.key {
                let text = text.to_string();
                TextInput::insert_text(this, &text, cursor_val.location, &mut values, &selections);
                TextInput::after_edit(
                    this,
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
                );
            }
        }
    }
}
impl TextInput {
    /// Was `InsertText::obs`'s text-mutation half: typing/paste append, or selection replacement
    /// (cursor lands after the inserted text, at the start of the removed range). Cursor
    /// placement/forwarding is the caller's job via `after_edit`. `cursor_location` is where the
    /// insert starts when there's no active selection to replace instead.
    fn insert_text(
        this: Entity,
        text: &str,
        cursor_location: GlyphOffset,
        values: &mut Query<&mut TextValue>,
        selections: &Query<&mut Selection>,
    ) {
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
    }
    /// Common tail after any text mutation: forward the new text, place the cursor at the end of
    /// what was just inserted, clear the (now-consumed) selection, and mark the input as
    /// actively being typed into.
    #[allow(clippy::too_many_arguments)]
    fn after_edit(
        this: Entity,
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
    ) {
        let new_location = values.get(this).unwrap().0.len();
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
        cursor: &Query<&mut Cursor>,
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
        values: &Query<&TextValue>,
        styles: &Query<&TextInputStyle>,
    ) {
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
            values,
        );
        let selections_ro = selections.as_readonly();
        TextInput::reselect_range(
            this,
            tree,
            handles,
            glyphs,
            &selections_ro,
            font,
            font_sizes,
            layout,
            styles,
        );
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
    ) {
        let this = trigger.event_target();
        let cursor_location = cursor.get(this).unwrap().location;
        TextInput::insert_text(
            this,
            &trigger.text,
            cursor_location,
            &mut values,
            &selections,
        );
        TextInput::after_edit(
            this,
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
    pub highlights: HashMap<GlyphOffset, Entity>,
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
