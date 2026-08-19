//! Two panels that share an edge, reflowing as the window resizes: stacked at `xs`, side by
//! side from `md` up. Run with `cargo run --example responsive_split -p foliage` and drag the
//! window narrow and wide.
//!
//! The seam is the thing to watch. Both panels are sized in percentages, so at most window
//! widths their shared edge lands on a fractional pixel -- and `Section::rounded` has to snap
//! it the same way from both sides or a 1px gap opens between them. It rounds the four edges
//! and derives the extent from those, rather than rounding position and size independently
//! (which put the seam at `round(left) + round(width)` on one side and `round(left)` on the
//! other). Drag slowly through a range of widths: the seam should stay closed at every one.

use foliage::{
    Bloom, Canopy, Color, Elevation, Foliage, FontSize, GridExt, HorizontalAlignment, Location,
    Panel, Root, Rounding, Text, VerticalAlignment,
};
use foliage::{Grows, Sprout};

/// Where the two panels meet, as a percentage of the window. Deliberately not 50 -- an even
/// split lands on a whole pixel far too often to show anything.
const SEAM_PCT: f32 = 42.5;
const MARGIN_PCT: f32 = 6.0;
/// `xs` stacks, so the seam is horizontal instead and sits here.
const XS_SEAM_PCT: f32 = 47.5;
const TOP_PCT: f32 = 18.0;
const BOTTOM_PCT: f32 = 82.0;

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((720, 480));

    foliage.root::<Split>();
    foliage.photosynthesize();
}

/// Nothing to keep: both panels are stated in percentages, so the reflow is the grid's work
/// and not the app's.
struct Split;

impl Root for Split {
    fn take_root(canopy: &mut Canopy) -> Self {
        // Left at xs -> top when stacked. Ends exactly where the other begins, on both axes.
        canopy.leaf(
            Panel::new()
                .rounding(Rounding::None)
                .color(Color::cyan(600))
                .at(Location::new()
                    .xs(
                        MARGIN_PCT
                            .pct()
                            .as_left()
                            .with((100.0 - MARGIN_PCT).pct().as_right()),
                        TOP_PCT.pct().as_top().with(XS_SEAM_PCT.pct().as_bottom()),
                    )
                    .md(
                        MARGIN_PCT.pct().as_left().with(SEAM_PCT.pct().as_right()),
                        TOP_PCT.pct().as_top().with(BOTTOM_PCT.pct().as_bottom()),
                    ))
                .elevate(Elevation::up(1)),
        );

        // Right at xs -> bottom when stacked. Its leading edge is the *same* percentage the
        // panel above ends on, which is what makes this a seam test rather than two rectangles
        // that happen to be near each other.
        canopy.leaf(
            Panel::new()
                .rounding(Rounding::None)
                .color(Color::orange(600))
                .at(Location::new()
                    .xs(
                        MARGIN_PCT
                            .pct()
                            .as_left()
                            .with((100.0 - MARGIN_PCT).pct().as_right()),
                        XS_SEAM_PCT
                            .pct()
                            .as_top()
                            .with(BOTTOM_PCT.pct().as_bottom()),
                    )
                    .md(
                        SEAM_PCT
                            .pct()
                            .as_left()
                            .with((100.0 - MARGIN_PCT).pct().as_right()),
                        TOP_PCT.pct().as_top().with(BOTTOM_PCT.pct().as_bottom()),
                    ))
                .elevate(Elevation::up(1)),
        );

        canopy.leaf(
            Text::new("drag the window -- stacked under md, split at md and up")
                .size(FontSize::new(13))
                .color(Color::gray(500))
                .at(Location::new().xs(
                    MARGIN_PCT
                        .pct()
                        .as_left()
                        .with((100.0 - MARGIN_PCT).pct().as_right()),
                    6.0.pct().as_top().with(10.0.pct().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .align(HorizontalAlignment::Center, VerticalAlignment::Middle),
        );
        Split
    }
    fn frame(&mut self, _canopy: &mut Canopy, _blooms: Vec<Bloom>) {}
}
