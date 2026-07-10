use crate::composite::list::{List, ListItems, RowHeight, SelectedIndex};
use crate::composite::{handle_replace, Root, Selected};
use crate::{
    Attachment, Button, Component, Composite, EcsExtension, Elevation, FontSize, Foliage, Grid,
    IntoTargets, Location, OnClick, Primary, Rounding, Secondary, Stem, Tertiary, TextValue, Tree,
    Trigger, Unfocused, Update, Visibility,
};
use crate::GridExt;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::{HookContext, Insert};
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;

/// A [`Button`] displaying the current choice that expands a [`List`] overlay of `ListItems`
/// beneath it. Selecting an option writes `SelectedIndex`, updates the button label, collapses
/// the overlay, and emits [`Selected`] at the root; clicking away (unfocus) collapses. Reuses
/// `ListItems`/`SelectedIndex` from [`List`] so the two read identically.
#[derive(Component, Clone)]
#[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
#[require(ListItems, SelectedIndex, RowHeight, Primary, Secondary, Tertiary, FontSize)]
#[require(Expanded)]
pub struct Dropdown {}

/// Whether the options overlay is currently open.
#[derive(Component, Copy, Clone, Default, Eq, PartialEq)]
pub struct Expanded(pub bool);

impl Default for Dropdown {
    fn default() -> Self {
        Self::new()
    }
}
impl Attachment for Dropdown {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Self::handle_trigger);
    }
}
impl Dropdown {
    pub fn new() -> Self {
        Self {}
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world
            .commands()
            .entity(this)
            .observe(Self::forward_items)
            .observe(Self::update_items)
            .observe(Self::forward_selected_index)
            .observe(Self::update_selected_index)
            .observe(Self::forward_expanded)
            .observe(Self::update_expanded)
            .observe(Self::forward_primary)
            .observe(Self::update_primary)
            .observe(Self::forward_secondary)
            .observe(Self::update_secondary)
            .observe(Self::forward_tertiary)
            .observe(Self::update_tertiary);
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let row_height = world.get::<RowHeight>(this).unwrap().0;
        let font_size = *world.get::<FontSize>(this).unwrap();
        world.commands().entity(this).insert(Grid::default());
        let button = world.commands().leaf((
            Button::new(),
            Rounding::Sm,
            font_size,
            Stem::some(this),
            Elevation::up(1),
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ),
            Root(this),
        ));
        world
            .commands()
            .entity(button)
            .observe(Self::button_clicked)
            .observe(Self::unfocused);
        let list = world.commands().leaf((
            List::new(),
            RowHeight(row_height),
            font_size,
            Stem::some(this),
            // overlays everything around it while expanded
            Elevation::up(10),
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                100.pct().top().adjust(4).with(0.px().height()),
            ),
            Visibility::new(false),
            Root(this),
        ));
        world.commands().entity(list).observe(Self::option_selected);
        let handle = Handle { button, list };
        world.commands().entity(this).insert(handle);
    }
    fn overlay_location(items: usize, row_height: i32) -> Location {
        let height = (items as i32) * (row_height + 2) + 2;
        Location::new().xs(
            0.pct().left().with(100.pct().right()),
            100.pct().top().adjust(4).with(height.px().height()),
        )
    }
    fn handle_trigger(trigger: Trigger<Insert, Handle>, mut tree: Tree) {
        let this = trigger.event_target();
        tree.trigger_targets(Update::<ListItems>::new(), this);
        tree.trigger_targets(Update::<SelectedIndex>::new(), this);
        tree.trigger_targets(Update::<Expanded>::new(), this);
        tree.trigger_targets(Update::<Primary>::new(), this);
        tree.trigger_targets(Update::<Secondary>::new(), this);
        tree.trigger_targets(Update::<Tertiary>::new(), this);
    }
    fn forward_items(trigger: Trigger<Insert, ListItems>, mut tree: Tree) {
        tree.trigger_targets(Update::<ListItems>::new(), trigger.event_target());
    }
    fn update_items(
        trigger: Trigger<Update<ListItems>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        items: Query<&ListItems>,
        row_heights: Query<&RowHeight>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let items = items.get(this).unwrap();
        let row_height = row_heights.get(this).unwrap().0;
        tree.entity(handle.list).insert(items.clone());
        tree.entity(handle.list)
            .insert(Self::overlay_location(items.0.len(), row_height));
        tree.trigger_targets(Update::<SelectedIndex>::new(), this);
    }
    fn forward_selected_index(trigger: Trigger<Insert, SelectedIndex>, mut tree: Tree) {
        tree.trigger_targets(Update::<SelectedIndex>::new(), trigger.event_target());
    }
    fn update_selected_index(
        trigger: Trigger<Update<SelectedIndex>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        selected: Query<&SelectedIndex>,
        items: Query<&ListItems>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let selected = selected.get(this).unwrap().0;
        let items = items.get(this).unwrap();
        let label = selected
            .and_then(|i| items.0.get(i).cloned())
            .unwrap_or_default();
        tree.entity(handle.button).insert(TextValue(label));
        tree.entity(handle.list).insert(SelectedIndex(selected));
        if let Some(index) = selected {
            tree.trigger_targets(Selected::new(index), this);
        }
    }
    fn forward_expanded(trigger: Trigger<Insert, Expanded>, mut tree: Tree) {
        tree.trigger_targets(Update::<Expanded>::new(), trigger.event_target());
    }
    fn update_expanded(
        trigger: Trigger<Update<Expanded>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        expanded: Query<&Expanded>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let expanded = expanded.get(this).unwrap().0;
        tree.entity(handle.list).insert(Visibility::new(expanded));
    }
    fn forward_primary(trigger: Trigger<Insert, Primary>, mut tree: Tree) {
        tree.trigger_targets(Update::<Primary>::new(), trigger.event_target());
    }
    fn update_primary(
        trigger: Trigger<Update<Primary>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        primaries: Query<&Primary>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let primary = *primaries.get(this).unwrap();
        tree.entity(handle.button).insert(primary);
        tree.entity(handle.list).insert(primary);
    }
    fn forward_secondary(trigger: Trigger<Insert, Secondary>, mut tree: Tree) {
        tree.trigger_targets(Update::<Secondary>::new(), trigger.event_target());
    }
    fn update_secondary(
        trigger: Trigger<Update<Secondary>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        secondaries: Query<&Secondary>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let secondary = *secondaries.get(this).unwrap();
        tree.entity(handle.button).insert(secondary);
        tree.entity(handle.list).insert(secondary);
    }
    fn forward_tertiary(trigger: Trigger<Insert, Tertiary>, mut tree: Tree) {
        tree.trigger_targets(Update::<Tertiary>::new(), trigger.event_target());
    }
    fn update_tertiary(
        trigger: Trigger<Update<Tertiary>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        tertiaries: Query<&Tertiary>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let tertiary = *tertiaries.get(this).unwrap();
        tree.entity(handle.list).insert(tertiary);
    }
    fn button_clicked(
        trigger: Trigger<OnClick>,
        mut tree: Tree,
        roots: Query<&Root>,
        expanded: Query<&Expanded>,
    ) {
        let root = roots.get(trigger.event_target()).unwrap().0;
        let now = !expanded.get(root).unwrap().0;
        tree.entity(root).insert(Expanded(now));
    }
    fn unfocused(trigger: Trigger<Unfocused>, mut tree: Tree, roots: Query<&Root>) {
        let root = roots.get(trigger.event_target()).unwrap().0;
        tree.entity(root).insert(Expanded(false));
    }
    fn option_selected(
        trigger: Trigger<Selected>,
        mut tree: Tree,
        roots: Query<&Root>,
        selected: Query<&SelectedIndex>,
    ) {
        let root = roots.get(trigger.event_target()).unwrap().0;
        let index = trigger.event().index;
        // only react to real changes so the List's initial Update bounce doesn't re-collapse
        if selected.get(root).unwrap().0 != Some(index) {
            tree.entity(root).insert(SelectedIndex(Some(index)));
        }
        tree.entity(root).insert(Expanded(false));
    }
}
impl Composite for Dropdown {
    type Handle = Handle;
    fn remove(handle: &Self::Handle) -> impl IntoTargets + Send + Sync + 'static {
        [handle.button, handle.list]
    }
}
#[derive(Component, Copy, Clone)]
#[component(on_discard = handle_replace::<Dropdown>)]
pub struct Handle {
    pub button: Entity,
    pub list: Entity,
}
