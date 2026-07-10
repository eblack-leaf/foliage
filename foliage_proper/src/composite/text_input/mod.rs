pub(crate) mod action;
pub(crate) mod keybindings;

use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::HookContext;
use crate::composite::handle_replace;
use crate::composite::Root;
use crate::interaction::CurrentInteraction;
use crate::text::monospaced::MonospacedFont;
use crate::text::{Glyphs, LineMetrics};
use crate::{
    auto, Attachment, AutoHeight, AutoWidth, Color, Component, Composite, Dragged, EcsExtension,
    Elevation, Engaged, Event, FocusBehavior, Foliage, FontSize, GlyphOffset, Grid, GridExt,
    InputSequence, InteractionListener, InteractionPropagation, Key, Layout, Location, Logical,
    Opacity, OverscrollPropagation, Panel, Primary, Secondary, Section, Stem, Tertiary, Text,
    TextValue, Tree, Unfocused, Update, View, Write,
};
use action::{InputAction, TextInputAction};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use crate::IntoTargets;
use crate::Trigger;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::{Query, Res};
use bevy_ecs::world::DeferredWorld;
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
        foliage.define(TextInputState::obs);
        foliage.define(PlaceCursor::obs);
        foliage.define(LocationFromClick::obs);
        foliage.define(MoveCursor::obs);
        foliage.define(Input::forward_to_text);
        foliage.define(Input::obs);
        foliage.define(Input::forward);
        foliage.define(InsertText::obs);
        foliage.define(ExtendRange::obs);
        foliage.define(ClearSelection::obs);
        foliage.define(ReselectRange::obs);
        foliage.define(Self::handle_trigger);
        foliage.world.insert_resource(KeyBindings::default());
    }
}
#[derive(Component, Copy, Clone)]
#[require(LineConstraint, Cursor, Selection, HintText, HintColor, RequestedLocation)]
#[require(Primary, Secondary, Tertiary, FontSize, TextValue)]
#[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
pub struct TextInput {}
impl TextInput {
    const HIGHLIGHT_SCROLL_THRESHOLD: f32 = 10.0;
    pub fn new() -> TextInput {
        TextInput {}
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        // observers
        world
            .commands()
            .entity(this)
            .observe(Self::unfocused)
            .observe(Self::forward_text_value)
            .observe(Self::update_text_value)
            .observe(Self::forward_primary)
            .observe(Self::update_primary)
            .observe(Self::forward_secondary)
            .observe(Self::update_secondary)
            .observe(Self::forward_tertiary)
            .observe(Self::update_tertiary)
            .observe(Self::forward_font_size)
            .observe(Self::update_font_size)
            .observe(Self::forward_hint)
            .observe(Self::update_hint);
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world.commands().entity(this).insert(Grid::default());
        let line_constraint = *world.get::<LineConstraint>(this).unwrap();
        let panel = world.commands().leaf((
            Panel::new_marker(),
            Stem::some(this),
            Grid::new(1.letters(), 1.letters()),
            InteractionListener::new(),
            Elevation::up(1),
            Location::new().xs(
                0.pct().left().adjust(4).with(100.pct().right().adjust(-4)),
                0.pct().top().adjust(4).with(100.pct().bottom().adjust(-4)),
            ),
            Root(this),
        ));
        world
            .commands()
            .entity(panel)
            .observe(Self::unfocused)
            .observe(PlaceCursor::forward);
        let cursor = world.commands().leaf((
            Stem::some(panel),
            InteractionListener::new(),
            InteractionPropagation::pass_through(),
            Elevation::up(6),
            Location::new().xs(
                1.col().left().with(1.col().right()),
                1.col().top().with(1.col().bottom()),
            ),
            Root(this),
        ));
        world
            .commands()
            .entity(cursor)
            .observe(Self::unfocused)
            .observe(Cursor::engaged)
            .observe(Selection::select);
        let visible = world.commands().leaf((
            Panel::new_marker(),
            Stem::some(panel),
            InteractionListener::new(),
            InteractionPropagation::pass_through(),
            FocusBehavior::ignore(),
            Elevation::up(3),
            Location::new().xs(
                1.col().left().with(1.col().right()),
                1.col().top().with(1.col().bottom()),
            ),
            Root(this),
        ));
        let text_location = Location::new().xs(
            match line_constraint {
                LineConstraint::Single => 0.pct().left().with(auto().width()),
                LineConstraint::Multiple => 0.pct().left().with(100.pct().right()),
            },
            match line_constraint {
                LineConstraint::Single => 0.pct().top().with(100.pct().bottom()),
                LineConstraint::Multiple => 0.pct().top().with(auto().height()),
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
        let text = world.commands().leaf((
            Text::new_marker(""),
            Stem::some(panel),
            InteractionListener::new(),
            Elevation::up(5),
            text_location,
            Root(this),
            auto_width,
            auto_height,
        ));
        world
            .commands()
            .entity(text)
            .observe(Self::unfocused)
            .observe(PlaceCursor::forward)
            .observe(Selection::reselect);
        let hint_text = world.commands().leaf((
            Text::new_marker(""),
            Stem::some(panel),
            InteractionPropagation::pass_through(),
            FocusBehavior::ignore(),
            Elevation::up(4),
            text_location,
            Root(this),
            auto_width,
            auto_height,
        ));
        let handle = Handle {
            panel,
            text,
            hint_text,
            cursor,
            visible,
            highlights: Default::default(),
        };
        world.commands().entity(this).insert(handle);
    }
    pub(crate) fn unfocused(
        trigger: Trigger<Unfocused>,
        mut tree: Tree,
        roots: Query<&Root>,
        handles: Query<&Handle>,
        current_interaction: Res<CurrentInteraction>,
    ) {
        let main = if let Ok(root) = roots.get(trigger.event_target()) {
            root.0
        } else {
            trigger.event_target()
        };
        let handle = handles.get(main).unwrap();
        if let Some(f) = current_interaction.focused {
            if f == main || f == handle.panel || f == handle.text || f == handle.cursor {
                return;
            }
        }
        tree.trigger_targets(ClearSelection { entity: Entity::PLACEHOLDER }, main);
        tree.trigger_targets(TextInputState::Inactive, main);
    }
    // forwarders for all colors + state
    fn handle_trigger(trigger: Trigger<Insert, Handle>, mut tree: Tree) {
        let this = trigger.event_target();
        tree.trigger_targets(Update::<TextValue>::new(), this);
        tree.trigger_targets(Update::<Primary>::new(), this);
        tree.trigger_targets(Update::<Secondary>::new(), this);
        tree.trigger_targets(Update::<Tertiary>::new(), this);
        tree.trigger_targets(Update::<FontSize>::new(), this);
        tree.trigger_targets(Update::<HintText>::new(), this);
    }
    fn forward_text_value(trigger: Trigger<Insert, TextValue>, mut tree: Tree) {
        tree.trigger_targets(Update::<TextValue>::new(), trigger.event_target());
    }
    fn update_text_value(trigger: Trigger<Update<TextValue>>, mut tree: Tree) {
        tree.trigger_targets(ForwardText { entity: Entity::PLACEHOLDER }, trigger.event_target());
        tree.trigger_targets(ClearSelection { entity: Entity::PLACEHOLDER }, trigger.event_target());
        tree.trigger_targets(TextInputState::Inactive, trigger.event_target());
    }
    fn forward_font_size(trigger: Trigger<Insert, FontSize>, mut tree: Tree) {
        tree.trigger_targets(Update::<FontSize>::new(), trigger.event_target());
    }
    fn update_font_size(
        trigger: Trigger<Update<FontSize>>,
        mut tree: Tree,
        font_sizes: Query<&FontSize>,
        handles: Query<&Handle>,
    ) {
        // give to handle.text
        let handle = handles.get(trigger.event_target()).unwrap();
        tree.entity(handle.text)
            .insert(font_sizes.get(trigger.event_target()).unwrap().clone());
        tree.entity(handle.hint_text)
            .insert(font_sizes.get(trigger.event_target()).unwrap().clone());
        tree.entity(handle.panel)
            .insert(font_sizes.get(trigger.event_target()).unwrap().clone());
    }
    fn update_primary(
        trigger: Trigger<Update<Primary>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        primary: Query<&Primary>,
    ) {
        // text-color
        let handle = handles.get(trigger.event_target()).unwrap();
        tree.entity(handle.text)
            .insert(primary.get(trigger.event_target()).unwrap().0);
        tree.entity(handle.hint_text)
            .insert(primary.get(trigger.event_target()).unwrap().0);
    }
    fn forward_primary(trigger: Trigger<Insert, Primary>, mut tree: Tree) {
        tree.trigger_targets(Update::<Primary>::new(), trigger.event_target());
    }
    fn update_secondary(
        trigger: Trigger<Update<Secondary>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        secondary: Query<&Secondary>,
    ) {
        // background (panel)
        let handle = handles.get(trigger.event_target()).unwrap();
        tree.entity(handle.panel)
            .insert(secondary.get(trigger.event_target()).unwrap().0);
    }
    fn forward_secondary(trigger: Trigger<Insert, Secondary>, mut tree: Tree) {
        tree.trigger_targets(Update::<Secondary>::new(), trigger.event_target());
    }
    fn update_tertiary(
        trigger: Trigger<Update<Tertiary>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        tertiary: Query<&Tertiary>,
    ) {
        // cursor color + highlight color
        let handle = handles.get(trigger.event_target()).unwrap();
        let color = tertiary.get(trigger.event_target()).unwrap().0;
        tree.entity(handle.visible).insert(color);
        for (o, e) in handle.highlights.iter() {
            tree.entity(*e).insert(color);
        }
    }
    fn forward_tertiary(trigger: Trigger<Insert, Tertiary>, mut tree: Tree) {
        tree.trigger_targets(Update::<Tertiary>::new(), trigger.event_target());
    }
    fn forward_hint(trigger: Trigger<Insert, HintText>, mut tree: Tree) {
        tree.trigger_targets(Update::<HintText>::new(), trigger.event_target());
    }
    fn update_hint(
        trigger: Trigger<Update<HintText>>,
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
#[derive(EntityEvent, Copy, Clone)]
pub(crate) struct TextInputState {
    entity: Entity,
    pub(crate) stage: TextInputStage,
}
crate::targeted_event!(TextInputState);
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
        tertiary: Query<&Tertiary>,
        primary: Query<&Primary>,
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
                    (Opacity::new(0.75), primary.get(trigger.event_target()).unwrap().0),
                )
            }
            TextInputStage::AwaitingInput => {
                virtual_keyboard
                    .open(crate::virtual_keyboard::VirtualKeyboardType::Keyboard);
                tree.write_to(trigger.event_target(), OverscrollPropagation(true));
                tree.write_to(handle.cursor, InteractionPropagation::grab().disable_drag());
                tree.write_to(
                    handle.visible,
                    (
                        Opacity::new(0.25),
                        tertiary.get(trigger.event_target()).unwrap().0,
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
            roots.get(trigger.event_target()).unwrap().0,
        );
    }
}
#[derive(EntityEvent, Copy, Clone)]
pub(crate) struct PlaceCursor {
    entity: Entity,
}
impl Default for PlaceCursor {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}
crate::targeted_event!(PlaceCursor);
impl PlaceCursor {
    pub(crate) fn forward(trigger: Trigger<Engaged>, mut tree: Tree, roots: Query<&Root>) {
        // trigger PlaceCursor on root.0
        if let Ok(root) = roots.get(trigger.event_target()) {
            tree.trigger_targets(PlaceCursor { entity: Entity::PLACEHOLDER }, root.0);
        }
    }
    pub(crate) fn obs(
        trigger: Trigger<PlaceCursor>,
        mut tree: Tree,
        current_interaction: Res<CurrentInteraction>,
        line_metrics: Query<&LineMetrics>,
    ) {
        tree.trigger_targets(ClearSelection { entity: Entity::PLACEHOLDER }, trigger.event_target());
        // initial placement of cursor + configure focus + interaction behavior
        tree.trigger_targets(TextInputState::AwaitingInput, trigger.event_target());
        // col / row from click => (cursor-from-click) [store in requested-location]
        tree.trigger_targets(
            LocationFromClick {
                entity: Entity::PLACEHOLDER,
                can_go_past_end: true,
            },
            trigger.event_target(),
        );
        // move-cursor with col/row
        tree.trigger_targets(MoveCursor { entity: Entity::PLACEHOLDER }, trigger.event_target());
    }
}
#[derive(EntityEvent, Copy, Clone)]
pub(crate) struct LocationFromClick {
    entity: Entity,
    pub(crate) can_go_past_end: bool,
}
impl LocationFromClick {
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        mut requested: Query<&mut RequestedLocation>,
        current_interaction: Res<CurrentInteraction>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        sections: Query<&Section<Logical>>,
        views: Query<&View>,
        handles: Query<&Handle>,
        line_metrics: Query<&LineMetrics>,
    ) {
        // offset for col-finding = u32::from(lfc.0); true -> 1 false -> 0
        let lfc = u32::from(trigger.can_go_past_end);
        let click = current_interaction.click.current;
        let fsv = font_sizes
            .get(trigger.event_target())
            .unwrap()
            .resolve(*layout)
            .value;
        let dims = font.character_block(fsv);
        let section = sections.get(trigger.event_target()).unwrap();
        let handle = handles.get(trigger.event_target()).unwrap();
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
        tree.write_to(trigger.event_target(), RequestedLocation::ColRow((column, row)))
    }
}
#[derive(Component, Copy, Clone)]
pub(crate) enum RequestedLocation {
    Offset(GlyphOffset),
    ColRow((u32, u32)),
}
impl Default for RequestedLocation {
    fn default() -> Self {
        RequestedLocation::Offset(0)
    }
}
#[derive(EntityEvent, Copy, Clone)]
pub(crate) struct MoveCursor {
    entity: Entity,
}
impl Default for MoveCursor {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}
crate::targeted_event!(MoveCursor);
impl MoveCursor {
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        requested: Query<&RequestedLocation>,
        glyphs: Query<&Glyphs>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        handles: Query<&Handle>,
        mut cursor: Query<&mut Cursor>,
        line_metrics: Query<&LineMetrics>,
    ) {
        let req = requested.get(trigger.event_target()).unwrap();
        let fsv = font_sizes
            .get(trigger.event_target())
            .unwrap()
            .resolve(*layout)
            .value;
        let dims = font.character_block(fsv);
        let handle = handles.get(trigger.event_target()).unwrap();
        let metrics = line_metrics.get(handle.text).unwrap();
        let mut cursor = cursor.get_mut(trigger.event_target()).unwrap();
        let text_glyphs = glyphs.get(handle.text).unwrap().layout.glyphs();
        let (location, col, row) = if let Some(found) = text_glyphs.iter().find(|glyph| match req {
            RequestedLocation::ColRow((column, row)) => {
                (glyph.x / dims.a()) as u32 == *column && (glyph.y / dims.b()) as u32 == *row
            }
            RequestedLocation::Offset(offset) => glyph.byte_offset == *offset,
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
                    let mut scan = *offset;
                    while let Some(s) = scan.checked_sub(1) {
                        if let Some(found) = text_glyphs.iter().find(|g| g.byte_offset == s) {
                            col = (found.x / dims.a()) as u32;
                            col = (col + 1).min(metrics.max_letter_idx_horizontal);
                            row = (found.y / dims.b()) as u32;
                            location = found.byte_offset + 1;
                            break;
                        } else {
                            if s == 0 {
                                col = 0;
                                row = 0;
                                location = 0;
                                break;
                            }
                        }
                        scan = s;
                    }
                }
                RequestedLocation::ColRow((c, r)) => {
                    let mut scan = *c;
                    while let Some(sc) = scan.checked_sub(1) {
                        if let Some(found) = text_glyphs.iter().find(|g| {
                            (g.x / dims.a()) as u32 == sc && (g.y / dims.b()) as u32 == *r
                        }) {
                            col = (sc + 1).min(metrics.max_letter_idx_horizontal);
                            row = *r;
                            location = found.byte_offset + 1;
                            break;
                        } else {
                            if sc == 0 {
                                col = 0;
                                row = *r;
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
            (col + 1).col().left().with((col + 1).col().right()),
            (row + 1).row().top().with((row + 1).row().bottom()),
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
        cursors: Query<&Cursor>,
    ) {
        let root = roots.get(trigger.event_target()).unwrap().0;
        tree.write_to(
            root,
            RequestedLocation::Offset(cursors.get(root).unwrap().location),
        );
        tree.trigger_targets(MoveCursor { entity: Entity::PLACEHOLDER }, root);
        tree.trigger_targets(ReselectRange { entity: Entity::PLACEHOLDER }, root);
    }
    pub(crate) fn select(trigger: Trigger<Dragged>, mut tree: Tree, roots: Query<&Root>) {
        // cursor is dragged => move view near edges + extend selection.range
        let root = roots.get(trigger.event_target()).unwrap().0;
        tree.trigger_targets(
            LocationFromClick {
                entity: Entity::PLACEHOLDER,
                can_go_past_end: false,
            },
            root,
        );
        // use RequestedLocation to extend highlight-range
        tree.trigger_targets(ExtendRange { entity: Entity::PLACEHOLDER }, root);
        // trigger reselect-range after updating the range above
        tree.trigger_targets(ReselectRange { entity: Entity::PLACEHOLDER }, root);
    }
}
#[derive(EntityEvent, Copy, Clone)]
pub(crate) struct ExtendRange {
    entity: Entity,
}
impl Default for ExtendRange {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}
crate::targeted_event!(ExtendRange);
impl ExtendRange {
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        requested: Query<&RequestedLocation>,
        cursors: Query<&Cursor>,
        mut selections: Query<&mut Selection>,
        glyphs: Query<&Glyphs>,
        handles: Query<&Handle>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        values: Query<&TextValue>,
    ) {
        let value = values.get(trigger.event_target()).unwrap();
        let handle = handles.get(trigger.event_target()).unwrap();
        let fsv = font_sizes
            .get(trigger.event_target())
            .unwrap()
            .resolve(*layout)
            .value;
        let dims = font.character_block(fsv);
        let cursor = cursors.get(trigger.event_target()).unwrap();
        let mut selection = selections.get_mut(trigger.event_target()).unwrap();
        let req = requested.get(trigger.event_target()).unwrap();
        match req {
            RequestedLocation::ColRow((c, r)) => {
                for glyph in glyphs.get(handle.text).unwrap().layout.glyphs() {
                    if (glyph.x / dims.a()) as u32 == *c && (glyph.y / dims.b()) as u32 == *r {
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
            _ => {}
        }
    }
}
#[derive(EntityEvent, Clone)]
pub(crate) struct ClearSelection {
    entity: Entity,
}
impl Default for ClearSelection {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}
crate::targeted_event!(ClearSelection);
impl ClearSelection {
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        mut selections: Query<&mut Selection>,
        mut handles: Query<&mut Handle>,
    ) {
        let mut selection = selections.get_mut(trigger.event_target()).unwrap();
        selection.range = Range::default();
        for (o, e) in handles
            .get_mut(trigger.event_target())
            .unwrap()
            .highlights
            .drain()
        {
            tree.remove(e);
        }
    }
}
#[derive(EntityEvent, Clone)]
pub(crate) struct ReselectRange {
    entity: Entity,
}
impl Default for ReselectRange {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}
crate::targeted_event!(ReselectRange);
impl ReselectRange {
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        mut handles: Query<&mut Handle>,
        glyphs: Query<&Glyphs>,
        selections: Query<&Selection>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        tertiary: Query<&Tertiary>,
    ) {
        let mut handle = handles.get_mut(trigger.event_target()).unwrap();
        let selection = selections.get(trigger.event_target()).unwrap();
        let glyph = glyphs.get(handle.text).unwrap();
        let fsv = font_sizes
            .get(trigger.event_target())
            .unwrap()
            .resolve(*layout)
            .value;
        let dims = font.character_block(fsv);
        // drop highlights that fell out of the range
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
        // create-or-move a highlight panel per in-range glyph
        for found in glyph.layout.glyphs() {
            if !selection.range.contains(&found.byte_offset) {
                continue;
            }
            let (col, row) = ((found.x / dims.a()) as u32, (found.y / dims.b()) as u32);
            let location = Location::new().xs(
                (col + 1).col().left().with((col + 1).col().right()),
                (row + 1).row().top().with((row + 1).row().bottom()),
            );
            if let Some(existing) = handle.highlights.get(&found.byte_offset) {
                tree.entity(*existing)
                    .insert(Opacity::new(1.0))
                    .insert(location);
            } else {
                let h = tree.leaf((
                    Panel::new_marker(),
                    Opacity::new(1.0),
                    Stem::some(handle.panel),
                    Elevation::up(2),
                    location,
                    tertiary.get(trigger.event_target()).unwrap().0,
                    InteractionPropagation::pass_through(),
                    FocusBehavior::ignore(),
                ));
                handle.highlights.insert(found.byte_offset, h);
            }
        }
    }
}

#[derive(EntityEvent, Clone)]
pub(crate) struct Input {
    entity: Entity,
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
        // if any focused => check if root or not (panel, text, cursor)
        if let Some(f) = current_interaction.focused {
            let main = if let Ok(root) = roots.get(f) {
                root.0
            } else {
                if handles.get(f).is_ok() {
                    f
                } else {
                    return;
                }
            };
            let handle = handles.get(main).unwrap();
            if f != main && f != handle.panel && f != handle.text && f != handle.cursor {
                return;
            }
            // forward Input to root
            tree.trigger_targets(
                Input {
                    entity: Entity::PLACEHOLDER,
                    sequence: trigger.event().clone(),
                },
                main,
            );
        }
    }
    pub(crate) fn forward_to_text(
        trigger: Trigger<ForwardText>,
        mut tree: Tree,
        values: Query<&TextValue>,
        handles: Query<&Handle>,
    ) {
        // get handle + send main TextValue => handle.text TextValue
        let handle = handles.get(trigger.event_target()).unwrap();
        let value = values.get(trigger.event_target()).unwrap();
        tree.write_to(handle.text, Text::new_marker(&value.0));
        // hint shows only while empty
        tree.write_to(
            handle.hint_text,
            crate::Visibility::new(value.0.is_empty()),
        );
        // public notification so enclosing composites (e.g. Prompt) can react to edits
        tree.trigger_targets(TextChanged::default(), trigger.event_target());
    }
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        key_bindings: Res<KeyBindings>,
        mut values: Query<&mut TextValue>,
        cursors: Query<&Cursor>,
        line_constraints: Query<&LineConstraint>,
        handles: Query<&Handle>,
        line_metrics: Query<&LineMetrics>,
        mut selections: Query<&mut Selection>,
        mut clipboard: bevy_ecs::system::ResMut<crate::Clipboard>,
    ) {
        // check type of interaction + if trigger action then send correct event
        let cursor = cursors.get(trigger.event_target()).unwrap();
        let lc = line_constraints.get(trigger.event_target()).unwrap();
        let mut value = values.get_mut(trigger.event_target()).unwrap();
        let handle = handles.get(trigger.event_target()).unwrap();
        let metrics = line_metrics.get(handle.text).unwrap();
        if let Some(action) = key_bindings.action(&trigger.event().sequence) {
            // every action is also broadcast at the root so enclosing composites (Prompt) can
            // route Up/Down/Tab/Escape/Enter without re-implementing key matching
            tree.trigger_targets(InputAction::new(action), trigger.event_target());
            match action {
                TextInputAction::Enter => match lc {
                    LineConstraint::Single => {}
                    LineConstraint::Multiple => {
                        tree.trigger_targets(
                            InsertText {
                                entity: Entity::PLACEHOLDER,
                                text: "\n".parse().unwrap(),
                            },
                            trigger.event_target(),
                        );
                    }
                },
                TextInputAction::Backspace => {
                    let selection = selections.get(trigger.event_target()).unwrap();
                    if !selection.range.is_empty() {
                        // deleting a selection == replacing it with nothing
                        tree.trigger_targets(InsertText::new(""), trigger.event_target());
                    } else if cursor.location > 0 && !value.0.is_empty() {
                        let idx = prev_boundary(&value.0, cursor.location);
                        value.0.remove(idx);
                        tree.trigger_targets(ForwardText::default(), trigger.event_target()); // manual forward
                        tree.write_to(trigger.event_target(), RequestedLocation::Offset(idx));
                        tree.trigger_targets(MoveCursor::default(), trigger.event_target());
                        tree.trigger_targets(TextInputState::AwaitingInput, trigger.event_target());
                        tree.trigger_targets(ClearSelection::default(), trigger.event_target());
                    }
                }
                TextInputAction::Delete => {
                    let selection = selections.get(trigger.event_target()).unwrap();
                    if !selection.range.is_empty() {
                        tree.trigger_targets(InsertText::new(""), trigger.event_target());
                    } else if cursor.location < value.0.len() {
                        value.0.remove(cursor.location);
                        tree.trigger_targets(ForwardText::default(), trigger.event_target()); // manual forward
                        tree.write_to(trigger.event_target(), RequestedLocation::Offset(cursor.location));
                        tree.trigger_targets(MoveCursor::default(), trigger.event_target());
                        tree.trigger_targets(TextInputState::AwaitingInput, trigger.event_target());
                        tree.trigger_targets(ClearSelection::default(), trigger.event_target());
                    }
                }
                TextInputAction::End => {
                    // one-past-last column; MoveCursor's scan-back resolves it exactly like a
                    // click past the end of the line does
                    let col = metrics
                        .lines
                        .get(cursor.row as usize)
                        .copied()
                        .unwrap_or_default();
                    tree.write_to(
                        trigger.event_target(),
                        RequestedLocation::ColRow((col, cursor.row)),
                    );
                    tree.trigger_targets(MoveCursor::default(), trigger.event_target());
                    tree.trigger_targets(TextInputState::AwaitingInput, trigger.event_target());
                    tree.trigger_targets(ClearSelection::default(), trigger.event_target());
                }
                TextInputAction::Home => {
                    tree.write_to(
                        trigger.event_target(),
                        RequestedLocation::ColRow((0, cursor.row)),
                    );
                    tree.trigger_targets(MoveCursor::default(), trigger.event_target());
                    tree.trigger_targets(TextInputState::AwaitingInput, trigger.event_target());
                    tree.trigger_targets(ClearSelection::default(), trigger.event_target());
                }
                TextInputAction::Copy => {
                    let selection = selections.get(trigger.event_target()).unwrap();
                    if !selection.range.is_empty() {
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
                        tree.trigger_targets(InsertText::new(text), trigger.event_target());
                    }
                }
                TextInputAction::SelectAll => {
                    if !value.0.is_empty() {
                        let mut selection = selections.get_mut(trigger.event_target()).unwrap();
                        selection.range = 0..value.0.len();
                        selection.inverted = false;
                        tree.write_to(
                            trigger.event_target(),
                            RequestedLocation::Offset(value.0.len()),
                        );
                        tree.trigger_targets(MoveCursor::default(), trigger.event_target());
                        tree.trigger_targets(ReselectRange::default(), trigger.event_target());
                        tree.trigger_targets(TextInputState::Highlighting, trigger.event_target());
                    }
                }
                TextInputAction::ExtendLeft => {
                    tree.write_to(
                        trigger.event_target(),
                        RequestedLocation::ColRow((
                            cursor.column.saturating_sub(1),
                            cursor.row,
                        )),
                    );
                    tree.trigger_targets(ExtendRange::default(), trigger.event_target());
                    tree.trigger_targets(ReselectRange::default(), trigger.event_target());
                    tree.trigger_targets(TextInputState::Highlighting, trigger.event_target());
                }
                TextInputAction::ExtendRight => {
                    tree.write_to(
                        trigger.event_target(),
                        RequestedLocation::ColRow((
                            (cursor.column + 1).min(metrics.max_letter_idx_horizontal),
                            cursor.row,
                        )),
                    );
                    tree.trigger_targets(ExtendRange::default(), trigger.event_target());
                    tree.trigger_targets(ReselectRange::default(), trigger.event_target());
                    tree.trigger_targets(TextInputState::Highlighting, trigger.event_target());
                }
                TextInputAction::ExtendUp => {
                    tree.write_to(
                        trigger.event_target(),
                        RequestedLocation::ColRow((
                            cursor.column,
                            cursor.row.saturating_sub(1),
                        )),
                    );
                    tree.trigger_targets(ExtendRange::default(), trigger.event_target());
                    tree.trigger_targets(ReselectRange::default(), trigger.event_target());
                    tree.trigger_targets(TextInputState::Highlighting, trigger.event_target());
                }
                TextInputAction::ExtendDown => {
                    tree.write_to(
                        trigger.event_target(),
                        RequestedLocation::ColRow((
                            cursor.column,
                            (cursor.row + 1)
                                .min(metrics.lines.len().checked_sub(1).unwrap_or_default() as u32),
                        )),
                    );
                    tree.trigger_targets(ExtendRange::default(), trigger.event_target());
                    tree.trigger_targets(ReselectRange::default(), trigger.event_target());
                    tree.trigger_targets(TextInputState::Highlighting, trigger.event_target());
                }
                TextInputAction::Up => {
                    tree.write_to(
                        trigger.event_target(),
                        RequestedLocation::ColRow((
                            cursor.column,
                            cursor.row.checked_sub(1).unwrap_or_default(),
                        )),
                    );
                    tree.trigger_targets(MoveCursor { entity: Entity::PLACEHOLDER }, trigger.event_target());
                    tree.trigger_targets(TextInputState::AwaitingInput, trigger.event_target());
                    tree.trigger_targets(ClearSelection { entity: Entity::PLACEHOLDER }, trigger.event_target());
                }
                TextInputAction::Down => {
                    tree.write_to(
                        trigger.event_target(),
                        RequestedLocation::ColRow((
                            cursor.column,
                            (cursor.row + 1)
                                .min(metrics.lines.len().checked_sub(1).unwrap_or_default() as u32),
                        )),
                    );
                    tree.trigger_targets(MoveCursor { entity: Entity::PLACEHOLDER }, trigger.event_target());
                    tree.trigger_targets(TextInputState::AwaitingInput, trigger.event_target());
                    tree.trigger_targets(ClearSelection { entity: Entity::PLACEHOLDER }, trigger.event_target());
                }
                TextInputAction::Left => {
                    tree.write_to(
                        trigger.event_target(),
                        RequestedLocation::Offset(if cursor.location > 0 {
                            prev_boundary(&value.0, cursor.location)
                        } else {
                            0
                        }),
                    );
                    tree.trigger_targets(MoveCursor { entity: Entity::PLACEHOLDER }, trigger.event_target());
                    tree.trigger_targets(TextInputState::AwaitingInput, trigger.event_target());
                    tree.trigger_targets(ClearSelection { entity: Entity::PLACEHOLDER }, trigger.event_target());
                }
                TextInputAction::Right => {
                    tree.trigger_targets(TextInputState::AwaitingInput, trigger.event_target());
                    tree.trigger_targets(ClearSelection { entity: Entity::PLACEHOLDER }, trigger.event_target());
                    tree.write_to(
                        trigger.event_target(),
                        RequestedLocation::Offset(next_boundary(&value.0, cursor.location)),
                    );
                    tree.trigger_targets(MoveCursor { entity: Entity::PLACEHOLDER }, trigger.event_target());
                }
                TextInputAction::Space => {
                    tree.trigger_targets(
                        InsertText {
                            entity: Entity::PLACEHOLDER,
                            text: " ".to_string(),
                        },
                        trigger.event_target(),
                    );
                }
                // no text mutation; enclosing composites react via the InputAction broadcast
                TextInputAction::Tab => {}
                TextInputAction::Escape => {}
            }
        } else {
            if let Key::Character(text) = &trigger.sequence.key {
                tree.trigger_targets(
                    InsertText {
                        entity: Entity::PLACEHOLDER,
                        text: text.to_string(),
                    },
                    trigger.event_target(),
                );
            }
        }
    }
}
/// Fired at the `TextInput` root whenever its text content changes (typing, deletion, paste,
/// programmatic `TextValue` writes). Subscribe with `tree.subscribe(input, ...)`.
#[derive(EntityEvent, Copy, Clone)]
pub struct TextChanged {
    entity: Entity,
}
impl Default for TextChanged {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}
crate::targeted_event!(TextChanged);
#[derive(EntityEvent, Clone)]
pub(crate) struct ForwardText {
    entity: Entity,
}
impl Default for ForwardText {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}
crate::targeted_event!(ForwardText);
#[derive(EntityEvent, Clone)]
pub struct InsertText {
    entity: Entity,
    pub text: String,
}
impl InsertText {
    pub fn new<S: AsRef<str>>(text: S) -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            text: text.as_ref().to_string(),
        }
    }
}
crate::targeted_event!(InsertText, Input, LocationFromClick);
impl InsertText {
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        selections: Query<&Selection>,
        mut values: Query<&mut TextValue>,
        cursors: Query<&Cursor>,
    ) {
        // typing append (or selection replacement — the cursor lands after the inserted text,
        // at the start of the removed range when the selection is consumed)
        let selection = selections.get(trigger.event_target()).unwrap();
        let mut value = values.get_mut(trigger.event_target()).unwrap();
        let cursor = cursors.get(trigger.event_target()).unwrap();
        let mut new_location = cursor.location.min(value.0.len());
        if !selection.range.is_empty() {
            let range = align_range(&value.0, &selection.range);
            new_location = range.start;
            value.0.replace_range(range, "");
        }
        while new_location > 0 && !value.0.is_char_boundary(new_location) {
            new_location -= 1;
        }
        value.0.insert_str(new_location, &trigger.text);
        new_location += trigger.text.len();
        tree.write_to(trigger.event_target(), RequestedLocation::Offset(new_location));
        tree.trigger_targets(MoveCursor::default(), trigger.event_target());
        tree.trigger_targets(ForwardText::default(), trigger.event_target());
        tree.trigger_targets(TextInputState::AwaitingInput, trigger.event_target());
        tree.trigger_targets(ClearSelection::default(), trigger.event_target());
    }
}
#[derive(Component, Clone, Debug)]
#[component(on_discard = handle_replace::<TextInput>)]
pub struct Handle {
    pub panel: Entity,
    pub text: Entity,
    pub hint_text: Entity,
    pub cursor: Entity,
    pub visible: Entity,
    pub highlights: HashMap<GlyphOffset, Entity>,
}
impl Composite for TextInput {
    type Handle = Handle;
    fn remove(handle: &Self::Handle) -> impl IntoTargets + Send + Sync + 'static {
        let mut targets = handle
            .highlights
            .iter()
            .map(|(_, e)| *e)
            .collect::<Vec<_>>();
        targets.push(handle.panel);
        targets.push(handle.text);
        targets.push(handle.hint_text);
        targets.push(handle.cursor);
        targets.push(handle.visible);
        targets
    }
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
