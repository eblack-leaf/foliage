//! Every `Line` weight from `MIN_LINE_WEIGHT` up, at the three angles that stress the
//! renderer differently. Run with `cargo run --example line_weights -p foliage`.
//!
//! The interesting rows are the first two. A 1px line is a single device pixel of ink: the
//! horizontal one lands on whole pixels and reads crisp, the shallow diagonal spreads that
//! same ink across two rows wherever it crosses between them and reads softer -- but it stays
//! *continuous*, and its apparent weight does not rise and fall along its own length. That
//! last part is what the floor of 3 used to exist for.

use foliage::{
    Canopy, Color, Elevation, Foliage, FontSize, GridExt, HorizontalAlignment, Line, Location,
    Text, VerticalAlignment,
};
use foliage::{Grows, Sprout};

/// Progressive rather than every integer: the difference between 1 and 2 is the whole
/// question, and the difference between 8 and 9 is nothing at all.
const WEIGHTS: [i32; 6] = [1, 2, 3, 4, 6, 8];

const ROW_H: i32 = 52;
const TOP: i32 = 16;
const GUTTER: i32 = 40;
/// Each angle's column, as (left, width, rise). Flat, then 1-in-6 -- shallow is where
/// sub-pixel coverage is most visible, and where a thin line used to break into dashes --
/// then 45 degrees, where a whole-pixel snap could not help even if it applied.
const COLUMNS: [(i32, i32, i32); 3] = [
    (GUTTER + 8, 150, 0),
    (GUTTER + 174, 150, 25),
    (GUTTER + 340, 36, 36),
];

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((460, TOP * 2 + ROW_H * WEIGHTS.len() as i32 + 24));

    let mut grown = false;
    foliage.photosynthesize(move |canopy: &mut Canopy| {
        if grown {
            return;
        }
        grown = true;

        for (i, weight) in WEIGHTS.into_iter().enumerate() {
            let row = TOP + i as i32 * ROW_H;
            let mid = row + ROW_H / 2;

            canopy.leaf(
                Text::new(format!("{weight}"))
                    .size(FontSize::new(14))
                    .color(Color::stone(400))
                    .at(Location::new().xs(
                        8.px().as_left().with(GUTTER.px().as_right()),
                        row.px().as_top().with(ROW_H.px().as_height()),
                    ))
                    .align(HorizontalAlignment::Right, VerticalAlignment::Middle)
                    .elevate(Elevation::up(1)),
            );

            // The flat column is the only one the CPU-side whole-pixel snap applies to; the
            // other two are the shader's feather alone.
            for (left, width, rise) in COLUMNS {
                canopy.leaf(
                    Line::new(weight)
                        .color(Color::stone(300))
                        .at(Location::new().xs(
                            left.px().as_x().with((mid + rise / 2).px().as_y()),
                            (left + width)
                                .px()
                                .as_x()
                                .with((mid - rise / 2).px().as_y()),
                        ))
                        .elevate(Elevation::up(1)),
                );
            }
        }
    });
}
