//! The aisles: a heading that counts what is on the shelves, the aisles' tabs, the order the
//! shelves are in, and the labels themselves.
//!
//! The grid is two, three or four labels across as the window allows, each a share of the width,
//! so a row never runs past the window's edge. When the aisle, the order or the search changes,
//! the labels that stay move to their new places one after another, and the rest are put away.

use foliage::{
    Boxed, Ease, Elevation, Grove, Grow, Leaf, Length, Location, Motion, Palette, Panel, Place,
    Plate, Pollen, Position, Sap, Source, Stem, Text, Timing, Tween, Vein, anchor, aspect,
    center_y, content, left, right, top, trunk,
};
use lichen::{Chip, Pick, Say, Voice, measure};

use arboretum::sprout::{Hover, Print, Printed};

use crate::card::{Card, Size};
use crate::catalog::{Aisle, GOODS, Sort, shelf};
use crate::icons::Icons;
use crate::theme::{self, Faces, HAIR, HEAVY, column};

/// The tabs: every aisle, then each.
const TABS: [&str; 5] = ["all", "palettes", "shaders", "type", "sound"];
/// The tabs' face size, the width of one of its letters, and the room between two tabs.
const TAB_SIZE: u32 = 18;
const TAB_LETTER: f32 = TAB_SIZE as f32 * 0.5;
const TAB_GAP: f32 = 22.0;
/// How wide the sort pick stands.
const SORT_W: f32 = 308.0;

/// How many labels across, the room between them, and how much taller than wide each is, at
/// each width: narrow, middling, wide.
const ACROSS: [f32; 3] = [2.0, 3.0, 4.0];
const GAP: [f32; 3] = [12.0, 18.0, 24.0];
const TAIL: [f32; 3] = [126.0, 130.0, 132.0];

struct Shelved {
    card: Card,
    /// Where it stands on the shelf now, if it is on it.
    place: Option<usize>,
    going: Option<Tween>,
}

pub struct Grid {
    root: Leaf,
    count: Say,
    tabs: Vec<Leaf>,
    underline: Leaf,
    sort: Pick,
    foot: Leaf,
    shelved: Vec<Shelved>,
    empty: Leaf,
    clear: Chip,
    empty_says: Leaf,
    aisle: Option<Aisle>,
    query: String,
    order: Sort,
    printed: Vec<Printed>,
    shown: bool,
}

/// What a press in the aisles asks of the shop.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Asked {
    Good(usize),
    /// To clear the search.
    Clear,
}

