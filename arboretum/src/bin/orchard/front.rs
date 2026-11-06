//! The front of the shop: the bar it is read under, the band running past, and the window -- the
//! headline, the shop's own label, and the good on show.

use foliage::{
    Boxed, Elevation, Grove, Grow, Leaf, Location, Palette, Panel, Place, Plate, Pollen, Source,
    Stem, Text, Vertical, anchor, aspect, bottom, content, left, right, top,
};
use lichen::{Chip, Field, Press, measure};

use arboretum::sprout::{Marquee, Print, Printed, Tally};

use crate::card::{Card, Size};
use crate::catalog::{Aisle, FEATURED, GOODS, weight};
use crate::icons::Icons;
use crate::theme::{self, BAR_NARROW, BAR_WIDE, Faces, HAIR, HEAVY, column};

/// What a press at the front asks of the shop.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Asked {
    /// To scroll down to the aisles.
    Aisles,
    /// To open the bag.
    Bag,
    /// To open the library of what was bought.
    Library,
    /// To see a good.
    Good(usize),
}

/// The bar over the shop: the wordmark, the search, the library once there is one, and the bag.
pub struct Bar {
    search: Field,
    bag: Chip,
    tally: Tally,
    library: Chip,
    has_library: bool,
    printed: Vec<Printed>,
}

impl Bar {
    pub fn grow(grove: &mut Grove, under: Leaf, faces: &Faces, icons: &Icons) -> Self {
        let m = measure();
        let bar = grove.branch(
            under,
            Panel::new()
                .color(Palette::Surface)
                .interactive()
                .elevate(Elevation::up(40))
                .at(Location::new()
                    .xs(
                        left(0.px()).right(100.pct()),
                        top(0.px()).height(BAR_NARROW.px()),
                    )
                    .lg(
                        left(0.px()).right(100.pct()),
                        top(0.px()).height(BAR_WIDE.px()),
                    )),
        );
        let rule = grove.branch(
            bar,
            Panel::new()
                .color(Palette::Ink)
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    bottom(100.pct()).height(HAIR.px()),
                )),
        );
        let col = grove.branch(
            bar,
            Stem::new()
                .intangible()
                .elevate(Elevation::up(1))
                .font_size(lichen::caption())
                .at(column(|| top(0.px()).bottom(100.pct()))),
        );
        let row = (BAR_WIDE - m.height) / 2.0;
        let wordmark = grove.branch(
            col,
            Text::new("bits & bytes")
                .font(faces.italic)
                .font_size(foliage::FontSize::new().xs(26).md(28))
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    left((-1).px()).width(content()),
                    foliage::center_y((row + m.height / 2.0).px()).height(1.letters()),
                )),
        );
        let bag_w = Chip::width("bag");
        let mut bag = Chip::grow(
            grove,
            col,
            Location::new().xs(
                right(100.pct()).width(bag_w.clone()),
                top(row.px()).height(m.height.px()),
            ),
            Elevation::up(1),
            icons.shopping_bag,
            "bag",
        );
        bag.arm(grove, Press::Rest);
        let tally = Tally::grow(grove, bag.leaf(), Palette::Accent, Palette::Contrast);
        let library = Chip::grow(
            grove,
            col,
            Location::new().xs(
                right(100.pct() - bag_w.clone() - m.gap.px()).width(Chip::width("library")),
                top(row.px()).height(m.height.px()),
            ),
            Elevation::up(1),
            icons.package,
            "library",
        );
        grove.visible(library.leaf(), false);
        // Under the wordmark on anything narrower than a desk; between it and the presses on one.
        let chips = Chip::width("bag") + m.gap.px() + Chip::width("library");
        let search = Field::grow(
            grove,
            col,
            Location::new()
                .xs(
                    left(0.px()).right(100.pct()),
                    top((row + m.height + 12.0).px()).height(m.height.px()),
                )
                .lg(
                    left(230.px())
                        .right(100.pct() - chips - 28.px())
                        .at_most(460.px()),
                    top(row.px()).height(m.height.px()),
                ),
            6.0,
            "find",
            false,
        );
        search.hint(grove, "rain, rust, aurora…");
        let printed = vec![
            Print::words(grove, wordmark),
            Print::plain(grove, search.leaf()),
            Print::plain(grove, bag.leaf()),
            Print::plain(grove, rule),
        ];
        Self {
            search,
            bag,
            tally,
            library,
            has_library: false,
            printed,
        }
    }

    pub fn printed(&self) -> &[Printed] {
        &self.printed
    }

    pub fn search(&mut self) -> &mut Field {
        &mut self.search
    }

    pub fn bag(&self) -> Leaf {
        self.bag.leaf()
    }

    /// Counts `n` on the bag.
    pub fn count(&mut self, grove: &mut Grove, n: u32) {
        self.tally.set(grove, n);
    }

    /// Puts the library in the bar, once there is something in it.
    pub fn library(&mut self, grove: &mut Grove) {
        if self.has_library {
            return;
        }
        self.has_library = true;
        grove.visible(self.library.leaf(), true);
        grove.opacity(self.library.leaf(), 0.0);
        grove.animate(
            self.library.leaf(),
            foliage::Motion::Opacity(1.0),
            foliage::Timing::ms(420).ease(foliage::Ease::Decelerate),
        );
    }

    /// Carries the bar. Whether the search's words moved, and what a press asked.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> (bool, Option<Asked>) {
        let edited = self.search.frame(grove, pollen);
        let asked = if self.bag.pressed(pollen) {
            Some(Asked::Bag)
        } else if self.has_library && self.library.pressed(pollen) {
            Some(Asked::Library)
        } else {
            None
        };
        (edited, asked)
    }
}

