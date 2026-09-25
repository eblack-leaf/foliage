//! A badge: something here wants a person.
//!
//! The lasting one of the three ways something asks to be noticed. A [`Ping`](crate::Ping) goes
//! by itself and a [`Say`](crate::Say) with the next change; a badge stays until whatever it is
//! about is dealt with, and the app says when that is. What it answers is a state and not an
//! event: something failed and is waiting to be tried again, something is waiting to go.
//!
//! A dot in the system's hue on the corner of what it is about -- on it and not in it, so what it
//! is on need not know it is there.

use foliage::{
    Boxed, Elevation, Grove, Grow, Leaf, Location, Motion, Panel, Place, Source, center_x, center_y,
};

use crate::chip::Chip;
use crate::measure::measure;
use crate::tone;

/// A badge, grown, and whether it is shown.
pub struct Badge {
    dot: Leaf,
    shown: bool,
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
        Self { dot, shown: false }
    }

    /// The dot, to anchor to.
    pub fn leaf(&self) -> Leaf {
        self.dot
    }

    /// Whether it is shown.
    pub fn shown(&self) -> bool {
        self.shown
    }

    /// Shows it, or takes it away. Nothing, if it is already as asked -- so it can be said every
    /// frame, from whatever decides it.
    pub fn show(&mut self, grove: &mut Grove, shown: bool) {
        if shown == self.shown {
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
