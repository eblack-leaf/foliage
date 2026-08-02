use crate::site;
use crate::site::router::Route;
use foliage::{
    Elevation, Foliage, Grid, GridExt, Stem, Location, Sprout, component,
};

/// Tags the app's one-and-only router, so anything that needs to find it can query for this
/// rather than walking the tree.
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
    let router = foliage.tree().leaf(
        Stem::new()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            // Route 0 is the hero. `router::attach` re-fires on it below, which is what
            // builds the opening screen -- there is no separate initial-build path.
            //
            // The `Grid` is what the route slot's own `Location` resolves against: every
            // scene is a full-bleed child of this entity, so without it the first route
            // spawned panics on a parent with no grid.
            .with((AppRouter, Route(0), Grid::default())),
    );

    // Sized to the rail's own footprint, never the whole screen -- a full-screen host sat on
    // top of the content, and `Stem` grabs interaction by default, so every click died in
    // it. Off-canvas on `xs` until the drawer brings it in.
    site::probe::build(&mut foliage.tree());
    let host = site::drawer::host(&mut foliage.tree());
    let controls = site::drawer::build(&mut foliage.tree(), host);
    // after `host` and `controls` exist, because the navigation reaction rebuilds both
    site::router::attach(&mut foliage.tree(), router, host, controls);
}
