//! `TextInput`'s styling surface -- hint text, foreground/background/accent, rounding,
//! outline. Config reused from `application/src/portfolio/composites.rs`. The box is
//! sized in percent of the viewport (not fixed px), and the window is resizable by
//! default, so dragging the OS window's edge actually grows/shrinks the box -- useful
//! for testing scroll-position behavior across a resize: type/paste enough multi-line
//! text to scroll away from the top, then resize the window and see whether the scroll
//! position holds or jumps back toward the cursor. Run with
//! `cargo run --example text_input -p foliage`.

use foliage::{
    Color, EcsExtension, Elevation, Foliage, GridExt, Location, Rounding, Sprout, TextInput,
};
use foliage_proper::LineConstraint;

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((280, 500));

    // A second monospace face, to see a registered font actually take effect. Registration
    // is startup-only and rejects anything proportional -- the whole layout addresses text
    // by a fixed character cell.
    let dejavu = foliage.font(include_bytes!("DejaVuSansMono.ttf").as_slice());

    // Single line, deliberately given a box taller than one line of text: `Single` stretches
    // its text to the field's full height, so this is where vertical placement shows. The
    // glyphs should sit centered in the box, not hugging an edge.
    foliage.world.leaf(
        TextInput::new()
            .line_constraint(LineConstraint::Single)
            .hint_text("single line -- dejavu")
            .foreground(Color::gray(200))
            .background(Color::gray(800))
            .accent(Color::green(600))
            .rounding(Rounding::None)
            .at(Location::new().xs(
                20.px().as_left().with(90.pct().as_right()),
                30.px().as_top().with(44.px().as_height()),
            ))
            .elevate(Elevation::up(1))
            .with(dejavu),
    );

    foliage.world.leaf(
        TextInput::new()
            .line_constraint(LineConstraint::Multiple)
            .hint_text("type here...")
            .foreground(Color::gray(200))
            .background(Color::gray(800))
            .accent(Color::green(600))
            .rounding(Rounding::None)
            .at(Location::new().xs(
                20.px().as_left().with(90.pct().as_right()),
                90.px().as_top().with(90.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    foliage.photosynthesize();
}
