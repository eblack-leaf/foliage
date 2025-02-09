use crate::{
    handle_replace, stack, Attachment, Disengaged, EcsExtension, Elevation, Engaged, FocusBehavior,
    Foliage, FontSize, Grid, GridExt, HorizontalAlignment, Icon, IconValue, InteractionListener,
    InteractionPropagation, Location, Outline, Panel, Primary, Rounding, Secondary, Stack, Stem,
    Text, TextValue, Tree, Update, VerticalAlignment, Visibility,
};
use crate::{Component, Composite};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::Query;
use bevy_ecs::world::{DeferredWorld, OnInsert};

#[derive(Component, Clone)]
#[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
#[require(Rounding, FontSize, IconValue, Outline, Primary, Secondary)]
pub struct Button {}
impl Attachment for Button {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Button::handle_trigger);
    }
}
impl Button {
    pub fn new() -> Self {
        Self {}
    }
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .entity(this)
            .observe(Self::engaged)
            .observe(Self::disengaged)
            .observe(Self::forward_text)
            .observe(Self::update_text)
            .observe(Self::forward_font_size)
            .observe(Self::update_font_size)
            .observe(Self::forward_icon)
            .observe(Self::update_icon)
            .observe(Self::forward_outline)
            .observe(Self::update_outline)
            .observe(Self::forward_rounding)
            .observe(Self::update_rounding)
            .observe(Self::forward_primary)
            .observe(Self::update_primary)
            .observe(Self::forward_secondary)
            .observe(Self::update_secondary);
    }
    fn handle_trigger(trigger: Trigger<OnInsert, Handle>, mut tree: Tree) {
        // trigger all
        let this = trigger.entity();
        tree.trigger_targets(Update::<TextValue>::new(), this);
        tree.trigger_targets(Update::<FontSize>::new(), this);
        tree.trigger_targets(Update::<IconValue>::new(), this);
        tree.trigger_targets(Update::<Outline>::new(), this);
        tree.trigger_targets(Update::<Rounding>::new(), this);
        tree.trigger_targets(Update::<Primary>::new(), this);
        tree.trigger_targets(Update::<Secondary>::new(), this);
        tree.trigger_targets(Update::<Secondary>::new(), this);
    }

    fn update_text(
        trigger: Trigger<Update<TextValue>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        values: Query<&TextValue>,
    ) {
        let this = trigger.entity();
        let handle = handles.get(this).unwrap();
        if let Some(value) = values.get(this).ok() {
            tree.entity(handle.text)
                .insert(Text::new(value.0.as_str()))
                .insert(
                    Location::new().xs(
                        50.pct()
                            .center_x()
                            .adjust(20)
                            .with(value.0.len().letters().width()),
                        1.row().top().with(1.row().bottom()),
                    ),
                );
        }
    }
    fn forward_text(trigger: Trigger<OnInsert, TextValue>, mut tree: Tree) {
        tree.trigger_targets(Update::<TextValue>::new(), trigger.entity());
    }
    fn update_font_size(
        trigger: Trigger<Update<FontSize>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        values: Query<&FontSize>,
    ) {
        let this = trigger.entity();
        let handle = handles.get(this).unwrap();
        let value = values.get(this).unwrap();
        tree.entity(handle.text).insert(*value);
    }
    fn forward_font_size(trigger: Trigger<OnInsert, FontSize>, mut tree: Tree) {
        tree.trigger_targets(Update::<FontSize>::new(), trigger.entity());
    }
    fn update_icon(
        trigger: Trigger<Update<IconValue>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        values: Query<&IconValue>,
    ) {
        let this = trigger.entity();
        let handle = handles.get(this).unwrap();
        let value = values.get(this).unwrap();
        tree.entity(handle.icon).insert(Icon::new(value.0));
    }
    fn forward_icon(trigger: Trigger<OnInsert, IconValue>, mut tree: Tree) {
        tree.trigger_targets(Update::<IconValue>::new(), trigger.entity());
    }
    fn update_outline(
        trigger: Trigger<Update<Outline>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        primaries: Query<&Primary>,
        secondaries: Query<&Secondary>,
        outlines: Query<&Outline>,
    ) {
        let this = trigger.entity();
        let handle = handles.get(this).unwrap();
        let outline = outlines.get(this).unwrap();
        let primary = primaries.get(this).unwrap();
        let secondary = secondaries.get(this).unwrap();
        let color = if outline == &Outline::default() {
            secondary.0
        } else {
            primary.0
        };
        tree.entity(handle.panel).insert(color).insert(*outline);
    }
    fn forward_outline(trigger: Trigger<OnInsert, Outline>, mut tree: Tree) {
        tree.trigger_targets(Update::<Outline>::new(), trigger.entity());
    }
    fn update_primary(
        trigger: Trigger<Update<Primary>>,
        handles: Query<&Handle>,
        mut tree: Tree,
        primaries: Query<&Primary>,
        outlines: Query<&Outline>,
    ) {
        let this = trigger.entity();
        let handle = handles.get(this).unwrap();
        let primary = primaries.get(this).unwrap();
        let outline = outlines.get(this).unwrap();
        tree.entity(handle.icon).insert(primary.0);
        tree.entity(handle.text).insert(primary.0);
        if outline != &Outline::default() {
            tree.entity(handle.panel).insert(primary.0);
        }
    }
    fn forward_primary(
        trigger: Trigger<OnInsert, Primary>,
        handles: Query<&Handle>,
        mut tree: Tree,
        primaries: Query<&Primary>,
        outlines: Query<&Outline>,
    ) {
        tree.trigger_targets(Update::<Primary>::new(), trigger.entity());
    }
    fn update_secondary(
        trigger: Trigger<OnInsert, Secondary>,
        handles: Query<&Handle>,
        mut tree: Tree,
        secondaries: Query<&Secondary>,
        outlines: Query<&Outline>,
    ) {
        let this = trigger.entity();
        let handle = handles.get(this).unwrap();
        let outline = outlines.get(this).unwrap();
        let secondary = secondaries.get(this).unwrap();
        if outline == &Outline::default() {
            tree.entity(handle.panel).insert(secondary.0);
        }
    }
    fn forward_secondary(trigger: Trigger<OnInsert, Secondary>, mut tree: Tree) {
        tree.trigger_targets(Update::<Secondary>::new(), trigger.entity());
    }
    fn engaged(
        trigger: Trigger<Engaged>,
        primaries: Query<&Primary>,
        secondaries: Query<&Secondary>,
        outlines: Query<&Outline>,
        handles: Query<&Handle>,
        mut tree: Tree,
    ) {
        let this = trigger.entity();
        let handle = handles.get(this).unwrap();
        let outline = outlines.get(this).unwrap();
        let secondary = secondaries.get(this).unwrap();
        let primary = primaries.get(this).unwrap();
        if outline == &Outline::default() {
            tree.entity(handle.panel).insert(primary.0);
            tree.entity(handle.icon).insert(secondary.0);
            tree.entity(handle.text).insert(secondary.0);
        } else {
            tree.entity(handle.panel).insert(Outline::default());
            tree.entity(handle.icon).insert(secondary.0);
            tree.entity(handle.text).insert(secondary.0);
        }
    }
    fn disengaged(
        trigger: Trigger<Disengaged>,
        primaries: Query<&Primary>,
        secondaries: Query<&Secondary>,
        outlines: Query<&Outline>,
        handles: Query<&Handle>,
        mut tree: Tree,
    ) {
        let this = trigger.entity();
        let handle = handles.get(this).unwrap();
        let outline = outlines.get(this).unwrap();
        let secondary = secondaries.get(this).unwrap();
        let primary = primaries.get(this).unwrap();
        if outline == &Outline::default() {
            tree.entity(handle.panel).insert(secondary.0);
            tree.entity(handle.icon).insert(primary.0);
            tree.entity(handle.text).insert(primary.0);
        } else {
            tree.entity(handle.panel).insert(*outline);
            tree.entity(handle.icon).insert(primary.0);
            tree.entity(handle.text).insert(primary.0);
        }
    }
    fn update_rounding(
        trigger: Trigger<Update<Rounding>>,
        roundings: Query<&Rounding>,
        handles: Query<&Handle>,
        mut tree: Tree,
    ) {
        let this = trigger.entity();
        let round = roundings.get(this).unwrap();
        tree.entity(this).insert(InteractionListener::new());
        let handle = handles.get(this).unwrap();
        let icon_location = match round {
            Rounding::Full => Location::new().xs(
                50.pct().center_x().with(24.px().width()),
                50.pct().center_y().with(24.px().height()),
            ),
            _ => Location::new().xs(
                stack().left().right().adjust(-8).with(24.px().width()),
                50.pct().center_y().with(24.px().height()),
            ),
        };
        tree.entity(handle.icon).insert(icon_location);
        match round {
            Rounding::Full => {
                tree.entity(handle.panel).insert(Rounding::Full);
                tree.entity(handle.text).insert(Visibility::new(false));
                tree.entity(handle.icon).insert(Stack::default());
            }
            _ => {
                tree.entity(handle.panel).insert(Rounding::Sm);
                tree.entity(handle.text).insert(Visibility::new(true));
                tree.entity(handle.icon).insert(Stack::new(handle.text));
            }
        }
    }
    fn forward_rounding(trigger: Trigger<OnInsert, Rounding>, mut tree: Tree) {
        tree.trigger_targets(Update::<Rounding>::new(), trigger.entity());
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let icon_value = *world.get::<IconValue>(this).unwrap();
        world
            .commands()
            .entity(this)
            .insert(Grid::new(1.col().gap(4), 1.row().gap(4)));
        let panel = world.commands().leaf((
            Panel::new(),
            Stem::some(this),
            Location::new().xs(
                1.col().left().with(1.col().right()),
                1.row().top().with(1.row().bottom()),
            ),
            InteractionPropagation::pass_through(),
            FocusBehavior::ignore(),
            Elevation::up(1),
        ));
        let icon = world.commands().leaf((
            Icon::new(icon_value.0),
            Elevation::up(2),
            Stem::some(this),
            InteractionPropagation::pass_through(),
            FocusBehavior::ignore(),
        ));
        let text = world.commands().leaf((
            Text::new(""),
            Elevation::up(2),
            Stem::some(this),
            HorizontalAlignment::Left,
            VerticalAlignment::Middle,
            Location::new().xs(
                50.pct().center_x().adjust(20).with(0.letters().width()),
                1.row().top().with(1.row().bottom()),
            ),
            InteractionPropagation::pass_through(),
            FocusBehavior::ignore(),
        ));
        let handle = Handle { panel, icon, text };
        world.commands().entity(this).insert(handle);
    }
}
impl Composite for Button {
    type Handle = Handle;
    fn remove(handle: &Self::Handle) -> impl TriggerTargets + Send + Sync + 'static {
        [handle.panel, handle.text, handle.icon]
    }
}
#[derive(Component, Copy, Clone)]
#[component(on_replace = handle_replace::<Button>)]
pub struct Handle {
    pub panel: Entity,
    pub icon: Entity,
    pub text: Entity,
}
