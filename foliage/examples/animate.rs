//! One motion, ended three ways.
//!
//! Three rows, each running the same pair of motions on a [`Polygon`] of its own: a long shape
//! motion, and a colour motion chained off its end. The chain is built out of the name
//! [`animate`](foliage::Grow::animate) hands back and nothing else -- the colour begins when the
//! shape reports [`finished`](Pollen::finished), and each row's two bars are filled from
//! [`tween`](Pollen::tween), which is how far that motion has come.
//!
//! What differs is how the first motion of each row ends:
//!
//! - **plays out.** Nothing interrupts it. Both bars fill in their own time.
//! - **stopped half way.** The row watches its own readout and calls
//!   [`stop`](foliage::Grow::stop) when the shape motion is half done. The bar freezes where it
//!   stopped, and the colour never runs, because nothing was reported for it to run off.
//! - **finished a fifth in.** The same watch, calling [`finish`](foliage::Grow::finish) instead.
//!   The bar snaps full and the colour begins at once, seconds before the row above it.
//!
//! **All three polygons end on the same shape.** Stopping gives up the duration and not the
//! destination: the target is what the element was told to be from the moment the motion started.
//! The whole of the difference between the last two rows is whether anything was told -- and so
//! whether what was waiting on that ending ever runs.
//!
//! ```sh
//! cargo run -p foliage --example animate
//! ```

use core::f32::consts::PI;

use foliage::{
    Area, Boxed, Ease, Foliage, FontSize, Grove, Grow, Leaf, Location, Motion, Palette, Panel,
    Place, Pollen, Polygon, Root, Rounding, Shape, Source, Text, Timing, Tween, content, left, top,
};

/// The margin around the page, and the rhythm everything else is stated in.
const MARGIN: f32 = 24.0;

/// The play button.
const BUTTON_W: f32 = 88.0;
const BUTTON_H: f32 = 32.0;

/// One row: the polygon's box, the gap past it, and the readouts beside it.
const POLY: f32 = 88.0;
const GUTTER: f32 = 20.0;
const BAR_W: f32 = 320.0;
const BAR_H: f32 = 8.0;
const ROW_H: f32 = 112.0;

/// Where the readout column starts, and the width the whole page is laid out across.
const COLUMN: f32 = MARGIN + POLY + GUTTER;
const PAGE_W: f32 = COLUMN + BAR_W + MARGIN;

/// How long each of the two motions takes. The first is long on purpose: an ending that arrives
/// seconds early has to have seconds to arrive early by.
const SHAPE_MS: u64 = 3600;
const COLOUR_MS: u64 = 600;

/// Where every row starts, and where its shape motion is going.
const START: Shape = Shape {
    sides: 3.0,
    rounding: 0.0,
    rotation: 0.0,
};
const END: Shape = Shape {
    sides: 7.0,
    rounding: 0.55,
    rotation: PI,
};

/// How a row's first motion ends.
#[derive(Copy, Clone, PartialEq)]
enum Fate {
    /// Nothing interrupts it.
    Plays,
    /// Stopped part way, which reports nothing.
    Stopped,
    /// Finished part way, which reports an arrival.
    Finished,
}

/// The three rows, in the order they are read.
const ROWS: [Fate; 3] = [Fate::Plays, Fate::Stopped, Fate::Finished];

impl Fate {
    /// How far into the shape motion the row interrupts it, if it does.
    ///
    /// Read off the motion's own progress rather than a clock of the row's own, which is what makes
    /// the interruption land at the same point every run.
    fn cut(self) -> Option<f32> {
        match self {
            Fate::Plays => None,
            Fate::Stopped => Some(0.5),
            Fate::Finished => Some(0.2),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Fate::Plays => "plays out",
            Fate::Stopped => "stopped half way",
            Fate::Finished => "finished a fifth in",
        }
    }

    /// What the row says once its first motion has ended, whichever way it did.
    fn outcome(self) -> &'static str {
        match self {
            Fate::Plays => "ran out, and the colour followed",
            Fate::Stopped => "on target, and nothing followed",
            Fate::Finished => "on target, and the colour followed",
        }
    }
}

