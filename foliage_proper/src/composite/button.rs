use crate::Justify::Far;
use crate::{
    handle_replace, Attachment, Color, EcsExtension, Elevation, Foliage, FontSize, Grid, GridExt,
    HorizontalAlignment, Icon, IconId, IconValue, Location, Outline, Panel, Primary, Rounding,
    Secondary, Stem, Text, TextValue, Tree, VerticalAlignment,
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
#[require(ButtonShape, FontSize, IconValue, Outline, Primary, Secondary)]
pub struct Button {}
impl Attachment for Button {
    fn attach(foliage: &mut Foliage) {

    }
}
#[derive(Component, Copy, Clone, PartialEq, Default, Debug)]
pub enum ButtonShape {
    Circle,
    #[default]
    Rectangle,
}
impl Button {
    pub fn new() -> Self {
        Self {}
    }
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        tracing::trace!("adding observers for {:?}", this);
        world.commands().entity(this)
            .observe(Self::forward_text)
            .observe(Self::forward_font_size)
            .observe(Self::forward_icon)
            .observe(Self::forward_outline)
            .observe(Self::forward_shape)
            .observe(Self::forward_primary)
            .observe(Self::forward_secondary);
    }
    fn forward_text(
        trigger: Trigger<OnInsert, TextValue>,
        mut tree: Tree,
        handles: Query<&Handle>,
        values: Query<&TextValue>,
    ) {
        let this = trigger.entity();
        let handle = handles.get(this).unwrap();
        let value = values.get(this).unwrap();
        tracing::trace!("forwarding text-value: {}", value.0);
        tree.entity(handle.text).insert(Text::new(value.0.as_str()));
    }
    fn forward_font_size(
        trigger: Trigger<OnInsert, FontSize>,
        mut tree: Tree,
        handles: Query<&Handle>,
        values: Query<&FontSize>,
    ) {
        let this = trigger.entity();
        let handle = handles.get(this).unwrap();
        let value = values.get(this).unwrap();
        tracing::trace!("forwarding font-sie: {}", value.xs);
        tree.entity(handle.text).insert(*value);
    }
    fn forward_icon(
        trigger: Trigger<OnInsert, IconValue>,
        mut tree: Tree,
        handles: Query<&Handle>,
        values: Query<&IconValue>,
    ) {
        let this = trigger.entity();
        let handle = handles.get(this).unwrap();
        let value = values.get(this).unwrap();
        tracing::trace!("forwarding icon: {}", value.0);
        tree.entity(handle.icon).insert(Icon::new(value.0));
    }
    fn forward_outline(
        trigger: Trigger<OnInsert, Outline>,
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
        tracing::trace!("forwarding outline: {}", outline.value);
        tree.entity(handle.panel).insert(color).insert(*outline);
    }
    fn forward_shape(
        trigger: Trigger<OnInsert, ButtonShape>,
        shapes: Query<&ButtonShape>,
        handles: Query<&Handle>,
        mut tree: Tree,
    ) {
        let this = trigger.entity();
        let shape = shapes.get(this).unwrap();
        let handle = handles.get(this).unwrap();
        let icon_location = match shape {
            ButtonShape::Circle => Location::new().xs(
                0.pct().to(100.pct()).min(24.px()).max(24.px()),
                0.pct().to(100.pct()).min(24.px()).max(24.px()),
            ),
            ButtonShape::Rectangle => Location::new().xs(
                1.col().to(1.col()).min(24.px()).max(24.px()).justify(Far),
                1.row().to(1.row()).min(24.px()).max(24.px()),
            ),
        };
        tracing::trace!("forwarding shape: {:?}", shape);
        tree.entity(handle.icon).insert(icon_location);
        match shape {
            ButtonShape::Circle => {
                tree.entity(handle.panel).insert(Rounding::Full);
            }
            ButtonShape::Rectangle => {
                tree.entity(handle.panel).insert(Rounding::Sm);
            }
        }
    }
    fn forward_primary(
        trigger: Trigger<OnInsert, Primary>,
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
        tracing::trace!("forwarding primary: {:?}", primary.0);
    }
    fn forward_secondary(
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
        tracing::trace!("forwarding secondary: {:?}", secondary.0);
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        // let this = trigger.entity();
        let icon_value = *world.get::<IconValue>(this).unwrap();
        world
            .commands()
            .entity(this)
            .insert(Grid::new(3.col(), 1.row()));
        let panel = world.commands().leaf((
            Panel::new(),
            Stem::some(this),
            Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())),
            Elevation::up(1),
        ));
        let icon =
            world
                .commands()
                .leaf((Icon::new(icon_value.0), Elevation::up(2), Stem::some(this)));
        let text = world.commands().leaf((
            Text::new(""),
            Elevation::up(2),
            Stem::some(this),
            HorizontalAlignment::Center,
            VerticalAlignment::Middle,
            Location::new().xs(2.col().to(3.col()), 1.row().to(1.row())),
        ));
        tracing::trace!("{:?}, {:?}, {:?}", panel, icon, text);
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
