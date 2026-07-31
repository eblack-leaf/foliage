use crate::site;
use foliage::{
    Branch, EcsExtension, Elevation, Entity, Foliage, GridExt, Location, PageChanged, Query,
    RouteFn, Router, RouterRoutes, Sprout, Tree, Trigger, component,
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
            // route 0 is the hero: full screen, no rail. Everything after it is a section
            // the rail lists, which is why `rail::SECTIONS` is indexed from 1.
            .routes(RouterRoutes::new([
                site::hero::build as RouteFn,
                site::overview::build as RouteFn,
                site::stub::leaf as RouteFn,
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

    // Sized to the rail's own footprint, never the whole screen -- a full-screen host sat on
    // top of the content, and `Leaf` grabs interaction by default, so every click died in
    // it. Off-canvas on `xs` until the drawer brings it in.
    site::probe::build(&mut foliage.world.commands());
    let host = site::drawer::host(&mut foliage.world.commands());
    let controls = site::drawer::build(&mut foliage.world.commands(), host);
    // nothing on the hero route -- the rail, and the button that opens it, appear once you
    // are inside the site
    site::rail::build(&mut foliage.world.commands(), host, router, None);
    site::drawer::set_available(&mut foliage.world.commands(), host, controls, false);

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
            // route 0 is the hero and has no rail; sections start at 1
            let section = index.checked_sub(1);
            site::rail::build(&mut tree, host, router, section);
            site::drawer::set_available(&mut tree, host, controls, section.is_some());
        },
    );
}
