use crate::color::Color;
use crate::remove::Remove;
use crate::{Attachment, Foliage, Location, Opacity, Tree, Update, Write};
use crate::{Layer, RenderTokenCache};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, Trigger};
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;
impl Attachment for Text {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Text::update);
        foliage.remove_queue::<Text>();
        foliage.differential::<Text, FontSize>();
        foliage.differential::<Text, Color>();
        foliage.differential::<Text, Opacity>();
        foliage.differential::<Text, Location>();
        foliage.differential::<Text, Layer>();
    }
}
#[derive(Component, Clone, PartialEq, Default)]
#[require(FontSize)]
#[require(UpdateCache)]
#[require(RenderTokenCache<Text, Opacity>)]
#[require(RenderTokenCache<Text, Color>)]
#[require(RenderTokenCache<Text, Location>)]
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
        world
            .commands()
            .entity(this)
            .observe(Remove::token_push::<Text>);
        world
            .commands()
            .entity(this)
            .observe(Self::update_from_location);
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .trigger_targets(Update::<Text>::new(), this);
    }
    fn update_from_location(trigger: Trigger<Write<Location>>, mut tree: Tree) {
        tree.trigger_targets(Update::<Text>::new(), trigger.entity());
    }
    fn update(
        trigger: Trigger<Update<Text>>,
        mut tree: Tree,
        texts: Query<&Text>,
        font_sizes: Query<&FontSize>,
        mut locations: Query<&mut Location>,
        mut cache: Query<&mut UpdateCache>,
    ) {
        // if config != current (made from current values) => process + set config
        // glyphs and such
    }
}
#[derive(Component, Clone, Copy, PartialEq)]
#[require(RenderTokenCache<Text, FontSize>)]
#[component(on_insert = FontSize::on_insert)]
pub struct FontSize {
    pub value: u32,
}
impl FontSize {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .trigger_targets(Update::<Text>::new(), this);
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
