use crate::composite::Root;
use crate::interaction::CurrentInteraction;
use crate::{Component, Composite, Dragged, Engaged, Event, GlyphOffset, InputSequence, Text, Tree, Unfocused, Write};
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::prelude::Trigger;
use bevy_ecs::query::With;
use bevy_ecs::system::{Query, Res};
use std::collections::HashMap;
use std::ops::Range;

#[derive(Component, Copy, Clone)]
#[require(LineConstraint, Cursor, Selection)]
pub struct TextInput {}
impl TextInput {
    pub fn new() -> TextInput {
        TextInput {}
    }
    fn on_add() {}
    fn on_insert() {}
    fn engaged(trigger: Trigger<Engaged>, mut tree: Tree) {
        // trigger PlaceCursor on trigger.entity()
    }
}
pub(crate) enum TextInputState {
    Inactive,
    Highlighting,
    AwaitingInput,
}
impl TextInputState {
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
    ) {
        // when changed => set OverscrollPropagation + FocusBehavior stuff
    }
}
#[derive(Component, Copy, Clone)]
pub enum LineConstraint {
    Single,
    Multiple,
}
#[derive(Component, Copy, Clone)]
pub(crate) struct Cursor {
    pub(crate) location: GlyphOffset,
    pub(crate) column: usize,
    pub(crate) row: usize,
}
impl Cursor {
    pub(crate) fn new() -> Self {
        Self {
            location: 0,
            column: 0,
            row: 0,
        }
    }
    pub(crate) fn engaged(
        trigger: Trigger<Engaged>,
        mut tree: Tree,
    ) {
        // we clicked explicitly on cursor, start drag behavior
    }
    pub(crate) fn unfocused(
        trigger: Trigger<Unfocused>,
        mut tree: Tree,
    ) {
        // if no focused / focused != text|panel|root
        // forward Unfocused to root.0
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct PlaceCursor {}
impl PlaceCursor {
    pub(crate) fn forward(trigger: Trigger<Engaged>, mut tree: Tree, roots: Query<&Root>) {
        // trigger PlaceCursor on root.0
    }
    pub(crate) fn obs(trigger: Trigger<PlaceCursor>, mut tree: Tree) {
        // initial placement of cursor + configure focus + interaction behavior
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct ClearCursor {}
impl ClearCursor {
    pub(crate) fn obs(trigger: Trigger<Self>, mut tree: Tree) {
        // opacity 0.0 for cursor + ClearSelection + set state Inactive?
        // or let Inactive handle the cleanup
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct MoveCursor {
    pub(crate) new_location: GlyphOffset,
}
impl MoveCursor {
    pub(crate) fn obs(trigger: Trigger<Self>, mut tree: Tree) {
        // attempt to find cursor.location in glyphs
        // if not found => scan backwards until 0 (no adjustment) or found (found + 1 col)
    }
}
#[derive(Component, Clone)]
pub(crate) struct Selection {
    pub(crate) range: Range<GlyphOffset>,
    pub(crate) inverted: bool,
}
impl Selection {
    pub(crate) fn reselect(trigger: Trigger<Write<Text>>, mut tree: Tree, roots: Query<&Root>) {
        let root = roots.get(trigger.entity()).unwrap().0;
        tree.trigger_targets(ReselectRange {}, root);
    }
    pub(crate) fn select(trigger: Trigger<Dragged>, mut tree: Tree) {
        // cursor is dragged => move view near edges + extend selection.range
        // create highlight panels as needed
    }
}
#[derive(Event)]
pub(crate) struct ClearSelection {}
impl ClearSelection {
    pub(crate) fn obs(trigger: Trigger<Self>, mut tree: Tree) {
        // despawn highlight panels + clear handle.highlights
    }
}
#[derive(Event)]
pub(crate) struct ReselectRange {}
impl ReselectRange {
    pub(crate) fn obs(trigger: Trigger<Self>, mut tree: Tree) {
        // iterate highlighted locations and if found glyph => create / update highlight
    }
}

#[derive(Event, Clone)]
pub(crate) struct Input {
    pub(crate) sequence: InputSequence,
}
impl Input {
    pub(crate) fn forward(
        trigger: Trigger<InputSequence>,
        mut tree: Tree,
        roots: Query<&Root>,
        already_root: Query<Entity, With<TextInput>>,
        current_interaction: Res<CurrentInteraction>,
    ) {
        // if any focused => check if root or not (panel, text, cursor)
        // forward Input to root
    }
    pub(crate) fn obs(trigger: Trigger<Self>, mut tree: Tree) {
        // check type of interaction + if trigger action then send correct event
        // [InsertText, CursorMove]
    }
}
#[derive(Event, Clone)]
pub struct InsertText {
    pub text: String,
}
#[derive(Event, Clone)]
pub(crate) struct Enter {}
#[derive(Event, Clone)]
pub(crate) struct Up {}
#[derive(Event, Clone)]
pub(crate) struct Down {}
#[derive(Event, Clone)]
pub(crate) struct Delete {}
#[derive(Component, Clone)]
#[component(on_replace = handle_replace::<TextInput>)]
pub struct Handle {
    pub panel: Entity,
    pub text: Entity,
    pub hint_text: Entity,
    pub cursor: Entity,
    pub highlights: HashMap<GlyphOffset, Entity>,
}
impl Composite for TextInput {
    type Handle = Handle;
    fn remove(handle: &Self::Handle) -> impl TriggerTargets + Send + Sync + 'static {
        let mut targets = handle
            .highlights
            .iter()
            .map(|(_, e)| *e)
            .collect::<Vec<_>>();
        targets.push(handle.panel);
        targets.push(handle.text);
        targets.push(handle.hint_text);
        targets.push(handle.cursor);
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
