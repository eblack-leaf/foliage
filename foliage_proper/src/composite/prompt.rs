use crate::composite::text_input::{HintColor, InputCommit};
use crate::handle_replace;
use crate::{
    Attachment, Component, Composite, EcsExtension, Elevation, Foliage, Grid, GridExt, Icon,
    IconValue, Location, Outline, Panel, Primary, Rounding, Secondary, Stem, Tertiary, TextInput,
    Tree, Update,
};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::prelude::Trigger;
use bevy_ecs::world::{DeferredWorld, OnInsert};

#[derive(Component)]
#[component(on_insert = Self::on_insert)]
pub struct Prompt {}
impl Prompt {
    pub fn new() -> Prompt {
        Prompt {}
    }
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .entity(this)
            .observe(Self::forward_primary)
            .observe(Self::update_primary)
            .observe(Self::forward_secondary)
            .observe(Self::update_secondary)
            .observe(Self::update_tertiary)
            .observe(Self::forward_tertiary);
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let panel = world.commands().leaf((
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ),
            Panel::new(),
            Rounding::Md,
            Outline::new(3),
            Elevation::abs(90),
            Stem::some(this),
            Grid::default(),
        ));
        let icon_value = *world.get::<IconValue>(this).unwrap();
        let icon = world.commands().leaf((
            Icon::new(icon_value.0),
            Elevation::up(1),
            Location::new().xs(
                8.px().left().with(24.px().width()),
                50.pct().center_y().with(24.px().height()),
            ),
            Stem::some(panel),
        ));
        let input = world.commands().leaf((
            TextInput::new(),
            Location::new().xs(
                40.px().left().with(100.pct().right().adjust(-8)),
                50.pct().center_y().with(1.letters().height().adjust(8)),
            ),
            Stem::some(panel),
            Elevation::up(1),
        ));
        world.commands().subscribe(input, Self::listen_input);
        let handle = Handle { panel, icon, input };
        world.commands().entity(this).insert(handle);
    }
    fn listen_input(trigger: Trigger<InputCommit>, mut tree: Tree) {
        // up-down suggestions + Tab to commit suggestion + Enter to Trigger "Submit" Event
    }
    fn handle_trigger(trigger: Trigger<OnInsert, Handle>, mut tree: Tree) {
        let this = trigger.entity();
        tree.trigger_targets(Update::<Primary>::new(), this);
        tree.trigger_targets(Update::<Secondary>::new(), this);
        tree.trigger_targets(Update::<Tertiary>::new(), this);
        tree.trigger_targets(Update::<HintColor>::new(), this);
        tree.trigger_targets(Update::<IconValue>::new(), this);
    }
    fn forward_primary(trigger: Trigger<OnInsert, Primary>) {}
    fn update_primary(trigger: Trigger<Update<Primary>>) {}
    fn forward_secondary(trigger: Trigger<OnInsert, Secondary>) {}
    fn update_secondary(trigger: Trigger<Update<Secondary>>) {}
    fn forward_tertiary(trigger: Trigger<OnInsert, Tertiary>) {}
    fn update_tertiary(trigger: Trigger<Update<Tertiary>>) {}
    fn forward_icon(trigger: Trigger<OnInsert, IconValue>) {}
    fn update_icon(trigger: Trigger<Update<IconValue>>) {}
    fn forward_hint(trigger: Trigger<OnInsert, HintColor>) {}
    fn update_hint(trigger: Trigger<Update<HintColor>>) {}
}
impl Attachment for Prompt {
    fn attach(foliage: &mut Foliage) {
        todo!()
    }
}
#[derive(Component, Clone, Default)]
pub(crate) struct CurrentlyTyped {
    pub(crate) value: String,
}
#[derive(Component, Clone, Default)]
pub struct Suggestion {
    pub value: String,
}
#[derive(Component, Copy, Clone, Default)]
pub enum SuggestionDirection {
    #[default]
    Up,
    Down,
}
#[derive(Component, Clone)]
#[component(on_replace = handle_replace::<Prompt>)]
pub struct Handle {
    pub panel: Entity,
    pub icon: Entity,
    pub input: Entity,
}
impl Composite for Prompt {
    type Handle = Handle;
    fn remove(handle: &Self::Handle) -> impl TriggerTargets + Send + Sync + 'static {
        [handle.panel, handle.input, handle.icon]
    }
}
