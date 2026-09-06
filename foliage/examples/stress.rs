//! Every phase of the frame, saturated on demand, for sampling the engine under load.
//!
//! An ordinary example spends its time waiting. A window opens, a page is grown once, and the
//! engine idles until something is pressed -- so a profile of one is a profile of startup and of
//! `epoll`. This grows a field of elements sized to order and writes to it every frame, and calls
//! [`again`](foliage::Grove::again) whatever the load is doing, so the engine never idles and what
//! a sampling profiler sees is the frame rather than the wait around it.
//!
//! # The loads
//!
//! One runs at a time, chosen with the number keys. Each writes to the same field, so what
//! separates two readings is the phase the writes land in rather than what is on the page.
//!
//! | key | load | what it saturates |
//! | --- | --- | --- |
//! | `1` | `idle` | nothing is written: the floor a frame costs at this element count |
//! | `2` | `layout` | every cell's placement, which is the whole of resolution |
//! | `3` | `color` | every cell's fill, which reaches extraction without moving a box |
//! | `4` | `text` | every label, with values no run has been shaped for |
//! | `5` | `animate` | a live motion on every cell, and the placements they write |
//! | `6` | `churn` | a slab of the field taken down and regrown, which is the drain |
//! | `7` | `repaint` | the scheme restated, which re-resolves every role in the tree |
//!
//! `-` halves the field and `+` doubles it, between 16 and 16384 cells. Each cell is two elements:
//! the shape a load writes to, and a label grown on it. Changing either the load or the count grows
//! the field again, because a load writes a property the one before it may have written differently
//! -- a fill stated outright is not a role any more, and would not answer a repaint.
//!
//! # Reading it
//!
//! The window presents in step with the display, so a load that finishes inside a refresh interval
//! spends the rest of that interval waiting to present, and a profile of it is mostly that wait.
//! **Raise the cell count until the frame rate falls below the display's.** Past that point the
//! frame is spent in the engine and the profile is about foliage.
//!
//! The rate is written to the page and to the trace once a second. It is the only thing here on a
//! cadence rather than every frame: the readout is a text run like any other, and one rewritten
//! every frame would measure itself alongside the load.
//!
//! # Running it
//!
//! The load and the cell count are also arguments, in either order, so the process can start in the
//! state being sampled rather than be typed into it.
//!
//! ```sh
//! cargo run -p foliage --release --example stress -- text 4096
//! ```
//!
//! Release, because a profile of a debug build is a profile of a debug build.
//!
//! # The phase table
//!
//! The engine states every phase of the frame as a span, so the breakdown is available without a
//! profiler. Printing each span as it closes is a line per phase per frame, which at these rates
//! costs more than the phases do and cannot be read; a layer keeps a running total per phase
//! instead, and one table is written a second beside the rate:
//!
//! ```text
//! layout on 4096 cells, 47 fps
//!   phase          calls    mean ms     share
//!   frame             47     21.180    100.0%
//!   resolve           47     13.941     65.8%
//!   axis              94      5.106     48.2%
//!   extract           47      4.302     20.3%
//!   drain             47      1.884      8.9%
//! ```
//!
//! The times are inclusive, so a phase's share is of the frame it sits in and the nested phases sum
//! past their parent. `frame` is the whole of one, and `calls` is how many times a phase ran over
//! the interval -- twice a frame for the one that resolves an axis, once for the rest.
//!
//! The table is written whatever `RUST_LOG` is set to, which governs only the ordinary log.

use core::f32::consts::TAU;
use core::time::Duration;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

