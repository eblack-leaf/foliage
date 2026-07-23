use crate::Trigger;
use crate::composite::pagination::PageChanged;
use crate::composite::{PageCount, PageIndex};
use crate::{
    Component, EcsExtension, Elevation, Entity, Grid, GridExt, LeafSprout, Location, Sprout, Tree,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;

/// A route's scene builder. Deliberately a bare `fn`, not a closure: a bare `fn` cannot
/// capture, so a scene can never close over another scene's live entities -- the mistake
/// that makes destructive switching unsafe becomes a compile error instead of a runtime
/// mystery. Everything a scene needs beyond its slot arrives through `Resource`s (asset
/// keyrings, app state), which by definition outlive any scene.
pub type RouteFn = fn(&mut Tree, Entity);

/// Destructive, element-level scene switching: [`Tabs`](crate::Tabs)' exact opposite on
/// content lifecycle. Tabs builds every page once and toggles `Visibility`; Router keeps
/// exactly ONE route's content alive -- navigating tears the current scene's subtree down
/// (`tree.remove`) and runs the next route's builder against a fresh full-size slot. Hidden
/// scenes therefore never exist: nothing ticks, nothing accumulates, and a route entered
/// twice is built from nothing both times, so it cannot depend on residue from its last
/// visit.
///
/// One entity: components in ([`RouterRoutes`], [`PageIndex`]), [`PageCount`] maintained,
/// [`PageChanged`] out -- the same paging vocabulary `Tabs`/`Carousel`/`Pagination` share,
/// so any of those (a bottom nav bar, a stepper) can drive a Router by forwarding an index.
/// Navigation is `tree.write_to(router, PageIndex(i))`; there is no richer protocol.
///
/// Element-level, not engine-level: like `List`/`Carousel`/`Tabs`, the author builds and
/// owns every scene -- this widget only owns which one currently exists. Anything that must
/// survive a switch (persistent chrome, shared state) lives outside the router's subtree or
/// in a `Resource`, never inside a scene. Router renders nothing of its own and has no
/// style vocabulary.
///
/// NO URL/browser-history integration -- decided, not pending. A full design was worked
/// through (opt-in sync, per-route slug blessing, typed URL params as the only scene-input
/// channel) and rejected: a deep link makes any route someone's FIRST scene, entered by a
/// user-generated URL rather than an authored call site, so a route whose scene reads a
/// navigation-coupled `Resource` cold-boots against defaults -- and no declared contract
/// survives indirect circumvention, it just relocates the footgun. In-session navigation
/// has no such class: every `PageIndex` write is a call site the author wrote, orderings
/// are enforced by where navigation UI exists, and mistakes render visibly in development.
/// The invariant is "top-level browser entry only": the app always boots at its root.
/// Don't re-propose URL sync in any form, composite-integrated or app-side.
#[derive(Component, Copy, Clone)]
pub struct Router {}
impl Router {
    pub fn new() -> RouterSprout {
        RouterSprout {
            leaf: LeafSprout::default(),
            routes: None,
            route: 0,
        }
    }
}

/// The route set, rewritten as one unit (config writes ARE the re-render API, same as
/// [`crate::TabsPages`]): rewriting it rebuilds the current route's scene from scratch.
#[derive(Component, Clone)]
pub struct RouterRoutes(pub Vec<RouteFn>);
impl RouterRoutes {
    pub fn new(routes: impl IntoIterator<Item = RouteFn>) -> Self {
        Self(routes.into_iter().collect())
    }
}

/// Private registry: which route index is currently built, and the slot holding it.
#[derive(Component, Clone)]
pub(crate) struct RouterHandle {
    built: Option<(usize, Entity)>,
}

pub struct RouterSprout {
    leaf: LeafSprout,
    routes: Option<RouterRoutes>,
    route: usize,
}
impl RouterSprout {
    pub fn routes(mut self, routes: RouterRoutes) -> Self {
        self.routes = Some(routes);
        self
    }
    pub fn route(mut self, i: usize) -> Self {
        self.route = i;
        self
    }
}
impl Sprout for RouterSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        let routes = self.routes.expect("Router::routes(..) is required");
        let count = routes.0.len();
        (
            Router {},
            PageIndex(self.route.min(count.saturating_sub(1))),
            PageCount(count),
            routes,
            Grid::default(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        tree.write_to(this, RouterHandle { built: None });

        // structure: a RouterRoutes rewrite rebuilds the current scene from scratch --
        // there is no patch-in-place for a scene, teardown/rebuild IS the semantic.
        tree.react::<RouterRoutes, _>(
            this,
            move |trigger: Trigger<Insert, RouterRoutes>,
                  routes: Query<&RouterRoutes>,
                  page_index: Query<&PageIndex>,
                  mut handles: Query<&mut RouterHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let set = routes.get(e).unwrap().clone();
                // kept in sync with the actual route count on every rewrite, for the same
                // stale-clamp reason Tabs writes PageCount in its structure reaction.
                tree.write_to(e, PageCount(set.0.len()));
                let mut handle = handles.get_mut(e).unwrap();
                if let Some((_, slot)) = handle.built.take() {
                    tree.remove(slot);
                }
                if set.0.is_empty() {
                    return;
                }
                let current = page_index
                    .get(e)
                    .unwrap()
                    .0
                    .min(set.0.len().saturating_sub(1));
                handle.built = Some(build_route(&mut tree, e, &set, current));
            },
        );
        // navigation: destroy the departing scene, build the arriving one. The equality
        // guard makes rewriting the same index a no-op rather than a rebuild -- and at
        // spawn, where the structure reaction has already built the initial route, this
        // fires only the initial PageChanged.
        tree.react::<PageIndex, _>(
            this,
            move |trigger: Trigger<Insert, PageIndex>,
                  routes: Query<&RouterRoutes>,
                  page_index: Query<&PageIndex>,
                  counts: Query<&PageCount>,
                  mut handles: Query<&mut RouterHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let count = counts.get(e).unwrap().0;
                if count == 0 {
                    return;
                }
                let current = page_index.get(e).unwrap().0.min(count.saturating_sub(1));
                let mut handle = handles.get_mut(e).unwrap();
                if handle.built.map(|(i, _)| i) != Some(current) {
                    if let Some((_, slot)) = handle.built.take() {
                        tree.remove(slot);
                    }
                    let set = routes.get(e).unwrap().clone();
                    handle.built = Some(build_route(&mut tree, e, &set, current));
                }
                tree.trigger_targets(PageChanged::new(current), e);
            },
        );
    }
}

/// One fresh full-size slot (the same transient-slot convention as every composite), with
/// the route's builder run against it.
fn build_route(tree: &mut Tree, root: Entity, routes: &RouterRoutes, index: usize) -> (usize, Entity) {
    let slot = tree.branch(
        root,
        crate::Leaf::sprout()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::default()),
    );
    (routes.0[index])(tree, slot);
    (index, slot)
}
