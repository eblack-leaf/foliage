//! A good on the shelf: its label.
//!
//! Every good is shown the same way, as the label it would be sold under -- its picture in a window
//! at the top, then its number and aisle, a hairline, its name, and under a heavy rule what it
//! weighs and what it costs. The rows are fixed, so a row of labels reads across as one table:
//! every name starts on the same line, and every price stands on the same rule.

use foliage::{
    Boxed, Elevation, FontSize, Grove, Grow, Leaf, Location, Palette, Panel, Place, Plate, Source,
    Stem, Text, Vertical, anchor, aspect, bottom, content, left, right, top,
};

use arboretum::sprout::{Lean, Print, Printed, Shade};

use crate::art::{Art, Scale};
use crate::catalog::{GOODS, price, weight};
use crate::theme::{self, Faces, HAIR, HEAVY};

/// How large a label is.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Size {
    /// One of the grid's.
    Grid,
    /// The good on show at the front: larger, and with its line of what it is.
    Hero,
}

impl Size {
    fn pad(self) -> f32 {
        match self {
            Size::Grid => 12.0,
            Size::Hero => 18.0,
        }
    }

    fn name(self) -> FontSize {
        match self {
            Size::Grid => theme::name(),
            Size::Hero => FontSize::new().xs(26).md(30),
        }
    }

    fn price(self) -> FontSize {
        match self {
            Size::Grid => theme::price(),
            Size::Hero => FontSize::new().xs(24).md(28),
        }
    }

    fn kicker(self) -> FontSize {
        match self {
            Size::Grid => theme::small(),
            Size::Hero => lichen::caption(),
        }
    }
}

/// A label, grown.
pub struct Card {
    root: Leaf,
    good: usize,
    art: Art,
    shade: Shade,
    lean: Lean,
    printed: Vec<Printed>,
}

impl Card {
    /// Grows `good`'s label under `under`, standing where `at` says, at `size`, with its shadow
    /// drawn from `shadow`.
    #[allow(clippy::too_many_arguments)]
    pub fn grow(
        grove: &mut Grove,
        under: Leaf,
        at: Location,
        good: usize,
        size: Size,
        faces: &Faces,
        shadow: Plate,
    ) -> Self {
        let g = &GOODS[good];
        let pad = size.pad();
        let root = grove.branch(under, Stem::new().interactive().at(at));
        let shade = Shade::grow(grove, root, shadow, 10.0);
        let label = grove.branch(
            root,
            Panel::new()
                .color(Palette::Raised)
                .outline(HAIR, Palette::Ink)
                .intangible()
                .at(Location::new()),
        );
        let inside =
            |v: Vertical| Location::new().xs(left(pad.px()).right(100.pct() - pad.px()), v);
        let window = match size {
            Size::Grid => inside(top(pad.px()).height(aspect(1.0))),
            Size::Hero => Location::new()
                .xs(
                    left(pad.px()).right(100.pct() - pad.px()),
                    top(pad.px()).height(aspect(1.25)),
                )
                .md(
                    left(pad.px()).right(100.pct() - pad.px()),
                    top(pad.px()).bottom(100.pct() - crate::front::HERO_BELOW.px()),
                ),
        };
        let art_scale = match size {
            Size::Grid => Scale::Card,
            Size::Hero => Scale::Hero,
        };
        let mut art = Art::grow(grove, label, window, good, art_scale, faces);
        grove.elevate(art.leaf(), Elevation::up(1));
        art.speed = 1.0;
        let fed = Print::feed(
            grove,
            art.leaf(),
            Palette::Raised,
            Palette::Accent,
            Elevation::up(8),
        );
        // The number and the aisle, on one line under the window.
        let no = grove.branch(
            label,
            Text::new(format!("No. {:02}", g.no))
                .font_size(size.kicker())
                .color(Palette::Ink)
                .intangible()
                .elevate(Elevation::up(1))
                .anchored(art.leaf())
                .at(Location::new().xs(
                    left(pad.px()).width(content()),
                    top(anchor().bottom() + 10.px()).height(1.letters()),
                )),
        );
        let aisle = grove.branch(
            label,
            Text::new(g.aisle.single())
                .font_size(size.kicker())
                .color(Palette::Muted)
                .intangible()
                .elevate(Elevation::up(1))
                .anchored(art.leaf())
                .at(Location::new().xs(
                    right(100.pct() - pad.px()).width(content()),
                    top(anchor().bottom() + 10.px()).height(1.letters()),
                )),
        );
        let hairline = grove.branch(
            label,
            Stem::new()
                .intangible()
                .elevate(Elevation::up(1))
                .anchored(no)
                .at(inside(top(anchor().bottom() + 8.px()).height(HAIR.px()))),
        );
        let hairline = Print::rule_in(grove, hairline, Palette::Ink);
        let name = grove.branch(
            label,
            Text::new(g.name)
                .font(faces.roman)
                .font_size(size.name())
                .color(Palette::Ink)
                .intangible()
                .elevate(Elevation::up(1))
                .anchored(hairline.leaf())
                .at(inside(top(anchor().bottom() + 8.px()).height(content()))),
        );
        let mut printed = vec![
            fed,
            Print::words(grove, no),
            Print::words(grove, aisle),
            hairline,
            Print::words(grove, name),
        ];
        // What it weighs and costs, standing on the heavy rule at the foot.
        let foot = 30.0;
        let heavy = grove.branch(
            label,
            Stem::new()
                .intangible()
                .elevate(Elevation::up(1))
                .at(inside(
                    bottom(100.pct() - (pad + foot).px()).height(HEAVY.px()),
                )),
        );
        printed.push(Print::rule_in(grove, heavy, Palette::Ink));
        let weighs = grove.branch(
            label,
            Text::new(format!("net {}", weight(g.bytes)))
                .font_size(size.kicker())
                .color(Palette::Muted)
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    left(pad.px()).width(content()),
                    bottom(100.pct() - (pad + 4.0).px()).height(1.letters()),
                )),
        );
        let costs = grove.branch(
            label,
            Text::new(price(g.cents))
                .font(faces.italic)
                .font_size(size.price())
                .color(Palette::Ink)
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    right(100.pct() - pad.px()).width(content()),
                    bottom(100.pct() - pad.px()).height(1.letters()),
                )),
        );
        printed.push(Print::words(grove, weighs));
        printed.push(Print::words(grove, costs));
        Self {
            root,
            good,
            art,
            shade,
            lean: Lean::new(root, 4.0),
            printed,
        }
    }

    /// The label itself, which takes the press.
    pub fn leaf(&self) -> Leaf {
        self.root
    }

    pub fn good(&self) -> usize {
        self.good
    }

    /// Everything on it, in the order it is printed.
    pub fn printed(&self) -> &[Printed] {
        &self.printed
    }

    pub fn art(&mut self) -> &mut Art {
        &mut self.art
    }

    /// Carries the label for a frame: lifted toward the pointer while it is `over` it, and its
    /// picture `awake` or asleep.
    pub fn frame(&mut self, grove: &mut Grove, over: bool, awake: bool, budget: &mut u32) {
        self.lean.frame(grove, over);
        self.shade.lift(grove, over);
        self.art.wake(grove, awake);
        self.art.frame(grove, budget);
    }
}
