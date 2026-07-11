use crate::{
    anchor, composite_on_insert, forward, handle_replace, Anchor, Attachment, Children,
    Disengaged, EcsExtension, Elevation, Engaged, FocusBehavior, Foliage, FontSize, Grid,
    GridExt, HorizontalAlignment, Icon, IconValue, InteractionListener, InteractionPropagation,
    Leaf, LeafBuilder, Location, Outline, Panel, Primary, Rounding, Secondary, Text,
    TextValue, Tree, Update, VerticalAlignment, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::HookContext;
use crate::{Component, Composite};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use crate::IntoTargets;
use crate::Trigger;
use bevy_ecs::system::Query;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::world::DeferredWorld;

#[derive(Component, Clone)]
#[component(on_add = Self::on_add)]
#[component(on_insert = composite_on_insert::<Button>)]
#[require(Rounding, FontSize, IconValue, Outline, Primary, Secondary)]
pub struct Button {}
impl Attachment for Button {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Button::handle_trigger);
    }
}
impl Button {
    pub fn new() -> ButtonSpec {
        ButtonSpec::default()
    }
    pub(crate) fn new_marker() -> Self {
        Self {}
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world
            .commands()
            .entity(this)
            .observe(Self::engaged)
            .observe(Self::disengaged)
            .observe(forward::<TextValue>)
            .observe(Self::update_text)
            .observe(forward::<FontSize>)
            .observe(Self::update_font_size)
            .observe(forward::<IconValue>)
            .observe(Self::update_icon)
            .observe(forward::<Outline>)
            .observe(Self::update_outline)
            .observe(forward::<Rounding>)
            .observe(Self::update_rounding)
            .observe(forward::<Primary>)
            .observe(Self::update_primary)
            .observe(forward::<Secondary>)
            .observe(Self::update_secondary);
    }
    fn handle_trigger(trigger: Trigger<Insert, Handle>, mut tree: Tree) {
        // trigger all
        let this = trigger.event_target();
        tree.trigger_targets(Update::<TextValue>::new(), this);
        tree.trigger_targets(Update::<FontSize>::new(), this);
        tree.trigger_targets(Update::<IconValue>::new(), this);
        tree.trigger_targets(Update::<Outline>::new(), this);
        tree.trigger_targets(Update::<Rounding>::new(), this);
        tree.trigger_targets(Update::<Primary>::new(), this);
        tree.trigger_targets(Update::<Secondary>::new(), this);
    }

    fn update_text(
        trigger: Trigger<Update<TextValue>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        values: Query<&TextValue>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        if let Some(value) = values.get(this).ok() {
            tree.entity(handle.text)
                .insert(Text::new_marker(value.0.as_str()))
                .insert(
                    Location::new().xs(
                        50.pct()
                            .as_center_x()
                            .adjust(20)
                            .with(value.0.len().letters().as_width()),
                        1.row().as_top().with(1.row().as_bottom()),
                    ),
                );
        }
    }
    fn update_font_size(
        trigger: Trigger<Update<FontSize>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        values: Query<&FontSize>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let value = values.get(this).unwrap();
        tree.entity(handle.text).insert(*value);
    }
    fn update_icon(
        trigger: Trigger<Update<IconValue>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        values: Query<&IconValue>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let value = values.get(this).unwrap();
        tracing::trace!(button = ?this, icon = ?handle.icon, id = value.0, "button: icon updated");
        tree.entity(handle.icon).insert(Icon::new_marker(value.0));
    }
    fn update_outline(
        trigger: Trigger<Update<Outline>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        primaries: Query<&Primary>,
        secondaries: Query<&Secondary>,
        outlines: Query<&Outline>,
    ) {
        let this = trigger.event_target();
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
    fn update_primary(
        trigger: Trigger<Update<Primary>>,
        handles: Query<&Handle>,
        mut tree: Tree,
        primaries: Query<&Primary>,
        outlines: Query<&Outline>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let primary = primaries.get(this).unwrap();
        let outline = outlines.get(this).unwrap();
        tree.entity(handle.icon).insert(primary.0);
        tree.entity(handle.text).insert(primary.0);
        if outline != &Outline::default() {
            tree.entity(handle.panel).insert(primary.0);
        }
    }
    fn update_secondary(
        trigger: Trigger<Insert, Secondary>,
        handles: Query<&Handle>,
        mut tree: Tree,
        secondaries: Query<&Secondary>,
        outlines: Query<&Outline>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let outline = outlines.get(this).unwrap();
        let secondary = secondaries.get(this).unwrap();
        if outline == &Outline::default() {
            tree.entity(handle.panel).insert(secondary.0);
        }
    }
    fn engaged(
        trigger: Trigger<Engaged>,
        primaries: Query<&Primary>,
        secondaries: Query<&Secondary>,
        outlines: Query<&Outline>,
        handles: Query<&Handle>,
        mut tree: Tree,
    ) {
        let this = trigger.event_target();
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
        let this = trigger.event_target();
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
        let this = trigger.event_target();
        let round = roundings.get(this).unwrap();
        tree.entity(this).insert(InteractionListener::new());
        let handle = handles.get(this).unwrap();
        tracing::trace!(button = ?this, full = round == &Rounding::Full, icon = ?handle.icon, "button: rounding updated");
        let icon_location = match round {
            Rounding::Full => Location::new().xs(
                50.pct().as_center_x().with(24.px().as_width()),
                50.pct().as_center_y().with(24.px().as_height()),
            ),
            _ => Location::new().xs(
                anchor().left().as_right().adjust(-8).with(24.px().as_width()),
                50.pct().as_center_y().with(24.px().as_height()),
            ),
        };
        tree.entity(handle.icon).insert(icon_location);
        match round {
            Rounding::Full => {
                tree.entity(handle.panel).insert(Rounding::Full);
                tree.entity(handle.text).insert(Visibility::new(false));
                tree.entity(handle.icon).insert(Anchor::default());
            }
            _ => {
                tree.entity(handle.panel).insert(Rounding::Sm);
                tree.entity(handle.text).insert(Visibility::new(true));
                tree.entity(handle.icon).insert(Anchor::new(handle.text));
            }
        }
    }
}
impl Composite for Button {
    type Handle = Handle;
    fn children(this: Entity, children: &mut Children<DeferredWorld>) -> Self::Handle {
        let icon_value = *children.tree().get::<IconValue>(this).unwrap();
        children
            .tree()
            .commands()
            .entity(this)
            .insert(Grid::new(1.col().gap(4), 1.row().gap(4)));

        let panel = children.spawn(
            Panel::new()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    1.row().as_top().with(1.row().as_bottom()),
                ))
                .with((InteractionPropagation::pass_through(), FocusBehavior::ignore())),
        );

