//! Driving the tree from another thread with a [`Sprig`]. Run with
//! `cargo run --example off_thread -p foliage`.
//!
//! The worker owns its own state and its own clock, never touches the engine, and never
//! blocks the frame. It holds `Leaf`s it grew itself -- names it allocated off-thread, which
//! the engine binds to real elements when the commands arrive. This is the path that makes
//! foliage's choice of ECS invisible: a thread doing this could be running its own, at its
//! own version, and nothing here would change.

use foliage::{
    Canopy, Color, Elevation, Foliage, GridExt, Grows, Location, Panel, Rounding, Sprout,
};
use std::time::Duration;

const BARS: usize = 5;

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
                        .at(bar(i, 0.2))
                        .elevate(Elevation::up(1)),
                )
            })
            .collect();

        let mut step = 0usize;
        loop {
            std::thread::sleep(Duration::from_millis(90));
            step += 1;
            for (i, bar_leaf) in bars.iter().enumerate() {
                // A plain sine, computed on this thread with no engine involvement at all.
                let phase = (step as f32 * 0.15) + i as f32 * 0.7;
                let height = 0.2 + 0.6 * (0.5 + 0.5 * phase.sin());
                sprig.location(*bar_leaf, bar(i, height));
            }
        }
    });

    foliage.define_frame(move |_canopy: &mut Canopy, _blooms| {
        // Nothing to do here. Everything on screen is being driven from the worker, which is
        // the point: the frame closure is not where an app has to live.
    });
    foliage.photosynthesize();
}

/// A bar `fraction` of the way up, in the `i`th column.
fn bar(i: usize, fraction: f32) -> Location {
    let left = 20 + i as i32 * 66;
    let height = (fraction * 150.0) as i32;
    Location::new().xs(
        left.px().as_left().with(50.px().as_width()),
        (190 - height).px().as_top().with(height.px().as_height()),
    )
}