use foliage::{
    Area, Boxed, Color, Ease, Foliage, FontSize, Grove, Grow, Key, Leaf, Location, Motion, Palette,
    Panel, Place, Pollen, Polygon, Root, Rounding, Scheme, Source, Step, Text, Timing, Tween,
    content, left, top,
};
use tracing::span::{Attributes, Id};
use tracing::{Subscriber, info, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;

/// The page the field is divided against, which the window opens at.
const PAGE_W: f32 = 1024.0;
const PAGE_H: f32 = 768.0;

/// The strip at the top of the page the two readout lines sit in, above the field.
const HEADER: f32 = 46.0;

/// How many cells the field starts with, and the bounds `-` and `+` move it between.
const CELLS: usize = 512;
const FEWEST: usize = 16;
const MOST: usize = 16384;

/// The fraction of its seat a cell fills, which leaves the gutter between them.
const INSET: f32 = 0.86;

/// How far a cell is displaced from its seat, as a fraction of the field.
const WANDER: f32 = 1.2;

/// How many cells [`Load::Churn`] takes down and regrows each frame.
const CHURN: usize = 32;

/// The shortest motion under [`Load::Animate`], and the spread across the field that keeps the
/// endings from all arriving on one frame.
const MOTION_MS: u64 = 700;
const SPREAD_MS: u64 = 900;

fn main() {
    // Two layers over one registry. The log is whatever `RUST_LOG` asks for; the timing is not,
    // because it needs the engine's phases at `trace` whatever the log is set to, and reports them
    // as a table rather than as the events a log would carry.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().with_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            ),
        )
        .with(Stopwatch.with_filter(Targets::new().with_target("foliage", LevelFilter::TRACE)))
        .init();
    let mut foliage = Foliage::new();
    foliage.title("foliage -- stress");
    foliage.app_id("foliage");
    foliage.desktop_size(Area::new(PAGE_W, PAGE_H));
    foliage.root::<Stress>();
    foliage.photosynthesize();
}

/// Which phase of the frame the running load's writes land in.
#[derive(Copy, Clone, PartialEq)]
enum Load {
    /// Nothing is written.
    Idle,
    /// Every cell's placement.
    Layout,
    /// Every cell's fill, stated outright.
    Color,
    /// Every label's value.
    Text,
    /// A live motion on every cell.
    Animate,
    /// A slab of the field, taken down and regrown.
    Churn,
    /// The scheme every role is resolved against.
    Repaint,
}

/// The loads in the order the number keys select them.
const LOADS: [Load; 7] = [
    Load::Idle,
    Load::Layout,
    Load::Color,
    Load::Text,
    Load::Animate,
    Load::Churn,
    Load::Repaint,
];

impl Load {
    /// What it is called, on the page, in the trace, and as an argument.
    fn name(self) -> &'static str {
        match self {
            Load::Idle => "idle",
            Load::Layout => "layout",
            Load::Color => "color",
            Load::Text => "text",
            Load::Animate => "animate",
            Load::Churn => "churn",
            Load::Repaint => "repaint",
        }
    }

    /// What it saturates, as the readout states it.
    fn phase(self) -> &'static str {
        match self {
            Load::Idle => "nothing written",
            Load::Layout => "resolve",
            Load::Color => "extract",
            Load::Text => "shape and extract",
            Load::Animate => "animate",
            Load::Churn => "drain",
            Load::Repaint => "extract, every role",
        }
    }

    /// The load `argument` names, if it names one.
    fn parse(argument: &str) -> Option<Self> {
        LOADS.into_iter().find(|load| load.name() == argument)
    }
}

/// The load and the cell count asked for on the command line, in either order and both optional.
fn requested() -> (Load, usize) {
    let mut load = Load::Layout;
    let mut cells = CELLS;
    for argument in std::env::args().skip(1) {
        if let Some(named) = Load::parse(&argument) {
            load = named;
        } else if let Ok(count) = argument.parse::<usize>() {
            cells = count.clamp(FEWEST, MOST);
        } else {
            warn!(argument, "not a load or a cell count");
        }
    }
    (load, cells)
}

/// How the field is divided, which is what fixes where a cell sits.
#[derive(Copy, Clone)]
struct Divided {
    columns: usize,
    rows: usize,
}

impl Divided {
    /// The division of `cells` that is as square as the page the field fills.
    fn of(cells: usize) -> Self {
        let across = (cells as f32 * PAGE_W / (PAGE_H - HEADER)).sqrt().ceil();
        let columns = (across as usize).max(1);
        Self {
            columns,
            rows: cells.div_ceil(columns),
        }
    }

    /// Where cell `n` sits, as a fraction of the field, displaced from its seat by `wander`.
    fn seat(self, n: usize, wander: f32) -> Location {
        let width = 100.0 / self.columns as f32;
        let height = 100.0 / self.rows as f32;
        let x = (n % self.columns) as f32 * width + wander;
        let y = (n / self.columns) as f32 * height + wander;
        Location::new().xs(
            left(x.pct()).width((width * INSET).pct()),
            top(y.pct()).height((height * INSET).pct()),
        )
    }
}

