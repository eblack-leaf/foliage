//! `Panel` + `Polygon` (sides/rounding variants) + a plain `Line` -- the framework's
//! smallest possible visual vocabulary. Run with `cargo run --example basic_shapes -p foliage`.
//!
//! Nothing exists before the loop starts: the tree is grown on the first frame, from inside
//! the closure, which is the only place an app ever touches foliage.

use foliage::{
    Canopy, Color, Elevation, Foliage, GridExt, Line, Location, Panel, Polygon, Rounding,
};
use foliage::{Grows, Sprout};

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((420, 290));

    let mut grown = false;
    foliage.define_frame(move |canopy: &mut Canopy, blooms| {
        if grown {
            return;
        }
        grown = true;
        canopy.leaf(
            Panel::new()
                .rounding(Rounding::Sm)
                .color(Color::gray(700))
                .at(Location::new().xs(
                    20.px().as_left().with(80.px().as_width()),
                    20.px().as_top().with(80.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );

        // Sharp triangle -> lightly-rounded pentagon -> more-rounded hexagon -> full
        // rounding, where any side count converges to a true circle.
        let polygons: [(f32, f32); 4] = [(3.0, 0.0), (5.0, 0.15), (6.0, 0.4), (8.0, 1.0)];
        for (i, (sides, rounding)) in polygons.into_iter().enumerate() {
            let left = 120 + i as i32 * 70;
            canopy.leaf(
                Polygon::new()
                    .sides(sides)
                    .rounding(rounding)
                    .color(Color::gray(300))
                    .at(Location::new().xs(
                        left.px().as_left().with(60.px().as_width()),
                        20.px().as_top().with(60.px().as_height()),
                    ))
                    .elevate(Elevation::up(1)),
            );
        }
        canopy.leaf(
            Line::new(3)
                .color(Color::green(400))
                .at(Location::new().xs(
                    20.px().as_x().with(120.px().as_y()),
                    380.px().as_x().with(180.px().as_y()),
                ))
                .elevate(Elevation::up(1)),
        );
    });
    foliage.photosynthesize();
}
