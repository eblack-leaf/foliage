//! A ping: something just happened here.
//!
//! The briefest of the three ways something asks to be noticed -- a [`Badge`](crate::Badge) stays
//! until it is cleared and a [`Say`](crate::Say) until the next change, and a ping goes by itself.
//! What it answers is a press whose result is otherwise invisible: a paste that landed, a copy that
//! was taken, a save that went through while the page stayed as it was.
//!
//! A ping is a cell with a mark in it, in the system's hue. At rest the cell holds a dash in the
//! grey between -- the place is there, and quiet -- or nothing; fired, the mark comes in over it,
//! holds, and goes.

use foliage::{
    Boxed, Ease, Elevation, Field, Grove, Grow, Icon, Leaf, Location, Motion, Panel, Place, Pollen,
    Source, Stem, Timing, Tween, center_x, center_y,
};

use crate::measure::measure;
use crate::tone;

/// How long a fired ping holds before it goes.
const HOLD_MS: u64 = 5_000;
/// How long its mark takes to come in, or to go.
const FLIP_MS: u64 = 160;
/// The dash a resting ping holds, wide and tall.
const DASH: (f32, f32) = (8.0, 2.0);

/// A ping, grown, and whether it is showing.
pub struct Ping {
    cell: Leaf,
    rest: Option<Leaf>,
    mark: Leaf,
    showing: Option<Tween>,
}

impl Ping {
    /// Grows a ping under `under`, standing where `at` says, with `mark` to show when it fires
    /// and a dash to hold at rest.
    pub fn grow(grove: &mut Grove, under: Leaf, at: Location, mark: Field) -> Self {
        let ping = Self::bare(grove, under, at, mark);
        let rest = grove.branch(
            ping.cell,
            Panel::new()
                .at(Location::new().xs(
                    center_x(50.pct()).width(DASH.0.px()),
                    center_y(50.pct()).height(DASH.1.px()),
                ))
                .elevate(Elevation::up(1))
                .intangible()
                .color(tone::INERT.ink),
        );
        Self {
            rest: Some(rest),
            ..ping
        }
    }

    /// Grows a ping that holds nothing at rest: a mark that comes and goes over whatever is
    /// there. For a corner of a chip, say, or a place that should not look like a place until
    /// something happens in it.
    pub fn bare(grove: &mut Grove, under: Leaf, at: Location, mark: Field) -> Self {
        let side = measure().ping;
        let cell = grove.branch(
            under,
            Stem::new().at(at).elevate(Elevation::up(1)).intangible(),
        );
        let mark = grove.branch(
            cell,
            Icon::new(mark)
                .at(Location::new().xs(
                    center_x(50.pct()).width(side.px()),
                    center_y(50.pct()).height(side.px()),
                ))
                .elevate(Elevation::up(2))
                .intangible()
                .opacity(0.0)
                .color(tone::HINT),
        );
        Self {
            cell,
            rest: None,
            mark,
            showing: None,
        }
    }

    /// The cell, to anchor to.
    pub fn leaf(&self) -> Leaf {
        self.cell
    }

    /// Whether it is showing.
    pub fn showing(&self) -> bool {
        self.showing.is_some()
    }

    /// Fires it: the mark comes in, holds, and goes. Firing one already showing holds it again
    /// from now.
    pub fn fire(&mut self, grove: &mut Grove) {
        self.show(grove, true);
        self.showing = Some(grove.timer(Timing::ms(HOLD_MS)));
    }

    /// Puts it back to rest now, whether or not it has held.
    pub fn quiet(&mut self, grove: &mut Grove) {
        if self.showing.take().is_some() {
            self.show(grove, false);
        }
    }

    /// Carries it for a frame: a ping that has held long enough goes.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        if self.showing.is_some_and(|timer| pollen.finished(timer)) {
            self.showing = None;
            self.show(grove, false);
        }
    }

    fn show(&self, grove: &mut Grove, shown: bool) {
        let timing = Timing::ms(FLIP_MS).ease(Ease::Decelerate);
        grove.animate(self.mark, Motion::Opacity(shown as u8 as f32), timing);
        if let Some(rest) = self.rest {
            grove.animate(rest, Motion::Opacity(!shown as u8 as f32), timing);
        }
    }
}