/// One cell of the field: the shape every load writes to, and the label grown on it.
struct Cell {
    shape: Leaf,
    label: Leaf,
}

/// The motion a cell is running, and which of its two seats it is heading for.
struct Live {
    tween: Tween,
    out: bool,
}

/// The field, the load running on it, and what the readout is drawn from.
struct Stress {
    load: Load,
    cells: Vec<Cell>,
    /// What each cell is running under [`Load::Animate`], and empty under every other load.
    motions: Vec<Live>,
    /// How the field is divided, which every seat is stated against.
    divided: Divided,
    /// What the cells are grown under.
    field: Leaf,
    /// What is running, and the rate it is running at.
    headline: Leaf,
    rate: Leaf,
    /// Where [`Load::Churn`] regrows next.
    churn: usize,
    /// Frames since the rate was last written, and the time it was written at.
    frames: u32,
    marked: Duration,
}

impl Root for Stress {
    fn take_root(grove: &mut Grove) -> Self {
        let (load, cells) = requested();
        let page = grove.plant(Panel::new());
        let headline = grove.branch(
            page,
            Text::new("")
                .color(Palette::Ink)
                .font_size(FontSize::new().xs(13))
                .intangible()
                .at(Location::new().xs(
                    left(12.px()).width(content()),
                    top(8.px()).height(content()),
                )),
        );
        let rate = grove.branch(
            page,
            Text::new("")
                .color(Palette::Ink.recede())
                .font_size(FontSize::new().xs(12))
                .intangible()
                .at(Location::new().xs(
                    left(12.px()).width(content()),
                    top(26.px()).height(content()),
                )),
        );
        // Nothing in the field receives, so every keystroke arrives with focus nowhere and the
        // controls cost the frame nothing.
        let field = grove.branch(
            page,
            Panel::new().intangible().at(Location::new().xs(
                left(0.px()).right(100.pct()),
                top(HEADER.px()).bottom(100.pct()),
            )),
        );
        let mut stress = Self {
            load,
            cells: Vec::new(),
            motions: Vec::new(),
            divided: Divided::of(cells),
            field,
            headline,
            rate,
            churn: 0,
            frames: 0,
            marked: Duration::ZERO,
        };
        stress.rebuild(grove, cells);
        stress
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        self.controls(grove, &pollen);
        self.apply(grove, &pollen);
        self.meter(grove);
        // The engine idles when nothing is owed, and a profile of an idle engine is a profile of
        // the wait. Every load asks for the next frame, including the one that writes nothing.
        grove.again();
    }
}

impl Stress {
    /// Reads the keyboard: which load runs, and how much field it runs on.
    fn controls(&mut self, grove: &mut Grove, pollen: &Pollen) {
        for stroke in pollen.root_keys() {
            let Key::Typed(typed) = stroke.key else {
                continue;
            };
            match typed {
                '1'..='7' => {
                    let load = LOADS[typed as usize - '1' as usize];
                    if load != self.load {
                        self.load = load;
                        self.rebuild(grove, self.cells.len());
                    }
                }
                '+' | '=' => self.resize(grove, self.cells.len() * 2),
                '-' | '_' => self.resize(grove, self.cells.len() / 2),
                _ => {}
            }
        }
    }

    /// Grows the field again at `cells`, clamped to what the example runs between.
    fn resize(&mut self, grove: &mut Grove, cells: usize) {
        let cells = cells.clamp(FEWEST, MOST);
        if cells != self.cells.len() {
            self.rebuild(grove, cells);
        }
    }

