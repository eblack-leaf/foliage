//! Click, prune, timers and sequences -- the paths that have no visual on their own, so
//! nothing else proves they work. Run with `cargo run --example interaction -p foliage`.
//!
//! Click the left panel and it fades away over a sequence; when the sequence reports done it
//! is pruned, and the `Leaf` naming it is checked to have withered. A timer then regrows it
//! a moment later, so the whole cycle repeats for as long as you keep clicking.

use foliage::{
    Bloom, Canopy, Color, Elevation, Foliage, Grid, GridExt, Grows, Leaf, Location, Motion, Panel,
    Presence, Rounding, Sprout, Text, Timing,
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

    let mut demo: Option<Demo> = None;
    foliage.define_frame(move |canopy: &mut Canopy, blooms| {
        let demo = demo.get_or_insert_with(|| grow(canopy));
        for bloom in blooms {
            match bloom {
                // One physical click can arrive for several elements; only ours matters.
                Bloom::Clicked(leaf) if Some(leaf) == demo.target => {
                    demo.clicks += 1;
                    canopy.text(demo.status, format!("clicked {}x -- fading", demo.clicks));
                    let sequence = canopy.sequence();
                    canopy.animate_during(leaf, Motion::Opacity(0.0), Timing::over(500), sequence);
                    demo.fading = Some(sequence);
                }
                // The fade finished. Prune what it faded, and start a countdown to regrow.
                Bloom::SequenceFinished(seq) if Some(seq) == demo.fading => {
                    demo.fading = None;
                    if let Some(target) = demo.target.take() {
                        canopy.prune(target);
                    }
                    demo.waiting = Some(canopy.timer(600));
                }
                Bloom::TimerFinished(timer) if Some(timer) == demo.waiting => {
                    demo.waiting = None;
                    demo.target = Some(panel(canopy));
                    canopy.text(demo.status, "click the square");
                }
                // The pruned element reporting itself gone -- and a good moment to check the
                // contract, since a withered `Leaf` must read as absent and swallow writes.
                Bloom::Withered(leaf) => {
                    debug_assert_eq!(canopy.presence(leaf), Presence::Withered);
                    debug_assert!(canopy.section(leaf).is_none());
                    canopy.color(leaf, Color::red(500));
                }
                _ => {}
            }
        }
    });
    foliage.photosynthesize();
}

fn grow(canopy: &mut Canopy) -> Demo {
    let status = canopy.leaf(
        Text::new("click the square")
            .color(Color::gray(400))
            .at(Location::new().xs(
                20.px().as_left().with(340.px().as_right()),
                20.px().as_top().with(24.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    Demo {
        target: Some(panel(canopy)),
        fading: None,
        waiting: None,
        status,
        clicks: 0,
    }
}

/// The clickable square. `.interactive()` is what puts it in the hit test at all.
fn panel(canopy: &mut Canopy) -> Leaf {
    canopy.leaf(
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
