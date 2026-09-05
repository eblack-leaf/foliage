//! What a theme is worth: one write, and every element carrying a tone follows.
//!
//! Nothing on this page is re-planted after it is grown. Every press queues a single
//! [`repaint`](foliage::Grow::repaint) with a whole [`Scheme`], and what moves is decided by what
//! each element declared -- which is the entire difference between a [`Palette`] tone and a
//! [`Color`] stated outright.
//!
//! Three things to press:
//!
//! - **The reading**, `dark` or `light`. The same six roles, seeded for the ground they are read
//!   against. Every step on the page inverts with it, because a step is named for where it
//!   stands relative to the ground rather than for which way it moves.
//! - **The accent**, one of five seeds. A scheme is stated in one color per role and derives the
//!   other four steps of each ramp from it, so this is one decision moving five colors -- and only
//!   the accent's row moves, because the other five roles were not restated.
//! - Nothing else. The swatches themselves are the only fills on the page written as literals, and
//!   they are the only things that do not move when the rest of it does.
//!
//! The grid is every role at every step. The strip beneath it is what the steps are for: a state is
//! a step on the role's own ramp rather than a color picked beside it, so it survives a repaint and
//! a change of reading without being restated.
//!
//! ```sh
//! cargo run -p foliage --example palette
//! ```

use foliage::{
    Area, Axes, Boxed, Color, Foliage, FontSize, Grove, Grow, Leaf, Location, Palette, Panel, Place,
    Polygon, Pollen, Root, Rounding, Scheme, Source, Step, Text, content, left, top,
};

/// The margin around the page, and the rhythm everything else is stated in.
const MARGIN: f32 = 24.0;

/// The width of the column of role names, which the grid indents past.
const LABELS: f32 = 92.0;

/// One cell of the role-by-step grid, and the polygon drawn in the middle of it.
const CELL_W: f32 = 104.0;
const CELL_H: f32 = 78.0;
const MARK: f32 = 52.0;

/// A button, and the swatch button that is square instead.
const BUTTON_W: f32 = 78.0;
const BUTTON_H: f32 = 32.0;
const SWATCH_W: f32 = 46.0;

/// The space a heading takes, and the space between sections.
const HEADING: f32 = 34.0;
const SECTION: f32 = 22.0;

/// How far the grid's own ground is inset past the cells it holds.
const CARD_PAD: f32 = 12.0;

/// One cell of the strip of states.
const STATE_W: f32 = 148.0;
const STATE_H: f32 = 72.0;

/// Every role, at its base step, with what to call it.
const ROLES: [(&str, Palette); 6] = [
    ("surface", Palette::Surface),
    ("raised", Palette::Raised),
    ("muted", Palette::Muted),
    ("accent", Palette::Accent),
    ("ink", Palette::Ink),
    ("contrast", Palette::Contrast),
];

/// Every step of a ramp, deepest into the ground first.
const STEPS: [(&str, Step); 5] = [
    ("farthest", Step::Farthest),
    ("far", Step::Far),
    ("base", Step::Base),
    ("near", Step::Near),
    ("nearest", Step::Nearest),
];

/// What the steps are for, as the states an app actually writes.
const STATES: [(&str, fn(Palette) -> Palette); 3] = [
    ("disabled", Palette::recede),
    ("at rest", |role| role),
    ("hovered", Palette::advance),
];

/// The accent seeds the swatches offer. Each is one color, and each moves five.
const SEEDS: [(f32, f32, f32); 5] = [
    (0.38, 0.71, 0.51),
    (0.36, 0.60, 0.90),
    (0.62, 0.47, 0.92),
    (0.92, 0.68, 0.24),
    (0.90, 0.35, 0.42),
];

/// The width the whole page is laid out across.
const PAGE_W: f32 = MARGIN * 2.0 + LABELS + CELL_W * STEPS.len() as f32;

fn main() {
    tracing_subscriber::fmt().init();
    let mut foliage = Foliage::new();
    foliage.title("foliage -- palette");
    foliage.app_id("foliage");
    foliage.desktop_size(Area::new(PAGE_W, 792.0));
    foliage.root::<Palettes>();
    foliage.photosynthesize();
}

/// A panel that receives the press and the label drawn over it, which does not.
///
/// Both are kept because a chosen button restates two tones at once: what it is filled with, and
/// what is legible on top of that -- which is the whole reason [`Palette::Contrast`] is a role.
struct Button {
    panel: Leaf,
    label: Leaf,
}

