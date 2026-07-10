use crate::{
    handle_replace, stack, Attachment, Disengaged, EcsExtension, Elevation, Engaged, FocusBehavior,
    Foliage, FontSize, Grid, GridExt, HorizontalAlignment, Icon, IconValue, InteractionListener,
    InteractionPropagation, Location, Outline, Panel, Primary, Rounding, Secondary, Stack, Stem,
    Text, TextValue, Tree, Update, VerticalAlignment, Visibility,
};
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
#[component(on_insert = Self::on_insert)]
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
                            .center_x()
                            .adjust(20)
                            .with(value.0.len().letters().width()),
                        1.row().top().with(1.row().bottom()),
                    ),
                );
        }
    }
    fn forward_text(trigger: Trigger<Insert, TextValue>, mut tree: Tree) {
        tree.trigger_targets(Update::<TextValue>::new(), trigger.event_target());
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
    fn forward_font_size(trigger: Trigger<Insert, FontSize>, mut tree: Tree) {
        tree.trigger_targets(Update::<FontSize>::new(), trigger.event_target());
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
        tree.entity(handle.icon).insert(Icon::new(value.0));
    }
    fn forward_icon(trigger: Trigger<Insert, IconValue>, mut tree: Tree) {
        tree.trigger_targets(Update::<IconValue>::new(), trigger.event_target());
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
    fn forward_outline(trigger: Trigger<Insert, Outline>, mut tree: Tree) {
        tree.trigger_targets(Update::<Outline>::new(), trigger.event_target());
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
    fn forward_primary(
        trigger: Trigger<Insert, Primary>,
        handles: Query<&Handle>,
        mut tree: Tree,
        primaries: Query<&Primary>,
        outlines: Query<&Outline>,
    ) {
        tree.trigger_targets(Update::<Primary>::new(), trigger.event_target());
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
    fn forward_secondary(trigger: Trigger<Insert, Secondary>, mut tree: Tree) {
        tree.trigger_targets(Update::<Secondary>::new(), trigger.event_target());
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
    fn forward_rounding(trigger: Trigger<Insert, Rounding>, mut tree: Tree) {
        tree.trigger_targets(Update::<Rounding>::new(), trigger.event_target());
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let icon_value = *world.get::<IconValue>(this).unwrap();
        world
            .commands()
            .entity(this)
            .insert(Grid::new(1.col().gap(4), 1.row().gap(4)));
        let panel = world.commands().leaf((
            Panel::new_marker(),
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
            Text::new_marker(""),
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
    pub fn spawn(self, tree: &mut impl EcsExtension) -> Entity {
        let e = tree.leaf((
            Button::new_marker(),
            self.leaf.location,
            self.leaf.stem,
            self.leaf.elevation,
        ));
        if let Some(v) = self.icon {
            tree.write_to(e, IconValue(v));
        }
        if let Some(v) = self.text {
            tree.write_to(e, TextValue(v));
        }
        if let Some((p, s)) = self.colors {
            tree.write_to(e, Primary(p));
            tree.write_to(e, Secondary(s));
        }
        if let Some(v) = self.rounding {
            tree.write_to(e, v);
        }
        if let Some(v) = self.outline {
            tree.write_to(e, Outline::new(v));
        }
        e
    }
}
#[derive(Component, Copy, Clone)]
#[component(on_discard = handle_replace::<Button>)]
pub struct Handle {
    pub panel: Entity,
    pub icon: Entity,
    pub text: Entity,
}
