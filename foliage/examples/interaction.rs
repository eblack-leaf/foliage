//! Click, prune, timers and sequences -- the paths that have no visual on their own, so
//! nothing else proves they work. Run with `cargo run --example interaction -p foliage`.
//!
//! Click the left panel and it fades away over a sequence; when the sequence reports done it
//! is pruned, and the `Leaf` naming it is checked to have withered. A timer then regrows it
//! a moment later, so the whole cycle repeats for as long as you keep clicking.

use foliage::{
    Moss, Forest, Color, Elevation, Foliage, Grid, GridExt, Grows, Leaf, Location, Motion, Panel,
    Presence, Root, Rounding, Sprout, Text, Timing,
};

struct Demo {
    /// The clickable panel, or `None` while it is gone and the timer is counting down.
    target: Option<Leaf>,
    /// The sequence the fade is joined to, and the timer that regrows afterwards.
    fading: Option<foliage::Leaf>,
    waiting: Option<foliage::Leaf>,
    status: Leaf,
    clicks: u32,
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((360, 220));

    foliage.root::<Demo>();
    foliage.photosynthesize();
}

impl Root for Demo {
    fn take_root(forest: &mut Forest) -> Self {
        grow(forest)
    }
    fn frame(&mut self, forest: &mut Forest, mosses: Vec<Moss>) {
        for moss in mosses {
            match moss {
                // One physical click can arrive for several elements; only ours matters.
                Moss::Clicked(leaf) if Some(leaf) == self.target => {
                    self.clicks += 1;
                    forest.text(self.status, format!("clicked {}x -- fading", self.clicks));
                    let sequence = forest.sequence();
                    forest.animate_during(leaf, Motion::Opacity(0.0), Timing::over(500), sequence);
                    self.fading = Some(sequence);
                }
                // The fade finished. Prune what it faded, and start a countdown to regrow.
                Moss::SequenceFinished(seq) if Some(seq) == self.fading => {
                    self.fading = None;
                    if let Some(target) = self.target.take() {
                        forest.prune(target);
                    }
                    self.waiting = Some(forest.timer(600));
                }
                Moss::TimerFinished(timer) if Some(timer) == self.waiting => {
                    self.waiting = None;
                    self.target = Some(panel(forest));
                    forest.text(self.status, "click the square");
                }
                // The pruned element reporting itself gone -- and a good moment to check the
                // contract, since a withered `Leaf` must read as absent and swallow writes.
                Moss::Withered(leaf) => {
                    debug_assert_eq!(forest.presence(leaf), Presence::Withered);
                    debug_assert!(forest.section(leaf).is_none());
                    forest.color(leaf, Color::red(500));
                }
                _ => {}
            }
        }
    }
}

fn grow(forest: &mut Forest) -> Demo {
    let status = forest.leaf(
        Text::new("click the square")
            .color(Color::gray(400))
            .at(Location::new().xs(
                20.px().as_left().with(340.px().as_right()),
                20.px().as_top().with(24.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    Demo {
        target: Some(panel(forest)),
        fading: None,
        waiting: None,
        status,
        clicks: 0,
    }
}

/// The clickable square. `.interactive()` is what puts it in the hit test at all.
fn panel(forest: &mut Forest) -> Leaf {
    forest.leaf(
        Panel::new()
            .color(Color::orange(600))
            .rounding(Rounding::Sm)
            .at(Location::new().xs(
                20.px().as_left().with(120.px().as_width()),
                70.px().as_top().with(120.px().as_height()),
            ))
            .elevate(Elevation::up(1))
            .grid(Grid::default())
            .interactive(),
    )
}
