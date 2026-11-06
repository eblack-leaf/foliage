//! The visitors' wall: what other people grew, and what this visitor grows, newest first.
//!
//! A grid that is two, three or four across as the window allows, every card a share of the width
//! rather than a fixed size, so a row never runs off the page's edge.

use foliage::{
    Area, Boxed, Ease, Fit, Font, Grove, Grow, Image, Leaf, Location, Motion, Palette, Place,
    Plate, Sap, Source, Stem, Text, Timing, Vein, anchor, aspect, content, left, top, trunk,
};

use arboretum::genart::{Painter, Params, Seed};
use arboretum::sprout::{Arrival, Emerge};

use crate::develop::{area, blank};
use crate::theme::{self, clip, column};
use crate::works::VISITORS;

/// How large a piece's picture is painted.
const PICTURE: u32 = 144;
/// How many cards across, narrow, middling, wide.
const ACROSS: [f32; 3] = [2.0, 3.0, 4.0];
/// The room between two cards.
const GAP: f32 = 14.0;
/// How much of a card is words, under its picture: two lines of prompt and one of who.
const WORDS: f32 = 74.0;
/// How many characters of a prompt a card shows, so that it stays in two lines.
const SAYS: usize = 34;
/// How many pieces the wall holds before the oldest comes down.
const HOLDS: usize = 12;

struct Card {
    root: Leaf,
    plate: Plate,
    painter: Option<Painter>,
}

pub struct Wall {
    root: Leaf,
    grid: Leaf,
    cards: Vec<Card>,
    tail: Leaf,
    next_handle: u32,
    head: Vec<Arrival>,
    shown: bool,
}