/// How tall the shop's label stands, a row of it, and how far under the presses it stands.
const FACT_ROW: f32 = 30.0;
const FACTS_H: f32 = 40.0 + HEAVY + 5.0 * FACT_ROW + 34.0;
const FACTS_GAP: f32 = 40.0;
/// How much of the featured card is under its window.
pub const HERO_BELOW: f32 = 146.0;

/// The marquee's words, round and round.
const BAND: &str = "free delivery on every order: it weighs nothing   ·   code FERN takes 15% off the bag   ·   every file weighed to the byte   ·   printed with your name   ·   sealed by hand   ·   ";

/// The front: the band, and the window under it.
pub struct Front {
    tail: Leaf,
    marquee: Marquee,
    featured: Card,
    browse: Chip,
    bag: Chip,
    printed: Vec<Printed>,
}

impl Front {
    pub fn grow(
        grove: &mut Grove,
        page: Leaf,
        faces: &Faces,
        icons: &Icons,
        shadow: Plate,
    ) -> Self {
        let m = measure();
        let band = grove.branch(
            page,
            Panel::new()
                .color(Palette::Ink)
                .intangible()
                .at(Location::new()
                    .xs(
                        left(0.px()).right(100.pct()),
                        top(BAR_NARROW.px()).height(36.px()),
                    )
                    .lg(
                        left(0.px()).right(100.pct()),
                        top(BAR_WIDE.px()).height(36.px()),
                    )),
        );
        let marquee = Marquee::grow(
            grove,
            band,
            Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).bottom(100.pct())),
            BAND,
            foliage::FontSize::new().xs(15).md(16),
            Some(faces.italic),
            Palette::Surface,
            36.0,
        );
        grove.elevate(marquee.leaf(), Elevation::up(1));
        let col = grove.branch(
            page,
            Stem::new()
                .intangible()
                .anchored(band)
                .font_size(lichen::caption())
                .at(column(|| top(anchor().bottom() + 30.px()).height(10.px()))),
        );
        // The masthead line: which catalogue this is, over a hairline.
        let issue = grove.branch(
            col,
            Text::new("no. 4 · autumn 2026")
                .font_size(lichen::caption())
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).width(content()),
                    top(0.px()).height(1.letters()),
                )),
        );
        let count = grove.branch(
            col,
            Text::new("16 goods · 4 aisles")
                .font_size(lichen::caption())
                .color(Palette::Muted)
                .intangible()
                .at(Location::new().xs(
                    right(100.pct()).width(content()),
                    top(0.px()).height(1.letters()),
                )),
        );
        let hairline = grove.branch(
            col,
            Stem::new()
                .intangible()
                .anchored(issue)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + 10.px()).height(HAIR.px()),
                )),
        );
        let hairline = Print::rule_in(grove, hairline, Palette::Ink);
        // The headline, two lines at every width: the second in italic, as the shop's name is.
        let first = grove.branch(
            col,
            Text::new("Digital goods,")
                .font(faces.roman)
                .font_size(theme::headline())
                .color(Palette::Ink)
                .intangible()
                .anchored(hairline.leaf())
                .at(Location::new().xs(
                    left((-2).px()).right(100.pct()),
                    top(anchor().bottom() + 22.px()).height(content()),
                )),
        );
        let second = grove.branch(
            col,
            Text::new("sold by the byte.")
                .font(faces.italic)
                .font_size(theme::headline())
                .color(Palette::Accent)
                .intangible()
                .anchored(first)
                .at(Location::new().xs(
                    left((-2).px()).right(100.pct()),
                    top(anchor().bottom()).height(content()),
                )),
        );
        // Under it, side by side on a wide window: what the shop is, and the good on show.
        let left_side = |v: Vertical| {
            Location::new()
                .xs(left(0.px()).right(100.pct()), v.clone())
                .md(left(0.px()).width(52.pct()), v)
        };
        let lead = grove.branch(
            col,
            Text::new(
                "Palettes, shaders, type specimens and sound packs. Every file is weighed to the \
                 byte, printed with the name of whoever holds its licence, and sealed before it \
                 leaves the shop.",
            )
            .font_size(theme::lead())
            .color(Palette::Ink)
            .intangible()
            .anchored(second)
            .at(left_side(
                top(anchor().bottom() + 28.px()).height(content()),
            )),
        );
        // The two presses side by side, but one over the other where the lead's column is too
        // narrow to hold both: beside the card on a middling window.
        let stacked = 2.0 * m.height + m.gap;
        let presses = grove.branch(
            col,
            Stem::new()
                .intangible()
                .anchored(lead)
                .font_size(lichen::caption())
                .at(Location::new()
                    .xs(
                        left(0.px()).right(100.pct()),
                        top(anchor().bottom() + 26.px()).height(m.height.px()),
                    )
                    .md(
                        left(0.px()).width(52.pct()),
                        top(anchor().bottom() + 26.px()).height(stacked.px()),
                    )
                    .lg(
                        left(0.px()).width(52.pct()),
                        top(anchor().bottom() + 26.px()).height(m.height.px()),
                    )),
        );
        let mut browse = Chip::grow(
            grove,
            presses,
            Location::new().xs(
                left(0.px()).width(Chip::width("browse the aisles")),
                top(0.px()).height(m.height.px()),
            ),
            Elevation::up(1),
            icons.arrow_right,
            "browse the aisles",
        );
        browse.arm(grove, Press::Armed);
        let bag = Chip::grow(
            grove,
            presses,
            Location::new()
                .xs(
                    left(Chip::width("browse the aisles") + m.gap.px())
                        .width(Chip::width("open the bag")),
                    top(0.px()).height(m.height.px()),
                )
                .md(
                    left(0.px()).width(Chip::width("open the bag")),
                    top((m.height + m.gap).px()).height(m.height.px()),
                )
                .lg(
                    left(Chip::width("browse the aisles") + m.gap.px())
                        .width(Chip::width("open the bag")),
                    top(0.px()).height(m.height.px()),
                ),
            Elevation::up(1),
            icons.shopping_bag,
            "open the bag",
        );
        // The shop's own label: what is on the shelves, weighed.
        let (_, mut fact_prints) = facts(grove, col, presses, faces);
        // The good on show: under the shop's label on a narrow window; beside the lead, the presses
        // and the label on a wider one, exactly as tall as the three, so neither column runs past
        // the other.
        let beside = 26.0 + m.height + FACTS_GAP + FACTS_H;
        let beside_stacked = beside + m.height + m.gap;
        let at_card = Location::new()
            .xs(
                left(0.px()).right(100.pct()),
                top(anchor().bottom() + (beside + 36.0).px())
                    .height(aspect(1.25) + (HERO_BELOW + 18.0 - 36.0 / 1.25).px()),
            )
            .md(
                right(100.pct()).width(42.pct()).at_most(460.px()),
                top(anchor().top() + 4.px()).bottom(anchor().bottom() + beside_stacked.px()),
            )
            .lg(
                right(100.pct()).width(42.pct()).at_most(460.px()),
                top(anchor().top() + 4.px()).bottom(anchor().bottom() + beside.px()),
            );
        let featured = Card::grow(grove, col, at_card, FEATURED, Size::Hero, faces, shadow);
        grove.anchor(featured.leaf(), lead);
        // The foot of the front, which is the card's on any window.
        let tail = grove.branch(
            col,
            Stem::new()
                .intangible()
                .anchored(featured.leaf())
                .at(Location::new().xs(
                    left(0.px()).width(1.px()),
                    top(anchor().bottom()).height(1.px()),
                )),
        );
        let mut printed = vec![
            Print::words(grove, issue),
            Print::words(grove, count),
            hairline,
            Print::words(grove, first),
            Print::words(grove, second),
            Print::words(grove, lead),
            Print::plain(grove, browse.leaf()),
            Print::plain(grove, bag.leaf()),
        ];
        printed.append(&mut fact_prints);
        printed.extend(featured.printed().iter().copied());
        Self {
            tail,
            marquee,
            featured,
            browse,
            bag,
            printed,
        }
    }

    pub fn tail(&self) -> Leaf {
        self.tail
    }

    pub fn printed(&self) -> &[Printed] {
        &self.printed
    }

    pub fn featured_leaf(&self) -> Leaf {
        self.featured.leaf()
    }

    /// Carries the front. `seen` is whether nothing covers it.
    pub fn frame(
        &mut self,
        grove: &mut Grove,
        pollen: &Pollen,
        over: bool,
        seen: bool,
        budget: &mut u32,
    ) -> Option<Asked> {
        self.marquee.frame(grove, pollen);
        self.featured.frame(grove, over, seen, budget);
        if self.browse.pressed(pollen) {
            return Some(Asked::Aisles);
        }
        if self.bag.pressed(pollen) {
            return Some(Asked::Bag);
        }
        if pollen.activated(self.featured.leaf()) {
            return Some(Asked::Good(FEATURED));
        }
        None
    }
}

