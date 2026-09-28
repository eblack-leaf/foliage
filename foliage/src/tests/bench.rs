//! Frame cost, by aspect. A measurement rather than a test of behaviour, so every case here is
//! `#[ignore]`d and runs only when asked for:
//!
//! ```sh
//! cargo test -p foliage --release --lib bench -- --ignored --nocapture --test-threads 1
//! ```
//!
//! Headless, against the same [`fern::run`](crate::fern::run) the platform loop calls, with the
//! clock moved by hand -- so what is timed is the frame on the processor and nothing else: no
//! surface, no present, no wait for the display. What the backend does with a batch is the one part
//! of the frame this cannot see, and the `stress` example under a real surface is where that is
//! read.
//!
//! Each load writes one kind of thing, so what separates two readings is where the writes land.
//! Every load runs at more than one size, because the claim worth checking about most of them is
//! how the cost grows with the tree rather than what it is at one size.
//!
//! # Knobs
//!
//! All optional, all environment variables so a filter reaches through `cargo test`:
//!
//! - `FOLIAGE_BENCH` -- a comma-separated list of load names to run, rather than all of them.
//! - `FOLIAGE_BENCH_CELLS` -- a comma-separated list of cell counts, rather than the default two.
//! - `FOLIAGE_BENCH_FRAMES` -- how many frames each reading is taken over.
//! - `FOLIAGE_BENCH_PHASES` -- set to anything to follow each reading with where its frames went,
//!   from the engine's own spans. A separate run, so the spans' own cost is not in the headline.

use core::time::Duration;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;

use tracing::Subscriber;
use tracing::span::{Attributes, Id};
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;

use crate::coordinate::{Area, Axes, Position};
use crate::interaction::input::{Input, Key};
use crate::tests::{advance, key, resize, tick};
use crate::{
    Boxed, Color, Ease, FontSize, Grove, Grow, Leaf, Location, Motion, Palette, Panel, Place,
    Polygon, Rounding, Scheme, Source, Stem, Step, Text, TextArea, Timing, Tween, content, left,
    top,
};

/// The page the field is divided against, which is what the stress example opens at.
const PAGE: Area = Area {
    width: 1024.0,
    height: 768.0,
};

/// How many cells each load runs at, unless `FOLIAGE_BENCH_CELLS` says otherwise. Each cell is two
/// elements: a shape, and a label grown on it.
const CELLS: [usize; 2] = [512, 4096];

/// Frames each reading is taken over, after the warm-up.
const FRAMES: usize = 120;

/// Frames run before anything is timed: the first frames grow the field and fill every cache, and
/// a load's steady state is what is being read.
const WARM: usize = 20;

/// One load: what it grows, and what it writes each frame.
struct Load {
    name: &'static str,
    /// What the load saturates, for the table.
    what: &'static str,
    grow: fn(&mut Grove, usize) -> Field,
    step: fn(&mut Grove, &mut Field, usize),
}

/// What a load grew, and whatever it keeps between frames.
#[derive(Default)]
struct Field {
    cells: Vec<Cell>,
    columns: usize,
    rows: usize,
    /// Whatever else a load names: the region, the page, the area being typed into.
    held: Vec<Leaf>,
    /// The motion running on each cell, for the loads that keep one live.
    tweens: Vec<Tween>,
    /// Where a rolling load writes next.
    cursor: usize,
}

struct Cell {
    shape: Leaf,
    label: Leaf,
}

