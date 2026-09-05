//! A path, written as a chain of [`Line`]s.
//!
//! foliage has no `Polyline`. A path is a chain of [`Line`]s meeting end to end, each one its own
//! element with its own instance and its own blend, and nothing joins them. [`Cap::Round`] is what
//! closes the chain: each end is a half-disc of half the weight centred on the point, so the disc
//! one stroke puts at a shared end is exactly the wedge two rectangles leave open on the outside of
//! the turn. What it costs is that both ends of the whole chain reach half a weight past where they
//! were told to stop, which is why it is the cap a path asks for and not the default.
//!
//! Three sections in one scrolling page:
//!
//! 1. **One join**, at six weights and five turn angles. A cell is two strokes sharing an end and
//!    nothing else, which is the join on its own at every angle it is drawn at.
//! 2. **Axis-aligned segments in a chain.** An axis-aligned stroke is put on whole device pixels so
//!    that a one-pixel rule is one lit row rather than two half-lit ones, which moves both of its
//!    ends and its weight. Alone that is correct; between two diagonals it is a segment that can
//!    stop meeting its neighbours and read at a different thickness.
//! 3. **A series**, at the size and the angles a page would actually draw one, and stated in the
//!    placement grammar so the whole chain follows the row it is in.
//!
//! ```sh
//! cargo run -p foliage --example polyline
//! ```

use foliage::{
    Area, Axes, Boxed, Cap, Foliage, FontSize, Grove, Grow, Leaf, Line, Location, Palette, Panel,
    Place, Point, Pollen, Root, Source, Stem, Text, content, left, top,
};

/// The weights the whole page is read at, in logical pixels. The last two are past anything a page
/// would draw a series in, and are here because an artifact invisible at 2px is obvious at 28.
const WEIGHTS: [f32; 6] = [1.0, 2.0, 4.0, 8.0, 16.0, 28.0];

/// The interior angles the join is read at, in degrees: from a turn barely worth the name to one
/// that folds the path back on itself.
const TURNS: [f32; 5] = [150.0, 120.0, 90.0, 60.0, 30.0];

/// The margin around the page, and the rhythm everything else is stated in.
const MARGIN: f32 = 24.0;

/// The width of the column of row labels, which every section indents past.
const LABELS: f32 = 64.0;

/// One cell of the join matrix, and how long each arm of the join inside it runs.
const CELL_W: f32 = 152.0;
const CELL_H: f32 = 104.0;
const ARM: f32 = 40.0;

/// The width the chains are drawn across, which is the matrix's own width.
const ROW_W: f32 = CELL_W * TURNS.len() as f32;

/// How tall one row of each chain section is.
const CHAIN_H: f32 = 132.0;
const SERIES_H: f32 = 76.0;

/// The space a heading takes, and the space between sections.
const HEADING: f32 = 34.0;
const SECTION: f32 = 28.0;

/// The inset from a row's own edges, so a heavy stroke has somewhere to be.
const PAD: f32 = 16.0;

/// A chain that turns through three axis-aligned segments: one horizontal, one vertical, one
/// horizontal again, each between two diagonals. Stated in the row's own pixels.
const AXIAL: [(f32, f32); 8] = [
    (10.0, 104.0),
    (90.0, 30.0),
    (250.0, 30.0),
    (330.0, 104.0),
    (330.0, 24.0),
    (430.0, 74.0),
    (590.0, 74.0),
    (700.0, 24.0),
];

/// The readings the series is drawn from, as fractions of the row's height.
const SERIES: [f32; 9] = [0.30, 0.62, 0.18, 0.74, 0.44, 0.90, 0.22, 0.55, 0.38];

fn main() {
    tracing_subscriber::fmt().init();
    let mut foliage = Foliage::new();
    foliage.title("foliage -- paths");
    foliage.app_id("foliage");
    foliage.desktop_size(Area::new(
        MARGIN * 2.0 + LABELS + ROW_W,
        MARGIN * 2.0 + 760.0,
    ));
    foliage.root::<Paths>();
    foliage.photosynthesize();
}

/// A page that is drawn once and never written to again: every element is planted in
/// [`take_root`](Root::take_root) and nothing here reads a frame.
struct Paths;

impl Root for Paths {
    fn take_root(grove: &mut Grove) -> Self {
        // The ground, and what scrolls. The page is longer than any window it is opened in, and a
        // region scrolls because it said so rather than because its children overflowed.
        let page = grove.plant(Panel::new().scrolls(Axes::Vertical));

        let mut y = MARGIN;
        y = heading(grove, page, y, "one join -- weight down, turn across");
        y = matrix(grove, page, y) + SECTION;
        y = heading(grove, page, y, "axis-aligned segments between diagonals");
        y = chains(grove, page, y) + SECTION;
        heading(grove, page, y, "a series, at the size a page draws one");
        series(grove, page, y + HEADING);

        Self
    }

    fn frame(&mut self, _grove: &mut Grove, _pollen: Pollen) {}
}