impl Grid {
    #[allow(clippy::too_many_arguments)]
    pub fn grow(
        grove: &mut Grove,
        page: Leaf,
        after: Leaf,
        faces: &Faces,
        icons: &Icons,
        hover: &mut Hover,
        shadow: Plate,
    ) -> Self {
        let m = measure();
        let root = grove.branch(
            page,
            Stem::new()
                .intangible()
                .anchored(after)
                .font_size(lichen::caption())
                .at(column(|| top(anchor().bottom() + 96.px()).height(10.px()))),
        );
        let heading = grove.branch(
            root,
            Text::new("The aisles")
                .font(faces.roman)
                .font_size(theme::heading())
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).width(content()),
                    top(0.px()).height(1.letters()),
                )),
        );
        let count = Say::grow(
            grove,
            root,
            Location::new().xs(
                right(100.pct()).width(content()),
                top(anchor().bottom() - 1.letters() - 4.px()).height(1.letters()),
            ),
            None,
        );
        grove.anchor(count.leaf(), heading);
        let rule = grove.branch(
            root,
            Stem::new()
                .intangible()
                .anchored(heading)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + 12.px()).height(HEAVY.px()),
                )),
        );
        let rule = Print::rule_in(grove, rule, Palette::Ink);
        // The tabs, and the order beside them on a wide window or under them on a narrow one.
        let controls = grove.branch(
            root,
            Stem::new()
                .intangible()
                .anchored(rule.leaf())
                .font_size(lichen::caption())
                .at(Location::new()
                    .xs(
                        left(0.px()).right(100.pct()),
                        top(anchor().bottom() + 14.px()).height((2.0 * m.height + 14.0).px()),
                    )
                    .lg(
                        left(0.px()).right(100.pct()),
                        top(anchor().bottom() + 14.px()).height(m.height.px()),
                    )),
        );
        let mut x = 0.0;
        let mut tabs = Vec::new();
        let mut printed_tabs = Vec::new();
        for name in TABS {
            let w = name.chars().count() as f32 * TAB_LETTER;
            let tab = grove.branch(
                controls,
                Stem::new().interactive().at(Location::new().xs(
                    left((x - 6.0).px()).width((w + 12.0).px()),
                    top(0.px()).height(m.height.px()),
                )),
            );
            let words = grove.branch(
                tab,
                Text::new(name)
                    .font(faces.roman)
                    .font_size(foliage::FontSize::new().xs(TAB_SIZE))
                    .color(Palette::Ink)
                    .intangible()
                    .at(Location::new().xs(
                        left(6.px()).width(content()),
                        center_y(50.pct() - 2.px()).height(1.letters()),
                    )),
            );
            printed_tabs.push(Print::words(grove, words));
            tabs.push(tab);
            x += w + TAB_GAP;
        }
        let underline = grove.branch(
            controls,
            Panel::new()
                .color(Palette::Accent)
                .intangible()
                .elevate(Elevation::up(1))
                .at(underline_at(0)),
        );
        let sort = Pick::grow(
            grove,
            controls,
            Location::new()
                .xs(
                    left(0.px()).width(SORT_W.px()),
                    top((m.height + 14.0).px()).height(m.height.px()),
                )
                .lg(
                    right(100.pct()).width(SORT_W.px()),
                    top(0.px()).height(m.height.px()),
                ),
            6.0,
            "sort",
            Sort::NAMES,
            1,
        );
        let grid = grove.branch(
            root,
            Stem::new()
                .intangible()
                .anchored(controls)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + 26.px()).height(1.px()),
                )),
        );
        let order = shelf(None, "", Sort::Featured);
        let shelved: Vec<Shelved> = (0..GOODS.len())
            .map(|good| {
                let place = order.iter().position(|&n| n == good);
                let card = Card::grow(
                    grove,
                    grid,
                    slot(place.unwrap_or(0)),
                    good,
                    Size::Grid,
                    faces,
                    shadow,
                );
                hover.watch(card.leaf());
                Shelved {
                    card,
                    place,
                    going: None,
                }
            })
            .collect();
        let foot = grove.branch(grid, Stem::new().intangible().at(foot(order.len())));
        // What the shelves say when nothing on them is found.
        let empty = grove.branch(
            grid,
            Panel::new()
                .color(Palette::Raised)
                .outline(HAIR, Palette::Ink)
                .intangible()
                .visible(false)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(0.px()).height(EMPTY_H.px()),
                )),
        );
        let empty_says = grove.branch(
            empty,
            Text::new("")
                .font(faces.roman)
                .font_size(foliage::FontSize::new().xs(20).md(24))
                .color(Palette::Ink)
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    left(18.px()).right(100.pct() - 18.px()),
                    top(18.px()).height(content()),
                )),
        );
        grove.branch(
            empty,
            Panel::new()
                .color(Palette::Ink)
                .intangible()
                .elevate(Elevation::up(1))
                .anchored(empty_says)
                .at(Location::new().xs(
                    left(18.px()).right(100.pct() - 18.px()),
                    top(anchor().bottom() + 12.px()).height(HEAVY.px()),
                )),
        );
        let hint = grove.branch(
            empty,
            Text::new("try rain, rust or aurora, or look along every aisle at once")
                .font_size(lichen::caption())
                .color(Palette::Muted)
                .intangible()
                .elevate(Elevation::up(1))
                .anchored(empty_says)
                .at(Location::new().xs(
                    left(18.px()).right(100.pct() - 18.px()),
                    top(anchor().bottom() + 26.px()).height(content()),
                )),
        );
        let clear = Chip::grow(
            grove,
            empty,
            Location::new().xs(
                left(18.px()).width(Chip::width("clear the search")),
                bottom_at(18.0),
            ),
            Elevation::up(2),
            icons.x,
            "clear the search",
        );
        let _ = hint;
        let mut printed = vec![Print::words(grove, heading), rule];
        printed.extend(printed_tabs);
        printed.push(Print::plain(grove, underline));
        printed.push(Print::plain(grove, sort.leaf()));
        let mut grid = Self {
            root,
            count,
            tabs,
            underline,
            sort,
            foot,
            shelved,
            empty,
            clear,
            empty_says,
            aisle: None,
            query: String::new(),
            order: Sort::Featured,
            printed,
            shown: false,
        };
        grid.say(grove, order.len());
        grid
    }

    pub fn leaf(&self) -> Leaf {
        self.root
    }

    /// The foot of the shelves, which whatever follows stands under.
    pub fn tail(&self) -> Leaf {
        self.foot
    }

    /// Shows what `query` finds.
    pub fn search(&mut self, grove: &mut Grove, query: &str) {
        if self.query == query.trim() {
            return;
        }
        self.query = query.trim().to_string();
        self.arrange(grove);
    }

    /// Prints the aisles in, once the page has come down to them. Whether it did so this frame.
    pub fn arrive(&mut self, grove: &mut Grove, print: &mut Print) -> bool {
        if self.shown {
            return false;
        }
        let Some(Sap::Section(drawn)) = grove.tap(self.root, Vein::Drawn) else {
            return false;
        };
        if drawn.top() > grove.viewport().height * 0.92 {
            return false;
        }
        self.shown = true;
        print.run(grove, &self.printed, 0, 50);
        let mut on: Vec<&Shelved> = self.shelved.iter().filter(|s| s.place.is_some()).collect();
        on.sort_by_key(|s| s.place);
        for (k, shelved) in on.iter().enumerate() {
            print.run(grove, shelved.card.printed(), 240 + k as u64 * 70, 40);
        }
        true
    }

    /// Carries the aisles for a frame.
    pub fn frame(
        &mut self,
        grove: &mut Grove,
        pollen: &Pollen,
        hover: &Hover,
        budget: &mut u32,
    ) -> Option<Asked> {
        let mut changed = false;
        for (n, &tab) in self.tabs.iter().enumerate() {
            if pollen.activated(tab) {
                let aisle = match n {
                    0 => None,
                    k => Some(Aisle::ALL[k - 1]),
                };
                if aisle != self.aisle {
                    self.aisle = aisle;
                    grove.animate(
                        self.underline,
                        Motion::Location(underline_at(n)),
                        Timing::ms(420).ease(Ease::Emphasis),
                    );
                    changed = true;
                }
            }
        }
        if self.sort.frame(grove, pollen) {
            self.order = Sort::ALL[self.sort.index()];
            changed = true;
        }
        if changed {
            self.arrange(grove);
        }
        if let Some(area) = pollen.resized() {
            for shelved in &mut self.shelved {
                shelved.card.art().fit(grove, area.width);
            }
        }
        let mut asked = None;
        for shelved in &mut self.shelved {
            if let Some(going) = shelved.going
                && pollen.finished(going)
            {
                shelved.going = None;
                grove.visible(shelved.card.leaf(), false);
            }
            let leaf = shelved.card.leaf();
            let over = hover.is_over(leaf);
            shelved.card.frame(grove, over, over, budget);
            if pollen.activated(leaf) {
                asked = Some(Asked::Good(shelved.card.good()));
            }
        }
        if self.clear.pressed(pollen) {
            asked = Some(Asked::Clear);
        }
        asked
    }

    /// Puts the shelves in the order the aisle, the search and the sort say: what stays moves to
    /// its place, what comes back comes up where it belongs, and what goes is put away.
    fn arrange(&mut self, grove: &mut Grove) {
        let order = shelf(self.aisle, &self.query, self.order);
        let mut moved = 0u64;
        for shelved in &mut self.shelved {
            let leaf = shelved.card.leaf();
            let place = order.iter().position(|&n| n == shelved.card.good());
            match (shelved.place, place) {
                (Some(was), Some(now)) => {
                    if was != now {
                        grove.animate(
                            leaf,
                            Motion::Location(slot(now)),
                            Timing::ms(560).after(moved * 26).ease(Ease::Emphasis),
                        );
                        moved += 1;
                    }
                }
                (None, Some(now)) => {
                    if let Some(going) = shelved.going.take() {
                        grove.stop(going);
                    }
                    grove.visible(leaf, true);
                    grove.at(leaf, slot(now));
                    grove.shift(leaf, Position::new(0.0, 14.0));
                    let timing = Timing::ms(420)
                        .after(140 + now as u64 * 30)
                        .ease(Ease::Decelerate);
                    grove.animate(leaf, Motion::Opacity(1.0), timing);
                    grove.animate(leaf, Motion::Shift(Position::default()), timing);
                }
                (Some(_), None) => {
                    grove.animate(
                        leaf,
                        Motion::Opacity(0.0),
                        Timing::ms(200).ease(Ease::Accelerate),
                    );
                    shelved.going = Some(grove.timer(Timing::ms(210)));
                }
                (None, None) => {}
            }
            shelved.place = place;
        }
        grove.at(
            self.foot,
            foot(order.len().max(if order.is_empty() { 0 } else { 1 })),
        );
        let empty = order.is_empty();
        grove.visible(self.empty, empty);
        if empty {
            grove.text(
                self.empty_says,
                format!("Nothing on these shelves weighs “{}”.", self.query),
            );
            grove.at(
                self.foot,
                Location::new().xs(left(0.px()).width(1.px()), top(EMPTY_H.px()).height(1.px())),
            );
        }
        self.say(grove, order.len());
    }

    /// Counts what the shelves show, beside the heading.
    fn say(&mut self, grove: &mut Grove, found: usize) {
        let of = match self.aisle {
            Some(aisle) => format!(
                "{found} {}",
                if found == 1 {
                    aisle.single().to_string()
                } else {
                    aisle.name().to_string()
                }
            ),
            None => format!("{found} good{}", if found == 1 { "" } else { "s" }),
        };
        let words = match (self.query.is_empty(), found) {
            (true, _) => format!("{of} on the shelves"),
            (false, 0) => format!("nothing weighs “{}”", clip(&self.query, 18)),
            (false, _) => format!("{of} for “{}”", clip(&self.query, 18)),
        };
        let voice = if found == 0 {
            Voice::Warn
        } else {
            Voice::Quiet
        };
        self.count.tell(grove, &words, voice);
    }
}

