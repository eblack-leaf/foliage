use crate::chapters;
use crate::chrome;
use crate::home::home;
use crate::navigator;
use crate::toc::toc;
use foliage::{
    EcsExtension, Elevation, Foliage, GridExt, Location, RouteFn, Router, RouterRoutes, Sprout,
    component,
};

/// Tags the app's one-and-only `Router` entity explicitly, so anywhere that needs to
/// find it (e.g. a route function that only gets its own `slot`, not `router`, since
/// `RouteFn = fn(&mut Tree, Entity)` has no room for a third parameter) can
/// `Query<Entity, With<AppRouter>>` instead of `Query<Entity, With<Router>>` -- `Router`
/// itself is just "this entity is *a* router" (a `foliage_proper` type, could in
/// principle describe more than one), where `AppRouter` means specifically "the one this
/// app actually built."
#[component]
#[derive(Copy, Clone)]
pub(crate) struct AppRouter;

pub fn build(foliage: &mut Foliage) {
    let router = foliage.world.leaf(
        Router::new()
            .routes(RouterRoutes::new([
                home as RouteFn,
                toc as RouteFn,
                chapters::location::build as RouteFn,
                chapters::elevate::build as RouteFn,
                chapters::relative::build as RouteFn,
                chapters::grid::build as RouteFn,
                chapters::anchor::build as RouteFn,
                chapters::animate::build as RouteFn,
                chapters::sequence::build as RouteFn,
                chapters::interact::build as RouteFn,
                chapters::text::build as RouteFn,
            ]))
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with(AppRouter),
    );

    // outside the router's subtree on purpose -- it survives every route switch and
    // drives navigation itself, rather than being torn down and rebuilt like scene
    // content.
    navigator::build(&mut foliage.world.commands(), router);
    // global, site-wide chrome (brand mark + github + home/ToC) -- also outside the
    // router's subtree, and elevated above the inner-site navigator so the two are always
    // visually distinct layers, never competing for the same space.
    chrome::build(&mut foliage.world.commands(), router);
}