/// The shop's own label, grown under `col` beneath `after`: a nutrition panel for a shop that sells
/// bytes. Hands back its foot, and what is printed on it.
fn facts(grove: &mut Grove, col: Leaf, after: Leaf, faces: &Faces) -> (Leaf, Vec<Printed>) {
    let heaviest = (0..GOODS.len())
        .max_by_key(|&n| GOODS[n].bytes)
        .unwrap_or(0);
    let lightest = (0..GOODS.len())
        .min_by_key(|&n| GOODS[n].bytes)
        .unwrap_or(0);
    let rows: [(&str, String); 5] = [
        ("goods on the shelves", GOODS.len().to_string()),
        ("aisles", Aisle::ALL.len().to_string()),
        (
            "heaviest",
            format!(
                "{} · {}",
                GOODS[heaviest].name,
                weight(GOODS[heaviest].bytes)
            ),
        ),
        (
            "lightest",
            format!(
                "{} · {}",
                GOODS[lightest].name,
                weight(GOODS[lightest].bytes)
            ),
        ),
        ("postage", "€0.00".to_string()),
    ];
    let row_h = FACT_ROW;
    let height = FACTS_H;
    let panel = grove.branch(
        col,
        Stem::new()
            .intangible()
            .anchored(after)
            .font_size(lichen::caption())
            .at(Location::new()
                .xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + FACTS_GAP.px()).height(height.px()),
                )
                .md(
                    left(0.px()).width(52.pct()).at_most(520.px()),
                    top(anchor().bottom() + FACTS_GAP.px()).height(height.px()),
                )),
    );
    let mut printed = Vec::new();
    let title = grove.branch(
        panel,
        Text::new("Shop facts")
            .font(faces.roman)
            .font_size(foliage::FontSize::new().xs(24).md(26))
            .color(Palette::Ink)
            .intangible()
            .at(Location::new().xs(
                left(0.px()).width(content()),
                top(0.px()).height(1.letters()),
            )),
    );
    printed.push(Print::words(grove, title));
    let per = grove.branch(
        panel,
        Text::new("per catalogue")
            .font_size(theme::small())
            .color(Palette::Muted)
            .intangible()
            .at(Location::new().xs(
                right(100.pct()).width(content()),
                top(12.px()).height(1.letters()),
            )),
    );
    printed.push(Print::words(grove, per));
    printed.push(Print::rule(
        grove,
        panel,
        Location::new().xs(
            left(0.px()).right(100.pct()),
            top(40.px()).height(HEAVY.px()),
        ),
        Palette::Ink,
    ));
    for (n, (name, value)) in rows.iter().enumerate() {
        let y = 40.0 + HEAVY + n as f32 * row_h;
        let key = grove.branch(
            panel,
            Text::new(*name)
                .font_size(lichen::caption())
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).width(content()),
                    foliage::center_y((y + row_h / 2.0).px()).height(1.letters()),
                )),
        );
        let value = grove.branch(
            panel,
            Text::new(value.as_str())
                .font_size(lichen::caption())
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    right(100.pct()).width(content()),
                    foliage::center_y((y + row_h / 2.0).px()).height(1.letters()),
                )),
        );
        printed.push(Print::words(grove, key));
        printed.push(Print::words(grove, value));
        let last = n + 1 == rows.len();
        printed.push(Print::rule(
            grove,
            panel,
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top((y + row_h - if last { 0.0 } else { HAIR }).px())
                    .height(if last { HEAVY } else { HAIR }.px()),
            ),
            Palette::Ink,
        ));
    }
    let foot = grove.branch(
        panel,
        Text::new("every weight to the byte · every file sealed")
            .font_size(theme::small())
            .color(Palette::Muted)
            .intangible()
            .at(Location::new().xs(
                left(0.px()).right(100.pct()),
                bottom(100.pct()).height(1.letters()),
            )),
    );
    printed.push(Print::words(grove, foot));
    (panel, printed)
}
