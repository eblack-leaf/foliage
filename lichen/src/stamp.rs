//! A rubber stamp: a word in a ring, pressed onto the page.

use foliage::{
    Boxed, Color, Ease, Elevation, Fill, Font, FontSize, Grove, Grow, Leaf, Location, Motion, Panel,
    Place, Rounding, Corners, Source, Stem, Text, Timing, center_x, center_y, content, left, top,
};

/// How long the press takes to land, and how far outside its place the ring starts.
const PRESS_MS: u64 = 340;
const SPREAD: f32 = 14.0;

/// A thump: overshoots its place a little and settles.
const THUMP: Ease = Ease::Curve {
    x1: 0.2,
    y1: 1.25,
    x2: 0.4,
    y2: 1.0,
};

/// A status pressed onto a page like a rubber stamp -- DRAFT, APPROVED, PAID -- in a ring the
/// colour of its ink.
///
/// The ring is a [`Panel`] filled with nothing and outlined, which leaves the outline alone. It
/// lands from a little larger than it ends, fading up as it comes down, which is as near to a
/// stamp's thump as a surface that cannot scale its glyphs gets: the word fades in once the ring
/// has landed round it.
pub struct Stamp {
    root: Leaf,
    ring: Leaf,
    word: Leaf,
    pressed: bool,
}

impl Stamp {
    /// A stamp under `under`, standing where `at` says, saying `word` at `size` in `font` and
    /// `ink`. Not yet pressed.
    pub fn grow(
        grove: &mut Grove,
        under: Leaf,
        at: Location,
        word: &str,
        size: FontSize,
        font: Option<Font>,
        ink: impl Into<Fill>,
    ) -> Self {
        let ink = ink.into();
        let root = grove.branch(
            under,
            Stem::new().intangible().font_size(size).elevate(Elevation::up(2)).at(at),
        );
        let ring = grove.branch(
            root,
            Panel::new()
                .color(Color::rgba(0.0, 0.0, 0.0, 0.0))
                .outline(2.0, ink)
                .rounding(Corners::all(Rounding::Sm))
                .intangible()
                .opacity(0.0)
                .at(spread(SPREAD)),
        );
        let text = Text::new(word)
            .color(ink)
            .font_size(size)
            .intangible()
            .opacity(0.0)
            .elevate(Elevation::up(1));
        let text = match font {
            Some(font) => text.font(font),
            None => text,
        };
        let word = grove.branch(
            root,
            text.at(Location::new().xs(
                center_x(50.pct()).width(content()),
                center_y(50.pct()).height(1.letters()),
            )),
        );
        Self {
            root,
            ring,
            word,
            pressed: false,
        }
    }

    /// The stamp's own box.
    pub fn leaf(&self) -> Leaf {
        self.root
    }

    /// Whether it has been pressed and not lifted.
    pub fn pressed(&self) -> bool {
        self.pressed
    }

    /// Presses the stamp onto the page, `after` ms from now. Nothing, if it is down.
    pub fn press(&mut self, grove: &mut Grove, after: u64) {
        if self.pressed {
            return;
        }
        self.pressed = true;
        grove.at(self.ring, spread(SPREAD));
        grove.animate(
            self.ring,
            Motion::Location(spread(0.0)),
            Timing::ms(PRESS_MS).after(after).ease(THUMP),
        );
        grove.animate(
            self.ring,
            Motion::Opacity(1.0),
            Timing::ms(PRESS_MS / 2).after(after).ease(Ease::Decelerate),
        );
        grove.animate(
            self.word,
            Motion::Opacity(1.0),
            Timing::ms(180).after(after + PRESS_MS / 2).ease(Ease::Decelerate),
        );
    }

    /// Takes the stamp off the page.
    pub fn lift(&mut self, grove: &mut Grove) {
        if !self.pressed {
            return;
        }
        self.pressed = false;
        let timing = Timing::ms(200).ease(Ease::Accelerate);
        grove.animate(self.ring, Motion::Opacity(0.0), timing);
        grove.animate(self.word, Motion::Opacity(0.0), timing);
    }

    /// Changes what it says and what ink it is in, over a moment. A pressed stamp stays pressed.
    pub fn say(&mut self, grove: &mut Grove, word: &str, ink: impl Into<Fill>) {
        let ink = ink.into();
        grove.text(self.word, word);
        let timing = Timing::ms(240).ease(Ease::Decelerate);
        grove.animate(self.word, Motion::from(ink), timing);
        grove.outline(self.ring, 2.0, ink);
    }
}

/// The stamp's box grown by `by` on every side.
fn spread(by: f32) -> Location {
    Location::new().xs(
        left((-by).px()).right(100.pct() + by.px()),
        top((-by).px()).bottom(100.pct() + by.px()),
    )
}