const LOADS: &[Load] = &[
    Load {
        name: "idle",
        what: "nothing written: the floor a frame costs at this size",
        grow: field,
        step: |_, _, _| {},
    },
    Load {
        name: "layout",
        what: "every cell placed somewhere new",
        grow: field,
        step: |grove, field, frame| wander(grove, field, frame, 1),
    },
    Load {
        name: "layout/64",
        what: "a sixty-fourth of the cells placed somewhere new",
        grow: field,
        step: |grove, field, frame| wander(grove, field, frame, 64),
    },
    Load {
        name: "color",
        what: "every cell refilled",
        grow: field,
        step: |grove, field, frame| recolor(grove, field, frame, 1),
    },
    Load {
        name: "color/64",
        what: "a sixty-fourth of the cells refilled",
        grow: field,
        step: |grove, field, frame| recolor(grove, field, frame, 64),
    },
    Load {
        name: "text",
        what: "every label rewritten with a value nothing has shaped",
        grow: field,
        step: |grove, field, frame| relabel(grove, field, frame, 1),
    },
    Load {
        name: "text/64",
        what: "a sixty-fourth of the labels rewritten",
        grow: field,
        step: |grove, field, frame| relabel(grove, field, frame, 64),
    },
    Load {
        name: "move",
        what: "a location motion live on every cell",
        grow: field,
        step: travel,
    },
    Load {
        name: "fade",
        what: "one opacity motion on the container of every cell",
        grow: field,
        step: fade,
    },
    Load {
        name: "hover",
        what: "one colour motion on one cell, as a hover would",
        grow: field,
        step: hover,
    },
    Load {
        name: "churn",
        what: "32 cells pruned and grown again",
        grow: field,
        step: churn,
    },
    Load {
        name: "repaint",
        what: "the scheme restated, so every role resolves again",
        grow: field,
        step: |grove, _, frame| {
            grove.repaint(Scheme::new().set(Palette::Accent, wheel(frame as f32 * 0.01)));
        },
    },
    Load {
        name: "toggle",
        what: "the field hidden and shown on alternate frames",
        grow: field,
        step: |grove, field, frame| grove.visible(field.held[0], frame % 2 == 0),
    },
    Load {
        name: "resize",
        what: "the viewport resized, which is every element written",
        grow: field,
        step: |grove, _, frame| {
            let width = PAGE.width - (frame % 2) as f32 * 64.0;
            resize(grove, Area::new(width, PAGE.height));
        },
    },
    Load {
        name: "scroll",
        what: "a region holding every cell, wheeled every frame",
        grow: region,
        step: wheeled,
    },
    Load {
        name: "deep",
        what: "a chain of stems nested cells deep, its root moved every frame",
        grow: chain,
        step: |grove, field, frame| {
            let x = (frame % 2) as f32;
            grove.at(
                field.held[0],
                Location::new().xs(left(x.px()).right(100.pct()), top(0.px()).bottom(100.pct())),
            );
        },
    },
    Load {
        name: "type",
        what: "a character typed and taken back in an area holding cells words",
        grow: area,
        step: |grove, _, frame| {
            key(
                grove,
                match frame % 2 {
                    0 => Key::Typed('x'),
                    _ => Key::Backspace,
                },
            );
        },
    },
];

#[test]
#[ignore = "a measurement; run with --ignored --nocapture"]
fn bench() {
    let chosen = std::env::var("FOLIAGE_BENCH").ok();
    let chosen: Option<Vec<&str>> = chosen.as_deref().map(|list| list.split(',').collect());
    let cells: Vec<usize> = std::env::var("FOLIAGE_BENCH_CELLS")
        .ok()
        .map(|list| {
            list.split(',')
                .filter_map(|n| n.trim().parse().ok())
                .collect()
        })
        .unwrap_or_else(|| CELLS.to_vec());
    let frames: usize = std::env::var("FOLIAGE_BENCH_FRAMES")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(FRAMES);
    let phases = std::env::var("FOLIAGE_BENCH_PHASES").is_ok();
    println!();
    println!(
        "  {:<11} {:>6} {:>9} {:>9} {:>9} {:>9}   what",
        "load", "cells", "elements", "median", "mean", "min"
    );
    for load in LOADS {
        if chosen
            .as_ref()
            .is_some_and(|chosen| !chosen.contains(&load.name))
        {
            continue;
        }
        for &count in &cells {
            let reading = measure(load, count, frames);
            println!(
                "  {:<11} {:>6} {:>9} {:>7.3}ms {:>7.3}ms {:>7.3}ms   {}",
                load.name,
                count,
                reading.elements,
                reading.median,
                reading.mean,
                reading.min,
                load.what
            );
            if phases {
                traced(load, count, frames);
            }
        }
    }
}

/// What a reading came to, in milliseconds per frame.
struct Reading {
    elements: usize,
    median: f64,
    mean: f64,
    min: f64,
}

