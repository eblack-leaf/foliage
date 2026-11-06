//! The foot of the page: the colophon, set as the fine print on a label is -- what the shop is set
//! in, what it is built with, and the way to the rest of the arboretum.

use foliage::{
    Boxed, FontSize, Grove, Grow, Leaf, Location, Palette, Place, Pollen, Sap, Source, Stem, Text,
    Vein, anchor, center_y, content, left, top,
};
use lichen::measure;

use arboretum::sprout::{App, Print, Printed, Signpost};

use crate::icons::Icons;
use crate::label;
use crate::theme::{Faces, HAIR, HEAVY, column};

/// What the colophon says, row by row.
const ROWS: [(&str, &str); 5] = [
    ("set in", "Xanh Mono, by Yellow Type"),
    ("specimens", "Major Mono, Plex Mono, JetBrains"),
    ("marks", "Feather icons, MIT licence"),
    ("built in", "Rust · foliage & lichen"),
    ("the till", "a demo: nothing is ever charged"),
];
/// How wide the names' column is.
const KEY: f32 = 96.0;

pub struct Colophon {
    root: Leaf,
    printed: Vec<Printed>,
    shown: bool,
    signpost: Signpost,
}

impl Colophon {
    /// The colophon under `page`, below `after`.
    pub fn grow(grove: &mut Grove, page: Leaf, after: Leaf, faces: &Faces, icons: &Icons) -> Self {
        let m = measure();
        let root = grove.branch(
            page,
            Stem::new()
                .intangible()
                .anchored(after)
                .font_size(lichen::caption())
                .at(column(|| top(anchor().bottom() + 64.px()).height(10.px()))),
        );
        let mut printed = Vec::new();
        printed.push(Print::rule(
            grove,
            root,
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top(0.px()).height(HEAVY.px()),
            ),
            Palette::Ink,
        ));
        let heading = grove.branch(
            root,
            Text::new("Colophon")
                .font(faces.roman)
                .font_size(FontSize::new().xs(21).md(24))
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).width(content()),
                    top(18.px()).height(1.letters()),
                )),
        );
        printed.push(Print::words(grove, heading));
        let names: Vec<&str> = ROWS.iter().map(|(name, _)| *name).collect();
        // The fine print runs no wider than it reads, on a wide window.
        let fine = grove.branch(
            root,
            Stem::new()
                .intangible()
                .font_size(lichen::caption())
                .at(Location::new()
                    .xs(left(0.px()).right(100.pct()), top(0.px()).height(10.px()))
                    .lg(left(0.px()).width(58.pct()), top(0.px()).height(10.px()))),
        );
        let (rows, values, row_printed) = label::rows(grove, fine, heading, 14.0, &names, KEY);
        for (leaf, (_, value)) in values.iter().zip(ROWS) {
            grove.text(*leaf, value);
        }
        printed.extend(row_printed);
        // The way elsewhere: named as a row is, the presses where its value would be.
        let (tall, short) = Signpost::heights();
        let way = grove.branch(
            root,
            Stem::new()
                .intangible()
                .anchored(rows)
                .font_size(lichen::caption())
                .at(Location::new()
                    .xs(
                        left(0.px()).right(100.pct()),
                        top(anchor().bottom() + 18.px()).height(tall.px()),
                    )
                    .sm(
                        left(0.px()).right(100.pct()),
                        top(anchor().bottom() + 18.px()).height(short.px()),
                    )),
        );
        let elsewhere = grove.branch(
            way,
            Text::new("elsewhere")
                .font_size(lichen::caption())
                .color(Palette::Muted)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).width(content()),
                    center_y((m.height / 2.0).px()).height(1.letters()),
                )),
        );
        printed.push(Print::words(grove, elsewhere));
        let signpost = Signpost::grow(
            grove,
            way,
            KEY,
            0.0,
            App::Orchard,
            icons.home,
            icons.arrow_right,
        );
        for leaf in signpost.leaves() {
            printed.push(Print::plain(grove, leaf));
        }
        let foot_rule = grove.branch(
            root,
            Stem::new()
                .intangible()
                .anchored(way)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + 18.px()).height(HAIR.px()),
                )),
        );
        printed.push(Print::rule_in(grove, foot_rule, Palette::Ink));
        let foot = grove.branch(
            root,
            Text::new("bits & bytes · no. 4 · autumn 2026")
                .font(faces.italic)
                .font_size(FontSize::new().xs(16))
                .color(Palette::Ink)
                .intangible()
                .anchored(foot_rule)
                .at(Location::new().xs(
                    left(0.px()).width(content()),
                    top(anchor().bottom() + 10.px()).height(1.letters()),
                )),
        );
        printed.push(Print::words(grove, foot));
        // A margin under it all, so the foot is not against the window's.
        grove.branch(
            root,
            Stem::new()
                .intangible()
                .anchored(foot)
                .at(Location::new().xs(
                    left(0.px()).width(1.px()),
                    top(anchor().bottom()).height(72.px()),
                )),
        );
        Self {
            root,
            printed,
            shown: false,
            signpost,
        }
    }

    /// Prints the colophon in once it comes into view.
    pub fn arrive(&mut self, grove: &mut Grove, print: &mut Print) {
        if self.shown {
            return;
        }
        let Some(Sap::Section(drawn)) = grove.tap(self.root, Vein::Drawn) else {
            return;
        };
        if drawn.top() > grove.viewport().height * 0.95 {
            return;
        }
        self.shown = true;
        print.run(grove, &self.printed, 0, 45);
    }

    /// Goes where a pressed signpost leads.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        self.signpost.frame(grove, pollen);
    }
}
