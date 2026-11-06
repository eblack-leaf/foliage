//! The colophon: what the exhibition is made of, and the way to the rest of the portfolio.

use foliage::{
    Boxed, Elevation, Font, Grove, Grow, HAIRLINE, Leaf, Location, Palette, Panel, Place, Pollen,
    Sap, Source, Stem, Text, Vein, anchor, content, left, top,
};

use arboretum::sprout::{App, Arrival, Emerge, Signpost};

use crate::icons::Icons;
use crate::theme::column;

const FICTION: &str = "latent flora is a fiction, made to show what foliage and lichen can do. ilex \
is not a model and nothing here was trained on anything: every picture is arithmetic, grown in your \
browser from a seed and a handful of numbers, and the same seed grows the same picture every time.";

const CREDITS: [&str; 3] = [
    "set in major mono display, by emre parlak, and jetbrains mono · both under the ofl",
    "marks from feather, by cole bemis · mit",
    "grown on foliage · dressed in lichen · no server was harmed",
];

pub struct Colophon {
    root: Leaf,
    signpost: Signpost,
    arriving: Vec<Arrival>,
    shown: bool,
}

impl Colophon {
    pub fn grow(grove: &mut Grove, page: Leaf, after: Leaf, face: Font, icons: &Icons) -> Self {
        let root = grove.branch(
            page,
            Stem::new()
                .intangible()
                .anchored(after)
                .font_size(lichen::caption())
                .at(Location::new().xs(column(), top(anchor().bottom() + 120.px()).height(10.px()))),
        );
        let rule = grove.branch(
            root,
            Panel::new()
                .color(Palette::Raised)
                .intangible()
                .opacity(0.0)
                .at(Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(HAIRLINE.px()))),
        );
        let heading = grove.branch(
            root,
            Text::new("colophon")
                .font(face)
                .font_size(foliage::FontSize::new().xs(18).md(20))
                .color(Palette::Ink)
                .intangible()
                .opacity(0.0)
                .at(Location::new().xs(left(0.px()).width(content()), top(36.px()).height(content()))),
        );
        let mut arriving = vec![rule.into(), Arrival::words(heading, Some(face), foliage::FontSize::new().xs(18).md(20), 5.0)];
        let mut last = grove.branch(
            root,
            Text::new(FICTION)
                .font_size(crate::theme::body())
                .color(Palette::Ink)
                .intangible()
                .opacity(0.0)
                .anchored(heading)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()).at_most(720.px()),
                    top(anchor().bottom() + 16.px()).height(content()),
                )),
        );
        arriving.push(Arrival::words(last, None, crate::theme::body(), 4.0));
        for (n, credit) in CREDITS.iter().enumerate() {
            last = grove.branch(
                root,
                Text::new(*credit)
                    .font_size(lichen::caption())
                    .color(Palette::Muted)
                    .intangible()
                    .opacity(0.0)
                    .anchored(last)
                    .at(Location::new().xs(
                        left(0.px()).right(100.pct()).at_most(720.px()),
                        top(anchor().bottom() + if n == 0 { 22.px() } else { 6.px() }).height(content()),
                    )),
            );
            arriving.push(Arrival::words(last, None, lichen::caption(), 3.0));
        }
        let way = grove.branch(
            root,
            Stem::new()
                .intangible()
                .anchored(last)
                .elevate(Elevation::up(1))
                .font_size(lichen::caption())
                .at(Location::new()
                    .xs(
                        left(0.px()).right(100.pct()),
                        top(anchor().bottom() + 30.px()).height((Signpost::heights().0 + 80.0).px()),
                    )
                    .sm(
                        left(0.px()).right(100.pct()),
                        top(anchor().bottom() + 30.px()).height((Signpost::heights().1 + 80.0).px()),
                    )),
        );
        let signpost = Signpost::grow(grove, way, 0.0, 0.0, App::Conservatory, icons.home, icons.external_link);
        for chip in signpost.leaves() {
            grove.opacity(chip, 0.0);
            arriving.push(chip.into());
        }
        Self {
            root,
            signpost,
            arriving,
            shown: false,
        }
    }

    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen, emerge: &mut Emerge) {
        if !self.shown
            && let Some(Sap::Section(drawn)) = grove.tap(self.root, Vein::Drawn)
            && drawn.top() < grove.viewport().height * 0.9
        {
            self.shown = true;
            emerge.bring(grove, &self.arriving, 0, 100);
        }
        self.signpost.frame(grove, pollen);
    }
}