/// Runs `load` at `count` cells and times `frames` of its steady state.
fn measure(load: &Load, count: usize, frames: usize) -> Reading {
    let (mut grove, mut field) = grown(load, count);
    let mut times = Vec::with_capacity(frames);
    for frame in WARM..WARM + frames {
        (load.step)(&mut grove, &mut field, frame);
        advance(&mut grove, 16);
        let started = Instant::now();
        tick(&mut grove);
        times.push(started.elapsed());
    }
    times.sort_unstable();
    let ms = |duration: Duration| duration.as_secs_f64() * 1000.0;
    Reading {
        elements: grove.elements.len(),
        median: ms(times[times.len() / 2]),
        mean: ms(times.iter().sum::<Duration>()) / times.len() as f64,
        min: ms(times[0]),
    }
}

/// A grove with `load`'s field grown on it and warmed into its steady state.
fn grown(load: &Load, count: usize) -> (Grove, Field) {
    let mut grove = Grove::new(PAGE);
    let mut field = (load.grow)(&mut grove, count);
    for frame in 0..WARM {
        (load.step)(&mut grove, &mut field, frame);
        advance(&mut grove, 16);
        tick(&mut grove);
    }
    (grove, field)
}

/// The same run under the engine's own spans, reported as where each frame went.
fn traced(load: &Load, count: usize, frames: usize) {
    let (mut grove, mut field) = grown(load, count);
    TALLIES.with(|tallies| tallies.borrow_mut().clear());
    let subscriber = tracing_subscriber::registry().with(Stopwatch);
    tracing::subscriber::with_default(subscriber, || {
        for frame in WARM..WARM + frames {
            (load.step)(&mut grove, &mut field, frame);
            advance(&mut grove, 16);
            tick(&mut grove);
        }
    });
    let tallies = TALLIES.with(|tallies| core::mem::take(&mut *tallies.borrow_mut()));
    let mut rows: Vec<(&'static str, Option<&'static str>, u64)> = tallies
        .into_iter()
        .map(|((phase, parent), nanos)| (phase, parent, nanos))
        .collect();
    rows.sort_by_key(|row| core::cmp::Reverse(row.2));
    branch(&rows, None, 0, frames);
}

/// Writes every phase sitting directly under `parent`, costliest first, then what sits under each.
fn branch(
    rows: &[(&'static str, Option<&'static str>, u64)],
    parent: Option<&'static str>,
    depth: usize,
    frames: usize,
) {
    for &(phase, above, nanos) in rows.iter().filter(|row| row.1 == parent) {
        let per = nanos as f64 / frames as f64 / 1.0e6;
        if per < 0.001 {
            continue;
        }
        println!(
            "      {:indent$}{:<width$} {:>8.3}ms",
            "",
            phase,
            per,
            indent = depth * 2,
            width = 16 - depth * 2,
        );
        let _ = above;
        branch(rows, Some(phase), depth + 1, frames);
    }
}

thread_local! {
    /// Time spent in each phase, keyed on its name and the name of what encloses it.
    static TALLIES: RefCell<HashMap<(&'static str, Option<&'static str>), u64>> =
        RefCell::new(HashMap::new());
}

/// How long a span has been entered for.
#[derive(Default)]
struct Busy {
    entered: Option<Instant>,
    nanos: u64,
}

/// The layer that times the engine's phases, as the stress example's does.
struct Stopwatch;

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Stopwatch {
    fn on_new_span(&self, _attributes: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        if let Some(span) = context.span(id) {
            span.extensions_mut().insert(Busy::default());
        }
    }

    fn on_enter(&self, id: &Id, context: Context<'_, S>) {
        if let Some(span) = context.span(id)
            && let Some(busy) = span.extensions_mut().get_mut::<Busy>()
        {
            busy.entered = Some(Instant::now());
        }
    }

    fn on_exit(&self, id: &Id, context: Context<'_, S>) {
        if let Some(span) = context.span(id)
            && let Some(busy) = span.extensions_mut().get_mut::<Busy>()
            && let Some(entered) = busy.entered.take()
        {
            busy.nanos += entered.elapsed().as_nanos() as u64;
        }
    }

    fn on_close(&self, id: Id, context: Context<'_, S>) {
        if let Some(span) = context.span(&id) {
            let nanos = span.extensions().get::<Busy>().map(|busy| busy.nanos);
            if let Some(nanos) = nanos {
                let key = (span.name(), span.parent().map(|parent| parent.name()));
                TALLIES.with(|tallies| *tallies.borrow_mut().entry(key).or_default() += nanos);
            }
        }
    }
}

// -- What the loads grow ---------------------------------------------------------------------------

