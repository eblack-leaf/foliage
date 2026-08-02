//! Destructive section switching, owned by the app rather than the library.
//!
//! Exactly one route's content exists at a time: navigating tears the current scene's
//! subtree down and runs the next route's builder against a fresh full-size slot. Hidden
//! scenes never exist, so nothing ticks in the background and a route entered twice is
//! built from nothing both times.
//!
//! Navigation is `tree.write_to(router, Route(i))` and nothing richer. The reaction that
//! answers it also rebuilds the rail and re-evaluates whether the drawer control applies,
//! because "which section is showing" is the only thing any of the three depend on.

use foliage::{
    Children, Elevation, Entity, EntityEvent, Grid, GridExt, Insert, Location, Stem,
    Query, Sprout, Tree, Trigger, component,
};

use crate::site;
use crate::site::drawer::Controls;

/// A route's scene builder. Deliberately a bare `fn`, not a closure: a bare `fn` cannot
/// capture, so a scene can never close over another scene's live entities -- the mistake
/// that makes destructive switching unsafe becomes a compile error instead of a runtime
/// mystery.
pub(crate) type RouteFn = fn(&mut Tree, Entity);

/// Route 0 is the hero: full screen, no rail. Everything after it is a section the rail
/// lists, which is why `rail::SECTIONS` is indexed from 1.
pub(crate) const ROUTES: [RouteFn; 7] = [
    site::hero::build,
    site::overview::build,
    site::stub::leaf,
    site::stub::layout,
    site::stub::motion,
    site::stub::composites,
    site::stub::text,
];

/// Which section is showing. The navigation channel: write it, everything follows.
#[component]
#[derive(Copy, Clone)]
pub(crate) struct Route(pub usize);

/// Which slot currently holds the built scene, so the next navigation knows what to tear
/// down. `None` only before the first build.
#[component]
#[derive(Copy, Clone, Default)]
pub(crate) struct Built(Option<(usize, Entity)>);

/// Wires the navigation reaction. Called once `host` and `controls` exist, since the
/// reaction rebuilds both; the build-time re-fire lands the initial route as a side effect.
pub(crate) fn attach(tree: &mut Tree, router: Entity, host: Entity, controls: Controls) {
    tree.write_to(router, Built::default());
    tree.react::<Route, _>(
        router,
        move |trigger: Trigger<Insert, Route>,
              routes: Query<&Route>,
              branches: Query<&Children>,
              mut built: Query<&mut Built>,
              mut tree: Tree| {
            let this = trigger.event_target();
            let index = routes.get(this).unwrap().0.min(ROUTES.len() - 1);
            let mut handle = built.get_mut(this).unwrap();
            if handle.0.map(|(i, _)| i) == Some(index) {
                return;
            }
            if let Some((_, slot)) = handle.0.take() {
                tree.remove(slot);
            }
            handle.0 = Some((index, build_route(&mut tree, this, index)));

            // The rail is respawned on a route change rather than diffed -- five labels and
            // an indicator is cheaper to rebuild than to reconcile, and it keeps "which
            // entry is active" a build-time input instead of live state to keep in sync.
            if let Ok(branch) = branches.get(host) {
                let existing = branch.ids.iter().copied().collect::<Vec<Entity>>();
                tree.remove(existing);
            }
            // route 0 is the hero and has no rail; sections start at 1
            let section = index.checked_sub(1);
            site::rail::build(&mut tree, host, this, section);
            site::drawer::set_available(&mut tree, host, controls, section.is_some());
        },
    );
}

/// One fresh full-size slot, with the route's builder run against it.
fn build_route(tree: &mut Tree, router: Entity, index: usize) -> Entity {
    let slot = tree.branch(
        router,
        Stem::new()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::default()),
    );
    (ROUTES[index])(tree, slot);
    slot
}
