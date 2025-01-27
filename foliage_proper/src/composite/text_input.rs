use crate::composite::handle_replace;
use crate::text::Glyphs;
use crate::{
    Attachment, Component, Composite, Dragged, EcsExtension, Elevation, Engaged, Event, Foliage,
    GlyphOffset, Grid, GridExt, InteractionListener, Location, Opacity, Panel, Primary, Secondary,
    Stem, Text, TextValue, Tree, Unfocused, Update, Write,
};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::Query;
use bevy_ecs::world::{DeferredWorld, OnInsert};
use std::collections::HashMap;
use std::ops::Range;

#[derive(Component, Clone)]
pub struct TextInput {
    pub(crate) highlight_range: Range<GlyphOffset>,
}
impl TextInput {
    pub fn new() -> Self {
        Self {
            highlight_range: Default::default(),
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
            panels: Default::default(),
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
    }
    fn forward_text_value(trigger: Trigger<OnInsert, TextValue>, mut tree: Tree) {}
    fn update_text_value(trigger: Trigger<Update<TextValue>>, mut tree: Tree) {
        // give to handle.text
    }
    fn update_primary(trigger: Trigger<Update<Primary>>, mut tree: Tree) {}
    fn forward_primary(trigger: Trigger<OnInsert, Primary>, mut tree: Tree) {}
    fn update_secondary(trigger: Trigger<Update<Secondary>>, mut tree: Tree) {}
    fn forward_secondary(trigger: Trigger<OnInsert, Secondary>, mut tree: Tree) {}
    fn highlight(trigger: Trigger<Dragged>, mut tree: Tree) {
        // highlight range (could be one) of span (include rows if covered)
        // using interaction.click.begin / current
        // set cursor state (one == add-char | two+ == replace-range)
    }
    fn place_cursor(trigger: Trigger<Engaged>, mut tree: Tree) {
        // highlight panel at interaction.click.begin => letter
        // set cursor location
    }
    fn unfocused(trigger: Trigger<Unfocused>, mut tree: Tree) {
        // cursor => opacity 0.0
    }
    fn on_write(trigger: Trigger<Write<Text>>, mut tree: Tree) {
        tree.trigger_targets(ConfigurePanels {}, trigger.entity());
    }
    fn configure_panels(
        trigger: Trigger<ConfigurePanels>,
        mut tree: Tree,
        glyphs: Query<&Glyphs>,
        handles: Query<&Handle>,
    ) {
        let this = trigger.entity();
        let handle = handles.get(this).unwrap();
        let text_glyphs = glyphs.get(this).unwrap();
        for g in text_glyphs.glyphs.iter() {
            let location = Location::new(); // TODO g.section => letter-slot l.col().left().with(l.col().right()) ...
            if let Some(p) = handle.panels.get(&g.offset) {
                tree.entity(*p).insert(location);
            } else {
                tree.trigger_targets(
                    AddPanel {
                        offset: g.offset,
                        location,
                    },
                    this,
                );
            }
        }
        for g in text_glyphs
            .glyphs
            .iter()
            .take(text_glyphs.glyphs.len())
            .skip(handle.panels.len())
        {
            tree.trigger_targets(RemovePanel { offset: g.offset }, this);
        }
    }
    fn add_panel(
        trigger: Trigger<AddPanel>,
        mut tree: Tree,
        mut handles: Query<&mut Handle>,
        primaries: Query<&Primary>,
    ) {
        let this = trigger.entity();
        let mut handle = handles.get_mut(this).unwrap();
        if handle.panels.contains_key(&trigger.offset) {
            return;
        }
        let panel = tree.leaf((
            Panel::new(),
            Stem::some(handle.panel),
            *primaries.get(this).unwrap(),
            Opacity::new(0.0),
            Elevation::up(1),
        ));
        handle.panels.insert(trigger.offset, panel);
    }
    fn remove_panel(
        trigger: Trigger<RemovePanel>,
        mut tree: Tree,
        mut handles: Query<&mut Handle>,
    ) {
        let this = trigger.entity();
        let mut handle = handles.get_mut(this).unwrap();
        if let Some(e) = handle.panels.remove(&trigger.offset) {
            tree.remove(e);
        }
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
    pub panels: HashMap<GlyphOffset, Entity>,
}
impl Composite for TextInput {
    type Handle = Handle;
    fn remove(handle: &Self::Handle) -> impl TriggerTargets + Send + Sync + 'static {
        let mut targets = handle.panels.iter().map(|(_, e)| *e).collect::<Vec<_>>();
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
pub(crate) struct AddPanel {
    pub(crate) offset: GlyphOffset,
    pub(crate) location: Location,
}
#[derive(Event, Copy, Clone)]
pub(crate) struct RemovePanel {
    pub(crate) offset: GlyphOffset,
}
#[derive(Event, Copy, Clone)]
pub(crate) struct ConfigurePanels {}