    /// Takes the field down and grows `cells` in its place.
    ///
    /// Every change of load comes through here rather than handing the standing field over: a load
    /// writes one property, and the property the load before it wrote is not the one it left behind
    /// -- a fill stated outright is no longer a role, and a cell that has wandered is no longer at
    /// its seat. Growing the field again is what makes two readings comparable.
    fn rebuild(&mut self, grove: &mut Grove, cells: usize) {
        for cell in self.cells.drain(..) {
            grove.prune(cell.shape);
        }
        self.motions.clear();
        self.churn = 0;
        self.divided = Divided::of(cells);
        let (divided, field) = (self.divided, self.field);
        self.cells = (0..cells).map(|n| grow(grove, field, divided, n)).collect();
        if self.load == Load::Animate {
            self.motions.reserve(cells);
            for n in 0..cells {
                let shape = self.cells[n].shape;
                self.motions.push(Live {
                    tween: begin(grove, shape, divided, n, true),
                    out: true,
                });
            }
        }
        grove.text(self.headline, self.headline());
        info!(
            load = self.load.name(),
            cells,
            elements = cells * 2,
            columns = divided.columns,
            rows = divided.rows,
            "field grown"
        );
    }

    /// One frame of the running load.
    fn apply(&mut self, grove: &mut Grove, pollen: &Pollen) {
        let elapsed = grove.elapsed().as_secs_f32();
        match self.load {
            Load::Idle => {}
            Load::Layout => self.wander(grove, elapsed),
            Load::Color => self.recolor(grove, elapsed),
            Load::Text => self.relabel(grove),
            Load::Animate => self.remotion(grove, pollen),
            Load::Churn => self.regrow(grove),
            Load::Repaint => self.restate(grove, elapsed),
        }
    }

    /// Rewrites every cell's placement, which is the whole of resolution: both axes, the extents
    /// that come out of them, the clips, and the rank.
    ///
    /// The displacement is a real one every frame. A placement written back at the value it already
    /// holds settles at the same box and extracts as unchanged, which would measure the check
    /// rather than the work.
    fn wander(&self, grove: &mut Grove, elapsed: f32) {
        let divided = self.divided;
        for (n, cell) in self.cells.iter().enumerate() {
            let phase = elapsed * 2.0 + n as f32 * 0.35;
            grove.at(cell.shape, divided.seat(n, phase.sin() * WANDER));
        }
    }

    /// Rewrites every cell's fill, which reaches extraction without moving a box: the same
    /// instances, rewritten where they already stand.
    fn recolor(&self, grove: &mut Grove, elapsed: f32) {
        let span = self.cells.len() as f32;
        for (n, cell) in self.cells.iter().enumerate() {
            grove.color(cell.shape, wheel(elapsed * 0.5 + n as f32 / span));
        }
    }

    /// Rewrites every label with a value nothing has shaped: five characters carrying the frame, so
    /// the run misses the cache, is shaped and measured and wrapped, has its glyphs extracted, and
    /// is swept at the end of the frame because nothing states it any more.
    fn relabel(&self, grove: &mut Grove) {
        for (n, cell) in self.cells.iter().enumerate() {
            grove.text(cell.label, format!("{:02x}{:03x}", self.frames & 0xff, n & 0xfff));
        }
    }

    /// Keeps a motion live on every cell, restarting each as it ends.
    ///
    /// A restart alternates which seat the cell is heading for, so a cell that has arrived has
    /// somewhere to go, and the durations are spread across the field so endings arrive on every
    /// frame rather than all on one -- which is what holds the count of live tweens flat.
    fn remotion(&mut self, grove: &mut Grove, pollen: &Pollen) {
        let divided = self.divided;
        for n in 0..self.motions.len() {
            if !pollen.finished(self.motions[n].tween) {
                continue;
            }
            let out = !self.motions[n].out;
            let shape = self.cells[n].shape;
            self.motions[n] = Live {
                tween: begin(grove, shape, divided, n, out),
                out,
            };
        }
    }

    /// Takes a slab of cells down and grows the same number back, which is the drain doing the two
    /// most expensive things an op asks of the tree.
    ///
    /// A rolling window rather than the same slab every frame, so the whole field turns over.
    fn regrow(&mut self, grove: &mut Grove) {
        let (divided, field) = (self.divided, self.field);
        for _ in 0..CHURN.min(self.cells.len()) {
            let n = self.churn % self.cells.len();
            grove.prune(self.cells[n].shape);
            self.cells[n] = grow(grove, field, divided, n);
            self.churn = self.churn.wrapping_add(1);
        }
    }

    /// Restates the scheme, which re-resolves every role in the tree at extraction.
    ///
    /// The accent turns, so what the roles derived from it resolve to genuinely differs from the
    /// frame before.
    fn restate(&self, grove: &mut Grove, elapsed: f32) {
        grove.repaint(Scheme::new().set(Palette::Accent, wheel(elapsed * 0.2)));
    }

