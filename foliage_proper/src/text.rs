use crate::{Attachment, Foliage, Location, Tree, Update, Write};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, Trigger};
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;
impl Attachment for Text {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Text::update);
    }
}
#[derive(Component, Clone, PartialEq, Default)]
#[require(FontSize, UpdateCache)]
#[component(on_add = Text::on_add)]
#[component(on_insert = Text::on_insert)]
pub struct Text {
    pub value: String,
}
impl Text {
    pub fn new<S: AsRef<str>>(value: S) -> Self {
        Self {
            value: value.as_ref().to_string(),
        }
    }
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.commands().entity(this).observe(Self::observe_location);
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.commands().trigger_targets(Update::<Text>::new(), this);
    }
    fn observe_location(trigger: Trigger<Write<Location>>, mut tree: Tree) {
        // trigger update
        tree.trigger_targets(Update::<Text>::new(), trigger.entity());
    }
    fn update(trigger: Trigger<Update<Text>>, mut tree: Tree, mut cache: Query<&mut UpdateCache>, locations: Query<&Location>) {
        // if config != current (made from current values) => process + set config
        // glyphs and such
    }
}
#[derive(Component, Clone, Copy, PartialEq)]
pub struct FontSize {
    pub value: u32,
}
impl FontSize {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.commands().trigger_targets(Update::<Text>::new(), this);
    }
}
impl Default for FontSize {
    fn default() -> Self {
        Self { value: 14 }
    }
}
#[derive(Component, Clone, PartialEq, Default)]
pub(crate) struct UpdateCache {
    pub(crate) font_size: FontSize,
    pub(crate) text: Text,
    pub(crate) location: Location,
}
