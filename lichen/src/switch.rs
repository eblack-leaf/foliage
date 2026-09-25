//! A switch: a track with a knob on it, and a name beside. The one shape a two-way choice is
//! offered in, so on and off read the same way wherever the choice is made: the knob a knob's
//! width along, and the track lit.
//!
//! The switch is the one thing that takes the press; the three on it are decoration, as a chip's
//! are.

use foliage::{
    Boxed, Ease, Elevation, Grove, Grow, Leaf, Length, Location, Motion, Panel, Place, Pollen,
    Source, Stem, Text, Timing, center_y, content, left,
};

use crate::chip::caption;
use crate::measure::measure;
use crate::tone::{self, Reach};

/// How long the knob takes to slide.
const FLIP_MS: u64 = 160;

/// A switch, grown, and where it stands.
pub struct Switch {
    /// What takes the press.
    switch: Leaf,
    track: Leaf,
    knob: Leaf,
    on: bool,
}

impl Switch {
    /// Grows a switch under `under`, standing where `at` says and `lift` in front of it, named
    /// `name`. Off.
    ///
    /// The switch composes at the caption size, so a width stated in its own letters -- see
    /// [`width`](Self::width) -- resolves against the cell the name is set in.
    pub fn grow(grove: &mut Grove, under: Leaf, at: Location, lift: Elevation, name: &str) -> Self {
        let m = measure();
        let switch = grove.branch(
            under,
            Stem::new()
                .at(at)
                .elevate(lift)
                .font_size(caption())
                .interactive(),
        );
        let track = grove.branch(
            switch,
            Panel::new()
                .at(Location::new().xs(
                    left(0.px()).width(m.track.0.px()),
                    center_y(50.pct()).height(m.track.1.px()),
                ))
                .elevate(Elevation::up(1))
                .intangible()
                .color(tone::BAR),
        );
        let knob = grove.branch(
            track,
            Panel::new()
                .at(knob_at(false))
                .elevate(Elevation::up(1))
                .intangible()
                .color(tone::REST.ink),
        );
        grove.branch(
            switch,
            Text::new(name)
                .at(Location::new().xs(
                    left((m.track.0 + m.gap - 2.0).px()).width(content()),
                    center_y(50.pct()).height(1.letters()),
                ))
                .elevate(Elevation::up(1))
                .intangible()
                .color(tone::REST.ink)
                .font_size(caption()),
        );
        Self {
            switch,
            track,
            knob,
            on: false,
        }
    }

    /// How wide a switch named `name` wants to be: the track, the room beside it, and the name.
    /// In the switch's own letters, so it is for the switch's placement and nothing else's.
    pub fn width(name: &str) -> Length {
        let m = measure();
        (m.track.0 + m.gap - 2.0).px() + (name.chars().count() as f32).letters()
    }

    /// The switch itself, to anchor to.
    pub fn leaf(&self) -> Leaf {
        self.switch
    }

    /// Where it stands.
    pub fn on(&self) -> bool {
        self.on
    }

    /// Carries the switch for a frame: a press flips it. Whether it flipped.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> bool {
        if !pollen.activated(self.switch) {
            return false;
        }
        self.set(grove, !self.on);
        true
    }

    /// Puts the switch on, or off: the knob slides to where that is, and the track lights for
    /// it, or not.
    pub fn set(&mut self, grove: &mut Grove, on: bool) {
        self.on = on;
        let timing = Timing::ms(FLIP_MS).ease(Ease::Emphasis);
        grove.animate(self.knob, Motion::Location(knob_at(on)), timing);
        let track = match on {
            true => tone::LIT,
            false => tone::BAR,
        };
        grove.animate(self.track, Motion::Palette(track), timing);
    }
}

impl Reach for Switch {
    /// Puts the switch in reach, or out of it. Out of reach it still stands and still says where
    /// it is; what it does not do is flip.
    fn reach(&self, grove: &mut Grove, within: bool) {
        match within {
            true => grove.enable(self.switch),
            false => grove.disable(self.switch),
        }
    }
}

/// Where the knob stands on its track, on or off.
fn knob_at(on: bool) -> Location {
    let m = measure();
    let x = match on {
        true => m.track.0 - m.knob_in - m.knob,
        false => m.knob_in,
    };
    Location::new().xs(
        left(x.px()).width(m.knob.px()),
        center_y(50.pct()).height(m.knob.px()),
    )
}
