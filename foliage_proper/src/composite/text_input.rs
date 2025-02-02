use crate::ash::clip::ClipSection;
use crate::composite::handle_replace;
use crate::ginkgo::ScaleFactor;
use crate::interaction::CurrentInteraction;
use crate::text::monospaced::MonospacedFont;
use crate::text::{Glyphs, LineMetrics};
use crate::{
    Attachment, Component, Composite, Dragged, EcsExtension, Elevation, Engaged, FocusBehavior,
    Foliage, FontSize, GlyphOffset, Grid, GridExt, InputSequence, InteractionListener,
    InteractionPropagation, Layout, Location, Logical, Opacity, OverscrollPropagation, Panel,
    Primary, Resource, Secondary, Section, Stem, Tertiary, Text, TextValue, Tree, Unfocused,
    Update, Write,
};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::prelude::{Res, Trigger};
use bevy_ecs::system::Query;
use bevy_ecs::world::{DeferredWorld, OnInsert};
use std::collections::HashMap;
use std::ops::Range;
use winit::keyboard::{Key, ModifiersState, NamedKey, SmolStr};

#[derive(Component, Clone)]
#[require(HintText, Primary, Secondary, Tertiary, FontSize, TextValue)]
#[component(on_insert = Self::on_insert)]
#[component(on_add = Self::on_add)]
pub struct TextInput {
    pub(crate) highlight_range: Range<GlyphOffset>,
    pub(crate) range_backwards: bool,
    pub(crate) cursor_location: GlyphOffset,
    pub(crate) cursor_col_row: (usize, usize),
}
impl TextInput {
    pub fn new() -> Self {
        Self {
            highlight_range: Default::default(),
            range_backwards: false,
            cursor_location: 0,
            cursor_col_row: (0, 0),
        }
    }
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .entity(this)
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
            .observe(Self::clear_cursor)
            .observe(Self::place_cursor);
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        println!("on-insert");
        world.commands().entity(this).insert(Grid::default());
        let panel = world.commands().leaf((
            Panel::new(),
            Location::new().xs(
                0.pct().left().adjust(4).with(100.pct().right().adjust(-4)),
                0.pct().top().adjust(4).with(100.pct().bottom().adjust(-4)),
            ),
            Grid::new(1.letters(), 1.letters()),
            Elevation::up(0),
            InteractionPropagation::pass_through(),
            FocusBehavior::ignore(),
            Stem::some(this),
        ));
        let cursor = world.commands().leaf((
            Panel::new(),
            Elevation::up(2),
            Stem::some(panel),
            Opacity::new(0.0),
            Location::new().xs(
                1.col().left().with(1.col().right()),
                1.col().top().with(1.col().bottom()),
            ),
            InteractionPropagation::pass_through(),
            FocusBehavior::ignore(),
            InteractionListener::new(),
            TextInputLink { root: this },
        ));
        world.commands().disable(cursor);
        world.commands().subscribe(cursor, Self::highlight_range);
        world.commands().subscribe(cursor, Self::engage_cursor);
        // TODO text SingleLine => AutoWidth(true) [no max-width + set width in Update] or MultiLine => AutoHeight(true) [existing impl]
        let text = world.commands().leaf((
            Stem::some(panel),
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ),
            Elevation::up(3),
            TextInputLink { root: this },
            InteractionPropagation::pass_through(),
            FocusBehavior::ignore(),
        ));
        world.commands().subscribe(text, Self::write_text);
        let handle = Handle {
            panel,
            text,
            cursor,
            highlights: Default::default(),
        };
        world
            .commands()
            .entity(this)
            .insert(handle)
            .insert(InteractionListener::new());
    }
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
    fn update_text_value(
        trigger: Trigger<Update<TextValue>>,
        mut tree: Tree,
        mut handles: Query<&mut Handle>,
        mut text_inputs: Query<&mut TextInput>,
        tv: Query<&TextValue>,
    ) {
        println!("update_text_value");
        // give to handle.text
        let t = tv.get(trigger.entity()).unwrap();
        let mut handle = handles.get_mut(trigger.entity()).unwrap();
        tree.entity(handle.text).insert(Text::new(&t.0));
        // clear highlighting as they are invalid offsets now text has changed
        tree.entity(handle.cursor)
            .insert(Opacity::new(0.0))
            .insert(InteractionPropagation::pass_through());
        tree.disable(handle.cursor);
        tree.entity(trigger.entity())
            .insert(OverscrollPropagation(true));
        for (o, e) in handle.highlights.drain() {
            tree.remove(e);
        }
        text_inputs
            .get_mut(trigger.entity())
            .unwrap()
            .highlight_range = Default::default();
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
        tree.entity(handle.cursor).insert(color);
        for (o, e) in handle.highlights.iter() {
            tree.entity(*e).insert(color);
        }
    }
    fn forward_tertiary(trigger: Trigger<OnInsert, Tertiary>, mut tree: Tree) {
        println!("forward_tertiary");
        tree.trigger_targets(Update::<Tertiary>::new(), trigger.entity());
    }
    fn typing(
        trigger: Trigger<InputSequence>,
        current_interaction: Res<CurrentInteraction>,
        key_bindings: Res<KeyBindings>,
        mut text_inputs: Query<&mut TextInput>,
        mut handles: Query<&mut Handle>,
        metrics: Query<&LineMetrics>,
        mut text_values: Query<&mut TextValue>,
        tertiaries: Query<&Tertiary>,
        glyphs: Query<&Glyphs>,
        mut tree: Tree,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
    ) {
        println!("typing");
        if let Some(focused) = current_interaction.focused {
            if let Ok(mut text_input) = text_inputs.get_mut(focused) {
                let mut handle = handles.get_mut(focused).unwrap();
                let mut current = text_values.get_mut(focused).unwrap();
                let metrics = metrics.get(handle.text).unwrap();
                let glyphs = glyphs.get(handle.text).unwrap();
                let fsv = font_sizes.get(handle.text).unwrap().resolve(*layout).value;
                let dims = font.character_block(fsv);
                if let Some(action) = key_bindings.action(&trigger.event()) {
                    // TODO process action
                    match action {
                        TextInputAction::Backspace => {
                            let a = SmolStr::new("");
                            if !text_input.highlight_range.is_empty() {
                                for i in text_input.highlight_range.clone().rev() {
                                    if current.0.get(i..i + 1).is_some() {
                                        current.0.remove(i);
                                    }
                                }
                                if text_input.range_backwards {
                                    text_input.cursor_location += 1; // adjust for cursor space
                                    text_input.cursor_location = text_input
                                        .cursor_location
                                        .checked_sub(text_input.highlight_range.len())
                                        .unwrap_or_default();
                                }
                            } else {
                                // delete previous character if there + move cursor back one
                                let idx_begin = text_input
                                    .cursor_location
                                    .checked_sub(1)
                                    .unwrap_or_default();
                                if current.0.get(idx_begin..idx_begin + 1).is_some() {
                                    current.0.remove(idx_begin);
                                }
                                text_input.cursor_location = idx_begin;
                                if let Some(found) = glyphs
                                    .layout
                                    .glyphs()
                                    .iter()
                                    .find(|g| g.byte_offset == text_input.cursor_location)
                                {
                                    let col = (found.x / dims.a()) as u32;
                                    let row = (found.y / dims.b()) as u32;
                                    let location = Location::new().xs(
                                        (col + 1).col().left().with((col + 1).col().right()),
                                        (row + 1).row().top().with((row + 1).row().bottom()),
                                    );
                                    text_input.cursor_col_row = (col as usize, row as usize);
                                    tree.entity(handle.cursor).insert(location);
                                } else {
                                    if text_input.cursor_location == 0 {
                                        let col = 0;
                                        let row = 0;
                                        let location = Location::new().xs(
                                            (col + 1).col().left().with((col + 1).col().right()),
                                            (row + 1).row().top().with((row + 1).row().bottom()),
                                        );
                                        text_input.cursor_col_row = (col as usize, row as usize);
                                        tree.entity(handle.cursor).insert(location);
                                    } else {
                                        let mut scan = Some(
                                            text_input
                                                .cursor_location
                                                .checked_sub(1)
                                                .unwrap_or_default(),
                                        );
                                        while let Some(s) = scan {
                                            if let Some(found) = glyphs
                                                .layout
                                                .glyphs()
                                                .iter()
                                                .find(|g| g.byte_offset == s)
                                            {
                                                let col = (found.x / dims.a()) as u32;
                                                let col = (col + 1)
                                                    .min(metrics.max_letter_idx_horizontal);
                                                let row = (found.y / dims.b()) as u32;
                                                let location = Location::new().xs(
                                                    (col + 1)
                                                        .col()
                                                        .left()
                                                        .with((col + 1).col().right()),
                                                    (row + 1)
                                                        .row()
                                                        .top()
                                                        .with((row + 1).row().bottom()),
                                                );
                                                text_input.cursor_col_row =
                                                    (col as usize, row as usize);
                                                tree.entity(handle.cursor).insert(location);
                                                break;
                                            } else {
                                                if s == 0 {
                                                    let col = 0;
                                                    let row = 0;
                                                    let location = Location::new().xs(
                                                        (col + 1)
                                                            .col()
                                                            .left()
                                                            .with((col + 1).col().right()),
                                                        (row + 1)
                                                            .row()
                                                            .top()
                                                            .with((row + 1).row().bottom()),
                                                    );
                                                    text_input.cursor_col_row =
                                                        (col as usize, row as usize);
                                                    tree.entity(handle.cursor).insert(location);
                                                    break;
                                                }
                                                scan = s.checked_sub(1);
                                            }
                                        }
                                    }
                                }
                            }
                            tree.entity(handle.text).insert(Text::new(&current.0));
                        }
                        _ => {}
                    }
                } else {
                    let (append, cursor_update) = match &trigger.event().key {
                        Key::Named(named) => {
                            match named {
                                NamedKey::ArrowLeft => (None, Some(CursorMove::Left)),
                                NamedKey::ArrowRight => (None, Some(CursorMove::Right)),
                                NamedKey::ArrowUp => (None, Some(CursorMove::Up)),
                                NamedKey::ArrowDown => (None, Some(CursorMove::Down)),
                                NamedKey::Space => (Some(SmolStr::new(" ")), None),
                                _ => {
                                    // unsupported specific key
                                    (None, None)
                                }
                            }
                        }
                        Key::Character(ch) => (Some(ch.clone()), None),
                        Key::Unidentified(_) => (None, None),
                        Key::Dead(_) => (None, None),
                    };
                    if let Some(a) = append {
                        if !text_input.highlight_range.is_empty() {
                            for i in text_input.highlight_range.clone().rev() {
                                if current.0.get(i..i + 1).is_some() {
                                    current.0.remove(i);
                                }
                            }
                            if text_input.range_backwards {
                                text_input.cursor_location += 1; // adjust for cursor space
                                text_input.cursor_location = text_input
                                    .cursor_location
                                    .checked_sub(text_input.highlight_range.len())
                                    .unwrap_or_default();
                            }
                        }
                        for c in a.chars().rev() {
                            current.0.insert(text_input.cursor_location, c);
                        }
                        text_input.cursor_location += a.len();
                        tree.entity(handle.text).insert(Text::new(&current.0));
                    } else if let Some(cursor_update) = cursor_update {
                        match cursor_update {
                            CursorMove::Left => {
                                text_input.cursor_location = text_input
                                    .cursor_location
                                    .checked_sub(1)
                                    .unwrap_or_default();
                                if let Some(found) = glyphs
                                    .layout
                                    .glyphs()
                                    .iter()
                                    .find(|g| g.byte_offset == text_input.cursor_location)
                                {
                                    let col = (found.x / dims.a()) as u32;
                                    let row = (found.y / dims.b()) as u32;
                                    let location = Location::new().xs(
                                        (col + 1).col().left().with((col + 1).col().right()),
                                        (row + 1).row().top().with((row + 1).row().bottom()),
                                    );
                                    text_input.cursor_col_row = (col as usize, row as usize);
                                    tree.entity(handle.cursor).insert(location);
                                } else {
                                    if text_input.cursor_location == 0 {
                                        let col = 0;
                                        let row = 0;
                                        let location = Location::new().xs(
                                            (col + 1).col().left().with((col + 1).col().right()),
                                            (row + 1).row().top().with((row + 1).row().bottom()),
                                        );
                                        text_input.cursor_col_row = (col as usize, row as usize);
                                        tree.entity(handle.cursor).insert(location);
                                    } else {
                                        let mut scan = Some(
                                            text_input
                                                .cursor_location
                                                .checked_sub(1)
                                                .unwrap_or_default(),
                                        );
                                        while let Some(s) = scan {
                                            if let Some(found) = glyphs
                                                .layout
                                                .glyphs()
                                                .iter()
                                                .find(|g| g.byte_offset == s)
                                            {
                                                let col = (found.x / dims.a()) as u32;
                                                let col = (col + 1)
                                                    .min(metrics.max_letter_idx_horizontal);
                                                let row = (found.y / dims.b()) as u32;
                                                let location = Location::new().xs(
                                                    (col + 1)
                                                        .col()
                                                        .left()
                                                        .with((col + 1).col().right()),
                                                    (row + 1)
                                                        .row()
                                                        .top()
                                                        .with((row + 1).row().bottom()),
                                                );
                                                text_input.cursor_col_row =
                                                    (col as usize, row as usize);
                                                tree.entity(handle.cursor).insert(location);
                                                break;
                                            } else {
                                                if s == 0 {
                                                    let col = 0;
                                                    let row = 0;
                                                    let location = Location::new().xs(
                                                        (col + 1)
                                                            .col()
                                                            .left()
                                                            .with((col + 1).col().right()),
                                                        (row + 1)
                                                            .row()
                                                            .top()
                                                            .with((row + 1).row().bottom()),
                                                    );
                                                    text_input.cursor_col_row =
                                                        (col as usize, row as usize);
                                                    tree.entity(handle.cursor).insert(location);
                                                    break;
                                                }
                                                scan = s.checked_sub(1);
                                            }
                                        }
                                    }
                                }
                            }
                            CursorMove::Right => {
                                text_input.cursor_location =
                                    (text_input.cursor_location + 1).min(current.0.len());
                                if let Some(found) = glyphs
                                    .layout
                                    .glyphs()
                                    .iter()
                                    .find(|g| g.byte_offset == text_input.cursor_location)
                                {
                                    let col = (found.x / dims.a()) as u32;
                                    let row = (found.y / dims.b()) as u32;
                                    let location = Location::new().xs(
                                        (col + 1).col().left().with((col + 1).col().right()),
                                        (row + 1).row().top().with((row + 1).row().bottom()),
                                    );
                                    text_input.cursor_col_row = (col as usize, row as usize);
                                    tree.entity(handle.cursor).insert(location);
                                } else {
                                    if text_input.cursor_location == 0 {
                                        let col = 0;
                                        let row = 0;
                                        let location = Location::new().xs(
                                            (col + 1).col().left().with((col + 1).col().right()),
                                            (row + 1).row().top().with((row + 1).row().bottom()),
                                        );
                                        text_input.cursor_col_row = (col as usize, row as usize);
                                        tree.entity(handle.cursor).insert(location);
                                    } else {
                                        let mut scan = Some(
                                            text_input
                                                .cursor_location
                                                .checked_sub(1)
                                                .unwrap_or_default(),
                                        );
                                        while let Some(s) = scan {
                                            if let Some(found) = glyphs
                                                .layout
                                                .glyphs()
                                                .iter()
                                                .find(|g| g.byte_offset == s)
                                            {
                                                let col = (found.x / dims.a()) as u32;
                                                let col = (col + 1)
                                                    .min(metrics.max_letter_idx_horizontal);
                                                let row = (found.y / dims.b()) as u32;
                                                let location = Location::new().xs(
                                                    (col + 1)
                                                        .col()
                                                        .left()
                                                        .with((col + 1).col().right()),
                                                    (row + 1)
                                                        .row()
                                                        .top()
                                                        .with((row + 1).row().bottom()),
                                                );
                                                text_input.cursor_col_row =
                                                    (col as usize, row as usize);
                                                tree.entity(handle.cursor).insert(location);
                                                break;
                                            } else {
                                                if s == 0 {
                                                    let col = 0;
                                                    let row = 0;
                                                    let location = Location::new().xs(
                                                        (col + 1)
                                                            .col()
                                                            .left()
                                                            .with((col + 1).col().right()),
                                                        (row + 1)
                                                            .row()
                                                            .top()
                                                            .with((row + 1).row().bottom()),
                                                    );
                                                    text_input.cursor_col_row =
                                                        (col as usize, row as usize);
                                                    tree.entity(handle.cursor).insert(location);
                                                    break;
                                                }
                                                scan = s.checked_sub(1);
                                            }
                                        }
                                    }
                                }
                            }
                            CursorMove::Up => {
                                let mut col = text_input.cursor_col_row.0;
                                let ar = text_input
                                    .cursor_col_row
                                    .1
                                    .checked_sub(1)
                                    .unwrap_or_default();
                                if let Some(found) = glyphs.layout.glyphs().iter().find(|g| {
                                    (g.x / dims.a()).floor() as usize == col
                                        && (g.y / dims.b()).floor() as usize == ar
                                }) {
                                    let location = Location::new().xs(
                                        (col + 1).col().left().with((col + 1).col().right()),
                                        (ar + 1).row().top().with((ar + 1).row().bottom()),
                                    );
                                    text_input.cursor_location = found.byte_offset;
                                    text_input.cursor_col_row = (col, ar);
                                    tree.entity(handle.cursor).insert(location);
                                } else {
                                    col = col.checked_sub(1).unwrap_or_default();
                                    loop {
                                        if let Some(f) = glyphs.layout.glyphs().iter().find(|g| {
                                            (g.x / dims.a()).floor() as usize == col
                                                && (g.y / dims.b()).floor() as usize == ar
                                        }) {
                                            let col = (col + 1)
                                                .min(metrics.max_letter_idx_horizontal as usize);
                                            let location = Location::new().xs(
                                                (col + 1)
                                                    .col()
                                                    .left()
                                                    .with((col + 1).col().right()),
                                                (ar + 1).row().top().with((ar + 1).row().bottom()),
                                            );
                                            text_input.cursor_location = f.byte_offset + 1;
                                            text_input.cursor_col_row = (col, ar);
                                            tree.entity(handle.cursor).insert(location);
                                            break;
                                        }
                                        if col == 0 {
                                            let c = 0;
                                            let location = Location::new().xs(
                                                (c + 1).col().left().with((c + 1).col().right()),
                                                (ar + 1).row().top().with((ar + 1).row().bottom()),
                                            );
                                            text_input.cursor_location = 0;
                                            text_input.cursor_col_row = (c as usize, ar);
                                            tree.entity(handle.cursor).insert(location);
                                            break;
                                        }
                                        col = col.checked_sub(1).unwrap_or_default();
                                    }
                                }
                            }
                            CursorMove::Down => {
                                let mut col = text_input.cursor_col_row.0;
                                let ar = (text_input.cursor_col_row.1 + 1)
                                    .min(metrics.lines.len().checked_sub(1).unwrap_or_default());
                                if let Some(found) = glyphs.layout.glyphs().iter().find(|g| {
                                    (g.x / dims.a()).floor() as usize == col
                                        && (g.y / dims.b()).floor() as usize == ar
                                }) {
                                    let location = Location::new().xs(
                                        (col + 1).col().left().with((col + 1).col().right()),
                                        (ar + 1).row().top().with((ar + 1).row().bottom()),
                                    );
                                    text_input.cursor_location = found.byte_offset;
                                    text_input.cursor_col_row = (col, ar);
                                    tree.entity(handle.cursor).insert(location);
                                } else {
                                    col = col.checked_sub(1).unwrap_or_default();
                                    loop {
                                        if let Some(f) = glyphs.layout.glyphs().iter().find(|g| {
                                            (g.x / dims.a()).floor() as usize == col
                                                && (g.y / dims.b()).floor() as usize == ar
                                        }) {
                                            let col = (col + 1)
                                                .min(metrics.max_letter_idx_horizontal as usize);
                                            let location = Location::new().xs(
                                                (col + 1)
                                                    .col()
                                                    .left()
                                                    .with((col + 1).col().right()),
                                                (ar + 1).row().top().with((ar + 1).row().bottom()),
                                            );
                                            text_input.cursor_location = f.byte_offset + 1;
                                            text_input.cursor_col_row = (col, ar);
                                            tree.entity(handle.cursor).insert(location);
                                            break;
                                        }
                                        if col == 0 {
                                            let c = 0;
                                            let location = Location::new().xs(
                                                (c + 1).col().left().with((c + 1).col().right()),
                                                (ar + 1).row().top().with((ar + 1).row().bottom()),
                                            );
                                            text_input.cursor_location = 0;
                                            text_input.cursor_col_row = (c as usize, ar);
                                            tree.entity(handle.cursor).insert(location);
                                            break;
                                        }
                                        col = col.checked_sub(1).unwrap_or_default();
                                    }
                                }
                            }
                        };
                    }
                }
                tree.entity(handle.cursor)
                    .insert(tertiaries.get(focused).unwrap().0)
                    .insert(InteractionPropagation::grab().disable_drag());
                text_input.highlight_range = Default::default();
                for (o, e) in handle.highlights.drain() {
                    tree.remove(e);
                }
            }
        }
    }
    fn write_text(
        trigger: Trigger<Write<Text>>,
        mut tree: Tree,
        links: Query<&TextInputLink>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        mut handles: Query<&mut Handle>,
        tertiary: Query<&Tertiary>,
        line_metrics: Query<&LineMetrics>,
        mut text_inputs: Query<&mut TextInput>,
        glyphs: Query<&Glyphs>,
        layout: Res<Layout>,
    ) {
        if let Ok(link) = links.get(trigger.entity()) {
            println!("write_text");
            let font_size = font_sizes.get(link.root).unwrap().resolve(*layout);
            let dims = font.character_block(font_size.value);
            let mut handle = handles.get_mut(link.root).unwrap();
            for (o, e) in handle.highlights.iter() {
                tree.write_to(*e, Opacity::new(0.0)); // turn off highlight before remaking range
            }
            let mut text_input = text_inputs.get_mut(link.root).unwrap();
            let glyph = glyphs.get(handle.text).unwrap();
            let metrics = line_metrics.get(handle.text).unwrap();
            if let Some(found) = glyph
                .layout
                .glyphs()
                .iter()
                .find(|g| g.byte_offset == text_input.cursor_location)
            {
                let col = (found.x / dims.a()) as u32;
                let row = (found.y / dims.b()) as u32;
                let location = Location::new().xs(
                    (col + 1).col().left().with((col + 1).col().right()),
                    (row + 1).row().top().with((row + 1).row().bottom()),
                );
                text_input.cursor_col_row = (col as usize, row as usize);
                tree.entity(handle.cursor).insert(location);
            } else {
                if text_input.cursor_location == 0 {
                    let col = 0;
                    let row = 0;
                    let location = Location::new().xs(
                        (col + 1).col().left().with((col + 1).col().right()),
                        (row + 1).row().top().with((row + 1).row().bottom()),
                    );
                    text_input.cursor_col_row = (col as usize, row as usize);
                    tree.entity(handle.cursor).insert(location);
                } else {
                    let mut scan = Some(
                        text_input
                            .cursor_location
                            .checked_sub(1)
                            .unwrap_or_default(),
                    );
                    while let Some(s) = scan {
                        if let Some(found) =
                            glyph.layout.glyphs().iter().find(|g| g.byte_offset == s)
                        {
                            let col = (found.x / dims.a()) as u32;
                            let col = (col + 1).min(metrics.max_letter_idx_horizontal);
                            let row = (found.y / dims.b()) as u32;
                            let location = Location::new().xs(
                                (col + 1).col().left().with((col + 1).col().right()),
                                (row + 1).row().top().with((row + 1).row().bottom()),
                            );
                            text_input.cursor_col_row = (col as usize, row as usize);
                            tree.entity(handle.cursor).insert(location);
                            break;
                        } else {
                            if s == 0 {
                                let col = 0;
                                let row = 0;
                                let location = Location::new().xs(
                                    (col + 1).col().left().with((col + 1).col().right()),
                                    (row + 1).row().top().with((row + 1).row().bottom()),
                                );
                                text_input.cursor_col_row = (col as usize, row as usize);
                                tree.entity(handle.cursor).insert(location);
                                break;
                            }
                            scan = s.checked_sub(1);
                        }
                    }
                }
            }
            for o in text_input.highlight_range.clone() {
                if let Some(g) = glyph.layout.glyphs().iter().find(|g| g.byte_offset == o) {
                    let col = (g.x / dims.a()) as u32;
                    let row = (g.y / dims.b()) as u32;
                    let location = Location::new().xs(
                        (col + 1).col().left().with((col + 1).col().right()),
                        (row + 1).row().top().with((row + 1).row().bottom()),
                    );
                    if let Some(existing) = handle.highlights.get(&g.byte_offset) {
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
                            tertiary.get(link.root).unwrap().0,
                            FocusBehavior::ignore(),
                        ));
                        handle.highlights.insert(g.byte_offset, h);
                    }
                }
            }
        }
    }
    const HIGHLIGHT_SCROLL_THRESHOLD: f32 = 10.0;
    fn engage_cursor(
        trigger: Trigger<Engaged>,
        mut tree: Tree,
        handles: Query<&Handle>,
        links: Query<&TextInputLink>,
        primaries: Query<&Primary>,
    ) {
        if let Ok(link) = links.get(trigger.entity()) {
            println!("engage-cursor");
            let primary = primaries.get(link.root).unwrap();
            let handle = handles.get(link.root).unwrap();
            tree.entity(handle.cursor)
                .insert(primary.0)
                .insert(Opacity::new(0.5));
        }
    }
    fn highlight_range(
        trigger: Trigger<Dragged>,
        mut tree: Tree,
        current_interaction: Res<CurrentInteraction>,
        links: Query<&TextInputLink>,
        clips: Query<&ClipSection>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        sections: Query<&Section<Logical>>,
        mut handles: Query<&mut Handle>,
        line_metrics: Query<&LineMetrics>,
        glyphs: Query<&Glyphs>,
        scale_factor: Res<ScaleFactor>,
        tertiary: Query<&Tertiary>,
        mut text_inputs: Query<&mut TextInput>,
    ) {
        if let Ok(link) = links.get(trigger.entity()) {
            println!("highlight_range");
            let current = current_interaction.click().current;
            let mut text_input = text_inputs.get_mut(link.root).unwrap();
            tree.entity(link.root).insert(OverscrollPropagation(false));
            let font_size = font_sizes.get(link.root).unwrap().resolve(*layout);
            let dims = font.character_block(font_size.value);
            let section = sections.get(link.root).unwrap();
            let clip = clips.get(link.root).unwrap();
            if current.left() - clip.0.left() < Self::HIGHLIGHT_SCROLL_THRESHOLD {
                // move left
            } else if current.top() - clip.0.top() < Self::HIGHLIGHT_SCROLL_THRESHOLD {
                // move up
            } else if clip.0.bottom() - current.top() < Self::HIGHLIGHT_SCROLL_THRESHOLD {
                // move down
            } else if clip.0.right() - current.left() < Self::HIGHLIGHT_SCROLL_THRESHOLD {
                // move right
            }
            let relative = current - section.position - (4, 4).into();
            let (x, y) = (
                (relative.left().max(0.0) / dims.a()) as u32,
                (relative.top().max(0.0) / dims.b()) as u32,
            );
            let mut handle = handles.get_mut(link.root).unwrap();
            tree.entity(handle.cursor)
                .insert(InteractionPropagation::pass_through());
            let metrics = line_metrics.get(handle.text).unwrap();
            let row = y.min(metrics.lines.len().checked_sub(1).unwrap_or_default() as u32);
            let column = x
                .min(
                    metrics
                        .lines
                        .get(row as usize)
                        .and_then(|l| Some(*l))
                        .unwrap_or_default(),
                )
                .min(metrics.max_letter_idx_horizontal);
            for (o, e) in handle.highlights.iter() {
                tree.write_to(*e, Opacity::new(0.0)); // turn off highlight before remaking range
            }
            let glyph = glyphs.get(handle.text).unwrap();
            for g in glyph.layout.glyphs() {
                if (g.x / dims.a()) as u32 == column {
                    if (g.y / dims.b()) as u32 == row {
                        if text_input.cursor_location < g.byte_offset {
                            text_input.range_backwards = false;
                            text_input.highlight_range =
                                text_input.cursor_location..(g.byte_offset + 1);
                        } else {
                            text_input.range_backwards = true;
                            text_input.highlight_range =
                                g.byte_offset..(text_input.cursor_location + 1);
                        }
                    }
                }
            }
            for o in text_input.highlight_range.clone() {
                if let Some(g) = glyph.layout.glyphs().iter().find(|g| g.byte_offset == o) {
                    let col = (g.x / dims.a()) as u32;
                    let row = (g.y / dims.b()) as u32;
                    let location = Location::new().xs(
                        (col + 1).col().left().with((col + 1).col().right()),
                        (row + 1).row().top().with((row + 1).row().bottom()),
                    );
                    if let Some(existing) = handle.highlights.get(&g.byte_offset) {
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
                            tertiary.get(link.root).unwrap().0,
                            InteractionPropagation::pass_through(),
                            FocusBehavior::ignore(),
                        ));
                        handle.highlights.insert(g.byte_offset, h);
                    }
                }
            }
        }
    }
    fn place_cursor(
        trigger: Trigger<Engaged>,
        mut tree: Tree,
        current_interaction: Res<CurrentInteraction>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        sections: Query<&Section<Logical>>,
        handles: Query<&Handle>,
        line_metrics: Query<&LineMetrics>,
        glyphs: Query<&Glyphs>,
        scale_factor: Res<ScaleFactor>,
        mut text_inputs: Query<&mut TextInput>,
        tertiaries: Query<&Tertiary>,
    ) {
        let mut text_input = text_inputs.get_mut(trigger.entity()).unwrap();
        tree.entity(trigger.entity())
            .insert(OverscrollPropagation(true));
        let handle = handles.get(trigger.entity()).unwrap();
        for (o, e) in handle.highlights.iter() {
            tree.write_to(*e, Opacity::new(0.0)); // turn off highlight before remaking range
        }
        println!("place_cursor");
        let begin = current_interaction.click().start;
        let font_size = font_sizes.get(trigger.entity()).unwrap().resolve(*layout);
        let dims = font.character_block(font_size.value);
        let section = sections.get(trigger.entity()).unwrap();
        let relative = begin - section.position - (4, 4).into();
        let (x, y) = (
            (relative.left().max(0.0) / dims.a()) as u32,
            (relative.top().max(0.0) / dims.b()) as u32,
        );
        let metrics = line_metrics.get(handle.text).unwrap();
        println!(
            "metrics {:?} {}",
            metrics.lines, metrics.max_letter_idx_horizontal
        );
        let row = y.min(metrics.lines.len().checked_sub(1).unwrap_or_default() as u32);
        let column = x
            .min(
                metrics
                    .lines
                    .get(row as usize)
                    .and_then(|l| Some(l + 1))
                    .unwrap_or_default(),
            )
            .min(metrics.max_letter_idx_horizontal);
        println!("column {} row {} begin {}", column, row, begin);
        tree.entity(handle.cursor)
            .insert(Location::new().xs(
                (column + 1).col().left().with((column + 1).col().right()),
                (row + 1).row().top().with((row + 1).row().bottom()),
            ))
            .insert(Opacity::new(1.0))
            .insert(InteractionPropagation::grab().disable_drag())
            .insert(tertiaries.get(trigger.entity()).unwrap().0);
        text_input.cursor_col_row = (column as usize, row as usize);
        tree.enable(handle.cursor);
        let glyph = glyphs.get(handle.text).unwrap();
        let co = glyph
            .layout
            .glyphs()
            .last()
            .and_then(|l| Some(l.byte_offset + 1))
            .unwrap_or_default();
        text_input.cursor_location = co;
        text_input.highlight_range = co..co;
        for g in glyph.layout.glyphs() {
            if (g.x / dims.a()) as u32 == column {
                if (g.y / dims.b()) as u32 == row {
                    println!("cursor on {}", g.byte_offset);
                    text_input.cursor_location = g.byte_offset;
                    text_input.highlight_range = g.byte_offset..g.byte_offset;
                }
            }
        }
    }
    fn clear_cursor(
        trigger: Trigger<Unfocused>,
        mut tree: Tree,
        mut handles: Query<&mut Handle>,
        mut text_inputs: Query<&mut TextInput>,
    ) {
        println!("clear_cursor");
        let this = trigger.entity();
        let mut handle = handles.get_mut(this).unwrap();
        tree.entity(handle.cursor)
            .insert(Opacity::new(0.0))
            .insert(InteractionPropagation::pass_through());
        tree.disable(handle.cursor);
        tree.entity(this).insert(OverscrollPropagation(true));
        let mut text_input = text_inputs.get_mut(this).unwrap();
        text_input.cursor_col_row = (0, 0);
        text_input.highlight_range = Default::default();
        for (o, e) in handle.highlights.drain() {
            tree.remove(e);
        }
    }
}
impl Attachment for TextInput {
    fn attach(foliage: &mut Foliage) {
        foliage.world.insert_resource(KeyBindings::default());
        foliage.define(Self::typing);
        foliage.define(Self::handle_trigger);
    }
}
#[derive(Component, Clone)]
#[component(on_replace = handle_replace::<TextInput>)]
pub struct Handle {
    pub panel: Entity,
    pub text: Entity,
    pub cursor: Entity,
    pub highlights: HashMap<GlyphOffset, Entity>,
}
#[derive(Component, Copy, Clone)]
pub(crate) struct TextInputLink {
    pub(crate) root: Entity,
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
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TextInputAction {
    Enter,
    Backspace,
    Delete,
    End,
    Home,
    Copy,
    Paste,
    SelectAll,
    ExtendLeft,
}
#[derive(Resource)]
pub struct KeyBindings {
    pub bindings: HashMap<InputSequence, TextInputAction>,
}
impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            bindings: {
                let mut map = HashMap::new();
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::Enter), ModifiersState::default()),
                    TextInputAction::Enter,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::Backspace), ModifiersState::default()),
                    TextInputAction::Backspace,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::Delete), ModifiersState::default()),
                    TextInputAction::Delete,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::End), ModifiersState::default()),
                    TextInputAction::End,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::Home), ModifiersState::default()),
                    TextInputAction::Home,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::Copy), ModifiersState::default()),
                    TextInputAction::Copy,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::Paste), ModifiersState::default()),
                    TextInputAction::Paste,
                );
                map.insert(
                    InputSequence::new(Key::Character(SmolStr::new("c")), ModifiersState::CONTROL),
                    TextInputAction::Copy,
                );
                map.insert(
                    InputSequence::new(Key::Character(SmolStr::new("v")), ModifiersState::CONTROL),
                    TextInputAction::Paste,
                );
                map.insert(
                    InputSequence::new(Key::Character(SmolStr::new("a")), ModifiersState::CONTROL),
                    TextInputAction::SelectAll,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::ArrowLeft), ModifiersState::SHIFT),
                    TextInputAction::ExtendLeft,
                );
                map
            },
        }
    }
}
impl KeyBindings {
    pub fn action(&self, i: &InputSequence) -> Option<TextInputAction> {
        self.bindings.iter().find_map(|(s, a)| {
            if i.key == s.key && i.mods.contains(s.mods) {
                Some(*a)
            } else {
                None
            }
        })
    }
}
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum CursorMove {
    Left,
    Right,
    Up,
    Down,
}
