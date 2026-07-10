use crate::composite::{handle_replace, ItemIndex, Root, Selected};
use crate::{
    Component, Composite, EcsExtension, Elevation, FocusBehavior, FontSize, Foliage, Grid,
    GridExt, HorizontalAlignment, IntoTargets, Location, OnClick, Panel, Primary, Rounding,
    Secondary, Stem, Text, Tree, Trigger, Update, VerticalAlignment,
};
use crate::{Attachment, InteractionListener};
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::{HookContext, Insert};
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;

/// Page-indicator composite: `<` and `>` arrows around one dot per page. `Page` is the
/// 0-based current page; clicking an arrow or dot writes `Page` and emits [`Selected`] at
/// the composite root. Theming: `Primary` = active dot + arrows, `Secondary` = idle dots.
#[derive(Component, Clone)]
#[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
#[require(PageCount, Page, Primary, Secondary, FontSize)]
pub struct Pagination {}

/// Number of pages the composite represents (dots rendered).
#[derive(Component, Copy, Clone, Debug)]
pub struct PageCount(pub usize);
impl Default for PageCount {
    fn default() -> Self {
        Self(1)
    }
}
/// 0-based current page. Insert this to change pages programmatically.
#[derive(Component, Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct Page(pub usize);

impl Default for Pagination {
    fn default() -> Self {
        Self::new()
    }
}
impl Attachment for Pagination {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Self::handle_trigger);
    }
}
impl Pagination {
    const DOT_SIZE: i32 = 8;
    pub fn new() -> Self {
        Self {}
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world
            .commands()
            .entity(this)
            .observe(Self::forward_page)
            .observe(Self::update_page)
            .observe(Self::forward_page_count)
            .observe(Self::update_page_count)
            .observe(Self::forward_primary)
            .observe(Self::update_primary)
            .observe(Self::forward_secondary)
            .observe(Self::update_secondary);
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let count = world.get::<PageCount>(this).unwrap().0.max(1);
        world
            .commands()
            .entity(this)
            .insert(Self::grid_for(count));
        let left = world.commands().leaf((
            Text::new("<"),
            Stem::some(this),
            Elevation::up(1),
            HorizontalAlignment::Center,
            VerticalAlignment::Middle,
            Location::new().xs(
                1.col().left().with(1.col().right()),
                1.row().top().with(1.row().bottom()),
            ),
            InteractionListener::new(),
            FocusBehavior::ignore(),
            Root(this),
        ));
        world.commands().entity(left).observe(Self::left_clicked);
        let right = world.commands().leaf((
            Text::new(">"),
            Stem::some(this),
            Elevation::up(1),
            HorizontalAlignment::Center,
            VerticalAlignment::Middle,
            Location::new().xs(
                (count + 2).col().left().with((count + 2).col().right()),
                1.row().top().with(1.row().bottom()),
            ),
            InteractionListener::new(),
            FocusBehavior::ignore(),
            Root(this),
        ));
        world.commands().entity(right).observe(Self::right_clicked);
        let handle = Handle {
            left,
            right,
            dots: vec![],
        };
        world.commands().entity(this).insert(handle);
    }
    fn grid_for(count: usize) -> Grid {
        Grid::new((count + 2).col().gap(4), 1.row())
    }
    fn handle_trigger(trigger: Trigger<Insert, Handle>, mut tree: Tree) {
        let this = trigger.event_target();
        tree.trigger_targets(Update::<PageCount>::new(), this);
        tree.trigger_targets(Update::<Page>::new(), this);
        tree.trigger_targets(Update::<Primary>::new(), this);
        tree.trigger_targets(Update::<Secondary>::new(), this);
    }
    fn forward_page_count(trigger: Trigger<Insert, PageCount>, mut tree: Tree) {
        tree.trigger_targets(Update::<PageCount>::new(), trigger.event_target());
    }
    fn update_page_count(
        trigger: Trigger<Update<PageCount>>,
        mut tree: Tree,
        mut handles: Query<&mut Handle>,
        counts: Query<&PageCount>,
        pages: Query<&Page>,
    ) {
        let this = trigger.event_target();
        let count = counts.get(this).unwrap().0.max(1);
        let mut handle = handles.get_mut(this).unwrap();
        // teardown + respawn: deterministic rebuild instead of diffing
        for dot in handle.dots.drain(..) {
            tree.remove(dot);
        }
        tree.entity(this).insert(Self::grid_for(count));
        tree.entity(handle.right).insert(Location::new().xs(
            (count + 2).col().left().with((count + 2).col().right()),
            1.row().top().with(1.row().bottom()),
        ));
        for i in 0..count {
            let dot = tree.leaf((
                Panel::new(),
                Rounding::Full,
                Stem::some(this),
                Elevation::up(1),
                Location::new().xs(
                    (i + 2)
                        .col()
                        .center_x()
                        .with(Self::DOT_SIZE.px().width()),
                    50.pct().center_y().with(Self::DOT_SIZE.px().height()),
                ),
                InteractionListener::new(),
                FocusBehavior::ignore(),
                ItemIndex(i),
                Root(this),
            ));
            tree.entity(dot).observe(Self::dot_clicked);
            handle.dots.push(dot);
        }
        // clamp the current page into the new count
        let page = pages.get(this).unwrap().0;
        if page >= count {
            tree.entity(this).insert(Page(count - 1));
        } else {
            tree.trigger_targets(Update::<Page>::new(), this);
        }
    }
    fn forward_page(trigger: Trigger<Insert, Page>, mut tree: Tree) {
        tree.trigger_targets(Update::<Page>::new(), trigger.event_target());
    }
    fn update_page(
        trigger: Trigger<Update<Page>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        pages: Query<&Page>,
        primaries: Query<&Primary>,
        secondaries: Query<&Secondary>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let page = pages.get(this).unwrap().0;
        let primary = primaries.get(this).unwrap().0;
        let secondary = secondaries.get(this).unwrap().0;
        for (i, dot) in handle.dots.iter().enumerate() {
            tree.entity(*dot)
                .insert(if i == page { primary } else { secondary });
        }
        tree.trigger_targets(Selected::new(page), this);
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
        tree.entity(handle.left).insert(primary);
        tree.entity(handle.right).insert(primary);
        tree.trigger_targets(Update::<Page>::new(), this);
    }
    fn forward_secondary(trigger: Trigger<Insert, Secondary>, mut tree: Tree) {
        tree.trigger_targets(Update::<Secondary>::new(), trigger.event_target());
    }
    fn update_secondary(trigger: Trigger<Update<Secondary>>, mut tree: Tree) {
        tree.trigger_targets(Update::<Page>::new(), trigger.event_target());
    }
    fn left_clicked(
        trigger: Trigger<OnClick>,
        mut tree: Tree,
        roots: Query<&Root>,
        pages: Query<&Page>,
    ) {
        let root = roots.get(trigger.event_target()).unwrap().0;
        let page = pages.get(root).unwrap().0;
        if page > 0 {
            tree.entity(root).insert(Page(page - 1));
        }
    }
    fn right_clicked(
        trigger: Trigger<OnClick>,
        mut tree: Tree,
        roots: Query<&Root>,
        pages: Query<&Page>,
        counts: Query<&PageCount>,
    ) {
        let root = roots.get(trigger.event_target()).unwrap().0;
        let page = pages.get(root).unwrap().0;
        let count = counts.get(root).unwrap().0.max(1);
        if page + 1 < count {
            tree.entity(root).insert(Page(page + 1));
        }
    }
    fn dot_clicked(
        trigger: Trigger<OnClick>,
        mut tree: Tree,
        roots: Query<&Root>,
        indices: Query<&ItemIndex>,
    ) {
        let root = roots.get(trigger.event_target()).unwrap().0;
        let index = indices.get(trigger.event_target()).unwrap().0;
        tree.entity(root).insert(Page(index));
    }
}
impl Composite for Pagination {
    type Handle = Handle;
    fn remove(handle: &Self::Handle) -> impl IntoTargets + Send + Sync + 'static {
        let mut targets = handle.dots.clone();
        targets.push(handle.left);
        targets.push(handle.right);
        targets
    }
}
#[derive(Component, Clone)]
#[component(on_discard = handle_replace::<Pagination>)]
pub struct Handle {
    pub left: Entity,
    pub right: Entity,
    pub dots: Vec<Entity>,
}