    /// Writes the frame rate, to the page and to the trace.
    ///
    /// Once a second rather than every frame, because the readout is a text run like any other and
    /// one rewritten every frame would be measuring itself alongside the load.
    fn meter(&mut self, grove: &mut Grove) {
        self.frames += 1;
        let elapsed = grove.elapsed();
        let since = elapsed.saturating_sub(self.marked);
        if since < Duration::from_secs(1) {
            return;
        }
        let rate = self.frames as f32 / since.as_secs_f32();
        let last = grove.frame_time().as_secs_f32() * 1000.0;
        info!(
            load = self.load.name(),
            cells = self.cells.len(),
            rate = rate as f64,
            frame_ms = last as f64,
            "stress"
        );
        grove.text(self.rate, format!("{rate:.0} fps, {last:.2} ms last frame"));
        report(self.load, self.cells.len(), rate);
        self.frames = 0;
        self.marked = elapsed;
    }

    /// What is running, on how much, and what it saturates.
    fn headline(&self) -> String {
        format!(
            "{}  {} cells, {} elements  -  {}  -  1..7 load, +/- size",
            self.load.name(),
            self.cells.len(),
            self.cells.len() * 2,
            self.load.phase(),
        )
    }
}

/// Grows cell `n`: the shape the loads write to, and the label grown on it.
///
/// Every fourth is a polygon, so the field is not one draw path measured as though it were the
/// engine. The label is grown on the shape rather than beside it, so it is placed against the box
/// that moves and is taken down with it.
fn grow(grove: &mut Grove, field: Leaf, divided: Divided, n: usize) -> Cell {
    let at = divided.seat(n, 0.0);
    let shape = match n % 4 {
        0 => grove.branch(
            field,
            Polygon::new()
                .sides(6.0)
                .rounding(0.25)
                .color(tone(n))
                .intangible()
                .at(at),
        ),
        _ => grove.branch(
            field,
            Panel::new()
                .color(tone(n))
                .rounding(Rounding::Sm)
                .intangible()
                .at(at),
        ),
    };
    let label = grove.branch(
        shape,
        Text::new(format!("{:03x}", n & 0xfff))
            .color(Palette::Contrast)
            .font_size(FontSize::new().xs(11))
            .intangible()
            .at(Location::new().xs(
                left(10.pct()).width(content()),
                top(10.pct()).height(content()),
            )),
    );
    Cell { shape, label }
}

/// Starts cell `n` toward the seat `out` selects, and hands back the name the motion runs under.
fn begin(grove: &mut Grove, shape: Leaf, divided: Divided, n: usize, out: bool) -> Tween {
    grove.animate(
        shape,
        Motion::Location(divided.seat(n, if out { WANDER } else { 0.0 })),
        Timing::ms(MOTION_MS + (n as u64 * 37) % SPREAD_MS).ease(Ease::Emphasis),
    )
}

/// The role cell `n` is filled with.
///
/// Roles rather than colors stated outright, so [`Load::Repaint`] has something to move and the
/// field is what a repaint costs on a real page.
fn tone(n: usize) -> Palette {
    match n % 5 {
        0 => Palette::Accent,
        1 => Palette::Raised.at(Step::Nearest),
        2 => Palette::Muted,
        3 => Palette::Ink.at(Step::Far),
        _ => Palette::Accent.at(Step::Far),
    }
}

/// A color at `turn` of the way round, stated outright.
fn wheel(turn: f32) -> Color {
    let channel = |offset: f32| ((turn + offset) * TAU).sin() * 0.4 + 0.5;
    Color::rgb(channel(0.0), channel(1.0 / 3.0), channel(2.0 / 3.0))
}

/// What one phase cost over an interval: what encloses it, how many times it ran, and the time
/// spent inside it.
struct Tally {
    phase: &'static str,
    parent: Option<&'static str>,
    calls: u64,
    nanos: u64,
}

/// Time spent in each of the engine's phases since the last report.
///
/// A static because the layer that fills it is installed before the engine is built, and the app
/// that drains it is grown by the engine -- so there is no value the two could be handed instead.
static PHASES: LazyLock<Phases> = LazyLock::new(Phases::default);

