//! The links tip: where else foliage is written down.
//!
//! Three presses, each worn in its own stretch of the leaf's ramp and marked with what it opens, and
//! a card for each saying what is there. A press -- or `Enter` with focus on one -- leaves the page.

use foliage::{Color, Fill, Font, Grove, Grow, Leaf, Location, Pollen, Source, content, left, top};
use lichen::{Chip, Tone};

use crate::icons::Icons;
use crate::parts::{self, Stack};
use crate::theme;

/// One place the site links to.
struct Link {
    name: &'static str,
    url: &'static str,
    /// The address as it is shown: without the scheme, and short enough for a phone's width.
    shown: &'static str,
    /// What is there, said on its card.
    says: &'static str,
}

const LINKS: [Link; 3] = [
    Link {
        name: "book",
        url: "https://eblack-leaf.github.io/foliage/book/",
        shown: "eblack-leaf.github.io/foliage/book",
        says: "The guide: what an element is, how it is placed, how it moves, and how an app is \
               put together from nothing but the verbs.",
    },
    Link {
        name: "docs",
        url: "https://eblack-leaf.github.io/foliage/api/foliage/",
        shown: "eblack-leaf.github.io/foliage/api",
        says: "The API reference. Its crate root is the map of the whole surface -- what every \
               type is for, and which verb writes it.",
    },
    Link {
        name: "github",
        url: "https://github.com/eblack-leaf/foliage",
        shown: "github.com/eblack-leaf/foliage",
        says: "The source: the engine, lichen, the icon baker, the Android driver, and this page.",
    },
];

/// What the controls hold, one to a row.
pub(crate) fn column() -> Vec<&'static str> {
    LINKS.iter().map(|link| link.name).collect()
}

/// How much nearer white a link's press is drawn while a pointer is on it.
const LIFT: f32 = 0.10;

/// The links tip, grown.
pub(crate) struct Links {
    chips: Vec<Chip>,
}

impl Links {
    pub(crate) fn grow(
        grove: &mut Grove,
        controls: Leaf,
        details: Leaf,
        icons: &Icons,
        italic: Font,
    ) -> Self {
        let marks = [icons.book_open, icons.file_text, icons.github];
        let presses: Vec<_> = LINKS
            .iter()
            .zip(marks)
            .map(|(link, mark)| (mark, link.name))
            .collect();
        let mut chips = parts::chips(grove, controls, &presses);
        for (chip, hue) in chips.iter_mut().zip(theme::HUES) {
            chip.wear(grove, worn(hue));
        }

        let mut stack = Stack::new(details);
        stack.heading(
            grove,
            "links",
            "Where the rest of foliage is written down. Each press leaves the page.",
            italic,
        );
        for (link, hue) in LINKS.iter().zip(theme::HUES) {
            // The address first, in the link's own hue, and what is there wrapped under it.
            let body = stack.card(grove, link.name, 96.0);
            parts::line(grove, body, link.shown, 0.0, 0.0, hue);
            parts::words(
                grove,
                body,
                link.says,
                Location::new().xs(
                    left(0.px()).width(100.pct()),
                    top(24.px()).height(content()),
                ),
                lichen::REST.ink,
            );
        }
        Self { chips }
    }

    pub(crate) fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        for ((chip, link), hue) in self.chips.iter_mut().zip(&LINKS).zip(theme::HUES) {
            if pollen.engaged(chip.leaf()) {
                chip.wear(grove, worn(hue.lightened(LIFT)));
            }
            if pollen.disengaged(chip.leaf()) {
                chip.wear(grove, worn(hue));
            }
            if chip.pressed(pollen) {
                grove.navigate(link.url);
            }
        }
    }
}

/// What a link's press wears: its hue, with near-black read on it.
fn worn(hue: Color) -> Tone {
    Tone {
        fill: Fill::Literal(hue),
        ink: Fill::Literal(theme::ON_HUE),
    }
}