        // no Location: icon's position is content-dependent (Rounding) and set reactively by
        // update_rounding once Handle exists — giving it an empty Location here would fail
        // resolution immediately and auto-hide it before the real value arrives.
        let icon = children.spawn(
            Leaf::spec()
                .elevate(Elevation::up(2))
                .with((Icon::new_marker(icon_value.0), InteractionPropagation::pass_through(), FocusBehavior::ignore())),
        );

        let text = children.spawn(
            Text::new("")
                .elevate(Elevation::up(2))
                .at(Location::new().xs(
                    50.pct().as_center_x().adjust(20).with(0.letters().as_width()),
                    1.row().as_top().with(1.row().as_bottom()),
                ))
                .with((
                    HorizontalAlignment::Left,
                    VerticalAlignment::Middle,
                    InteractionPropagation::pass_through(),
                    FocusBehavior::ignore(),
                )),
        );

        Handle { panel, icon, text }
    }
    fn remove(handle: &Self::Handle) -> impl IntoTargets + Send + Sync + 'static {
        [handle.panel, handle.text, handle.icon]
    }
}
#[derive(Default)]
pub struct ButtonSpec {
    leaf: crate::LeafSpec,
    icon: Option<crate::IconId>,
    text: Option<String>,
    colors: Option<(crate::Color, crate::Color)>,
    rounding: Option<Rounding>,
    outline: Option<i32>,
}
impl crate::LeafBuilder for ButtonSpec {
    fn leaf_spec(&mut self) -> &mut crate::LeafSpec {
        &mut self.leaf
    }
    fn bundle(self) -> impl Bundle {
        let (primary, secondary) = self.colors.unwrap_or_default();
        (
            Button::new_marker(),
            self.leaf.location,
            self.leaf.stem,
            self.leaf
                .elevation
                .expect("elevation not set -- call .elevate(...) before spawning"),
            IconValue(self.icon.unwrap_or_default()),
            TextValue(self.text.unwrap_or_default()),
            Primary(primary),
            Secondary(secondary),
            self.rounding.unwrap_or_default(),
            self.outline.map(Outline::new).unwrap_or_default(),
        )
    }
}
impl ButtonSpec {
    pub fn icon(mut self, icon: crate::IconId) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
    pub fn colors(mut self, primary: crate::Color, secondary: crate::Color) -> Self {
        self.colors = Some((primary, secondary));
        self
    }
    pub fn rounding(mut self, r: Rounding) -> Self {
        self.rounding = Some(r);
        self
    }
    pub fn outline(mut self, w: i32) -> Self {
        self.outline = Some(w);
        self
    }
}
#[derive(Component, Copy, Clone)]
#[component(on_discard = handle_replace::<Button>)]
pub struct Handle {
    pub panel: Entity,
    pub icon: Entity,
    pub text: Entity,
}
