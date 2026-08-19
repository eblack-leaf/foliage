//! `TextInput`'s styling surface, and reading what was typed.
//!
//! The boxes are sized in percent of the viewport, and the window is resizable, so dragging
//! its edge grows and shrinks them -- useful for checking that scroll position holds across a
//! resize. Type into either box and the title reports what the multi-line one contains, which
//! is the whole read path: an emission says *something changed*, a sample says *what it is*.
//! Run with `cargo run --example text_input -p foliage`.

use foliage::{
    Bloom, Canopy, Color, Elevation, Foliage, FontSize, Grid, GridExt, HorizontalAlignment, Leaf,
    LineConstraint, Location, Panel, Rounding, Text, TextInput, VerticalAlignment,
};
use foliage::{Grows, Sprout};

struct Boxes {
    title: Leaf,
    multi: Leaf,
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((280, 500));

    // A second monospace face, to see a registered font take effect. Registration is
    // startup-only and rejects anything proportional -- the layout addresses text by a fixed
    // character cell.
    let dejavu = foliage.font(include_bytes!("DejaVuSansMono.ttf").as_slice());

    let mut boxes: Option<Boxes> = None;
    foliage.define_frame(move |canopy: &mut Canopy, blooms| {
        let boxes = boxes.get_or_insert_with(|| grow(canopy, dejavu));
        for bloom in blooms {
            // The emission carries the new value, but sampling would answer just as well --
            // and does, for anything that did not change this frame.
            if let Bloom::TextChanged { leaf, value } = bloom
                && leaf == boxes.multi
            {
                let count = value.chars().count();
                canopy.text(boxes.title, format!("{count} chars"));
            }
        }
    });
}

fn grow(canopy: &mut Canopy, dejavu: foliage::FontId) -> Boxes {
    let title = canopy.leaf(
        Text::new("0 chars")
            .size(FontSize::new(14))
            .color(Color::gray(400))
            .at(Location::new().xs(
                20.px().as_left().with(90.pct().as_right()),
                6.px().as_top().with(20.px().as_height()),
            ))
            .elevate(Elevation::up(1))
            .align(HorizontalAlignment::Left, VerticalAlignment::Middle),
    );

    // `TextInput` draws no backdrop of its own -- these are the example's, one behind each
    // field. `Rounding::None` needs no inset between backdrop and field: a square corner
    // doesn't clip a glyph sitting flush against it the way a rounded one would.
    let single_backdrop = canopy.leaf(
        Panel::new()
            .color(Color::gray(800))
            .rounding(Rounding::None)
            .at(Location::new().xs(
                20.px().as_left().with(90.pct().as_right()),
                30.px().as_top().with(44.px().as_height()),
            ))
            .elevate(Elevation::up(1))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            .pass_through(),
    );
    // Single line, given a box taller than one line: `Single` stretches its text to the
    // field's full height, so this is where vertical placement shows.
    canopy.branch(
        single_backdrop,
        TextInput::new()
            .line_constraint(LineConstraint::Single)
            .hint_text("single line -- dejavu")
            .foreground(Color::gray(200))
            .accent(Color::green(600))
            .at(Location::new().xs(
                4.px().as_left().with(100.pct().as_right().adjust(-4)),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .font(dejavu),
    );

    let multi_backdrop = canopy.leaf(
        Panel::new()
            .color(Color::gray(800))
            .rounding(Rounding::None)
            .at(Location::new().xs(
                20.px().as_left().with(90.pct().as_right()),
                90.px().as_top().with(90.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            .pass_through(),
    );
    let multi = canopy.branch(
        multi_backdrop,
        TextInput::new()
            .line_constraint(LineConstraint::Multiple)
            .hint_text("type here...")
            .foreground(Color::gray(200))
            .accent(Color::green(600))
            .at(Location::new().xs(
                4.px().as_left().with(100.pct().as_right().adjust(-4)),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    Boxes { title, multi }
}
