use crate::composite::{handle_replace, ItemIndex, Root, Selected};
use crate::{
    Attachment, Component, Composite, EcsExtension, Elevation, FocusBehavior, FontSize, Foliage,
    Grid, GridExt, HorizontalAlignment, InteractionListener, InteractionPropagation, IntoTargets,
    Location, OnClick, Panel, Primary, Rounding, Secondary, Stem, Tertiary, Text, Tree, Trigger,
    Update, VerticalAlignment,
};
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::{HookContext, Insert};
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;

/// Scrollable selectable text rows. `ListItems` is the whole data model — writing it rebuilds
/// every row (teardown + respawn; deterministic, no diffing), so the practical ceiling is
/// hundreds of rows, not thousands (no virtualization). Clicking a row writes
/// `SelectedIndex(Some(i))` and emits [`Selected`] at the root. Theming: `Primary` = row text,
/// `Secondary` = row background, `Tertiary` = selected row background.
#[derive(Component, Clone)]
#[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
#[require(ListItems, SelectedIndex, RowHeight, Primary, Secondary, Tertiary, FontSize)]
pub struct List {}

/// The rows' text content — the list's entire data model.
#[derive(Component, Clone, Default)]
pub struct ListItems(pub Vec<String>);
/// 0-based selected row, if any. Insert to select programmatically.
#[derive(Component, Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct SelectedIndex(pub Option<usize>);
/// Row height in logical pixels.
#[derive(Component, Copy, Clone, Debug)]
pub struct RowHeight(pub i32);
impl Default for RowHeight {
    fn default() -> Self {
        Self(36)
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}
impl Attachment for List {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Self::handle_trigger);
    }
}
impl List {
    const ROW_GAP: i32 = 2;
    const TEXT_PAD: i32 = 8;
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
            .observe(Self::forward_primary)
            .observe(Self::update_primary)
            .observe(Self::forward_secondary)
            .observe(Self::update_secondary)
            .observe(Self::forward_tertiary)
            .observe(Self::update_tertiary)
            .observe(Self::forward_font_size)
            .observe(Self::update_font_size);
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let row_height = world.get::<RowHeight>(this).unwrap().0;
        world.commands().entity(this).insert(Grid::default());
        // the scroll container: Grid requires View, so drag/wheel scrolling comes for free
        let panel = world.commands().leaf((
            Stem::some(this),
            Grid::new(
                1.col().gap(Self::ROW_GAP),
                row_height.px().gap(Self::ROW_GAP),
            ),
            Elevation::up(1),
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ),
            InteractionListener::new(),
            FocusBehavior::ignore(),
            Root(this),
        ));
        let handle = Handle {
            panel,
            rows: vec![],
            texts: vec![],
        };
        world.commands().entity(this).insert(handle);
    }
    fn handle_trigger(trigger: Trigger<Insert, Handle>, mut tree: Tree) {
        let this = trigger.event_target();
        tree.trigger_targets(Update::<ListItems>::new(), this);
    }
    fn forward_items(trigger: Trigger<Insert, ListItems>, mut tree: Tree) {
        tree.trigger_targets(Update::<ListItems>::new(), trigger.event_target());
    }
    fn update_items(
        trigger: Trigger<Update<ListItems>>,
        mut tree: Tree,
        mut handles: Query<&mut Handle>,
        items: Query<&ListItems>,
        primaries: Query<&Primary>,
        secondaries: Query<&Secondary>,
        font_sizes: Query<&FontSize>,
    ) {
        let this = trigger.event_target();
        let mut handle = handles.get_mut(this).unwrap();
        let items = items.get(this).unwrap();
        let primary = primaries.get(this).unwrap().0;
        let secondary = secondaries.get(this).unwrap().0;
        let font_size = *font_sizes.get(this).unwrap();
        // teardown + respawn: deterministic rebuild instead of diffing
        for e in handle.rows.drain(..) {
            tree.remove(e);
        }
        for e in handle.texts.drain(..) {
            tree.remove(e);
        }
        for (i, item) in items.0.iter().enumerate() {
            let row = tree.leaf((
                Panel::new(),
                Rounding::Sm,
                secondary,
                Stem::some(handle.panel),
                Elevation::up(1),
                Location::new().xs(
                    1.col().left().with(1.col().right()),
                    (i + 1).row().top().with((i + 1).row().bottom()),
                ),
                InteractionListener::new(),
                FocusBehavior::ignore(),
                ItemIndex(i),
                Root(this),
            ));
            tree.entity(row).observe(Self::row_clicked);
            let text = tree.leaf((
                Text::new(item),
                font_size,
                primary,
                HorizontalAlignment::Left,
                VerticalAlignment::Middle,
                Stem::some(handle.panel),
                Elevation::up(2),
                Location::new().xs(
                    1.col()
                        .left()
                        .adjust(Self::TEXT_PAD)
                        .with(1.col().right().adjust(-Self::TEXT_PAD)),
                    (i + 1).row().top().with((i + 1).row().bottom()),
                ),
                InteractionPropagation::pass_through(),
                FocusBehavior::ignore(),
                Root(this),
            ));
            handle.rows.push(row);
            handle.texts.push(text);
        }
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
        secondaries: Query<&Secondary>,
        tertiaries: Query<&Tertiary>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let selected = selected.get(this).unwrap().0;
        let secondary = secondaries.get(this).unwrap().0;
        let tertiary = tertiaries.get(this).unwrap().0;
        for (i, row) in handle.rows.iter().enumerate() {
            tree.entity(*row).insert(if Some(i) == selected {
                tertiary
            } else {
                secondary
            });
        }
        if let Some(index) = selected {
            tree.trigger_targets(Selected::new(index), this);
        }
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
        let primary = primaries.get(this).unwrap().0;
        for text in handle.texts.iter() {
            tree.entity(*text).insert(primary);
        }
    }
    fn forward_secondary(trigger: Trigger<Insert, Secondary>, mut tree: Tree) {
        tree.trigger_targets(Update::<SelectedIndex>::new(), trigger.event_target());
    }
    fn update_secondary(trigger: Trigger<Update<Secondary>>, mut tree: Tree) {
        tree.trigger_targets(Update::<SelectedIndex>::new(), trigger.event_target());
    }
    fn forward_tertiary(trigger: Trigger<Insert, Tertiary>, mut tree: Tree) {
        tree.trigger_targets(Update::<SelectedIndex>::new(), trigger.event_target());
    }
    fn update_tertiary(trigger: Trigger<Update<Tertiary>>, mut tree: Tree) {
        tree.trigger_targets(Update::<SelectedIndex>::new(), trigger.event_target());
    }
    fn forward_font_size(trigger: Trigger<Insert, FontSize>, mut tree: Tree) {
        tree.trigger_targets(Update::<FontSize>::new(), trigger.event_target());
    }
    fn update_font_size(
        trigger: Trigger<Update<FontSize>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        font_sizes: Query<&FontSize>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let font_size = *font_sizes.get(this).unwrap();
        for text in handle.texts.iter() {
            tree.entity(*text).insert(font_size);
        }
    }
    fn row_clicked(
        trigger: Trigger<OnClick>,
        mut tree: Tree,
        roots: Query<&Root>,
        indices: Query<&ItemIndex>,
    ) {
        let root = roots.get(trigger.event_target()).unwrap().0;
        let index = indices.get(trigger.event_target()).unwrap().0;
        tree.entity(root).insert(SelectedIndex(Some(index)));
    }
}
impl Composite for List {
    type Handle = Handle;
    fn remove(handle: &Self::Handle) -> impl IntoTargets + Send + Sync + 'static {
        let mut targets = handle.rows.clone();
        targets.extend(handle.texts.iter().copied());
        targets.push(handle.panel);
        targets
    }
}
#[derive(Component, Clone)]
#[component(on_discard = handle_replace::<List>)]
pub struct Handle {
    pub panel: Entity,
    pub rows: Vec<Entity>,
    pub texts: Vec<Entity>,
}
