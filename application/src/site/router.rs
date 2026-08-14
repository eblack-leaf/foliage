//! Destructive section switching, owned by the app rather than the library.
//!
//! Exactly one route's content exists at a time: navigating tears the current scene's
//! subtree down and runs the next route's builder against a fresh full-size slot. Hidden
//! scenes never exist, so nothing ticks in the background and a route entered twice is
//! built from nothing both times.
//!
//! Navigation is [`Site::navigate`](crate::entry::Site::navigate) and nothing richer. It also
//! rebuilds the rail and re-evaluates whether the drawer control applies, because "which
//! section is showing" is the only thing any of the three depend on.

use foliage::{Bare, Elevation, Grid, GridExt, Grows, Leaf, Location, Sprout};

use crate::site;
use crate::site::Grow;

/// A route's scene builder. Deliberately a bare `fn`, not a closure: a bare `fn` cannot
/// capture, so a scene can never close over another scene's live leaves -- the mistake that
/// makes destructive switching unsafe becomes a compile error instead of a runtime mystery.
pub(crate) type RouteFn = fn(&mut Grow, Leaf);

/// Route 0 is the hero: full screen, no rail. Everything after it is a section the rail
/// lists, which is why [`rail::SECTIONS`](site::rail::SECTIONS) is indexed from 1.
///
/// This list and that one have to stay the same length. They were not: the rail listed eight
/// sections against seven routes, and `assets` -- the last entry -- was clamped by
/// [`Site::navigate`](crate::entry::Site::navigate) onto the last route that did exist, so the
/// rail's own entry and the overview card pointing at it both quietly opened `text` instead.
pub(crate) const ROUTES: [RouteFn; 9] = [
    site::hero::build,
    site::overview::build,
    site::leaf::build,
    site::layout::build,
    site::renderers::build,
    site::anim::build,
    site::input::build,
    site::text::build,
    site::assets::build,
];

/// One fresh full-size slot, with the route's builder run against it.
///
/// The slot is what a navigation prunes, so a route's whole scene has exactly one root and
/// tearing it down cannot leave anything behind.
pub(crate) fn build_route(g: &mut Grow, router: Leaf, index: usize) -> Leaf {
    let slot = g.canopy.branch(
        router,
        Bare::new()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .grid(Grid::default()),
    );
    (ROUTES[index])(g, slot);
    slot
}
