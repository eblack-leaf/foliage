use crate::composite::handle_replace;
use crate::text::Glyphs;
use crate::{
    Attachment, Component, Composite, Dragged, EcsExtension, Elevation, Engaged, Event, Foliage,
    GlyphOffset, Grid, GridExt, Location, Opacity, Panel, Primary, Stem, Text, Tree, Write,
};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;
use std::collections::HashMap;

#[derive(Component, Clone)]
pub struct TextInput {}
impl TextInput {
    pub fn new() -> Self {
        Self {}
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
        let text = world.commands().leaf((
            Stem::some(panel),
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ),
            Elevation::up(1),
        ));
        let handle = Handle {
            panel,
            text,
            panels: Default::default(),
        };
        world.commands().entity(this).insert(handle);
    }
    fn highlight(trigger: Trigger<Dragged>, mut tree: Tree) {
        // highlight range (could be one) of span (include rows if covered)
        // using interaction.click.begin / current
        // set cursor state (one == add-char | two+ == replace-range)
    }
    fn place_cursor(trigger: Trigger<Engaged>, mut tree: Tree) {
        // highlight panel at interaction.click.begin => letter
        // set cursor location
    }
    fn on_write(trigger: Trigger<Write<Text>>, mut tree: Tree) {
        // glyphs changed => reconfigure + add / remove panels (use last???)
    }
    fn configure_panels(
        trigger: Trigger<ConfigurePanels>,
        mut tree: Tree,
        glyphs: Query<&Glyphs>,
        handles: Query<&Handle>,
    ) {
        let this = trigger.entity();
        let handle = handles.get(this).unwrap();
        for g in glyphs.get(this).unwrap().glyphs.iter() {
            let location = Location::new(); // TODO g.section => letter-slot l.col().left().with(l.col().right()) ...
            tree.entity(*handle.panels.get(&g.offset).unwrap())
                .insert(location);
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
}
#[derive(Event, Copy, Clone)]
pub(crate) struct RemovePanel {
    pub(crate) offset: GlyphOffset,
}
#[derive(Event, Copy, Clone)]
pub(crate) struct ConfigurePanels {}
