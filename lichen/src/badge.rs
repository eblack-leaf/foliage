//! A badge: something here wants a person.
//!
//! The lasting one of the three ways something asks to be noticed. A [`Ping`](crate::Ping) goes
//! by itself and a [`Say`](crate::Say) with the next change; a badge stays until whatever it is
//! about is dealt with, and the app says when that is. What it answers is a state and not an
//! event: something failed and is waiting to be tried again, something is waiting to go.
//!
//! A dot in the system's hue on the corner of what it is about -- on it and not in it, so what it
//! is on need not know it is there. Or, where how many matters -- what is in a bag, how many
//! clashes -- a count in a pill, which hops when it goes up and is gone at nothing.

use foliage::{
    Boxed, Ease, Elevation, FontSize, Grove, Grow, Leaf, Location, Motion, Panel, Place, Position,
    Source, Text, Timing, center_x, center_y, content,
};

use crate::chip::Chip;
use crate::measure::measure;
use crate::tone::{self, Tone};

/// How tall a counting badge's pill is, and its count's size.
const PILL: f32 = 18.0;
const COUNT: u32 = 11;
/// How far a count hops when it goes up, and how long it takes to land.
const HOP: f32 = 7.0;
const HOP_MS: u64 = 420;
/// A hop up and a little bounce on landing.
const LANDING: Ease = Ease::Curve {
    x1: 0.25,
    y1: 1.6,
    x2: 0.5,
    y2: 1.0,
};

/// A badge, grown, and whether it is shown.
pub struct Badge {
    dot: Leaf,
    shown: bool,
    /// What a counting badge says, and how many it counts.
    count: Option<(Leaf, u32)>,
}

impl Badge {
    /// Grows a badge on `chip`'s top corner at its end, where it reads as on the chip and not
    /// beside it. Hidden.
    pub fn on(grove: &mut Grove, chip: &Chip) -> Self {
        let side = measure().dot;
        Self::grow(
            grove,
            chip.leaf(),
            Location::new().xs(
                center_x(100.pct()).width(side.px()),
                center_y(0.pct()).height(side.px()),
            ),
        )
    }

    /// Grows a badge under `under`, standing where `at` says. Hidden.
    pub fn grow(grove: &mut Grove, under: Leaf, at: Location) -> Self {
        let dot = grove.branch(
            under,
            Panel::new()
                .at(at)
                .elevate(Elevation::up(4))
                .intangible()
                .opacity(0.0)
                .color(tone::HINT)
                .rounding(tone::corner()),
        );
        Self {
            dot,
            shown: false,
            count: None,
        }
    }

    /// Grows a counting badge on the top corner of `on` at its end, in `tone`: a pill with a number
    /// in it, shown at more than nothing. Counting nothing.
    pub fn counting(grove: &mut Grove, on: Leaf, tone: Tone) -> Self {
        let pill = grove.branch(
            on,
            Panel::new()
                .color(tone.fill)
                .rounding(foliage::Corners::all(foliage::Rounding::Full))
                .intangible()
                .elevate(Elevation::up(4))
                .visible(false)
                .at(Location::new().xs(
                    center_x(100.pct() - 3.px()).width(content() + 10.px()).at_least(PILL.px()),
                    center_y(3.px()).height(PILL.px()),
                )),
        );
        let count = grove.branch(
            pill,
            Text::new("0")
                .color(tone.ink)
                .font_size(FontSize::new().xs(COUNT))
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    center_x(50.pct()).width(content()),
                    center_y(50.pct()).height(1.letters()),
                )),
        );
        Self {
            dot: pill,
            shown: false,
            count: Some((count, 0)),
        }
    }

    /// How many a counting badge counts: nothing, for a dot.
    pub fn count(&self) -> u32 {
        self.count.map_or(0, |(_, n)| n)
    }

    /// Counts `n`: shown at more than nothing, gone at nothing, and hopping when it went up.
    /// Nothing, if it counts that already, or is a dot.
    pub fn set(&mut self, grove: &mut Grove, n: u32) {
        let Some((count, was)) = self.count else {
            return;
        };
        if was == n {
            return;
        }
        self.count = Some((count, n));
        self.shown = n > 0;
        grove.text(count, if n > 99 { "99+".to_string() } else { n.to_string() });
        grove.visible(self.dot, n > 0);
        if n > was && n > 0 {
            grove.shift(self.dot, Position::new(0.0, -HOP));
            grove.animate(
                self.dot,
                Motion::Shift(Position::default()),
                Timing::ms(HOP_MS).ease(LANDING),
            );
        }
    }

    /// The dot, or a counting badge's pill, to anchor to.
    pub fn leaf(&self) -> Leaf {
        self.dot
    }

    /// Whether it is shown.
    pub fn shown(&self) -> bool {
        self.shown
    }

    /// Shows it, or takes it away. Nothing, if it is already as asked -- so it can be said every
    /// frame, from whatever decides it. A counting badge is shown by what it counts.
    pub fn show(&mut self, grove: &mut Grove, shown: bool) {
        if shown == self.shown || self.count.is_some() {
            return;
        }
        self.shown = shown;
        grove.animate(
            self.dot,
            Motion::Opacity(shown as u8 as f32),
            tone::timing(),
        );
    }
}