impl Button {
    /// One button, with its label inset from its own left edge.
    fn grow(grove: &mut Grove, page: Leaf, x: f32, y: f32, width: f32, text: &str) -> Self {
        let panel = grove.branch(
            page,
            Panel::new()
                .color(Palette::Raised)
                .rounding(Rounding::Sm)
                .interactive()
                .at(Location::new().xs(
                    left(x.px()).width(width.px()),
                    top(y.px()).height(BUTTON_H.px()),
                )),
        );
        let label = grove.branch(
            page,
            Text::new(text)
                .color(Palette::Ink)
                .font_size(FontSize::new().xs(13))
                .pass_through()
                .at(Location::new().xs(
                    left((x + 12.0).px()).width(content()),
                    top((y + 9.0).px()).height(content()),
                )),
        );
        Self { panel, label }
    }

    /// States the two tones a button is drawn in: what it is filled with, and what is read on top.
    ///
    /// Both are tones, so this survives the repaint it is written beside -- a button marked in the
    /// accent is marked in whatever the accent has become.
    fn mark(&self, grove: &mut Grove, fill: Palette, ink: Palette) {
        grove.color(self.panel, fill);
        grove.color(self.label, ink);
    }
}

/// The page, and the two choices it is drawn from.
struct Palettes {
    readings: [Button; 2],
    swatches: [Button; SEEDS.len()],
    reading: usize,
    seed: usize,
}

impl Root for Palettes {
    fn take_root(grove: &mut Grove) -> Self {
        // The ground. It carries the surface role like anything else, so the page itself is part of
        // what a repaint moves.
        let page = grove.plant(Panel::new().scrolls(Axes::Vertical));

        let mut y = MARGIN;
        heading(grove, page, y, "press to state what a role resolves to");
        y += HEADING;

        let readings = [
            Button::grow(grove, page, MARGIN, y, BUTTON_W, "dark"),
            Button::grow(grove, page, MARGIN + BUTTON_W + 8.0, y, BUTTON_W, "light"),
        ];
        let swatches = std::array::from_fn(|n| {
            let x = MARGIN + 2.0 * (BUTTON_W + 8.0) + 24.0 + n as f32 * (SWATCH_W + 8.0);
            let button = Button::grow(grove, page, x, y, SWATCH_W, "");
            // The one literal on the page, and the reason it is here: a fill stated outright is not
            // part of any scheme, so this circle is what the accent would become rather than
            // anything that follows what it already is.
            let (red, green, blue) = SEEDS[n];
            grove.branch(
                page,
                Polygon::circle()
                    .color(Color::rgb(red, green, blue))
                    .pass_through()
                    .at(Location::new().xs(
                        left((x + (SWATCH_W - 18.0) / 2.0).px()).width(18.0.px()),
                        top((y + (BUTTON_H - 18.0) / 2.0).px()).height(18.0.px()),
                    )),
            );
            button
        });
        y += BUTTON_H + SECTION;

        y = grid(grove, page, y) + SECTION;
        heading(grove, page, y, "a state is a step, not a color beside one");
        states(grove, page, y + HEADING);

        let page = Self {
            readings,
            swatches,
            reading: 0,
            seed: 0,
        };
        page.chosen(grove);
        page
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        let mut moved = false;
        for (n, button) in self.readings.iter().enumerate() {
            if pollen.clicked(button.panel) && self.reading != n {
                self.reading = n;
                moved = true;
            }
        }
        for (n, button) in self.swatches.iter().enumerate() {
            if pollen.clicked(button.panel) && self.seed != n {
                self.seed = n;
                moved = true;
            }
        }
        if moved {
            // The whole of what a press does. One op, naming no element: every tone the page
            // declared is resolved against the new scheme at extraction, and the elements that move
            // are exactly the ones painted in a color that changed.
            grove.repaint(self.scheme());
            self.chosen(grove);
        }
    }
}

impl Palettes {
    /// The scheme the two choices make: a reading, and one role restated on top of it.
    fn scheme(&self) -> Scheme {
        let reading = match self.reading {
            0 => Scheme::new(),
            _ => Scheme::light(),
        };
        let (red, green, blue) = SEEDS[self.seed];
        reading.set(Palette::Accent, Color::rgb(red, green, blue))
    }

