//! `TextInput`'s styling surface, and reading what was typed.
//!
//! The boxes are sized in percent of the viewport, and the window is resizable, so dragging
//! its edge grows and shrinks them -- useful for checking that scroll position holds across a
//! resize. Type into either box and the title reports what the multi-line one contains, which
//! is the whole read path: an emission says *something changed*, a sample says *what it is*.
//! Run with `cargo run --example text_input -p foliage`.

use foliage::{
    Bloom, Canopy, Color, Elevation, Foliage, FontSize, GridExt, HorizontalAlignment, Leaf,
    LineConstraint, Location, Rounding, Text, TextInput, VerticalAlignment,
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
    foliage.photosynthesize(move |canopy: &mut Canopy| {
        let boxes = boxes.get_or_insert_with(|| grow(canopy, dejavu));
        for bloom in canopy.take() {
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

    // Single line, given a box taller than one line: `Single` stretches its text to the
    // field's full height, so this is where vertical placement shows.
    canopy.leaf(
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
            .font(dejavu),
    );

    let multi = canopy.leaf(
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

    Boxes { title, multi }
}
