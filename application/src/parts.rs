//! What every section is built from: a column of presses for its controls, and a stack of labelled
//! cards for what it opens.
//!
//! A stack places each thing it is handed under the last, anchored to it, so something whose
//! height is its wrapped text's measures itself and moves everything under it -- and nothing in a
//! section has to know how tall anything above it came out.

use foliage::{
    Boxed, Corners, Elevation, Field, Fill, Font, FontSize, Grove, Grow, Horizontal, Leaf,
    Location, Palette, Panel, Place, Rounding, Seed, Source, Stem, Text, VerticalLength, anchor,
    content, left, top,
};
use lichen::{Chip, Ping, measure};

use crate::theme::GAP;

/// How wide a card is at most. A widget is drawn for a hand's width of screen, and a card as wide
/// as a desk's window reads as a strip rather than a thing.
pub(crate) const WIDEST: f32 = 620.0;

/// How tall the name over a card is, and how far a card's body stands in from its ground.
const TITLE: f32 = 22.0;
const INSET: f32 = 14.0;

/// Where press `n` of a column of them stands: the column's whole width, one under the other.
pub(crate) fn row(n: usize) -> Location {
    let m = measure();
    Location::new().xs(
        left(0.px()).width(100.pct()),
        top((n as f32 * (m.height + m.gap)).px()).height(m.height.px()),
    )
}

/// Grows a column of chips under `under`, one per `(mark, name)`, the whole width of it.
pub(crate) fn chips(grove: &mut Grove, under: Leaf, presses: &[(Field, &str)]) -> Vec<Chip> {
    presses
        .iter()
        .enumerate()
        .map(|(n, &(mark, name))| Chip::grow(grove, under, row(n), Elevation::up(1), mark, name))
        .collect()
}

/// How far down a widget's body row `n` of the chip's shape starts: rows a chip tall, a gap apart.
pub(crate) fn lane(n: usize) -> f32 {
    let m = measure();
    n as f32 * (m.height + m.gap)
}

/// A box `across`, `height` pixels tall from `top_px` down.
pub(crate) fn at(across: Horizontal, top_px: f32, height: f32) -> Location {
    Location::new().xs(across, top(top_px.px()).height(height.px()))
}

/// A box in row `n`, a chip tall, `across`.
pub(crate) fn in_lane(across: Horizontal, n: usize) -> Location {
    at(across, lane(n), measure().height)
}

/// A line of words at the caption size, standing where `at` says, read in `fill`. Nothing presses
/// it.
pub(crate) fn words(
    grove: &mut Grove,
    under: Leaf,
    text: impl Into<String>,
    at: Location,
    fill: impl Into<Fill>,
) -> Leaf {
    grove.branch(
        under,
        Text::new(text)
            .color(fill)
            .font_size(lichen::caption())
            .elevate(Elevation::up(1))
            .intangible()
            .at(at),
    )
}

/// A line of words one letter tall, `left_px` in and `top_px` down, in `fill`.
pub(crate) fn line(
    grove: &mut Grove,
    under: Leaf,
    text: impl Into<String>,
    left_px: f32,
    top_px: f32,
    fill: impl Into<Fill>,
) -> Leaf {
    words(
        grove,
        under,
        text,
        Location::new().xs(
            left(left_px.px()).width(content()),
            top(top_px.px()).height(1.letters()),
        ),
        fill,
    )
}

/// A chip with a mark and no name: its cell, and nothing else.
pub(crate) fn mark(grove: &mut Grove, under: Leaf, at: Location, mark: Field) -> Chip {
    Chip::mark(grove, under, at, Elevation::up(1), mark)
}

/// A ping standing where `at` says, a chip's cell square.
///
/// In a box of its own set at the caption size, so that `at` may be stated in letters -- beside a
/// chip, say, whose width is. A ping's own cell is set at no size, and a letter of no size is no
/// width at all.
pub(crate) fn ping(grove: &mut Grove, under: Leaf, at: Location, mark: Field) -> Ping {
    let cell = grove.branch(
        under,
        Stem::new().intangible().font_size(lichen::caption()).at(at),
    );
    Ping::grow(grove, cell, Location::new(), mark)
}

