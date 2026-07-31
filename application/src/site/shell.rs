//! The persistent frame: the navigation rail, and the scroll container routes draw into.
//!
//! Both live *outside* the router's subtree, so switching sections tears down content
//! without rebuilding the frame around it.

use foliage::{
    ConfigurationDescriptor, EcsExtension, Elevation, Entity, Grid, GridExt, Leaf, Location, Panel,
    Rounding, Sprout, Tree,
};

use crate::site::{role, space};

/// Rail width on `md`+. Narrow enough to be chrome, wide enough for a label.
pub(crate) const RAIL_W: i32 = 148;
/// Prose never runs wider than this -- long lines are hard to read, and a docs page on a
/// 1600px monitor should not span it.
pub(crate) const MEASURE_MAX: i32 = 720;
/// One step wider than the reading measure -- enough that a drawing visibly breaks out of the
/// text block, not so much that it becomes a letterbox strip across the window.
pub(crate) const FIGURE_MAX: i32 = 900;

/// The scroll container a route renders into: full viewport width, so a hero can bleed
/// edge to edge.
///
/// The `Grid` brings a `View` with it, and `extent_check` grows the scrollable range to
/// cover whatever the route puts inside -- nothing here needs to know how tall a page is.
pub(crate) fn content_area(tree: &mut Tree, parent: Entity) -> Entity {
    tree.branch(
        parent,
        Leaf::sprout()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.px().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with((
                Grid::new(1.col().gap(0), 1.row().gap(0)),
                crate::site::probe::Scroller,
            )),
    )
}

/// How wide an element in the content column runs, at `xs` and at `md`+.
///
/// This used to be a box: a measured column inside the scroll container, with everything
/// written into it. A box needs a `Grid`, a `Grid` brings a `View`, and a second view inside
/// the scroll container is what stopped the page scrolling when the pointer sat in the side
/// gutters -- the wheel walks up to the *first* view under it, found the inner one over the
/// text and the outer one (with no extent of its own) over the gutters. So the measure is a
/// pair of descriptors applied per element instead, and the container is the only view.
///
/// `md`+ clears the rail and caps the measure; `xs` just takes a margin, since the rail is a
/// drawer there. Fluid up to the cap rather than a fixed width -- `max` keeps the column
/// filling a narrow window and only stops growing once prose would get too wide to read.
pub(crate) fn measure() -> (ConfigurationDescriptor, ConfigurationDescriptor) {
    capped(MEASURE_MAX)
}

/// The measure a figure runs at: the same insets, capped wider than the reading one.
///
/// Prose is capped because long lines are hard to read. A drawing has no such limit -- but it
/// does have a point past which more width is just a thinner drawing, and uncapped it simply
/// ran the whole window. So it breaks the measure by a step and stops.
///
/// Both caps centre in the same span, so a figure grows symmetrically past the paragraphs
/// around it rather than hanging off one side.
pub(crate) fn figure_measure() -> (ConfigurationDescriptor, ConfigurationDescriptor) {
    capped(FIGURE_MAX)
}

fn capped(max: i32) -> (ConfigurationDescriptor, ConfigurationDescriptor) {
    (
        space::MD
            .px()
            .as_left()
            .with(100.pct().as_right().adjust(-space::MD)),
        (RAIL_W + space::XL)
            .px()
            .as_left()
            .with(100.pct().as_right().adjust(-space::XL))
            .max(max as f32),
    )
}

/// Where the rail host sits, open or closed.
///
/// `xs` is the drawer: off-canvas by its own width plus the inset, so no sliver shows.
/// `md`+ ignores `open` entirely -- the rail is permanent there, and a menu button would be
/// a control for something already on screen.
pub(crate) fn rail_host_location(open: bool) -> Location {
    let closed_left = -(RAIL_W + space::SM * 2);
    Location::new()
        .xs(
            if open { 0 } else { closed_left }
                .px()
                .as_left()
                .with(RAIL_W.px().as_width()),
            0.pct().as_top().with(100.pct().as_bottom()),
        )
        .md(
            0.px().as_left().with(RAIL_W.px().as_width()),
            0.pct().as_top().with(100.pct().as_bottom()),
        )
}

/// The rail's own surface -- a panel behind the section entries.
///
/// Inset from the window on every side so it reads as a floating surface, which is what
/// makes the rounding visible at all: flush to the edges, three of the four corners are
/// offscreen. `Xs` because `Rounding` is proportional -- `Md` is half the short side, which
/// on a 148px rail is a 74px radius, i.e. a lozenge.
pub(crate) fn rail_surface(tree: &mut Tree, parent: Entity) -> Entity {
    tree.branch(
        parent,
        Panel::new()
            .color(role::surface())
            .rounding(Rounding::Xs)
            .at(Location::new().xs(
                space::SM
                    .px()
                    .as_left()
                    .with(100.pct().as_right().adjust(-space::SM)),
                space::SM
                    .px()
                    .as_top()
                    .with(100.pct().as_bottom().adjust(-space::SM)),
            ))
            .elevate(Elevation::up(6))
            .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
    )
}
