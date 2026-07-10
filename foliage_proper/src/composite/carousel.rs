use crate::composite::pagination::{Page, PageCount, Pagination};
use crate::composite::{handle_replace, Root, Selected};
use crate::interaction::CurrentInteraction;
use crate::{
    Animation, AssetKey, Attachment, Color, Component, Composite, Disengaged, EcsExtension,
    Elevation, FocusBehavior, Foliage, Grid, GridExt, Icon, IconId, Image, ImageView,
    InteractionListener, InteractionPropagation, IntoTargets, Location, MemoryId, Panel, Primary,
    Secondary, Stem, Tree, Trigger, Update,
};
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::{HookContext, Insert};
use bevy_ecs::system::{Query, Res};
use bevy_ecs::world::DeferredWorld;

/// One page of a [`Carousel`]: a solid color, an image (memory id + asset key), or an icon.
#[derive(Copy, Clone)]
pub enum CarouselItem {
    Color(Color),
    Image(MemoryId, AssetKey),
    Icon(IconId),
}
/// The carousel's data model — one entry per page.
#[derive(Component, Clone, Default)]
pub struct CarouselItems(pub Vec<CarouselItem>);

/// Swipeable horizontal pager with an embedded [`Pagination`] indicator. Pages sit on a wide
/// strip whose `Location` animates on page change (drag past 25% of the width to page; the
/// strip does not live-follow the finger — a swipe is a paging gesture, not a free scroll).
/// Reuses `Page` from [`Pagination`]; emits [`Selected`] at the root on page change.
#[derive(Component, Clone)]
#[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
#[require(CarouselItems, Page, Primary, Secondary)]
pub struct Carousel {}