/// A chip with a mark and a name, as wide as its name makes it.
pub(crate) fn chip(grove: &mut Grove, under: Leaf, at: Location, mark: Field, name: &str) -> Chip {
    Chip::grow(grove, under, at, Elevation::up(1), mark, name)
}

/// Things placed one under the other, each anchored to the one before.
pub(crate) struct Stack {
    under: Leaf,
    last: Option<Leaf>,
}

impl Stack {
    /// An empty stack, filling `under` from its top.
    pub(crate) fn new(under: Leaf) -> Self {
        Self { under, last: None }
    }

    /// Whatever was placed last, to anchor more under.
    pub(crate) fn last(&self) -> Option<Leaf> {
        self.last
    }

    /// Places `seed` under whatever was placed last, `gap` below it, `across` as stated and
    /// `height` tall.
    pub(crate) fn place<S: Seed + Place + Boxed>(
        &mut self,
        grove: &mut Grove,
        seed: S,
        across: Horizontal,
        height: impl Into<VerticalLength>,
        gap: f32,
    ) -> Leaf {
        let placed = match self.last {
            Some(last) => seed
                .anchored(last)
                .at(Location::new().xs(across, top(anchor().bottom() + gap.px()).height(height))),
            None => seed.at(Location::new().xs(across, top(0.px()).height(height))),
        };
        let leaf = grove.branch(self.under, placed);
        self.last = Some(leaf);
        leaf
    }

    /// Places `seed` under whatever was placed last, `height` tall and as wide as the stack is --
    /// but no wider than [`WIDEST`].
    pub(crate) fn push<S: Seed + Place + Boxed>(
        &mut self,
        grove: &mut Grove,
        seed: S,
        height: impl Into<VerticalLength>,
    ) -> Leaf {
        let across = left(0.px()).width(100.pct()).at_most(WIDEST.px());
        self.place(grove, seed, across, height, GAP)
    }

    /// A title and the line that says what follows it: what heads a section, or one theme of it.
    pub(crate) fn heading(&mut self, grove: &mut Grove, title: &str, blurb: &str, italic: Font) {
        self.push(
            grove,
            Text::new(title)
                .color(Palette::Ink)
                .font_size(FontSize::new().xs(20).md(24))
                .intangible(),
            content(),
        );
        self.push(
            grove,
            Text::new(blurb)
                .color(lichen::INERT.ink)
                .font(italic)
                .font_size(lichen::caption())
                .intangible(),
            content(),
        );
    }

    /// A card: `title` over a raised ground whose body is `body` pixels tall. Hands back the body,
    /// inset from the ground's edges, to grow a widget into.
    pub(crate) fn card(&mut self, grove: &mut Grove, title: &str, body: f32) -> Leaf {
        let card = self.push(
            grove,
            Stem::new().intangible(),
            (TITLE + body + 2.0 * INSET).px(),
        );
        grove.branch(
            card,
            Text::new(title)
                .color(lichen::INERT.ink)
                .font_size(lichen::caption())
                .intangible()
                .at(Location::new().xs(
                    left(2.px()).width(content()),
                    top(0.px()).height(1.letters()),
                )),
        );
        let ground = grove.branch(
            card,
            Panel::new()
                .color(Palette::Raised.recede())
                .rounding(Corners::all(Rounding::Sm))
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).width(100.pct()),
                    top(TITLE.px()).bottom(100.pct()),
                )),
        );
        grove.branch(
            ground,
            Stem::new()
                .intangible()
                .elevate(Elevation::up(1))
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(INSET.px()).right(100.pct() - INSET.px()),
                    top(INSET.px()).bottom(100.pct() - INSET.px()),
                )),
        )
    }
}
