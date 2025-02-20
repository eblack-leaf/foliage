pub(crate) mod action;
pub(crate) mod keybindings;

use crate::composite::handle_replace;
use crate::composite::Root;
use crate::interaction::CurrentInteraction;
use crate::text::monospaced::MonospacedFont;
use crate::text::{Glyphs, LineMetrics};
use crate::{
    auto, Attachment, AutoHeight, AutoWidth, Color, Component, Composite, Dragged, EcsExtension,
    Elevation, Engaged, Event, FocusBehavior, Foliage, FontSize, GlyphOffset, Grid, GridExt,
    InputSequence, InteractionListener, InteractionPropagation, Layout, Location, Logical, Opacity,
    OverscrollPropagation, Panel, Primary, Secondary, Section, Stem, Tertiary, Text, TextValue,
    Tree, Unfocused, Update, View, Write,
};
use action::TextInputAction;
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::prelude::{OnInsert, Trigger};
use bevy_ecs::system::{Query, Res};
use bevy_ecs::world::DeferredWorld;
use keybindings::KeyBindings;
use std::collections::HashMap;
use std::ops::Range;
use winit::keyboard::Key;

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
#[require(LineConstraint, Cursor, Selection, HintText, RequestedLocation)]
#[require(Primary, Secondary, Tertiary, FontSize, TextValue)]
#[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
pub struct TextInput {}
impl TextInput {
    const HIGHLIGHT_SCROLL_THRESHOLD: f32 = 10.0;
    pub fn new() -> TextInput {
        TextInput {}
    }
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
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
            .observe(Self::update_font_size);
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.commands().entity(this).insert(Grid::default());
        let line_constraint = *world.get::<LineConstraint>(this).unwrap();
        let panel = world.commands().leaf((
            Panel::new(),
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
            Panel::new(),
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
            Text::new(""),
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
            Text::new(""),
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
        println!("handle: {:?} for {:?}", handle, this);
        world.commands().entity(this).insert(handle);
    }
    pub(crate) fn unfocused(
        trigger: Trigger<Unfocused>,
        mut tree: Tree,
        roots: Query<&Root>,
        handles: Query<&Handle>,
        current_interaction: Res<CurrentInteraction>,
    ) {
        let main = if let Ok(root) = roots.get(trigger.entity()) {
            root.0
        } else {
            trigger.entity()
        };
        let handle = handles.get(main).unwrap();
        if let Some(f) = current_interaction.focused {
            if f == main || f == handle.panel || f == handle.text || f == handle.cursor {
                return;
            }
        }
        tree.trigger_targets(ClearSelection {}, main);
        tree.trigger_targets(TextInputState::Inactive, main);
    }
    // forwarders for all colors + state
    fn handle_trigger(trigger: Trigger<OnInsert, Handle>, mut tree: Tree) {
        println!("handle_trigger");
        let this = trigger.entity();
        tree.trigger_targets(Update::<TextValue>::new(), this);
        tree.trigger_targets(Update::<Primary>::new(), this);
        tree.trigger_targets(Update::<Secondary>::new(), this);
        tree.trigger_targets(Update::<Tertiary>::new(), this);
        tree.trigger_targets(Update::<FontSize>::new(), this);
    }
    fn forward_text_value(trigger: Trigger<OnInsert, TextValue>, mut tree: Tree) {
        println!("forward_text_value");
        tree.trigger_targets(Update::<TextValue>::new(), trigger.entity());
    }
    fn update_text_value(trigger: Trigger<Update<TextValue>>, mut tree: Tree) {
        println!("update_text_value");
        tree.trigger_targets(ForwardText {}, trigger.entity());
        tree.trigger_targets(ClearSelection {}, trigger.entity());
        tree.trigger_targets(TextInputState::Inactive, trigger.entity());
    }
    fn forward_font_size(trigger: Trigger<OnInsert, FontSize>, mut tree: Tree) {
        println!("forward_font_size");
        tree.trigger_targets(Update::<FontSize>::new(), trigger.entity());
    }
    fn update_font_size(
        trigger: Trigger<Update<FontSize>>,
        mut tree: Tree,
        font_sizes: Query<&FontSize>,
        handles: Query<&Handle>,
    ) {
        println!("update_font_size");
        // give to handle.text
        let handle = handles.get(trigger.entity()).unwrap();
        tree.entity(handle.text)
            .insert(font_sizes.get(trigger.entity()).unwrap().clone());
        tree.entity(handle.hint_text)
            .insert(font_sizes.get(trigger.entity()).unwrap().clone());
        tree.entity(handle.panel)
            .insert(font_sizes.get(trigger.entity()).unwrap().clone());
    }
    fn update_primary(
        trigger: Trigger<Update<Primary>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        primary: Query<&Primary>,
    ) {
        println!("update_primary");
        // text-color
        let handle = handles.get(trigger.entity()).unwrap();
        tree.entity(handle.text)
            .insert(primary.get(trigger.entity()).unwrap().0);
        tree.entity(handle.hint_text)
            .insert(primary.get(trigger.entity()).unwrap().0);
    }
    fn forward_primary(trigger: Trigger<OnInsert, Primary>, mut tree: Tree) {
        println!("forward_primary");
        tree.trigger_targets(Update::<Primary>::new(), trigger.entity());
    }
    fn update_secondary(
        trigger: Trigger<Update<Secondary>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        secondary: Query<&Secondary>,
    ) {
        println!("update_secondary");
        // background (panel)
        let handle = handles.get(trigger.entity()).unwrap();
        tree.entity(handle.panel)
            .insert(secondary.get(trigger.entity()).unwrap().0);
    }
    fn forward_secondary(trigger: Trigger<OnInsert, Secondary>, mut tree: Tree) {
        println!("forward_secondary");
        tree.trigger_targets(Update::<Secondary>::new(), trigger.entity());
    }
    fn update_tertiary(
        trigger: Trigger<Update<Tertiary>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        tertiary: Query<&Tertiary>,
    ) {
        println!("update_tertiary");
        // cursor color + highlight color
        let handle = handles.get(trigger.entity()).unwrap();
        let color = tertiary.get(trigger.entity()).unwrap().0;
        tree.entity(handle.visible).insert(color);
        for (o, e) in handle.highlights.iter() {
            tree.entity(*e).insert(color);
        }
    }
    fn forward_tertiary(trigger: Trigger<OnInsert, Tertiary>, mut tree: Tree) {
        println!("forward_tertiary");
        tree.trigger_targets(Update::<Tertiary>::new(), trigger.entity());
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) enum TextInputState {
    Inactive,
    Highlighting,
    AwaitingInput,
}
impl TextInputState {
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        handles: Query<&Handle>,
        tertiary: Query<&Tertiary>,
        primary: Query<&Primary>,
    ) {
        let value = trigger.event();
        let handle = handles.get(trigger.entity()).unwrap();
        match value {
            TextInputState::Inactive => {
                tree.write_to(trigger.entity(), OverscrollPropagation(true));
                tree.write_to(handle.visible, Opacity::new(0.0));
                tree.write_to(handle.cursor, InteractionPropagation::pass_through());
                tree.disable(handle.cursor);
            }
            TextInputState::Highlighting => {
                tree.write_to(trigger.entity(), OverscrollPropagation(false));
                tree.write_to(
                    handle.visible,
                    (Opacity::new(0.75), primary.get(trigger.entity()).unwrap().0),
                )
            }
            TextInputState::AwaitingInput => {
                tree.write_to(trigger.entity(), OverscrollPropagation(true));
                tree.write_to(handle.cursor, InteractionPropagation::grab().disable_drag());
                tree.write_to(
                    handle.visible,
                    (
                        Opacity::new(0.25),
                        tertiary.get(trigger.entity()).unwrap().0,
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
            roots.get(trigger.entity()).unwrap().0,
        );
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct PlaceCursor {}
impl PlaceCursor {
    pub(crate) fn forward(trigger: Trigger<Engaged>, mut tree: Tree, roots: Query<&Root>) {
        // trigger PlaceCursor on root.0
        println!("forwarding place-cursor");
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
        println!("place-cursor");
        tree.trigger_targets(ClearSelection {}, trigger.entity());
        // initial placement of cursor + configure focus + interaction behavior
        tree.trigger_targets(TextInputState::AwaitingInput, trigger.entity());
        // col / row from click => (cursor-from-click) [store in requested-location]
        tree.trigger_targets(
            LocationFromClick {
                can_go_past_end: true,
            },
            trigger.entity(),
        );
        // move-cursor with col/row
        tree.trigger_targets(MoveCursor {}, trigger.entity());
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct LocationFromClick {
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
        println!("location from click");
        let lfc = u32::from(trigger.can_go_past_end);
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
        line_metrics: Query<&LineMetrics>,
    ) {
        println!("move-cursor");
        let req = requested.get(trigger.entity()).unwrap();
        let fsv = font_sizes
            .get(trigger.entity())
            .unwrap()
            .resolve(*layout)
            .value;
        let dims = font.character_block(fsv);
        let handle = handles.get(trigger.entity()).unwrap();
        let metrics = line_metrics.get(handle.text).unwrap();
        let mut cursor = cursor.get_mut(trigger.entity()).unwrap();
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
        println!("reselect");
        let root = roots.get(trigger.entity()).unwrap().0;
        tree.write_to(
            root,
            RequestedLocation::Offset(cursors.get(root).unwrap().location),
        );
        tree.trigger_targets(MoveCursor {}, root);
        tree.trigger_targets(ReselectRange {}, root);
    }
    pub(crate) fn select(trigger: Trigger<Dragged>, mut tree: Tree, roots: Query<&Root>) {
        // cursor is dragged => move view near edges + extend selection.range
        println!("select");
        let root = roots.get(trigger.entity()).unwrap().0;
        tree.trigger_targets(
            LocationFromClick {
                can_go_past_end: false,
            },
            root,
        );
        // use RequestedLocation to extend highlight-range
        tree.trigger_targets(ExtendRange {}, root);
        // trigger reselect-range after updating the range above
        tree.trigger_targets(ReselectRange {}, root);
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
        glyphs: Query<&Glyphs>,
        handles: Query<&Handle>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
    ) {
        println!("extend-range");
        let handle = handles.get(trigger.entity()).unwrap();
        let fsv = font_sizes
            .get(trigger.entity())
            .unwrap()
            .resolve(*layout)
            .value;
        let dims = font.character_block(fsv);
        let cursor = cursors.get(trigger.entity()).unwrap();
        let mut selection = selections.get_mut(trigger.entity()).unwrap();
        let req = requested.get(trigger.entity()).unwrap();
        match req {
            RequestedLocation::ColRow((c, r)) => {
                for glyph in glyphs.get(handle.text).unwrap().layout.glyphs() {
                    if (glyph.x / dims.a()) as u32 == *c && (glyph.y / dims.b()) as u32 == *r {
                        if cursor.location < glyph.byte_offset {
                            selection.inverted = false;
                            selection.range = cursor.location..(glyph.byte_offset + 1);
                        } else {
                            selection.inverted = true;
                            selection.range = glyph.byte_offset..(cursor.location + 1);
                        }
                    }
                }
            }
            _ => {}
        }
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
        println!("clear-selection");
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
        println!("reselect-range");
        let mut handle = handles.get_mut(trigger.entity()).unwrap();
        let glyph = glyphs.get(handle.text).unwrap();
        let cursor = cursors.get(trigger.entity()).unwrap();
        let fsv = font_sizes
            .get(trigger.entity())
            .unwrap()
            .resolve(*layout)
            .value;
        let dims = font.character_block(fsv);
        for (o, e) in handle.highlights.clone().iter() {
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
                        Elevation::up(2),
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
        handles: Query<&Handle>,
    ) {
        println!("forward-input");
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
        let handle = handles.get(trigger.entity()).unwrap();
        let value = values.get(trigger.entity()).unwrap();
        println!("forward-text {}", value.0);
        tree.write_to(handle.text, Text::new(&value.0));
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
    ) {
        println!("Input::obs");
        // check type of interaction + if trigger action then send correct event
        let cursor = cursors.get(trigger.entity()).unwrap();
        let lc = line_constraints.get(trigger.entity()).unwrap();
        let mut value = values.get_mut(trigger.entity()).unwrap();
        let handle = handles.get(trigger.entity()).unwrap();
        let metrics = line_metrics.get(handle.text).unwrap();
        if let Some(action) = key_bindings.action(&trigger.event().sequence) {
            // handle action
            match action {
                TextInputAction::Enter => match lc {
                    LineConstraint::Single => {
                        tree.trigger_targets(TextInputAction::Enter, trigger.entity())
                    }
                    LineConstraint::Multiple => {
                        tree.trigger_targets(
                            InsertText {
                                text: "\n".parse().unwrap(),
                            },
                            trigger.entity(),
                        );
                    }
                },
                TextInputAction::Backspace => {
                    // if we have previous => delete that char + update cursor
                    if let Some(idx) = cursor.location.checked_sub(1) {
                        if value.0.get(idx..idx + 1).is_some() {
                            value.0.remove(idx);
                        }
                        tree.trigger_targets(ForwardText {}, trigger.entity()); // manual forward
                        tree.write_to(trigger.entity(), RequestedLocation::Offset(idx));
                        tree.trigger_targets(MoveCursor {}, trigger.entity());
                        tree.trigger_targets(TextInputState::AwaitingInput, trigger.entity());
                        tree.trigger_targets(ClearSelection {}, trigger.entity());
                    }
                }
                TextInputAction::Delete => {
                    // delete char at cursor.location
                    tree.trigger_targets(ForwardText {}, trigger.entity()); // manual forward
                    tree.trigger_targets(TextInputState::AwaitingInput, trigger.entity());
                    tree.trigger_targets(ClearSelection {}, trigger.entity());
                }
                TextInputAction::End => {
                    // move cursor to end of line
                    tree.trigger_targets(TextInputState::AwaitingInput, trigger.entity());
                    tree.trigger_targets(ClearSelection {}, trigger.entity());
                }
                TextInputAction::Home => {
                    // move cursor to begin of line
                    tree.trigger_targets(TextInputState::AwaitingInput, trigger.entity());
                    tree.trigger_targets(ClearSelection {}, trigger.entity());
                }
                TextInputAction::Copy => {
                    // capture selection.range
                }
                TextInputAction::Paste => {
                    // InsertText for all of clipboard
                }
                TextInputAction::SelectAll => {
                    // selection.range => all
                }
                TextInputAction::ExtendLeft => {
                    // like down
                }
                TextInputAction::ExtendRight => {
                    // like down
                }
                TextInputAction::ExtendUp => {
                    // like down
                }
                TextInputAction::ExtendDown => {
                    tree.write_to(
                        trigger.entity(),
                        RequestedLocation::ColRow((
                            cursor.column,
                            (cursor.row + 1)
                                .min(metrics.lines.len().checked_sub(1).unwrap_or_default() as u32),
                        )),
                    );
                    tree.trigger_targets(ExtendRange {}, trigger.entity());
                }
                TextInputAction::Up => {
                    tree.write_to(
                        trigger.entity(),
                        RequestedLocation::ColRow((
                            cursor.column,
                            cursor.row.checked_sub(1).unwrap_or_default(),
                        )),
                    );
                    tree.trigger_targets(MoveCursor {}, trigger.entity());
                    tree.trigger_targets(TextInputState::AwaitingInput, trigger.entity());
                    tree.trigger_targets(ClearSelection {}, trigger.entity());
                }
                TextInputAction::Down => {
                    tree.write_to(
                        trigger.entity(),
                        RequestedLocation::ColRow((
                            cursor.column,
                            (cursor.row + 1)
                                .min(metrics.lines.len().checked_sub(1).unwrap_or_default() as u32),
                        )),
                    );
                    tree.trigger_targets(MoveCursor {}, trigger.entity());
                    tree.trigger_targets(TextInputState::AwaitingInput, trigger.entity());
                    tree.trigger_targets(ClearSelection {}, trigger.entity());
                }
                TextInputAction::Left => {
                    tree.write_to(
                        trigger.entity(),
                        RequestedLocation::Offset(
                            cursor.location.checked_sub(1).unwrap_or_default(),
                        ),
                    );
                    tree.trigger_targets(MoveCursor {}, trigger.entity());
                    tree.trigger_targets(TextInputState::AwaitingInput, trigger.entity());
                    tree.trigger_targets(ClearSelection {}, trigger.entity());
                }
                TextInputAction::Right => {
                    tree.trigger_targets(TextInputState::AwaitingInput, trigger.entity());
                    tree.trigger_targets(ClearSelection {}, trigger.entity());
                    tree.write_to(
                        trigger.entity(),
                        RequestedLocation::Offset((cursor.location + 1).min(value.0.len())),
                    );
                    tree.trigger_targets(MoveCursor {}, trigger.entity());
                }
                TextInputAction::Space => {
                    tree.trigger_targets(
                        InsertText {
                            text: " ".to_string(),
                        },
                        trigger.entity(),
                    );
                }
            }
        } else {
            if let Key::Character(text) = &trigger.sequence.key {
                tree.trigger_targets(
                    InsertText {
                        text: text.to_string(),
                    },
                    trigger.entity(),
                );
            }
        }
    }
}
#[derive(Event, Clone)]
pub(crate) struct ForwardText {}
#[derive(Event, Clone)]
pub struct InsertText {
    pub text: String,
}
impl InsertText {
    pub(crate) fn obs(
        trigger: Trigger<Self>,
        mut tree: Tree,
        selections: Query<&Selection>,
        mut values: Query<&mut TextValue>,
        cursors: Query<&Cursor>,
    ) {
        println!("inserting text");
        // typing append
        let selection = selections.get(trigger.entity()).unwrap();
        let mut value = values.get_mut(trigger.entity()).unwrap();
        let cursor = cursors.get(trigger.entity()).unwrap();
        let mut new_location = cursor.location;
        if !selection.range.is_empty() {
            for i in selection.range.clone().rev() {
                if value.0.get(i..i + 1).is_some() {
                    value.0.remove(i);
                }
            }
            if selection.inverted {
                new_location += 1;
                new_location = new_location
                    .checked_sub(selection.range.len())
                    .unwrap_or_default();
            }
        }
        for ch in trigger.text.chars().rev() {
            value.0.insert(new_location, ch);
        }
        new_location += trigger.text.len();
        tree.write_to(trigger.entity(), RequestedLocation::Offset(new_location));
        tree.trigger_targets(MoveCursor {}, trigger.entity());
        tree.trigger_targets(ForwardText {}, trigger.entity());
        tree.trigger_targets(TextInputState::AwaitingInput, trigger.entity());
        tree.trigger_targets(ClearSelection {}, trigger.entity());
    }
}
#[derive(Component, Clone, Debug)]
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
#[derive(Component, Clone, Default)]
pub struct HintColor(pub Color);
