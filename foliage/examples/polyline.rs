//! Plain vs. dashed `Polyline`, with a draw-progress loop driven by a tween. Run with
//! `cargo run --example polyline -p foliage`.
//!
//! The tween is the shape of every "value changing over time" in an app now: foliage owns the
//! clock and the easing, hands back plain numbers each frame, and what they mean is entirely
//! the app's business -- here, how much of a line is drawn.

use foliage::{
    Bloom, Canopy, Color, DashPattern, Elevation, Foliage, GridExt, Leaf, Location, Polyline,
    Position, Repeat, Root, Timing, Tween,
};
use foliage::{Grows, Sprout};

const DRAW_CYCLE_MS: u64 = 1500;

/// What this app keeps between frames. Ordinary Rust state in an ordinary struct -- it is
/// never handed to the engine, and the engine has no way to reach it.
struct Drawing {
    line: Leaf,
    cycle: Tween,
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((420, 160));

    foliage.root::<Drawing>();
    foliage.photosynthesize();
}

impl Root for Drawing {
    fn take_root(canopy: &mut Canopy) -> Self {
        let zigzag: Vec<Position<foliage::Logical>> = vec![
            (10, 90).into(),
            (50, 30).into(),
            (90, 90).into(),
            (130, 30).into(),
            (170, 70).into(),
        ];
        grow(canopy, &zigzag)
    }
    fn frame(&mut self, canopy: &mut Canopy, blooms: Vec<Bloom>) {
        for bloom in blooms {
            if let Bloom::Tween { tween, values } = bloom
                && tween == self.cycle
            {
                canopy.draw_progress(self.line, values[0]);
            }
        }
    }
}

/// Sprout the tree on the first frame and keeps the two handles that matter.
fn grow(canopy: &mut Canopy, points: &[Position<foliage::Logical>]) -> Drawing {
    let line = canopy.leaf(
        Polyline::new()
            .points(points.to_vec())
            .weight(3)
            .color(Color::gray(300))
            .at(Location::new().xs(
                20.px().as_left().with(180.px().as_width()),
                20.px().as_top().with(100.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    canopy.leaf(
        Polyline::new()
            .points(points.to_vec())
            .weight(3)
            .color(Color::gray(300))
            .dash(DashPattern::new(10.0, 6.0))
            .at(Location::new().xs(
                220.px().as_left().with(180.px().as_width()),
                20.px().as_top().with(100.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    // One channel running 0 to 1, forever.
    let cycle = canopy.tween(
        vec![(0.0, 1.0)],
        Timing::over(DRAW_CYCLE_MS).repeat(Repeat::Forever),
    );
    Drawing { line, cycle }
}
