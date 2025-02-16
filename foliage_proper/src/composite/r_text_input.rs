use crate::composite::handle_replace;
use crate::composite::Root;
use crate::interaction::CurrentInteraction;
use crate::text::monospaced::MonospacedFont;
use crate::text::{Glyphs, LineMetrics};
use crate::{
    auto, AutoHeight, AutoWidth, Component, Composite, Dragged, EcsExtension, Elevation, Engaged,
    Event, FocusBehavior, FontSize, GlyphOffset, Grid, GridExt, InputSequence, InteractionListener,
    InteractionPropagation, Layout, Location, Logical, Opacity, Panel, Section, Stem, Tertiary,
    Text, Tree, Unfocused, View, Write,
};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::{Query, Res};
use bevy_ecs::world::DeferredWorld;
use fontdue::layout::GlyphPosition;
use std::collections::HashMap;
use std::ops::Range;

#[derive(Component, Copy, Clone)]
#[require(LineConstraint, Cursor, Selection)]
pub struct TextInput {}
impl TextInput {
    pub fn new() -> TextInput {
        TextInput {}
    }
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        // observers
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.commands().entity(this).insert(Grid::default());
        let line_constraint = *world.get::<LineConstraint>(this).unwrap();
        let auto_width = match line_constraint {
            LineConstraint::Single => AutoWidth(true),
            LineConstraint::Multiple => AutoWidth(false),
        };
        let auto_height = match line_constraint {
            LineConstraint::Single => AutoHeight(false),
            LineConstraint::Multiple => AutoHeight(true),
        };
        let panel = world.commands().leaf((
            Panel::new(),
            Stem::some(this),
            Grid::new(1.letters(), 1.letters()),
            InteractionListener::new(),
            Elevation::up(0),
            Location::new().xs(
                0.pct().left().adjust(4).with(100.pct().right().adjust(-4)),
                0.pct().top().adjust(4).with(100.pct().bottom().adjust(-4)),
            ),
            Root(this),
        ));
        let cursor = world.commands().leaf((
            Stem::some(panel),
            InteractionListener::new(),
            InteractionPropagation::pass_through(),
            Elevation::up(5),
            Location::new().xs(
                1.col().left().with(1.col().right()),
                1.col().top().with(1.col().bottom()),
            ),
            Root(this),
        ));
        let visible = world.commands().leaf((
            Panel::new(),
            Stem::some(panel),
            InteractionListener::new(),
            InteractionPropagation::pass_through(),
            Elevation::up(2),
            Location::new().xs(
                1.col().left().with(1.col().right()),
                1.col().top().with(1.col().bottom()),
            ),
            Root(this),
        ));
        let text = world.commands().leaf((
            Text::new(""),
            Stem::some(panel),
            InteractionListener::new(),
            Elevation::up(4),
            Location::new().xs(
                match line_constraint {
                    LineConstraint::Single => 0.pct().left().with(auto().width()),
                    LineConstraint::Multiple => 0.pct().left().with(100.pct().right()),
                },
                match line_constraint {
                    LineConstraint::Single => 0.pct().top().with(100.pct().bottom()),
                    LineConstraint::Multiple => 0.pct().top().with(auto().height()),
                },
            ),
            Root(this),
        ));
        let hint_text = world.commands().leaf((
            Text::new(""),
            Stem::some(panel),
            InteractionPropagation::pass_through(),
            Elevation::up(3),
            Root(this),
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
    // forwarders for all colors + state
    // handle-trigger
}
#[derive(Event, Copy, Clone)]
pub(crate) enum TextInputState {
    Inactive,
    Highlighting,
    AwaitingInput,
}
impl TextInputState {
    pub(crate) fn obs(trigger: Trigger<Self>, mut tree: Tree) {
        // when changed => set OverscrollPropagation + FocusBehavior stuff
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
    pub(crate) fn engaged(trigger: Trigger<Engaged>, mut tree: Tree) {
        // we clicked explicitly on cursor, start drag behavior
        tree.trigger_targets(TextInputState::Highlighting, trigger.entity());
    }
    pub(crate) fn unfocused(trigger: Trigger<Unfocused>, mut tree: Tree) {
        // if no focused / focused != text|panel
        // forward Unfocused to root.0
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct PlaceCursor {}
impl PlaceCursor {
    pub(crate) fn forward(trigger: Trigger<Engaged>, mut tree: Tree, roots: Query<&Root>) {
        // trigger PlaceCursor on root.0
        if let Ok(root) = roots.get(trigger.entity()) {
            tree.trigger_targets(PlaceCursor {}, root.0);
        }
    }
    pub(crate) fn obs(
        trigger: Trigger<PlaceCursor>,
        mut tree: Tree,
        current_interaction: Res<CurrentInteraction>,
        line_metrics: Query<&LineMetrics>,
    ) {
        tree.trigger_targets(ClearSelection {}, trigger.entity());
        // initial placement of cursor + configure focus + interaction behavior
        tree.trigger_targets(TextInputState::AwaitingInput, trigger.entity());
        // col / row from click => (cursor-from-click) [store in requested-location]
        tree.trigger_targets(LocationFromClick(true), trigger.entity());
        // move-cursor with col/row
        tree.trigger_targets(MoveCursor {}, trigger.entity());
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct LocationFromClick(pub(crate) bool);
impl LocationFromClick {
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        mut requested: Query<&mut RequestedLocation>,
        values: Query<&LocationFromClick>,
        current_interaction: Res<CurrentInteraction>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        sections: Query<&Section<Logical>>,
        views: Query<&View>,
        handles: Query<&Handle>,
        line_metrics: Query<&LineMetrics>,
    ) {
        let lfc = u32::from(values.get(trigger.entity()).unwrap().0);
        // offset for col-finding = i32::from(lfc.0); true -> 1 false -> 0
        let click = current_interaction.click.current;
        let fsv = font_sizes
            .get(trigger.entity())
            .unwrap()
            .resolve(*layout)
            .value;
        let dims = font.character_block(fsv);
        let section = sections.get(trigger.entity()).unwrap();
        let handle = handles.get(trigger.entity()).unwrap();
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
        tree.write_to(trigger.entity(), RequestedLocation::ColRow((column, row)))
    }
}
#[derive(Component, Copy, Clone, Default)]
pub(crate) enum RequestedLocation {
    #[default]
    Offset(GlyphOffset),
    ColRow((u32, u32)),
}
#[derive(Event, Copy, Clone)]
pub(crate) struct MoveCursor {}
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
    ) {
        // attempt to find cursor.location in glyphs
        let req = requested.get(trigger.entity()).unwrap();
        let fsv = font_sizes
            .get(trigger.entity())
            .unwrap()
            .resolve(*layout)
            .value;
        let dims = font.character_block(fsv);
        let handle = handles.get(trigger.entity()).unwrap();
        let pred = |glyph: &GlyphPosition<()>| {
            match req {
                RequestedLocation::ColRow((column, row)) => {
                    (glyph.x / dims.a()) as u32 == *column && (glyph.y / dims.b()) as u32 == *row
                }
                RequestedLocation::Offset(offset) => glyph.byte_offset == *offset,
            };
        };
        let mut cursor = cursor.get_mut(trigger.entity()).unwrap();
        if let Some(found) = glyphs
            .get(handle.text)
            .unwrap()
            .layout
            .glyphs()
            .iter()
            .find(pred)
        {
            // found so place there + update cursor
            let col = (found.x / dims.a()) as u32;
            let row = (found.y / dims.b()) as u32;
            let location = Location::new().xs(
                (col + 1).col().left().with((col + 1).col().right()),
                (row + 1).row().top().with((row + 1).row().bottom()),
            );
        } else {
            // if not found => scan backwards until 0 (no adjustment) or found (found + 1 col)
            // set to 0 initially
            // while let Some(scan) = cursor.location.checked_sub(1) {
            //     if we can scan backward => set to scanned + 1 or 0 w/ break
            // }
            //
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
        trigger: Trigger<Write<Text>>,
        mut tree: Tree,
        roots: Query<&Root>,
        cursors: Query<&Cursor>,
    ) {
        let root = roots.get(trigger.entity()).unwrap().0;
        tree.write_to(
            root,
            RequestedLocation::Offset(cursors.get(root).unwrap().location),
        );
        tree.trigger_targets(MoveCursor {}, root);
        tree.trigger_targets(ReselectRange {}, root);
    }
    pub(crate) fn select(trigger: Trigger<Dragged>, mut tree: Tree) {
        // cursor is dragged => move view near edges + extend selection.range
        tree.trigger_targets(LocationFromClick(false), trigger.entity());
        // use RequestedLocation to extend highlight-range
        tree.trigger_targets(ExtendRange {}, trigger.entity());
        // trigger reselect-range after updating the range above
        tree.trigger_targets(ReselectRange {}, trigger.entity());
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct ExtendRange {}
impl ExtendRange {
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        requested: Query<&RequestedLocation>,
        cursors: Query<&Cursor>,
        mut selections: Query<&mut Selection>,
    ) {
        // find requested-location col/row in glyphs
        // range.inverted + found.offset -> cursor.offset
    }
}
#[derive(Event)]
pub(crate) struct ClearSelection {}
impl ClearSelection {
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        mut selections: Query<&mut Selection>,
        mut handles: Query<&mut Handle>,
    ) {
        // despawn highlight panels + clear handle.highlights
        let mut selection = selections.get_mut(trigger.entity()).unwrap();
        selection.range = Range::default();
        for (o, e) in handles
            .get_mut(trigger.entity())
            .unwrap()
            .highlights
            .drain()
        {
            tree.remove(e);
        }
    }
}
#[derive(Event)]
pub(crate) struct ReselectRange {}
impl ReselectRange {
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        mut handles: Query<&mut Handle>,
        glyphs: Query<&Glyphs>,
        cursors: Query<&Cursor>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        tertiary: Query<&Tertiary>,
    ) {
        // iterate highlighted locations and if found glyph => create / update highlight
        let handle = handles.get_mut(trigger.entity()).unwrap();
        let glyph = glyphs.get(handle.text).unwrap();
        let cursor = cursors.get(trigger.entity()).unwrap();
        let fsv = font_sizes
            .get(trigger.entity())
            .unwrap()
            .resolve(*layout)
            .value;
        let dims = font.character_block(fsv);
        for (o, e) in handle.highlights.iter() {
            if let Some(found) = glyph.layout.glyphs().iter().find(|g| g.byte_offset == *o) {
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
                        Panel::new(),
                        Opacity::new(1.0),
                        Stem::some(handle.panel),
                        Elevation::up(1),
                        location,
                        tertiary.get(trigger.entity()).unwrap().0,
                        InteractionPropagation::pass_through(),
                        FocusBehavior::ignore(),
                    ));
                    handle.highlights.insert(found.byte_offset, h);
                }
            } else {
                tree.write_to(*e, Opacity::new(0.0));
            }
        }
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
impl InsertText {
    pub(crate) fn obs(trigger: Trigger<Self>, mut tree: Tree) {
        // typing append
    }
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
    pub visible: Entity,
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
