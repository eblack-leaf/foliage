//! A count worn on a press's corner.

use foliage::{
    Boxed, Ease, Elevation, Fill, Grove, Grow, Leaf, Location, Motion, Panel, Place, Position,
    Rounding, Corners, Source, Text, Timing, center_x, center_y, content,
};

/// A hop up and a little bounce on landing.
const HOP: Ease = Ease::Curve {
    x1: 0.25,
    y1: 1.6,
    x2: 0.5,
    y2: 1.0,
};

/// A number worn on the corner of a press -- what is in the bag, how many clashes -- that hops
/// when it changes and is gone at zero.
///
/// lichen's [`Badge`](lichen::Badge) says that something wants a person, and only that: a dot.
/// This says how many.
pub struct Tally {
    pill: Leaf,
    count: Leaf,
    n: u32,
}

impl Tally {
    /// A tally on the top right corner of `on`, in `fill` with `ink` read on it. At zero, unseen.
    pub fn grow(grove: &mut Grove, on: Leaf, fill: impl Into<Fill>, ink: impl Into<Fill>) -> Self {
        let pill = grove.branch(
            on,
            Panel::new()
                .color(fill)
                .rounding(Corners::all(Rounding::Full))
                .intangible()
                .elevate(Elevation::up(4))
                .visible(false)
                .at(Location::new().xs(
                    center_x(100.pct() - 3.px()).width(content() + 10.px()).at_least(18.px()),
                    center_y(3.px()).height(18.px()),
                )),
        );
        let count = grove.branch(
            pill,
            Text::new("0")
                .color(ink)
                .font_size(foliage::FontSize::new().xs(11))
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    center_x(50.pct()).width(content()),
                    center_y(50.pct()).height(1.letters()),
                )),
        );
        Self { pill, count, n: 0 }
    }

    /// The pill.
    pub fn leaf(&self) -> Leaf {
        self.pill
    }

    /// What it counts.
    pub fn count(&self) -> u32 {
        self.n
    }

    /// Counts `n`: hops if it went up, shows at more than zero and hides at zero. Nothing, if it
    /// counts that already.
    pub fn set(&mut self, grove: &mut Grove, n: u32) {
        if self.n == n {
            return;
        }
        let up = n > self.n;
        self.n = n;
        grove.text(self.count, if n > 99 { "99+".to_string() } else { n.to_string() });
        grove.visible(self.pill, n > 0);
        if n > 0 && up {
            grove.shift(self.pill, Position::new(0.0, -7.0));
            grove.animate(self.pill, Motion::Shift(Position::default()), Timing::ms(420).ease(HOP));
        }
    }
}