impl Wall {
    pub fn grow(grove: &mut Grove, page: Leaf, after: Leaf, face: Font) -> Self {
        let root = grove.branch(
            page,
            Stem::new()
                .intangible()
                .anchored(after)
                .at(Location::new().xs(column(), top(anchor().bottom() + 110.px()).height(10.px()))),
        );
        let heading = grove.branch(
            root,
            Text::new("visitors' wall")
                .font(face)
                .font_size(theme::heading())
                .color(Palette::Ink)
                .intangible()
                .opacity(0.0)
                .at(Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(content()))),
        );
        let blurb = grove.branch(
            root,
            Text::new("grown in the studio today, newest first")
                .font_size(lichen::caption())
                .color(Palette::Muted)
                .intangible()
                .opacity(0.0)
                .anchored(heading)
                .at(Location::new().xs(left(0.px()).right(100.pct()), top(anchor().bottom() + 10.px()).height(content()))),
        );
        let grid = grove.branch(
            root,
            Stem::new()
                .intangible()
                .anchored(blurb)
                .at(Location::new().xs(left(0.px()).right(100.pct()), top(anchor().bottom() + 26.px()).height(1.px()))),
        );
        let mut wall = Self {
            root,
            grid,
            cards: Vec::new(),
            tail: grid,
            next_handle: 413,
            head: vec![
                Arrival::words(heading, Some(face), theme::heading(), 7.0),
                Arrival::words(blurb, None, lichen::caption(), 3.0),
            ],
            shown: false,
        };
        for visitor in VISITORS.iter() {
            let handle = format!("#{:04} · {}", visitor.handle, visitor.ago);
            let painter = Painter::new(
                visitor.style,
                Params::of(visitor.style, Seed::new(visitor.seed)),
                &theme::ramp_for(visitor.prompt),
                PICTURE,
                PICTURE,
                0.0,
            );
            let slot = wall.cards.len();
            let card = wall.card(grove, slot, &handle, visitor.prompt, None);
            grove.opacity(card.root, 0.0);
            wall.cards.push(Card {
                painter: Some(painter),
                ..card
            });
        }
        // The last row's foot, which the colophon stands below.
        wall.tail = grove.branch(wall.grid, Stem::new().intangible().at(foot(VISITORS.len())));
        wall
    }

    pub fn tail(&self) -> Leaf {
        self.tail
    }

    /// Hangs a piece at the head of the wall, moving every other along a place.
    pub fn hang(&mut self, grove: &mut Grove, prompt: &str, picture: Vec<u8>, emerge: &mut Emerge) {
        let handle = format!("#{:04} · you · now", self.next_handle);
        self.next_handle += 1;
        // Everything moves along a place, the oldest coming down if the wall is full.
        if self.cards.len() >= HOLDS
            && let Some(last) = self.cards.pop()
        {
            grove.prune(last.root);
        }
        for (n, card) in self.cards.iter().enumerate() {
            grove.animate(
                card.root,
                Motion::Location(slot(n + 1)),
                Timing::ms(520).after(n as u64 * 30).ease(Ease::Emphasis),
            );
        }
        let card = self.card(grove, 0, &handle, prompt, Some(picture));
        let root = card.root;
        self.cards.insert(0, card);
        emerge.now(grove, root.into());
        grove.at(self.tail, foot(self.cards.len()));
    }

    /// Brings the wall in once it is in view, and paints what is waiting, one card at a time, once
    /// it is near.
    pub fn frame(&mut self, grove: &mut Grove, emerge: &mut Emerge) {
        let Some(Sap::Section(drawn)) = grove.tap(self.root, Vein::Drawn) else {
            return;
        };
        let view = grove.viewport().height;
        if !self.shown && drawn.top() < view * 0.86 {
            self.shown = true;
            emerge.bring(grove, &self.head, 0, 110);
            let cards: Vec<Arrival> = self.cards.iter().map(|card| Arrival::from(card.root)).collect();
            emerge.bring(grove, &cards, 260, 90);
        }
        if drawn.top() > view * 1.6 {
            return;
        }
        if let Some(card) = self.cards.iter_mut().find(|card| card.painter.is_some()) {
            grove.again();
            if let Some(painter) = &mut card.painter
                && painter.step()
                && let Some(painter) = card.painter.take()
            {
                grove.load(card.plate, painter.into_pixels(), Area::new(PICTURE as f32, PICTURE as f32));
            }
        }
    }

    fn card(&mut self, grove: &mut Grove, at: usize, handle: &str, prompt: &str, picture: Option<Vec<u8>>) -> Card {
        let root = grove.branch(self.grid, Stem::new().intangible().at(slot(at)));
        let plate = match picture {
            Some(picture) => grove.pixels(picture, Area::new(256.0, 256.0)),
            None => {
                // Dark until it is painted: a plate waiting.
                let plate = grove.plate();
                grove.load(plate, blank((PICTURE, PICTURE)), area((PICTURE, PICTURE)));
                plate
            }
        };
        grove.branch(
            root,
            Image::new(plate)
                .fit(Fit::Crop)
                .intangible()
                .at(Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(aspect(1.0)))),
        );
        let says = grove.branch(
            root,
            Text::new(format!("“{}”", clip(prompt, SAYS)))
                .font_size(lichen::caption())
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(100.pct() - (WORDS - 10.0).px()).height(content()),
                )),
        );
        grove.branch(
            root,
            Text::new(handle)
                .font_size(lichen::caption())
                .color(Palette::Muted)
                .intangible()
                .anchored(says)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + 4.px()).height(content()),
                )),
        );
        Card {
            root,
            plate,
            painter: None,
        }
    }
}

/// How wide a card is when `across` stand in a row: its share of the row, less the gaps.
fn share(across: f32) -> foliage::Length {
    (100.pct() - ((across - 1.0) * GAP).px()) * (1.0 / across)
}

/// How far down row `row` starts, when `across` stand in a row.
fn down(row: f32, across: f32) -> foliage::Length {
    (trunk().width() - ((across - 1.0) * GAP).px()) * (row / across) + (row * (WORDS + GAP)).px()
}

/// Where the card in place `n` stands, at each width of window.
fn slot(n: usize) -> Location {
    let at = |across: f32| {
        let (column, row) = ((n as f32) % across, (n as f32 / across).floor());
        (
            left(share(across) * column + (column * GAP).px()).width(share(across)),
            top(down(row, across)).height(aspect(1.0) + WORDS.px()),
        )
    };
    let ((xl, xt), (ml, mt), (ll, lt)) = (at(ACROSS[0]), at(ACROSS[1]), at(ACROSS[2]));
    Location::new().xs(xl, xt).md(ml, mt).lg(ll, lt)
}

/// The foot of a wall holding `cards`, at each width of window.
fn foot(cards: usize) -> Location {
    let at = |across: f32| {
        let rows = (cards as f32 / across).ceil();
        (left(0.px()).width(1.px()), top(down(rows, across)).height(1.px()))
    };
    let ((xl, xt), (ml, mt), (ll, lt)) = (at(ACROSS[0]), at(ACROSS[1]), at(ACROSS[2]));
    Location::new().xs(xl, xt).md(ml, mt).lg(ll, lt)
}