fn main() {
    tracing_subscriber::fmt().init();
    let mut foliage = Foliage::new();
    foliage.title("foliage -- animate");
    foliage.app_id("foliage");
    foliage.desktop_size(Area::new(PAGE_W, 512.0));
    foliage.root::<Endings>();
    foliage.photosynthesize();
}

/// One bar: a track, and the fill written across it every frame its motion runs.
struct Readout {
    fill: Leaf,
    y: f32,
}

impl Readout {
    fn grow(grove: &mut Grove, page: Leaf, y: f32, colour: Palette) -> Self {
        grove.branch(
            page,
            Panel::new()
                .color(Palette::Muted)
                .rounding(Rounding::Sm)
                .at(Location::new().xs(
                    left(COLUMN.px()).width(BAR_W.px()),
                    top(y.px()).height(BAR_H.px()),
                )),
        );
        let fill = grove.branch(
            page,
            Panel::new()
                .color(colour)
                .rounding(Rounding::Sm)
                .at(Location::new().xs(
                    left(COLUMN.px()).width(0.px()),
                    top(y.px()).height(BAR_H.px()),
                )),
        );
        Self { fill, y }
    }

    /// Draws the bar `at` of the way across its track.
    fn at(&self, grove: &mut Grove, at: f32) {
        grove.at(
            self.fill,
            Location::new().xs(
                left(COLUMN.px()).width((BAR_W * at).px()),
                top(self.y.px()).height(BAR_H.px()),
            ),
        );
    }
}

/// Which of a row's two motions is running, and the name it is running under.
#[derive(Copy, Clone)]
struct Running {
    stage: usize,
    tween: Tween,
}

/// One row: a polygon, the two bars its two motions fill, and how its first motion ends.
struct Row {
    fate: Fate,
    polygon: Leaf,
    bars: [Readout; 2],
    outcome: Leaf,
    running: Option<Running>,
    /// Whether the interruption has been issued. An op takes a frame to land, so the progress that
    /// asked for it is still being reported on the frame after -- this is what keeps one
    /// interruption from being asked for twice.
    cut: bool,
}

impl Row {
    fn grow(grove: &mut Grove, page: Leaf, y: f32, fate: Fate) -> Self {
        let polygon = grove.branch(
            page,
            Polygon::new()
                .sides(START.sides)
                .rounding(START.rounding)
                .rotation(START.rotation)
                .color(Palette::Accent)
                .at(Location::new().xs(
                    left(MARGIN.px()).width(POLY.px()),
                    top(y.px()).height(POLY.px()),
                )),
        );
        grove.branch(
            page,
            Text::new(fate.label())
                .color(Palette::Ink)
                .font_size(FontSize::new().xs(13))
                .at(Location::new().xs(
                    left(COLUMN.px()).width(content()),
                    top(y.px()).height(content()),
                )),
        );
        let bars = [
            Readout::grow(grove, page, y + 24.0, Palette::Accent),
            Readout::grow(grove, page, y + 40.0, Palette::Ink),
        ];
        let outcome = grove.branch(
            page,
            Text::new("")
                .color(Palette::Ink.recede())
                .font_size(FontSize::new().xs(12))
                .at(Location::new().xs(
                    left(COLUMN.px()).width(content()),
                    top((y + 58.0).px()).height(content()),
                )),
        );
        Self {
            fate,
            polygon,
            bars,
            outcome,
            running: None,
            cut: false,
        }
    }

    /// Puts the row back where it started and runs it again.
    fn restart(&mut self, grove: &mut Grove) {
        // Two direct writes, so whatever was moving either property is cancelled and the polygon is
        // at what was written. Nothing has to stop a running motion first.
        grove.reshape(self.polygon, START);
        grove.color(self.polygon, Palette::Accent);
        for bar in &self.bars {
            bar.at(grove, 0.0);
        }
        grove.text(self.outcome, "");
        self.cut = false;
        self.begin(grove, 0);
    }

    /// Starts one of the row's two motions and holds the name it runs under.
    fn begin(&mut self, grove: &mut Grove, stage: usize) {
        let tween = match stage {
            0 => grove.animate(
                self.polygon,
                Motion::Polygon(END),
                Timing::ms(SHAPE_MS).ease(Ease::Linear),
            ),
            _ => grove.animate(
                self.polygon,
                Motion::Palette(Palette::Ink),
                Timing::ms(COLOUR_MS).ease(Ease::Emphasis),
            ),
        };
        self.running = Some(Running { stage, tween });
    }