/// One cell per weight and turn: two strokes sharing an end, and nothing over the join.
///
/// The vertex sits at the foot of the cell and the arms run up from it, so the turn opens toward
/// the reader at every angle and the join is always on the near side of the stroke.
fn matrix(grove: &mut Grove, page: Leaf, top_y: f32) -> f32 {
    for (column, turn) in TURNS.iter().enumerate() {
        label(
            grove,
            page,
            MARGIN + LABELS + column as f32 * CELL_W + PAD,
            top_y,
            format!("{turn:.0} deg"),
        );
    }

    let first = top_y + HEADING;
    for (row, weight) in WEIGHTS.iter().enumerate() {
        let row_y = first + row as f32 * CELL_H;
        label(
            grove,
            page,
            MARGIN,
            row_y + CELL_H / 2.0,
            format!("{weight:.0} px"),
        );
        for (column, turn) in TURNS.iter().enumerate() {
            let cell = grove.branch(
                page,
                Stem::new().at(Location::new().xs(
                    left((MARGIN + LABELS + column as f32 * CELL_W).px()).width(CELL_W.px()),
                    top(row_y.px()).height(CELL_H.px()),
                )),
            );
            join(grove, cell, *turn, *weight);
        }
    }
    first + WEIGHTS.len() as f32 * CELL_H
}

/// Two strokes meeting at one point, `turn` degrees apart, drawn in the cell's own pixels.
fn join(grove: &mut Grove, cell: Leaf, turn: f32, weight: f32) {
    let half = turn.to_radians() / 2.0;
    let (vertex_x, vertex_y) = (CELL_W / 2.0, CELL_H - PAD);
    let (across, up) = (ARM * half.sin(), ARM * half.cos());
    for arm in [-1.0, 1.0] {
        stroke(
            grove,
            cell,
            weight,
            (vertex_x, vertex_y),
            (vertex_x + arm * across, vertex_y - up),
        );
    }
}

/// The axial chain, at three weights: one row each, and the same seven segments in every row.
fn chains(grove: &mut Grove, page: Leaf, top_y: f32) -> f32 {
    for (row, weight) in [4.0, 12.0, 24.0].iter().enumerate() {
        let row_y = top_y + row as f32 * CHAIN_H;
        label(
            grove,
            page,
            MARGIN,
            row_y + CHAIN_H / 2.0,
            format!("{weight:.0} px"),
        );
        let strip = grove.branch(
            page,
            Stem::new().at(Location::new().xs(
                left((MARGIN + LABELS).px()).width(ROW_W.px()),
                top(row_y.px()).height(CHAIN_H.px()),
            )),
        );
        for pair in AXIAL.windows(2) {
            stroke(grove, strip, *weight, pair[0], pair[1]);
        }
    }
    top_y + 3.0 * CHAIN_H
}

/// The series, at every weight the matrix is read at.
///
/// Each reading is a point in the placement grammar rather than a number worked out here, so the
/// whole chain follows the row it is in: `x` is a fraction of the row's width and the strokes
/// between the readings stretch with it.
fn series(grove: &mut Grove, page: Leaf, top_y: f32) -> f32 {
    for (row, weight) in WEIGHTS.iter().enumerate() {
        let row_y = top_y + row as f32 * SERIES_H;
        label(
            grove,
            page,
            MARGIN,
            row_y + SERIES_H / 2.0,
            format!("{weight:.0} px"),
        );
        let strip = grove.branch(
            page,
            Stem::new().at(Location::new().xs(
                left((MARGIN + LABELS).px()).width(ROW_W.px()),
                top(row_y.px()).height(SERIES_H.px()),
            )),
        );
        let reading = |n: usize| {
            let along = n as f32 / (SERIES.len() - 1) as f32;
            let height = SERIES_H - 2.0 * PAD;
            Point::new(
                PAD.px() + (100.pct() - (2.0 * PAD).px()) * along,
                ((SERIES_H - PAD) - height * SERIES[n]).px(),
            )
        };
        for n in 0..SERIES.len() - 1 {
            grove.branch(
                strip,
                Line::new()
                    .weight(*weight)
                    .cap(Cap::Round)
                    .color(Palette::Accent)
                    .between(reading(n), reading(n + 1)),
            );
        }
    }
    top_y + WEIGHTS.len() as f32 * SERIES_H
}

/// One segment of a chain, in its trunk's own pixels.
fn stroke(grove: &mut Grove, trunk: Leaf, weight: f32, from: (f32, f32), to: (f32, f32)) {
    grove.branch(
        trunk,
        Line::new()
            .weight(weight)
            .cap(Cap::Round)
            .color(Palette::Accent)
            .between(
                Point::new(from.0.px(), from.1.px()),
                Point::new(to.0.px(), to.1.px()),
            ),
    );
}

/// A section's title, and how far down the page it leaves the next thing.
fn heading(grove: &mut Grove, page: Leaf, y: f32, title: &str) -> f32 {
    grove.branch(
        page,
        Text::new(title)
            .color(Palette::Ink)
            .font_size(FontSize::new().xs(15))
            .at(Location::new().xs(
                left(MARGIN.px()).width(content()),
                top(y.px()).height(content()),
            )),
    );
    y + HEADING
}

/// A label for a row or a column, drawn where it is asked for.
fn label(grove: &mut Grove, page: Leaf, x: f32, y: f32, text: String) {
    grove.branch(
        page,
        Text::new(text)
            .color(Palette::Muted)
            .font_size(FontSize::new().xs(12))
            .at(Location::new().xs(left(x.px()).width(content()), top(y.px()).height(content()))),
    );
}
