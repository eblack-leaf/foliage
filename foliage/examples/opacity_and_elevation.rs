//! Nested `Panel`s at different `Opacity`/`Elevation::up(n)` -- a visual counterpart to
//! the blend-math/stacking-order tests in `foliage_proper/tests/{opacity,elevation}.rs`,
//! since this is exactly the surface no test can fully cover on its own (correct numbers
//! don't guarantee it *looks* right). Run with
//! `cargo run --example opacity_and_elevation -p foliage`.

use foliage::{
    Color, EcsExtension, Elevation, Foliage, Grid, GridExt, Location, Opacity, Panel, Sprout,
};

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((300, 200));

    // three overlapping panels, each further in front and more transparent than the
    // last -- correct blending should read as a soft stack, not a hard-edged collage.
    let base = foliage.world.leaf(
        Panel::new()
            .color(Color::orange(700))
            .at(Location::new().xs(
                20.px().as_left().with(140.px().as_width()),
                20.px().as_top().with(140.px().as_height()),
            ))
            .elevate(Elevation::abs(0))
            // any entity that's going to have children with a real (non-empty) Location
            // needs `Grid` on itself, regardless of whether those children anchor with
            // px or pct/col/row -- resolving a child's Location always reads its
            // parent's Grid unconditionally, not just for percentage-relative anchors.
            .with((Opacity::new(1.0), Grid::default())),
    );
    foliage.world.branch(
        base,
        Panel::new()
            .color(Color::green(500))
            .at(Location::new().xs(
                50.px().as_left().with(140.px().as_width()),
                50.px().as_top().with(140.px().as_height()),
            ))
            .elevate(Elevation::up(1))
            .with(Opacity::new(0.6)),
    );
    foliage.world.branch(
        base,
        Panel::new()
            .color(Color::gray(200))
            .at(Location::new().xs(
                80.px().as_left().with(140.px().as_width()),
                80.px().as_top().with(140.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with(Opacity::new(0.6)),
    );

    foliage.photosynthesize();
}