    /// One frame of the row, which is one read of what its running motion is doing.
    fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        let Some(running) = self.running else {
            return;
        };
        // The frame a motion ends reports its end value and its finish together, so the bar is
        // written from one place and never has to read a full one out of a report that stopped
        // arriving.
        if let Some(at) = pollen.tween(running.tween) {
            self.bars[running.stage].at(grove, at);
            if running.stage == 0 {
                self.interrupt(grove, running.tween, at);
            }
        }
        if pollen.finished(running.tween) {
            match running.stage {
                // The chain, and the whole of it: what runs next runs off the ending of what ran
                // before. A stopped motion reports no ending, so nothing here runs.
                0 => self.begin(grove, 1),
                _ => {
                    self.running = None;
                    grove.text(self.outcome, self.fate.outcome());
                }
            }
        }
    }

    /// Ends the shape motion early, once it has come as far as this row waits for.
    fn interrupt(&mut self, grove: &mut Grove, tween: Tween, at: f32) {
        let Some(cut) = self.fate.cut() else {
            return;
        };
        if self.cut || at < cut {
            return;
        }
        self.cut = true;
        match self.fate {
            // Ended in silence. The polygon still arrives -- it was told where to go when the
            // motion started -- but the bar stays where it stopped and the colour never begins.
            Fate::Stopped => {
                grove.stop(tween);
                self.running = None;
                grove.text(self.outcome, self.fate.outcome());
            }
            // Ended as an arrival. Nothing else is written here: the colour begins where it always
            // begins, off the `finished` this produces in the next frame.
            Fate::Finished => grove.finish(tween),
            Fate::Plays => {}
        }
    }
}

/// The page.
struct Endings {
    rows: [Row; ROWS.len()],
    play: Leaf,
}

impl Root for Endings {
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(Panel::new());

        note(
            grove,
            page,
            MARGIN,
            14,
            Palette::Ink,
            "one motion, ended three ways",
        );
        note(
            grove,
            page,
            MARGIN + 22.0,
            13,
            Palette::Ink.recede(),
            "a shape motion, then a colour chained off its end",
        );

        let play = grove.branch(
            page,
            Panel::new()
                .color(Palette::Raised)
                .rounding(Rounding::Sm)
                .interactive()
                .at(Location::new().xs(
                    left(MARGIN.px()).width(BUTTON_W.px()),
                    top((MARGIN + 52.0).px()).height(BUTTON_H.px()),
                )),
        );
        grove.branch(
            page,
            Text::new("play")
                .color(Palette::Ink)
                .font_size(FontSize::new().xs(13))
                .intangible()
                .at(Location::new().xs(
                    left((MARGIN + 14.0).px()).width(content()),
                    top((MARGIN + 61.0).px()).height(content()),
                )),
        );

        let top_of_rows = MARGIN + 100.0;
        let rows = std::array::from_fn(|n| {
            Row::grow(grove, page, top_of_rows + n as f32 * ROW_H, ROWS[n])
        });

        let below = top_of_rows + ROWS.len() as f32 * ROW_H - 8.0;
        note(
            grove,
            page,
            below,
            13,
            Palette::Ink.recede(),
            "all three end on the same shape",
        );
        note(
            grove,
            page,
            below + 22.0,
            13,
            Palette::Ink.recede(),
            "what differs is the report, and so what follows",
        );

        Self { rows, play }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        for row in &mut self.rows {
            row.frame(grove, &pollen);
        }
        // The one control. It resets every row and runs them together, so what the three endings do
        // differently is read off one press rather than off three presses timed by hand.
        if pollen.clicked(self.play) {
            for row in &mut self.rows {
                row.restart(grove);
            }
        }
    }
}

/// One line of text at the page's left margin.
fn note(grove: &mut Grove, page: Leaf, y: f32, size: u32, colour: Palette, text: &str) {
    grove.branch(
        page,
        Text::new(text)
            .color(colour)
            .font_size(FontSize::new().xs(size))
            .at(Location::new().xs(
                left(MARGIN.px()).width(content()),
                top(y.px()).height(content()),
            )),
    );
}