/// A field of `count` cells filling the page, under one container: panels, with every fourth a
/// polygon, and a label grown on each. What the stress example grows.
fn field(grove: &mut Grove, count: usize) -> Field {
    let page = grove.plant(Panel::new());
    let container = grove.branch(
        page,
        Stem::new()
            .intangible()
            .at(Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).bottom(100.pct()))),
    );
    let columns = ((count as f32 * PAGE.width / PAGE.height).sqrt().ceil() as usize).max(1);
    let rows = count.div_ceil(columns);
    let mut field = Field {
        columns,
        rows,
        held: vec![container],
        ..Field::default()
    };
    field.cells = (0..count)
        .map(|n| cell(grove, container, &field, n))
        .collect();
    field
}

/// Cell `n`, grown under `under` at its seat.
fn cell(grove: &mut Grove, under: Leaf, field: &Field, n: usize) -> Cell {
    let at = seat(field, n, 0.0);
    let shape = match n % 4 {
        0 => grove.branch(
            under,
            Polygon::new()
                .sides(6.0)
                .rounding(0.25)
                .color(tone(n))
                .intangible()
                .at(at),
        ),
        _ => grove.branch(
            under,
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

/// Where cell `n` sits, as a fraction of what it is grown under, displaced by `wander`.
fn seat(field: &Field, n: usize, wander: f32) -> Location {
    let width = 100.0 / field.columns as f32;
    let height = 100.0 / field.rows as f32;
    let x = (n % field.columns) as f32 * width + wander;
    let y = (n / field.columns) as f32 * height + wander;
    Location::new().xs(
        left(x.pct()).width((width * 0.86).pct()),
        top(y.pct()).height((height * 0.86).pct()),
    )
}

/// The role cell `n` is filled with, so a repaint has something to move.
fn tone(n: usize) -> Palette {
    match n % 5 {
        0 => Palette::Accent,
        1 => Palette::Raised.at(Step::Nearest),
        2 => Palette::Muted,
        3 => Palette::Ink.at(Step::Far),
        _ => Palette::Accent.at(Step::Far),
    }
}

/// A colour at `turn` of the way round.
fn wheel(turn: f32) -> Color {
    let channel = |offset: f32| ((turn + offset) * core::f32::consts::TAU).sin() * 0.4 + 0.5;
    Color::rgb(channel(0.0), channel(1.0 / 3.0), channel(2.0 / 3.0))
}

/// The same cells, in a region four pages tall that scrolls, laid out in pixels so the content
/// reaches past it.
fn region(grove: &mut Grove, count: usize) -> Field {
    let region = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).bottom(100.pct())))
            .scrolls(Axes::Vertical),
    );
    let content = grove.branch(
        region,
        Stem::new().intangible().at(Location::new().xs(
            left(0.px()).right(100.pct()),
            top(0.px()).height((PAGE.height * 4.0).px()),
        )),
    );
    let columns = ((count as f32 * PAGE.width / (PAGE.height * 4.0))
        .sqrt()
        .ceil() as usize)
        .max(1);
    let rows = count.div_ceil(columns);
    let mut field = Field {
        columns,
        rows,
        held: vec![region, content],
        ..Field::default()
    };
    field.cells = (0..count)
        .map(|n| cell(grove, content, &field, n))
        .collect();
    field
}

/// A chain of `count` stems, each grown under the last, with a label on every one.
fn chain(grove: &mut Grove, count: usize) -> Field {
    let root = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).bottom(100.pct()))),
    );
    let mut under = root;
    let mut field = Field {
        held: vec![root],
        ..Field::default()
    };
    for n in 0..count {
        let shape = grove.branch(
            under,
            Panel::new()
                .color(tone(n))
                .intangible()
                .at(Location::new().xs(
                    left(0.05.pct()).width(99.9.pct()),
                    top(0.05.pct()).height(99.9.pct()),
                )),
        );
        let label = grove.branch(
            shape,
            Text::new(format!("{:03x}", n & 0xfff))
                .font_size(FontSize::new().xs(11))
                .at(Location::new()
                    .xs(left(0.px()).width(content()), top(0.px()).height(content()))),
        );
        field.cells.push(Cell { shape, label });
        under = shape;
    }
    field
}

