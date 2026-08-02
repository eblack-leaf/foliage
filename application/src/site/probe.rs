//! Throwaway on-screen scroll instrumentation.
//!
//! Delete this module, its `mod` line, and the `Probe` on `Site` once the scroll question is
//! settled. It exists because the numbers have to be read on a device where a log is not
//! reachable.
//!
//! Pinned to the viewport as a root-level leaf, the way the drawer's scrim is -- inside the
//! router it would scroll away with the content it is measuring.

use foliage::{
    Canopy, Color, Elevation, FontSize, Grid, GridExt, Grows, HorizontalAlignment, Leaf, Location,
    Panel, Rounding, Sprout, Text, VerticalAlignment,
};

use crate::site::{role, space, type_scale};

const H: i32 = 18;

/// The readings, and the line they are written to.
pub(crate) struct Probe {
    line: Leaf,
    /// Last offset seen, to difference against.
    last: Option<f32>,
    /// px the view moved on the previous tick.
    per_tick: f32,
    /// Largest px/tick seen since the last press -- a coast's peak is over in a few frames
    /// and a live number is unreadable at that speed.
    peak: f32,
    /// Velocity handed off at the last release, px/ms. No latch needed -- the engine resets
    /// the pointer velocity only when a gesture starts, so after a release it holds the exact
    /// value the coast was given until the next touch.
    release: f32,
    shown: Option<String>,
}

/// A line pinned to the bottom of the viewport.
pub(crate) fn build(canopy: &mut Canopy) -> Probe {
    let backing = canopy.leaf(
        Panel::new()
            .color(role::surface())
            .rounding(Rounding::None)
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                100.pct().as_bottom().with(H.px().as_height()),
            ))
            .elevate(Elevation::up(9))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0))),
    );
    let line = canopy.branch(
        backing,
        Text::new("scroll  --")
            .size(FontSize::new(type_scale::LABEL))
            .color(Color::amber(400))
            .at(Location::new().xs(
                space::SM.px().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(10))
            .align(HorizontalAlignment::Left, VerticalAlignment::Middle),
    );
    Probe {
        line,
        last: None,
        per_tick: 0.0,
        peak: 0.0,
        release: 0.0,
        shown: None,
    }
}

impl Probe {
    /// One frame of it. `container` is the current page's scroll container, or `None` on a
    /// route that has nothing to scroll.
    pub(crate) fn drive(&mut self, canopy: &mut Canopy, container: Option<Leaf>) {
        let Some(offset) = container.and_then(|view| canopy.scroll_offset(view)) else {
            return;
        };
        let offset = offset.top();
        self.release = canopy.pointer_velocity().top();
        // the velocity is zeroed when a gesture starts, so this is "a new touch began" --
        // without it `peak` holds the first flick's number forever and every later one reads
        // as smaller
        if self.release == 0.0 {
            self.peak = 0.0;
        }

        if let Some(last) = self.last {
            self.per_tick = offset - last;
            if self.per_tick.abs() > self.peak.abs() {
                self.peak = self.per_tick;
            }
        }
        self.last = Some(offset);

        // frame time here rather than inferred from peak/v: the hero's own readout is on a
        // route with nothing to scroll, so it can never be on screen while this is happening
        let line = format!(
            "v {:>5.2}  now {:>6.1}  peak {:>6.1}  {:>4.1}ms  off {:>5.0}",
            self.release,
            self.per_tick,
            self.peak,
            canopy.frame_time().as_secs_f32() * 1000.0,
            offset
        );
        if self.shown.as_deref() == Some(line.as_str()) {
            return;
        }
        self.shown = Some(line.clone());
        canopy.text(self.line, line);
    }
}
