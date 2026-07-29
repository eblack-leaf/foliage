use crate::site;
use foliage::{
    Branch, EcsExtension, Elevation, Entity, Foliage, Grid, GridExt, InteractionPropagation, Leaf,
    Location, PageChanged, Query, RouteFn, Router, RouterRoutes, Sprout, Tree, Trigger, component,
};

/// Tags the app's one-and-only `Router`, so anything that needs to find it can query for
/// this rather than for `Router` (which only means "this entity is *a* router").
#[component]
#[derive(Copy, Clone)]
pub(crate) struct AppRouter;

/// Holds the rail. Lives outside the router's subtree, so switching sections tears down
/// content without rebuilding the frame -- and gives the rail one parent to clear when the
/// active entry changes.
#[component]
#[derive(Copy, Clone)]
pub(crate) struct RailHost;

pub fn build(foliage: &mut Foliage) {
    let router = foliage.world.leaf(
        Router::new()
            .routes(RouterRoutes::new([
                site::overview::build as RouteFn,
                site::stub::layout as RouteFn,
                site::stub::motion as RouteFn,
                site::stub::composites as RouteFn,
                site::stub::text as RouteFn,
            ]))
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with(AppRouter),
    );

    // Sized to the rail's own footprint, never the whole screen. A full-screen host sat on
    // top of the content: `Leaf` grabs interaction by default so every click died in it,
    // and its `Grid` brought a `View`, so dragging anywhere scrolled the host and slid the
    // off-canvas rail in. Off-canvas on `xs` means it covers nothing there at all.
    let host = foliage.world.leaf(
        Leaf::sprout()
            .at(Location::new()
                .xs(
                    (-(site::shell::RAIL_W + 16))
                        .px()
                        .as_left()
                        .with(site::shell::RAIL_W.px().as_width()),
                    0.pct().as_top().with(100.pct().as_bottom()),
                )
                .md(
                    0.px().as_left().with(site::shell::RAIL_W.px().as_width()),
                    0.pct().as_top().with(100.pct().as_bottom()),
                ))
            .elevate(Elevation::up(5))
            .with((
                RailHost,
                // the rail surface positions itself against this, so it needs a grid even
                // though nothing addresses a cell
                Grid::new(1.col().gap(0), 1.row().gap(0)),
                // never swallow a click meant for the content behind it
                InteractionPropagation::pass_through(),
            )),
    );
    site::rail::build(&mut foliage.world.commands(), host, router, 0);

    // The rail is respawned on a route change rather than diffed -- five labels and an
    // indicator is cheaper to rebuild than to reconcile, and it keeps "which entry is
    // active" a build-time input instead of live state to keep in sync.
    foliage.world.commands().subscribe(
        router,
        move |trigger: Trigger<PageChanged>, branches: Query<&Branch>, mut tree: Tree| {
            let index = trigger.event().index;
            if let Ok(branch) = branches.get(host) {
                let existing = branch.ids.iter().copied().collect::<Vec<Entity>>();
                tree.remove(existing);
            }
            site::rail::build(&mut tree, host, router, index);
        },
    );
}
