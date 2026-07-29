//! The persistent frame: the navigation rail, and the scroll container routes draw into.
//!
//! Both live *outside* the router's subtree, so switching sections tears down content
//! without rebuilding the frame around it.

use foliage::{
    Anchor, Color, EcsExtension, Elevation, Entity, Grid, GridExt, Leaf, Location, Panel, Rounding,
    Sprout, Tree, ValueDescriptor, anchor,
};

use crate::site::{role, space};

/// Rail width on `md`+. Narrow enough to be chrome, wide enough for a label.
pub(crate) const RAIL_W: i32 = 148;
/// Prose never runs wider than this -- long lines are hard to read, and a docs page on a
/// 1600px monitor should not span it.
pub(crate) const MEASURE_MAX: i32 = 720;

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
            .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
    )
}

/// The measured column inside the scroll container -- where prose and cards go.
///
/// The inset lives here rather than on the container so full-bleed content (the hero) can
/// span the viewport while text stays readable. `md`+ clears the rail and caps the measure;
/// `xs` just takes a margin, since the rail is a drawer there.
///
/// `below` stacks this under something already in the container -- the hero. Both are
/// children of the same scroll container, so without it they share `top: 0` and render on
/// top of each other.
pub(crate) fn measured_column(tree: &mut Tree, container: Entity, below: Option<Entity>) -> Entity {
    let top = |height: ValueDescriptor| match below {
        Some(_) => anchor().bottom().as_top().with(height),
        None => 0.px().as_top().with(height),
    };
    let column = tree.branch(
        container,
        Leaf::sprout()
            .at(Location::new()
                .xs(
                    space::MD
                        .px()
                        .as_left()
                        .with(100.pct().as_right().adjust(-space::MD)),
                    top(100.pct().as_height()),
                )
                .md(
                    (RAIL_W + space::XL)
                        .px()
                        .as_left()
                        .with(MEASURE_MAX.px().as_width()),
                    top(100.pct().as_height()),
                ))
            .elevate(Elevation::up(1))
            .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
    );
    if let Some(anchor_to) = below {
        tree.write_to(column, Anchor::new(anchor_to));
    }
    column
}

/// The rail's own surface -- a full-height panel behind the section entries.
pub(crate) fn rail_surface(tree: &mut Tree, parent: Entity) -> Entity {
    tree.branch(
        parent,
        Panel::new()
            .color(Color::slate(role::SURFACE))
            .rounding(Rounding::None)
            .at(Location::new().xs(
                0.px().as_left().with(100.pct().as_right()),
                0.px().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(6))
            .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
    )
}