impl Default for Carousel {
    fn default() -> Self {
        Self::new()
    }
}
impl Attachment for Carousel {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Self::handle_trigger);
    }
}
impl Carousel {
    const SNAP_MILLIS: u64 = 250;
    const SWIPE_FRACTION: f32 = 0.25;
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
            .observe(Self::forward_page)
            .observe(Self::update_page)
            .observe(Self::forward_primary)
            .observe(Self::update_primary)
            .observe(Self::forward_secondary)
            .observe(Self::update_secondary);
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world.commands().entity(this).insert(Grid::default());
        // swipe surface: drag disabled so the gesture is ours, not the View scroller's
        let viewport = world.commands().leaf((
            Stem::some(this),
            Elevation::up(1),
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ),
            InteractionListener::new(),
            InteractionPropagation::grab().disable_drag(),
            FocusBehavior::ignore(),
            Root(this),
        ));
        world
            .commands()
            .entity(viewport)
            .observe(Self::swipe_released);
        // the strip is re-laid-out per items-count; starts as a single empty page
        let strip = world.commands().leaf((
            Stem::some(viewport),
            Grid::default(),
            Elevation::up(1),
            Self::strip_location(1, 0),
            InteractionPropagation::pass_through(),
            FocusBehavior::ignore(),
            Root(this),
        ));
        let pagination = world.commands().leaf((
            Pagination::new(),
            Stem::some(this),
            Elevation::up(3),
            Location::new().xs(
                50.pct().center_x().with(40.pct().width()),
                100.pct().bottom().adjust(-8).with(24.px().height()),
            ),
            Root(this),
        ));
        world
            .commands()
            .entity(pagination)
            .observe(Self::indicator_selected);
        let handle = Handle {
            viewport,
            strip,
            pages: vec![],
            pagination,
        };
        world.commands().entity(this).insert(handle);
    }
    /// The strip is `count` viewports wide, shifted one whole viewport left per page.
    fn strip_location(count: usize, page: usize) -> Location {
        let count = count.max(1) as i32;
        let page = page as i32;
        Location::new().xs(
            (-100 * page)
                .pct()
                .left()
                .with((100 * (count - page)).pct().right()),
            0.pct().top().with(100.pct().bottom()),
        )
    }
    fn handle_trigger(trigger: Trigger<Insert, Handle>, mut tree: Tree) {
        let this = trigger.event_target();
        tree.trigger_targets(Update::<CarouselItems>::new(), this);
        tree.trigger_targets(Update::<Primary>::new(), this);
        tree.trigger_targets(Update::<Secondary>::new(), this);
    }
    fn forward_items(trigger: Trigger<Insert, CarouselItems>, mut tree: Tree) {
        tree.trigger_targets(Update::<CarouselItems>::new(), trigger.event_target());
    }
    fn update_items(
        trigger: Trigger<Update<CarouselItems>>,
        mut tree: Tree,
        mut handles: Query<&mut Handle>,
        items: Query<&CarouselItems>,
        pages: Query<&Page>,
    ) {
        let this = trigger.event_target();
        let mut handle = handles.get_mut(this).unwrap();
        let items = items.get(this).unwrap();
        let count = items.0.len().max(1);
        // teardown + respawn: deterministic rebuild instead of diffing
        for e in handle.pages.drain(..) {
            tree.remove(e);
        }
        tree.entity(handle.strip)
            .insert(Grid::new(count.col(), 1.row()));
        for (i, item) in items.0.iter().enumerate() {
            let location = Location::new().xs(
                (i + 1).col().left().with((i + 1).col().right()),
                1.row().top().with(1.row().bottom()),
            );
            let base = (
                Stem::some(handle.strip),
                Elevation::up(1),
                location,
                InteractionPropagation::pass_through(),
                FocusBehavior::ignore(),
                Root(this),
            );
            let page = match item {
                CarouselItem::Color(color) => tree.leaf((Panel::new(), *color, base)),
                CarouselItem::Image(memory_id, key) => {
                    tree.leaf((Image::new(*memory_id, *key), ImageView::Crop, base))
                }
                CarouselItem::Icon(icon_id) => tree.leaf((Icon::new(*icon_id), base)),
            };
            handle.pages.push(page);
        }
        tree.entity(handle.pagination).insert(PageCount(count));
        // clamp + re-snap the strip for the new count
        let page = pages.get(this).unwrap().0;
        if page >= count {
            tree.entity(this).insert(Page(count - 1));
        } else {
            tree.entity(handle.strip)
                .insert(Self::strip_location(count, page));
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
        items: Query<&CarouselItems>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let count = items.get(this).unwrap().0.len().max(1);
        let page = pages.get(this).unwrap().0.min(count - 1);
        let seq = tree.sequence();
        tree.animate(
            Animation::new(Self::strip_location(count, page))
                .start(0)
                .finish(Self::SNAP_MILLIS)
                .during(seq)
                .targeting(handle.strip),
        );
        tree.entity(handle.pagination).insert(Page(page));
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
        tree.entity(handle.pagination)
            .insert(*primaries.get(this).unwrap());
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
        tree.entity(handle.pagination)
            .insert(*secondaries.get(this).unwrap());
    }
    fn swipe_released(
        trigger: Trigger<Disengaged>,
        mut tree: Tree,
        roots: Query<&Root>,
        pages: Query<&Page>,
        items: Query<&CarouselItems>,
        sections: Query<&crate::Section<crate::Logical>>,
        current_interaction: Res<CurrentInteraction>,
    ) {
        let root = roots.get(trigger.event_target()).unwrap().0;
        let click = current_interaction.click();
        let Some(end) = click.end else {
            return;
        };
        let dx = end.left() - click.start.left();
        let width = sections.get(trigger.event_target()).unwrap().width();
        if width <= 0.0 || dx.abs() < width * Self::SWIPE_FRACTION {
            return;
        }
        let count = items.get(root).unwrap().0.len().max(1);
        let page = pages.get(root).unwrap().0;
        if dx < 0.0 {
            // swiped left => next page
            if page + 1 < count {
                tree.entity(root).insert(Page(page + 1));
            }
        } else if page > 0 {
            tree.entity(root).insert(Page(page - 1));
        }
    }
    fn indicator_selected(
        trigger: Trigger<Selected>,
        mut tree: Tree,
        roots: Query<&Root>,
        pages: Query<&Page>,
    ) {
        let root = roots.get(trigger.event_target()).unwrap().0;
        let index = trigger.event().index;
        // only real changes: the indicator echoes Page writes we made ourselves
        if pages.get(root).unwrap().0 != index {
            tree.entity(root).insert(Page(index));
        }
    }
}
impl Composite for Carousel {
    type Handle = Handle;
    fn remove(handle: &Self::Handle) -> impl IntoTargets + Send + Sync + 'static {
        let mut targets = handle.pages.clone();
        targets.push(handle.strip);
        targets.push(handle.viewport);
        targets.push(handle.pagination);
        targets
    }
}
#[derive(Component, Clone)]
#[component(on_discard = handle_replace::<Carousel>)]
pub struct Handle {
    pub viewport: Entity,
    pub strip: Entity,
    pub pages: Vec<Entity>,
    pub pagination: Entity,
}
