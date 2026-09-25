//! A chip: a mark in a cell, a bar, and a name. The one shape a press is offered in, so what a
//! chip wears -- at rest, inert, armed, chosen, dangerous -- is the whole of what says its state,
//! and a press anywhere is read the same way.
//!
//! The chip is the one thing that takes the press; the three on it are decoration, so the visual
//! is free to change without the input changing with it. A press is a tap, or `Enter` or the
//! space bar with focus on the chip -- [`Pollen::activated`] -- so what a keyboard steps to, it
//! can press.

use foliage::{
    Boxed, Elevation, Field, FontSize, Grove, Grow, Horizontal, Icon, Leaf, Length, Location,
    Motion, Panel, Place, Pollen, Source, Stem, Text, center_x, center_y, content, left, top,
};

use crate::measure::measure;
use crate::tone::{self, Press, Reach, Tone};

/// A chip, grown, and the parts of it that change with what it wears.
pub struct Chip {
    /// What takes the press.
    chip: Leaf,
    /// The fill.
    back: Leaf,
    mark: Leaf,
    label: Leaf,
    /// The state it was last armed in, or `None` where it was dressed by hand since.
    press: Option<Press>,
}

impl Chip {
    /// Grows a chip under `under`, standing where `at` says and `lift` in front of it, with
    /// `mark` in its cell and `name` beside. At rest, and in reach.
    ///
    /// The chip composes at the caption size, so a width stated in its own letters -- see
    /// [`width`](Self::width) -- resolves against the cell the name is set in.
    pub fn grow(
        grove: &mut Grove,
        under: Leaf,
        at: Location,
        lift: Elevation,
        mark: Field,
        name: &str,
    ) -> Self {
        let m = measure();
        let chip = grove.branch(
            under,
            Stem::new()
                .at(at)
                .elevate(lift)
                .font_size(caption())
                .interactive(),
        );
        let back = grove.branch(
            chip,
            Panel::new()
                .at(Location::new())
                .elevate(Elevation::up(1))
                .intangible()
                .color(tone::REST.fill)
                .rounding(tone::corner()),
        );
        let mark = grove.branch(
            back,
            Icon::new(mark)
                .at(Location::new().xs(
                    center_x((m.height / 2.0).px()).width(m.glyph.px()),
                    center_y(50.pct()).height(m.glyph.px()),
                ))
                .elevate(Elevation::up(1))
                .intangible()
                .color(tone::REST.ink),
        );
        grove.branch(
            back,
            Panel::new()
                .at(Location::new().xs(
                    left(m.height.px()).width(1.px()),
                    top(m.notch.px()).bottom(100.pct() - m.notch.px()),
                ))
                .elevate(Elevation::up(1))
                .intangible()
                .color(tone::BAR),
        );
        // In the middle of what is right of the cell and the bar, however wide the chip is.
        let label = grove.branch(
            back,
            Text::new(name)
                .at(Location::new().xs(
                    center_x(50.pct() + ((m.height + 1.0) / 2.0).px()).width(content()),
                    center_y(50.pct()).height(1.letters()),
                ))
                .elevate(Elevation::up(1))
                .intangible()
                .color(tone::REST.ink)
                .font_size(caption()),
        );
        Self {
            chip,
            back,
            mark,
            label,
            press: Some(Press::Rest),
        }
    }

    /// How wide a chip with `name` on it wants to be: the cell, the bar, and the name with a
    /// letter's room either side. In the chip's own letters, so it is for the chip's placement
    /// and nothing else's.
    pub fn width(name: &str) -> Length {
        (measure().height + 1.0).px() + (name.chars().count() as f32 + 2.0).letters()
    }

    /// The chip itself, to anchor to or to hang more on.
    pub fn leaf(&self) -> Leaf {
        self.chip
    }

    /// The name, to be written over with `Grow::text` -- a count beside it, say.
    pub fn label(&self) -> Leaf {
        self.label
    }

    /// Whether the frame reports a press on the chip.
    pub fn pressed(&self, pollen: &Pollen) -> bool {
        pollen.activated(self.chip)
    }

    /// The state it was last armed in, or `None` where it has been dressed by hand since.
    pub fn press(&self) -> Option<Press> {
        self.press
    }

    /// Stands the chip in `press`: dressed for it, and in reach unless it is inert. Nothing, if it
    /// stands there already -- so it can be said every frame, from whatever decides it.
    pub fn arm(&mut self, grove: &mut Grove, press: Press) {
        if self.press == Some(press) {
            return;
        }
        self.press = Some(press);
        self.dress(grove, press.tone());
        self.set_reach(grove, press.within());
    }

    /// Dresses the chip in `tone` by hand: its fill, and the ink its mark and name are read in.
    /// For a tone none of the [`Press`] states is; what is in reach is left as it was.
    pub fn wear(&mut self, grove: &mut Grove, tone: Tone) {
        self.press = None;
        self.dress(grove, tone);
    }

    fn dress(&self, grove: &mut Grove, tone: Tone) {
        let timing = tone::timing();
        grove.animate(self.back, Motion::from(tone.fill), timing);
        grove.animate(self.mark, Motion::from(tone.ink), timing);
        grove.animate(self.label, Motion::from(tone.ink), timing);
    }

    fn set_reach(&self, grove: &mut Grove, within: bool) {
        match within {
            true => grove.enable(self.chip),
            false => grove.disable(self.chip),
        }
    }
}

impl Reach for Chip {
    /// Puts the chip in reach, or out of it, without changing what it wears. Out of reach it still
    /// stands and still stops a press, which is what an inert chip is; [`arm`](Chip::arm) is what
    /// does both at once.
    fn reach(&self, grove: &mut Grove, within: bool) {
        self.set_reach(grove, within);
    }
}

/// A bar on its own, standing between things in a row the way it stands between a chip's mark
/// and name: a chip's height less its notches, centred in `under`, at `across`.
pub fn bar(grove: &mut Grove, under: Leaf, across: Horizontal) -> Leaf {
    let m = measure();
    grove.branch(
        under,
        Panel::new()
            .at(Location::new().xs(
                across,
                center_y(50.pct()).height((m.height - 2.0 * m.notch).px()),
            ))
            .elevate(Elevation::up(1))
            .intangible()
            .color(tone::BAR),
    )
}

/// The size everything in the chip's shape is set at.
pub fn caption() -> FontSize {
    FontSize::new().xs(measure().caption)
}
