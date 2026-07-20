//! `TextInput`'s styling surface -- hint text, foreground/background/accent, rounding,
//! outline. Config reused from `application/src/portfolio/composites.rs`. Run with
//! `cargo run --example text_input -p foliage`.

use foliage::{Color, EcsExtension, Elevation, Foliage, GridExt, Location, Rounding, Sprout, TextInput};
use foliage_proper::LineConstraint;

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((280, 500));

    foliage.world.leaf(
        TextInput::new().line_constraint(LineConstraint::Multiple)
            .hint_text("type here...")
            .foreground(Color::gray(200))
            .background(Color::gray(800))
            .accent(Color::green(600))
            .rounding(Rounding::None)
            .outline(3)
            .at(Location::new().xs(
                20.px().as_left().with(240.px().as_width()),
                30.px().as_top().with(400.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );

    foliage.photosynthesize();
}
