//! The wall text: what the exhibition is, in two columns on a wide window, and a line from the
//! model. It comes in out of the dark the first time it comes into view.

use foliage::{
    Boxed, Elevation, Font, Grove, Grow, Leaf, Location, Palette, Panel, Place, Sap, Source, Stem,
    Text, Vein, anchor, content, left, top,
};

use arboretum::sprout::{Arrival, Emerge};

use crate::theme::{self, column};

const FIRST: &str = "ilex was trained on eleven thousand herbarium plates: pressed, labelled, dated, \
and for the most part dead. it has never seen a flower move. what it knows of growing it learned \
from things that had stopped. these twelve works are what it makes when it is asked to grow anyway.";

const SECOND: &str = "every picture in these rooms begins as a dim, soft wash of its own colours \
and is resolved, a step at a time, toward something the model believes is a plant. nothing here was drawn. each work is a seed \
and a handful of numbers, and the room grows it again whenever you walk in: the same picture every \
time, out of nothing, in front of you.";

const QUOTE: &str = "a model cannot see a flower. it can only remember ten thousand of them, slightly wrong.";

pub struct Statement {
    root: Leaf,
    parts: Vec<Arrival>,
    tail: Leaf,
    shown: bool,
}

impl Statement {
    /// The wall text, under `page`, below `after`.
    pub fn grow(grove: &mut Grove, page: Leaf, after: Leaf, face: Font) -> Self {
        let root = grove.branch(
            page,
            Stem::new()
                .intangible()
                .anchored(after)
                .at(Location::new().xs(
                    column(),
                    top(anchor().bottom() + 96.px()).height(1.px()),
                )),
        );
        let heading = grove.branch(
            root,
            Text::new("statement")
                .font(face)
                .font_size(theme::heading())
                .color(Palette::Ink)
                .intangible()
                .opacity(0.0)
                .at(Location::new().xs(
                    left(0.px()).width(content()),
                    top(0.px()).height(content()),
                )),
        );
        let paragraph = |text: &str| Text::new(text).font_size(theme::body()).color(Palette::Ink).intangible().opacity(0.0);
        let first = grove.branch(
            root,
            paragraph(FIRST).anchored(heading).at(
                Location::new()
                    .xs(
                        left(0.px()).right(100.pct()).at_most(640.px()),
                        top(anchor().bottom() + 24.px()).height(content()),
                    )
                    .lg(
                        left(0.px()).width(47.pct()),
                        top(anchor().bottom() + 30.px()).height(content()),
                    ),
            ),
        );
        let second = grove.branch(
            root,
            paragraph(SECOND).anchored(first).at(
                Location::new()
                    .xs(
                        left(0.px()).right(100.pct()).at_most(640.px()),
                        top(anchor().bottom() + 18.px()).height(content()),
                    )
                    .lg(
                        left(53.pct()).right(100.pct()),
                        top(anchor().top()).height(content()),
                    ),
            ),
        );
        let quote = grove.branch(
            root,
            Text::new(QUOTE)
                .font(face)
                .font_size(theme::quote())
                .color(Palette::Ink)
                .intangible()
                .opacity(0.0)
                .anchored(second)
                .at(Location::new()
                    .xs(
                        left(18.px()).right(100.pct()),
                        top(anchor().bottom() + 44.px()).height(content()),
                    )
                    .md(
                        left(24.px()).right(100.pct()).at_most(900.px()),
                        top(anchor().bottom() + 56.px()).height(content()),
                    )),
        );
        let rule = grove.branch(
            root,
            Panel::new()
                .color(Palette::Accent)
                .intangible()
                .opacity(0.0)
                .elevate(Elevation::up(1))
                .anchored(quote)
                .at(Location::new().xs(
                    left(0.px()).width(2.px()),
                    top(anchor().top() + 4.px()).bottom(anchor().bottom() - 4.px()),
                )),
        );
        let said = grove.branch(
            root,
            Text::new("ilex, asked what it was looking at · 2026")
                .font_size(lichen::caption())
                .color(Palette::Muted)
                .intangible()
                .opacity(0.0)
                .anchored(quote)
                .at(Location::new().xs(
                    left(anchor().left()).right(100.pct()),
                    top(anchor().bottom() + 14.px()).height(content()),
                )),
        );
        Self {
            root,
            parts: vec![
                Arrival::words(heading, Some(face), theme::heading(), 7.0),
                Arrival::words(first, None, theme::body(), 4.0),
                Arrival::words(second, None, theme::body(), 4.0),
                rule.into(),
                Arrival::words(quote, Some(face), theme::quote(), 6.0),
                Arrival::words(said, None, lichen::caption(), 3.0),
            ],
            tail: said,
            shown: false,
        }
    }

    /// The last thing in the section, which the next one stands below.
    pub fn tail(&self) -> Leaf {
        self.tail
    }

    /// Brings the text in out of the dark the first time the section comes into view.
    pub fn frame(&mut self, grove: &mut Grove, emerge: &mut Emerge) {
        if self.shown {
            return;
        }
        let view = grove.viewport();
        if let Some(Sap::Section(drawn)) = grove.tap(self.root, Vein::Drawn)
            && drawn.top() < view.height * 0.86
        {
            self.shown = true;
            emerge.bring(grove, &self.parts, 0, 120);
        }
    }
}
