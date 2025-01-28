use crate::composite::handle_replace;
use crate::ginkgo::ScaleFactor;
use crate::text::monospaced::MonospacedFont;
use crate::text::{Glyphs, LineMetrics};
use crate::{
    Attachment, Component, Composite, Dragged, EcsExtension, Elevation, Engaged, Event, Foliage,
    FontSize, GlyphOffset, Grid, GridExt, InteractionListener, Layout, Location, Logical, Opacity,
    Panel, Primary, Secondary, Section, Stem, Tertiary, Text, TextValue, Tree, Unfocused, Update,
    Write,
};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::prelude::{Res, Trigger};
use bevy_ecs::system::Query;
use bevy_ecs::world::{DeferredWorld, OnInsert};
use std::collections::HashMap;
use std::ops::Range;

#[derive(Component, Clone)]
pub struct TextInput {
    pub(crate) highlight_range: Range<GlyphOffset>,
    pub(crate) cursor_location: GlyphOffset,
}
impl TextInput {
    pub fn new() -> Self {
        Self {
            highlight_range: Default::default(),
            cursor_location: 0,
        }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let panel = world.commands().leaf((
            Location::new().xs(
                0.pct().left().adjust(4).with(100.pct().right().adjust(-4)),
                0.pct().top().adjust(4).with(100.pct().bottom().adjust(-4)),
            ),
            Grid::new(1.letters(), 1.letters()),
            Elevation::up(0),
            Stem::some(this),
        ));
        let cursor = world.commands().leaf((
            Panel::new(),
            Elevation::up(1),
            Stem::some(panel),
            Opacity::new(0.0),
            Location::new().xs(
                1.col().left().with(1.col().right()),
                1.col().top().with(1.col().bottom()),
            ),
        ));
        let text = world.commands().leaf((
            Stem::some(panel),
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ),
            Elevation::up(2),
        ));
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
            .insert(InteractionListener::new().scroll(true));
    }
    fn handle_trigger(trigger: Trigger<OnInsert, Handle>, mut tree: Tree) {
        let this = trigger.entity();
        tree.trigger_targets(Update::<TextValue>::new(), this);
        tree.trigger_targets(Update::<Primary>::new(), this);
        tree.trigger_targets(Update::<Secondary>::new(), this);
        tree.trigger_targets(Update::<Tertiary>::new(), this);
    }
    fn forward_text_value(trigger: Trigger<OnInsert, TextValue>, mut tree: Tree) {}
    fn update_text_value(trigger: Trigger<Update<TextValue>>, mut tree: Tree) {
        // give to handle.text
        // clear highlighting as they are invalid offsets now text has changed
        // regular editing will be consistent and adapt as needed
        // hopefully only need to purge highlighting on hard change
        // but can always just re-highlight text (not user-friendly tho)
    }
    fn forward_font_size(trigger: Trigger<OnInsert, FontSize>, mut tree: Tree) {}
    fn update_font_size(trigger: Trigger<Update<FontSize>>, mut tree: Tree) {
        // give to handle.text
    }
    fn update_primary(trigger: Trigger<Update<Primary>>, mut tree: Tree) {
        // text-color
    }
    fn forward_primary(trigger: Trigger<OnInsert, Primary>, mut tree: Tree) {}
    fn update_secondary(trigger: Trigger<Update<Secondary>>, mut tree: Tree) {
        // background (panel)
    }
    fn forward_secondary(trigger: Trigger<OnInsert, Secondary>, mut tree: Tree) {}
    fn update_tertiary(trigger: Trigger<Update<Tertiary>>, mut tree: Tree) {
        // cursor color + highlight color
    }
    fn forward_tertiary(trigger: Trigger<OnInsert, Tertiary>, mut tree: Tree) {}
    fn write_text(trigger: Trigger<Write<Text>>, mut tree: Tree) {
        // reconfigure highlights from where offsets currently are (glyph iter w/ col + row derive)
    }
    fn highlight_range(
        trigger: Trigger<Dragged>,
        mut tree: Tree,
        listeners: Query<&InteractionListener>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        sections: Query<&Section<Logical>>,
        handles: Query<&Handle>,
        line_metrics: Query<&LineMetrics>,
        glyphs: Query<&Glyphs>,
        scale_factor: Res<ScaleFactor>,
        tertiary: Query<&Tertiary>,
        mut text_inputs: Query<&mut TextInput>,
    ) {
        let current = listeners.get(trigger.entity()).unwrap().click.current;
        let font_size = font_sizes.get(trigger.entity()).unwrap().resolve(*layout);
        let dims = font.character_block(font_size.value);
        let section = sections.get(trigger.entity()).unwrap();
        let relative = current - section.position - (4, 4).into();
        let (x, y) = (
            (relative.left().max(0.0) / dims.a()) as u32,
            (relative.top().max(0.0) / dims.b()) as u32,
        );
        let handle = handles.get(trigger.entity()).unwrap();
        let metrics = line_metrics.get(handle.text).unwrap();
        let row = y.min(metrics.lines.len().checked_sub(1).unwrap_or_default() as u32);
        let column = x
            .min(*metrics.lines.get(row as usize).unwrap_or(&0))
            .min(metrics.max_letter_idx_horizontal);
        let mut text_input = text_inputs.get_mut(trigger.entity()).unwrap();
        for (o, e) in handle.highlights.iter() {
            tree.write_to(*e, Opacity::new(0.0)); // turn off highlight before remaking range
        }
        let glyph = glyphs.get(handle.text).unwrap();
        for g in glyph.layout.glyphs() {
            if (g.x / dims.a()) as u32 == column {
                if (g.y / dims.b()) as u32 == row {
                    if text_input.cursor_location < g.byte_offset {
                        text_input.highlight_range = text_input.cursor_location..g.byte_offset;
                    } else {
                        text_input.highlight_range = g.byte_offset..text_input.cursor_location;
                    }
                }
            }
        }
        for o in text_input.highlight_range.clone() {
            if let Some(g) = glyph.layout.glyphs().iter().find(|g| g.byte_offset == o) {
                let col = (g.x / dims.a()) as u32;
                let row = (g.y / dims.b()) as u32;
                // TODO panel creation is if not have in handle.highlights (growth only until un-focus w/ opacity cull above)
                // TODO if created => tertiary color give (not existent when forwarding value)
                // get panel entity => then
                // highlight location with col / row
                // opacity -> 0.5 (turn-on)
            }
        }
    }
    fn place_cursor(
        trigger: Trigger<Engaged>,
        mut tree: Tree,
        listeners: Query<&InteractionListener>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
        layout: Res<Layout>,
        sections: Query<&Section<Logical>>,
        handles: Query<&Handle>,
        line_metrics: Query<&LineMetrics>,
        glyphs: Query<&Glyphs>,
        scale_factor: Res<ScaleFactor>,
        mut text_inputs: Query<&mut TextInput>,
    ) {
        let begin = listeners.get(trigger.entity()).unwrap().click.start;
        let font_size = font_sizes.get(trigger.entity()).unwrap().resolve(*layout);
        let dims = font.character_block(font_size.value);
        let section = sections.get(trigger.entity()).unwrap();
        let relative = begin - section.position - (4, 4).into();
        let (x, y) = (
            (relative.left().max(0.0) / dims.a()) as u32,
            (relative.top().max(0.0) / dims.b()) as u32,
        );
        let handle = handles.get(trigger.entity()).unwrap();
        let metrics = line_metrics.get(handle.text).unwrap();
        let row = y.min(metrics.lines.len().checked_sub(1).unwrap_or_default() as u32);
        let column = x
            .min(*metrics.lines.get(row as usize).unwrap_or(&0))
            .min(metrics.max_letter_idx_horizontal);
        tree.entity(handle.cursor)
            .insert(Location::new().xs(
                (column + 1).col().left().with((column + 1).col().right()),
                (row + 1).row().top().with((row + 1).row().bottom()),
            ))
            .insert(Opacity::new(0.5));
        for g in glyphs.get(handle.text).unwrap().layout.glyphs() {
            if (g.x / dims.a()) as u32 == column {
                if (g.y / dims.b()) as u32 == row {
                    let mut text_input = text_inputs.get_mut(trigger.entity()).unwrap();
                    text_input.cursor_location = g.byte_offset;
                    text_input.highlight_range = g.byte_offset..g.byte_offset;
                }
            }
        }
    }
    fn clear_cursor(trigger: Trigger<Unfocused>, mut tree: Tree) {
        // cursor => opacity 0.0
        // remove highlights
    }
}
impl Attachment for TextInput {
    fn attach(foliage: &mut Foliage) {
        todo!()
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
        targets
    }
}
#[derive(Component, Clone)]
pub struct HintText(pub(crate) String);
impl HintText {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct ConfigurePanels {}
#[derive(Event, Copy, Clone)]
pub(crate) struct Highlight(pub(crate) GlyphOffset);
#[derive(Event, Copy, Clone)]
pub(crate) struct UnHighlight(pub(crate) GlyphOffset);
