//! Driving the tree from another thread with a [`Sprig`]. Run with
//! `cargo run --example off_thread -p foliage`.
//!
//! The worker owns its own state and its own clock, never touches the engine, and never
//! blocks the frame. It holds `Leaf`s it grew itself -- names it allocated off-thread, which
//! the engine binds to real elements when the commands arrive. This is the path that makes
//! foliage's choice of ECS invisible: a thread doing this could be running its own, at its
//! own version, and nothing here would change.

use foliage::{
    Moss, Forest, Color, Elevation, Foliage, GridExt, Grows, Location, Panel, Root, Rounding,
    Sample, Sap, Sprout,
};
use std::time::Duration;

const BARS: usize = 5;
/// Where the bars stand until the first frame has published a viewport to stand them against.
const FLOOR: i32 = 190;

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((360, 220));

    let mut sprig = foliage.sprig();
    std::thread::spawn(move || {
        // Grown from off the main thread, before the loop has even started. The names come
        // back immediately; the elements appear once the frame picks the commands up.
        let bars: Vec<_> = (0..BARS)
            .map(|i| {
                sprig.leaf(
                    Panel::new()
                        .color(Color::cyan(500))
                        .rounding(Rounding::Xs)
                        .at(bar(i, 0.2, FLOOR))
                        .elevate(Elevation::up(1))
                        .interactive(),
                )
            })
            .collect();

        // A dot that rides the first bar's cap. The worker sets that bar's *location* but
        // never resolves it -- percentages, the grid and the breakpoint are the engine's
        // arithmetic -- so where the bar actually ended up has to be reported back. That is
        // what a watch is: the read this side cannot take for itself.
        let marker = sprig.leaf(
            Panel::new()
                .color(Color::gray(100))
                .rounding(Rounding::Md)
                .at(bar(0, 0.0, FLOOR))
                .elevate(Elevation::up(2)),
        );
        sprig.watch(bars[0], Sap::Section);

        let mut step = 0usize;
        let mut held: Option<usize> = None;
        loop {
            std::thread::sleep(Duration::from_millis(50));
            step += 1;

            // The other direction. `mosses` hands over everything the tree reported since the
            // last pass, so this thread hears its own bars being clicked without the root
            // relaying anything -- it never wakes the frame and the frame never waits on it.
            for moss in sprig.mosses() {
                match moss {
                    Moss::Clicked(leaf) => {
                        let Some(i) = bars.iter().position(|bar| *bar == leaf) else {
                            continue;
                        };
                        let previous = held.take();
                        if let Some(prev) = previous {
                            sprig.color(bars[prev], Color::cyan(500));
                        }
                        // Clicking the held bar again just clears it.
                        if previous != Some(i) {
                            held = Some(i);
                            sprig.color(leaf, Color::orange(400));
                        }
                    }
                    // Where the first bar actually landed, as the engine resolved it.
                    Moss::Reading {
                        value: Sample::Section(section),
                        ..
                    } => {
                        let left = section.left() as i32 + 21;
                        let top = section.top() as i32 - 12;
                        sprig.location(
                            marker,
                            Location::new().xs(
                                left.px().as_left().with(8.px().as_width()),
                                top.px().as_top().with(8.px().as_height()),
                            ),
                        );
                    }
                    _ => {}
                }
            }

            // The window's own size, which this thread has no way to sample -- published each
            // frame because it is neither large nor per-element. Falls back until the first
            // frame has run.
            let floor = sprig
                .conditions()
                .map(|conditions| conditions.viewport.height() as i32 - 30)
                .unwrap_or(FLOOR);
            for (i, bar_leaf) in bars.iter().enumerate() {
                // A plain sine, computed on this thread with no engine involvement at all.
                let phase = (step as f32 * 0.15) + i as f32 * 0.7;
                let height = 0.2 + 0.6 * (0.5 + 0.5 * phase.sin());
                sprig.location(*bar_leaf, bar(i, height, floor));
            }
        }
    });

    foliage.root::<Idle>();
    foliage.photosynthesize();
}

/// Nothing to do here. Everything on screen is being driven from the worker, which is the
/// point: the root is not where an app has to live.
struct Idle;

impl Root for Idle {
    fn take_root(_forest: &mut Forest) -> Self {
        Idle
    }
    fn frame(&mut self, _forest: &mut Forest, _mosses: Vec<Moss>) {}
}

/// A bar `fraction` of the way up from `floor`, in the `i`th column.
fn bar(i: usize, fraction: f32, floor: i32) -> Location {
    let left = 20 + i as i32 * 66;
    let height = (fraction * (floor - 40) as f32) as i32;
    Location::new().xs(
        left.px().as_left().with(50.px().as_width()),
        (floor - height).px().as_top().with(height.px().as_height()),
    )
}