/// How tall the shelves' word for nothing found stands.
const EMPTY_H: f32 = 190.0;

fn bottom_at(from_foot: f32) -> foliage::Vertical {
    foliage::bottom(100.pct() - from_foot.px()).height(measure().height.px())
}

/// Where the underline stands under tab `n`.
fn underline_at(n: usize) -> Location {
    let x: f32 = TABS[..n]
        .iter()
        .map(|t| t.chars().count() as f32 * TAB_LETTER + TAB_GAP)
        .sum();
    let w = TABS[n].chars().count() as f32 * TAB_LETTER;
    Location::new().xs(
        left(x.px()).width(w.px()),
        top((measure().height - 4.0).px()).height(2.px()),
    )
}

/// How wide a label is with `across` in a row.
fn share(across: f32, gap: f32) -> Length {
    (100.pct() - ((across - 1.0) * gap).px()) * (1.0 / across)
}

/// How far down row `row` starts.
fn down(row: f32, across: f32, gap: f32, tail: f32) -> foliage::VerticalCoordinate {
    let side = (trunk().width() - ((across - 1.0) * gap).px()) * (1.0 / across);
    (side * row + (row * (tail + gap)).px()).into()
}

/// Where the label in place `n` stands, at each width.
pub fn slot(n: usize) -> Location {
    let at = |k: usize| {
        let (across, gap, tail) = (ACROSS[k], GAP[k], TAIL[k]);
        let (column, row) = ((n as f32) % across, (n as f32 / across).floor());
        (
            left(share(across, gap) * column + (column * gap).px()).width(share(across, gap)),
            top(down(row, across, gap, tail)).height(aspect(1.0) + tail.px()),
        )
    };
    let ((xh, xv), (mh, mv), (lh, lv)) = (at(0), at(1), at(2));
    Location::new().xs(xh, xv).md(mh, mv).lg(lh, lv)
}

/// The foot of shelves holding `count` labels.
fn foot(count: usize) -> Location {
    let at = |k: usize| {
        let (across, gap, tail) = (ACROSS[k], GAP[k], TAIL[k]);
        let rows = (count as f32 / across).ceil();
        let side = (trunk().width() - ((across - 1.0) * gap).px()) * (1.0 / across);
        (
            left(0.px()).width(1.px()),
            top(side * rows + (rows * (tail + gap) - gap).max(0.0).px()).height(1.px()),
        )
    };
    let ((xh, xv), (mh, mv), (lh, lv)) = (at(0), at(1), at(2));
    Location::new().xs(xh, xv).md(mh, mv).lg(lh, lv)
}

/// `text` cut to `most` characters, with an ellipsis where it was cut.
pub fn clip(text: &str, most: usize) -> String {
    if text.chars().count() <= most {
        return text.to_string();
    }
    let cut: String = text.chars().take(most.saturating_sub(1)).collect();
    format!("{}…", cut.trim_end())
}