/// The tallies, keyed on the phase's name.
#[derive(Default)]
struct Phases(Mutex<HashMap<&'static str, Tally>>);

impl Phases {
    /// Adds one run of `phase` under `parent`, which spent `nanos` entered.
    fn add(&self, phase: &'static str, parent: Option<&'static str>, nanos: u64) {
        let mut held = self.0.lock().expect("phase tallies");
        let tally = held.entry(phase).or_insert(Tally {
            phase,
            parent,
            calls: 0,
            nanos: 0,
        });
        tally.calls += 1;
        tally.nanos += nanos;
    }

    /// Everything tallied since the last take, costliest first, leaving the tallies empty.
    fn take(&self) -> Vec<Tally> {
        let held = core::mem::take(&mut *self.0.lock().expect("phase tallies"));
        let mut phases: Vec<Tally> = held.into_values().collect();
        phases.sort_by(|left, right| right.nanos.cmp(&left.nanos));
        phases
    }
}

/// How long a span has been entered for, kept on the span itself.
///
/// Entered time rather than wall time between opening and closing: a phase that is entered twice in
/// a frame is one phase costing what both entries cost, and the gap between them belongs to
/// whatever ran in it.
#[derive(Default)]
struct Busy {
    entered: Option<Instant>,
    nanos: u64,
}

/// The layer that times the engine's phases.
struct Stopwatch;

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Stopwatch {
    fn on_new_span(&self, _attributes: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        if let Some(span) = context.span(id) {
            span.extensions_mut().insert(Busy::default());
        }
    }

    fn on_enter(&self, id: &Id, context: Context<'_, S>) {
        if let Some(span) = context.span(id) {
            if let Some(busy) = span.extensions_mut().get_mut::<Busy>() {
                busy.entered = Some(Instant::now());
            }
        }
    }

    fn on_exit(&self, id: &Id, context: Context<'_, S>) {
        if let Some(span) = context.span(id) {
            if let Some(busy) = span.extensions_mut().get_mut::<Busy>() {
                if let Some(entered) = busy.entered.take() {
                    busy.nanos += entered.elapsed().as_nanos() as u64;
                }
            }
        }
    }

    fn on_close(&self, id: Id, context: Context<'_, S>) {
        if let Some(span) = context.span(&id) {
            let nanos = span.extensions().get::<Busy>().map(|busy| busy.nanos);
            if let Some(nanos) = nanos {
                PHASES.add(span.name(), span.parent().map(|parent| parent.name()), nanos);
            }
        }
    }
}

/// Writes what the interval's frames were spent in, as the tree the engine states them in.
///
/// The denominator is the sum of the phases nothing encloses -- the frame, and the passes that draw
/// what it settled, which run after it has returned -- because between them those are a frame end
/// to end. A nested phase's share is of that whole, and siblings under one parent sum to no more
/// than their parent's.
fn report(load: Load, cells: usize, rate: f32) {
    let phases = PHASES.take();
    let total: u64 = phases
        .iter()
        .filter(|tally| tally.parent.is_none())
        .map(|tally| tally.nanos)
        .sum();
    if total == 0 {
        return;
    }
    println!("\n{} on {cells} cells, {rate:.0} fps", load.name());
    println!(
        "  {:<20} {:>6} {:>10} {:>9}",
        "phase", "calls", "mean ms", "share"
    );
    branch(&phases, None, 0, total as f64);
}

/// Writes every phase sitting directly under `parent`, costliest first, and then what sits under
/// each of them.
fn branch(phases: &[Tally], parent: Option<&'static str>, depth: usize, total: f64) {
    if depth > 6 {
        return;
    }
    let indent = depth * 2;
    for tally in phases.iter().filter(|tally| tally.parent == parent) {
        println!(
            "  {:indent$}{:<width$} {:>6} {:>10.3} {:>8.1}%",
            "",
            tally.phase,
            tally.calls,
            tally.nanos as f64 / tally.calls as f64 / 1.0e6,
            100.0 * tally.nanos as f64 / total,
            indent = indent,
            width = 20 - indent,
        );
        branch(phases, Some(tally.phase), depth + 1, total);
    }
}