    /// Marks which button in each row is the one the page is drawn from.
    ///
    /// A reading is marked in the accent, with its label in what reads against one. A swatch cannot
    /// be: the accent is what a swatch is showing, so filling it with what it seeds would paint the
    /// circle out. It is marked with a step on the surface it already sits on instead, which is the
    /// same thing the strip of states is about.
    fn chosen(&self, grove: &mut Grove) {
        for (n, button) in self.readings.iter().enumerate() {
            match n == self.reading {
                true => button.mark(grove, Palette::Accent, Palette::Contrast),
                false => button.mark(grove, Palette::Raised, Palette::Ink),
            }
        }
        for (n, button) in self.swatches.iter().enumerate() {
            match n == self.seed {
                true => button.mark(grove, Palette::Raised.at(Step::Nearest), Palette::Ink),
                false => button.mark(grove, Palette::Raised, Palette::Ink),
            }
        }
    }
}

/// Every role at every step: a column per step, a row per role, and a polygon in each cell.
///
/// The polygons are the point of the grid -- a role's five steps beside each other is the only way
/// to see that a ramp holds its hue and moves only how far it stands out.
fn grid(grove: &mut Grove, page: Leaf, top_y: f32) -> f32 {
    // A raised ground under the grid, and not decoration. Half the cells are the colors a page is
    // made of, so on the page itself the surface row and the contrast base are painted in exactly
    // what is behind them and read as missing. One surface in front of another is the role that
    // exists for this, and against it only the raised base disappears -- which is the truth about
    // that cell rather than a gap in the grid.
    let height = 22.0 + ROLES.len() as f32 * CELL_H + 2.0 * CARD_PAD;
    grove.branch(
        page,
        Panel::new()
            .color(Palette::Raised)
            .rounding(Rounding::Md)
            .pass_through()
            .at(Location::new().xs(
                left((MARGIN - CARD_PAD).px()).width((PAGE_W - 2.0 * (MARGIN - CARD_PAD)).px()),
                top((top_y - CARD_PAD).px()).height(height.px()),
            )),
    );

    for (column, (name, _)) in STEPS.iter().enumerate() {
        label(
            grove,
            page,
            MARGIN + LABELS + column as f32 * CELL_W,
            top_y,
            name,
        );
    }

    let first = top_y + 22.0;
    for (row, (name, role)) in ROLES.iter().enumerate() {
        let row_y = first + row as f32 * CELL_H;
        label(grove, page, MARGIN, row_y + (CELL_H - MARK) / 2.0 + 18.0, name);
        for (column, (_, step)) in STEPS.iter().enumerate() {
            grove.branch(
                page,
                Polygon::new()
                    .sides(6.0)
                    .rounding(0.3)
                    .color(role.at(*step))
                    .at(Location::new().xs(
                        left((MARGIN + LABELS + column as f32 * CELL_W).px()).width(MARK.px()),
                        top((row_y + (CELL_H - MARK) / 2.0).px()).height(MARK.px()),
                    )),
            );
        }
    }
    first + ROLES.len() as f32 * CELL_H
}

/// The accent's three working steps, each carrying a label in what reads against it.
///
/// What the grid shows as a ramp, written the way an app writes it: one role, three conditions, and
/// no color named anywhere.
fn states(grove: &mut Grove, page: Leaf, top_y: f32) {
    for (column, (name, step)) in STATES.iter().enumerate() {
        let x = MARGIN + column as f32 * (STATE_W + 12.0);
        grove.branch(
            page,
            Panel::new()
                .color(step(Palette::Accent))
                .rounding(Rounding::Md)
                .at(Location::new().xs(
                    left(x.px()).width(STATE_W.px()),
                    top(top_y.px()).height(STATE_H.px()),
                )),
        );
        grove.branch(
            page,
            Text::new(*name)
                .color(Palette::Contrast)
                .font_size(FontSize::new().xs(14))
                .at(Location::new().xs(
                    left((x + 16.0).px()).width(content()),
                    top((top_y + STATE_H / 2.0 - 9.0).px()).height(content()),
                )),
        );
    }
}

/// A section's title.
fn heading(grove: &mut Grove, page: Leaf, y: f32, title: &str) {
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
}

/// A label for a row or a column, drawn where it is asked for.
fn label(grove: &mut Grove, page: Leaf, x: f32, y: f32, text: &str) {
    grove.branch(
        page,
        Text::new(text)
            .color(Palette::Muted)
            .font_size(FontSize::new().xs(12))
            .at(Location::new().xs(left(x.px()).width(content()), top(y.px()).height(content()))),
    );
}
