//! Nested `Panel`s at different opacity and elevation -- the surface no test fully covers on
//! its own, since correct numbers don't guarantee it *looks* right. Run with
//! `cargo run --example opacity_and_elevation -p foliage`.

use foliage::{Bloom, Canopy, Color, Elevation, Foliage, Grid, GridExt, Location, Panel, Root};
use foliage::{Grows, Sprout};

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((300, 200));
    foliage.root::<Stack>();
    foliage.photosynthesize();
}

/// Nothing to keep: the panels are grown once and never change again.
struct Stack;

impl Root for Stack {
    fn take_root(canopy: &mut Canopy) -> Self {
        // Three overlapping panels, each further forward and more transparent than the last
        // -- correct blending reads as a soft stack, not a hard-edged collage.
        let base = canopy.leaf(
            Panel::new()
                .color(Color::orange(700))
                .at(Location::new().xs(
                    20.px().as_left().with(140.px().as_width()),
                    20.px().as_top().with(140.px().as_height()),
                ))
                .elevate(Elevation::abs(0))
                .opacity(1.0)
                // Anything with children needs a grid of its own: a child's location always
                // resolves against its parent's, whether or not it is expressed in percent.
                .grid(Grid::default()),
        );
        canopy.branch(
            base,
            Panel::new()
                .color(Color::green(500))
                .at(Location::new().xs(
                    50.px().as_left().with(140.px().as_width()),
                    50.px().as_top().with(140.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .opacity(0.6),
        );
        canopy.branch(
            base,
            Panel::new()
                .color(Color::gray(200))
                .at(Location::new().xs(
                    80.px().as_left().with(140.px().as_width()),
                    80.px().as_top().with(140.px().as_height()),
                ))
                .elevate(Elevation::up(2))
                .opacity(0.6),
        );
        Stack
    }
    fn frame(&mut self, _canopy: &mut Canopy, _blooms: Vec<Bloom>) {}
}
