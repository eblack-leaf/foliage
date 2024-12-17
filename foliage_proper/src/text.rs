use crate::color::Color;
use crate::remove::Remove;
use crate::{
    Attachment, Foliage, Location, Opacity, RenderQueue, RenderRemoveQueue, RenderToken, Tree,
    Update, Write,
};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, Trigger};
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;

impl Attachment for Text {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Text::update);
        foliage
            .world
            .insert_resource(RenderRemoveQueue::<Text>::new());
        foliage
            .world
            .insert_resource(RenderQueue::<Text, FontSize>::new());
        foliage.define(RenderQueue::<Text, FontSize>::token_fetch);
        foliage
            .world
            .insert_resource(RenderQueue::<Text, Color>::new());
        foliage.define(RenderQueue::<Text, Color>::token_fetch);
        foliage
            .world
            .insert_resource(RenderQueue::<Text, Location>::new());
        foliage.define(RenderQueue::<Text, Location>::token_fetch);
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
        world
            .commands()
            .entity(this)
            .observe(Opacity::token_push::<Text>);
        world
            .commands()
            .entity(this)
            .observe(Color::token_push::<Text>);
        world
            .commands()
            .entity(this)
            .observe(Remove::token_queue::<Text>);
        world
            .commands()
            .entity(this)
            .observe(Location::token_push::<Text>);
        world
            .commands()
            .entity(this)
            .observe(Self::update_from_location);
    }
    fn update_from_location(trigger: Trigger<Write<Location>>, mut tree: Tree) {
        tree.trigger_targets(Update::<Text>::new(), trigger.entity());
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .trigger_targets(Update::<Text>::new(), this);
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
        let font_size = *world.get::<FontSize>(this).unwrap();
        world
            .commands()
            .trigger_targets(RenderToken::<Text, _>::new(font_size), this);
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