/// An area filling the page with `count` words in it, focused, with the caret at the end.
fn area(grove: &mut Grove, count: usize) -> Field {
    let leaf = grove.plant(
        TextArea::new()
            .at(Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).bottom(100.pct()))),
    );
    let words: Vec<String> = (0..count).map(|n| format!("w{n:x}")).collect();
    grove.text(leaf, words.join(" "));
    tick(grove);
    grove.focus(leaf);
    Field {
        held: vec![leaf],
        ..Field::default()
    }
}

// -- What the loads write --------------------------------------------------------------------------

/// Which cells a rolling load writes this frame: a `1/share` slice of the field, moving on.
fn window(field: &mut Field, share: usize) -> impl Iterator<Item = usize> + use<> {
    let count = field.cells.len();
    let taken = count.div_ceil(share).max(1);
    let from = field.cursor;
    field.cursor = (from + taken) % count.max(1);
    (0..taken).map(move |step| (from + step) % count)
}

fn wander(grove: &mut Grove, field: &mut Field, frame: usize, share: usize) {
    for n in window(field, share) {
        let phase = frame as f32 * 0.03 + n as f32 * 0.35;
        grove.at(field.cells[n].shape, seat(field, n, phase.sin() * 1.2));
    }
}

fn recolor(grove: &mut Grove, field: &mut Field, frame: usize, share: usize) {
    let span = field.cells.len() as f32;
    for n in window(field, share) {
        grove.color(
            field.cells[n].shape,
            wheel(frame as f32 * 0.01 + n as f32 / span),
        );
    }
}

fn relabel(grove: &mut Grove, field: &mut Field, frame: usize, share: usize) {
    for n in window(field, share) {
        grove.text(
            field.cells[n].label,
            format!("{:02x}{:03x}", frame & 0xff, n & 0xfff),
        );
    }
}

/// Keeps a location motion live on every cell, starting each again as it lands.
fn travel(grove: &mut Grove, field: &mut Field, frame: usize) {
    if field.tweens.is_empty() {
        field.tweens = (0..field.cells.len())
            .map(|n| begin(grove, field, n, frame))
            .collect();
        return;
    }
    // A motion that has landed is no longer running, and finishing one that is not is dropped --
    // so the cheap way to keep every cell moving is to start them all again on a cadence shorter
    // than the shortest of them.
    if frame.is_multiple_of(30) {
        for n in 0..field.cells.len() {
            field.tweens[n] = begin(grove, field, n, frame);
        }
    }
}

fn begin(grove: &mut Grove, field: &Field, n: usize, frame: usize) -> Tween {
    let out = (frame / 30).is_multiple_of(2);
    grove.animate(
        field.cells[n].shape,
        Motion::Location(seat(field, n, if out { 1.2 } else { 0.0 })),
        Timing::ms(600 + (n as u64 * 37) % 400).ease(Ease::Emphasis),
    )
}

/// Keeps the container fading one way and then the other.
fn fade(grove: &mut Grove, field: &mut Field, frame: usize) {
    if frame.is_multiple_of(30) {
        let to = if (frame / 30).is_multiple_of(2) {
            0.4
        } else {
            1.0
        };
        grove.animate(field.held[0], Motion::Opacity(to), Timing::ms(1000));
    }
}

/// Keeps one cell's fill moving, the way a pointer resting on a control would.
fn hover(grove: &mut Grove, field: &mut Field, frame: usize) {
    if frame.is_multiple_of(30) {
        let to = if (frame / 30).is_multiple_of(2) {
            Palette::Accent
        } else {
            Palette::Muted
        };
        grove.animate(field.cells[0].shape, Motion::Palette(to), Timing::ms(1000));
    }
}

fn churn(grove: &mut Grove, field: &mut Field, _frame: usize) {
    let container = field.held[0];
    for _ in 0..32.min(field.cells.len()) {
        let n = field.cursor % field.cells.len();
        grove.prune(field.cells[n].shape);
        field.cells[n] = cell(grove, container, field, n);
        field.cursor = field.cursor.wrapping_add(1);
    }
}

/// A wheel notch at the middle of the page every frame, down for a stretch and then back up.
fn wheeled(grove: &mut Grove, _field: &mut Field, frame: usize) {
    let down = (frame / 50).is_multiple_of(2);
    grove.incoming.take(Input::Wheeled {
        at: Position::new(PAGE.width / 2.0, PAGE.height / 2.0),
        delta: Position::new(0.0, if down { -20.0 } else { 20.0 }),
    });
}
