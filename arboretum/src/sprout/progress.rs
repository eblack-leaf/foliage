//! How far along something is.

use foliage::{
    Boxed, Ease, Elevation, Fill, Grove, Grow, Leaf, Location, Motion, Palette, Panel, Place,
    Rounding, Corners, Source, Timing, left, top,
};

/// A bar that fills as something gets along, with a tick at each of its steps.
pub struct Progress {
    root: Leaf,
    fill: Leaf,
    at: f32,
}

impl Progress {
    /// A bar under `under`, standing where `at` says, filled in `fill` on `track`, marked into
    /// `steps` with ticks where there are more than one. Empty.
    pub fn grow(
        grove: &mut Grove,
        under: Leaf,
        at: Location,
        steps: usize,
        track: impl Into<Fill>,
        fill: impl Into<Fill>,
    ) -> Self {
        let root = grove.branch(
            under,
            Panel::new()
                .color(track)
                .rounding(Corners::all(Rounding::Full))
                .intangible()
                .at(at),
        );
        let fill = grove.branch(
            root,
            Panel::new()
                .color(fill)
                .rounding(Corners::all(Rounding::Full))
                .intangible()
                .elevate(Elevation::up(1))
                .at(filled(0.0)),
        );
        for n in 1..steps {
            let x = n as f32 / steps as f32 * 100.0;
            grove.branch(
                root,
                Panel::new()
                    .color(Palette::Surface)
                    .intangible()
                    .elevate(Elevation::up(2))
                    .at(Location::new().xs(
                        left(x.pct() - 1.px()).width(2.px()),
                        top(0.px()).height(100.pct()),
                    )),
            );
        }
        Self { root, fill, at: 0.0 }
    }

    /// The bar.
    pub fn leaf(&self) -> Leaf {
        self.root
    }

    /// How full it is, in `0.0..=1.0`.
    pub fn at(&self) -> f32 {
        self.at
    }

    /// Fills the bar to `to`, in `0.0..=1.0`, over `ms`. At once where `ms` is zero.
    pub fn set(&mut self, grove: &mut Grove, to: f32, ms: u64) {
        let to = to.clamp(0.0, 1.0);
        if (to - self.at).abs() < 1e-4 {
            return;
        }
        self.at = to;
        match ms {
            0 => grove.at(self.fill, filled(to)),
            ms => {
                grove.animate(
                    self.fill,
                    Motion::Location(filled(to)),
                    Timing::ms(ms).ease(Ease::Decelerate),
                );
            }
        }
    }

    /// Recolours what is filled.
    pub fn tint(&mut self, grove: &mut Grove, fill: impl Into<Fill>, ms: u64) {
        grove.animate(self.fill, Motion::from(fill.into()), Timing::ms(ms));
    }
}

fn filled(to: f32) -> Location {
    Location::new().xs(
        left(0.px()).width((to * 100.0).pct()),
        top(0.px()).height(100.pct()),
    )
}
