//! A good, opened: its picture large, and its label printed out in full -- the price rolling to the
//! licence chosen, the formats it comes in, a way to try it, and the panel saying what is inside.
//!
//! An overlay under the bar, so the bag stays in reach and an added good can be seen going into it.
//! Beside the picture on a wide window, under it on a narrow one; every part a share of the column
//! it stands in. Escape closes it, as focus leaving it for nowhere.

use foliage::{
    Axes, Boxed, Ease, Elevation, Font, FontSize, Grove, Grow, Leaf, Location, Motion, Palette,
    Panel, Place, Pollen, Polygon, Sap, ScrollTo, Section, Source, Stem, Text, TextInput, Timing,
    Vein, anchor, aspect, center_x, center_y, content, left, right, top, trunk,
};
use lichen::{Chip, Origin, Overlay, Pick, Ping, Press, Roll, Say, Switch, Voice, dress, measure};

use arboretum::sprout::{Print, Printed, Progress};

use crate::api;
use crate::art::{Art, Scale};
use crate::cart::Licence;
use crate::catalog::{Aisle, Face, GOODS, Look, exact, money, price, related, seconds, weight};
use crate::icons::Icons;
use crate::label;
use crate::theme::{self, BAR_NARROW, BAR_WIDE, Faces, HAIR, HEAVY, column};

/// How wide the sheet's presses run at most, however wide the sheet.
const CONTROLS: f32 = 440.0;
/// How far a shader can be sped up or slowed down.
const SLOWEST: f32 = 0.2;
const FASTEST: f32 = 3.0;

/// What the detail asks of the shop.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Asked {
    /// A seat of a good, under a licence, as a format, into the bag -- from where the press was.
    Add {
        good: usize,
        licence: Licence,
        format: usize,
        from: Section,
    },
    /// It closed.
    Closed,
}

/// The ways to try a good, one for each aisle.
struct Previews {
    holders: [Leaf; 4],
    swatches: Vec<Leaf>,
    knob: Leaf,
    track: Leaf,
    speed_says: Leaf,
    freeze: Switch,
    inputs: [Leaf; 4],
    play: Chip,
    clock: Leaf,
    progress: Progress,
}

struct Related {
    root: Leaf,
    good: usize,
    art: Art,
}

pub struct Detail {
    overlay: Overlay,
    body: Leaf,
    good: usize,
    window: Leaf,
    art: Option<Art>,
    playhead: Leaf,
    kicker: Leaf,
    weighs: Leaf,
    title: Leaf,
    blurb: Leaf,
    price: Roll,
    per: Leaf,
    licence: Pick,
    formats: Vec<Pick>,
    previews: Previews,
    add_row: Leaf,
    add: Chip,
    ping: Ping,
    say: Say,
    values: Vec<Leaf>,
    shelf: Leaf,
    related: Vec<Related>,
    back: Chip,
    close: Chip,
    rules: Vec<Printed>,
    fixed: Vec<Printed>,
    /// How far the price is drawn left, so its first figure stands at the sheet's edge.
    flush: f32,
    speed: f32,
    frozen: bool,
    playing: bool,
    played: f32,
    faces: Faces,
    /// The marks the player swaps between: play, and pause.
    icons: (foliage::Field, foliage::Field),
}

impl Detail {
    pub fn grow(grove: &mut Grove, shop: Leaf, page: Leaf, faces: &Faces, icons: &Icons) -> Self {
        let m = measure();
        let mut overlay = Overlay::grow(
            grove,
            shop,
            page,
            Location::new()
                .xs(
                    left(0.px()).right(100.pct()),
                    top(BAR_NARROW.px()).bottom(100.pct()),
                )
                .lg(
                    left(0.px()).right(100.pct()),
                    top(BAR_WIDE.px()).bottom(100.pct()),
                ),
            Elevation::up(30),
        );
        let body = grove.branch(
            overlay.sheet(),
            Stem::new()
                .interactive()
                .scrolls(Axes::Vertical)
                .elevate(Elevation::up(1))
                .at(Location::new()),
        );
        // The body holds focus while it is up, so the keys that scroll it reach it.
        overlay.hold(body);
        let col = grove.branch(
            body,
            Stem::new()
                .intangible()
                .font_size(lichen::caption())
                .at(column(|| top(22.px()).height(10.px()))),
        );
        let back = Chip::grow(
            grove,
            col,
            Location::new().xs(
                left(0.px()).width(Chip::width("the aisles")),
                top(0.px()).height(m.height.px()),
            ),
            Elevation::up(1),
            icons.arrow_left,
            "the aisles",
        );
        let close = Chip::mark(
            grove,
            col,
            Location::new().xs(
                right(100.pct()).width(Chip::square()),
                top(0.px()).height(m.height.px()),
            ),
            Elevation::up(1),
            icons.x,
        );
        // The picture's window, framed as a label's is.
        let frame = grove.branch(
            col,
            Panel::new()
                .color(Palette::Raised)
                .outline(HAIR, Palette::Ink)
                .intangible()
                .at(Location::new()
                    .xs(
                        left(0.px()).right(100.pct()).at_most(560.px()),
                        top((m.height + 22.0).px()).height(aspect(1.0)),
                    )
                    .lg(
                        left(0.px()).width(50.pct()),
                        top((m.height + 22.0).px()).height(aspect(1.0)),
                    )),
        );
        let window = grove.branch(
            frame,
            Stem::new()
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    left(14.px()).right(100.pct() - 14.px()),
                    top(14.px()).bottom(100.pct() - 14.px()),
                )),
        );
        let playhead = grove.branch(
            window,
            Panel::new()
                .color(Palette::Accent)
                .intangible()
                .elevate(Elevation::up(6))
                .visible(false)
                .at(playhead_at(0.0)),
        );
        // The sheet: beside the picture on a wide window, under it on a narrower one.
        let sheet = grove.branch(
            col,
            Stem::new()
                .intangible()
                .anchored(frame)
                .font_size(lichen::caption())
                .at(Location::new()
                    .xs(
                        left(0.px()).right(100.pct()).at_most(560.px()),
                        top(anchor().bottom() + 28.px()).height(10.px()),
                    )
                    .lg(
                        left(55.pct()).right(100.pct()),
                        top(anchor().top() - 4.px()).height(10.px()),
                    )),
        );
        let kicker = grove.branch(
            sheet,
            Text::new("")
                .font_size(lichen::caption())
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).width(content()),
                    top(0.px()).height(1.letters()),
                )),
        );
        let weighs = grove.branch(
            sheet,
            Text::new("")
                .font_size(lichen::caption())
                .color(Palette::Muted)
                .intangible()
                .at(Location::new().xs(
                    right(100.pct()).width(content()),
                    top(0.px()).height(1.letters()),
                )),
        );
        let hairline = label::rule(grove, sheet, kicker, 10.0, HAIR);
        let title = label::words(
            grove,
            sheet,
            hairline.leaf(),
            14.0,
            "",
            Some(faces.roman),
            theme::title(),
            Palette::Ink,
        );
        let blurb = label::words(
            grove,
            sheet,
            title,
            10.0,
            "",
            None,
            theme::body(),
            Palette::Muted,
        );
        let heavy = label::rule(grove, sheet, blurb, 22.0, HEAVY);
        // The price, rolling to whatever the licence makes it.
        let price = Roll::grow(
            grove,
            sheet,
            Location::new().xs(
                left((-2).px()).width(8.letters()),
                top(anchor().bottom() + 18.px()).height(1.letters()),
            ),
            8,
            "€.",
            theme::figure(),
            Some(faces.italic),
            Palette::Ink,
        );
        grove.anchor(price.leaf(), heavy.leaf());
        let per = label::words(
            grove,
            sheet,
            price.leaf(),
            6.0,
            "",
            None,
            lichen::caption(),
            Palette::Muted,
        );
        let picks = grove.branch(
            sheet,
            Stem::new()
                .intangible()
                .anchored(per)
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()).at_most(CONTROLS.px()),
                    top(anchor().bottom() + 20.px()).height((2.0 * m.height + m.gap).px()),
                )),
        );
        let row = |n: f32| {
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top((n * (m.height + m.gap)).px()).height(m.height.px()),
            )
        };
        let licence = Pick::grow(grove, picks, row(0.0), 8.0, "licence", Licence::NAMES, 1);
        let formats: Vec<Pick> = Aisle::ALL
            .iter()
            .map(|aisle| {
                let pick = Pick::grow(grove, picks, row(1.0), 8.0, "format", aisle.formats(), 1);
                grove.visible(pick.leaf(), false);
                pick
            })
            .collect();
        let previews = previews(grove, sheet, picks, faces, icons);
        // The press to make, what answers it, and the tick when it lands.
        let add_row = grove.branch(
            sheet,
            Stem::new()
                .intangible()
                .anchored(previews.holders[0])
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()).at_most(CONTROLS.px()),
                    top(anchor().bottom() + 22.px()).height(m.height.px()),
                )),
        );
        let mut add = Chip::grow(
            grove,
            add_row,
            Location::new().xs(
                left(0.px()).width(Chip::width("add to bag")),
                top(0.px()).height(100.pct()),
            ),
            Elevation::up(1),
            icons.plus,
            "add to bag",
        );
        add.arm(grove, Press::Armed);
        let ping = Ping::bare(
            grove,
            add_row,
            Location::new().xs(
                left(Chip::width("add to bag") + 4.px()).width(Chip::square()),
                top(0.px()).height(100.pct()),
            ),
            icons.check,
        );
        let say = Say::grow(
            grove,
            sheet,
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top(anchor().bottom() + 10.px()).height(1.letters()),
            ),
            None,
        );
        grove.anchor(say.leaf(), add_row);
        // What is inside, as the side of a box says it.
        let heading = label::words(
            grove,
            sheet,
            say.leaf(),
            34.0,
            "Contents",
            Some(faces.roman),
            FontSize::new().xs(22).md(24),
            Palette::Ink,
        );
        let top_rule = label::rule(grove, sheet, heading, 8.0, HEAVY);
        let (panel, values, rows_printed) = label::rows(
            grove,
            sheet,
            top_rule.leaf(),
            0.0,
            &[
                "contents",
                "files",
                "formats",
                "net weight",
                "licence",
                "made by",
            ],
            104.0,
        );
        // The rest of the aisle, along the foot.
        let shelf = grove.branch(
            col,
            Stem::new()
                .intangible()
                .anchored(panel)
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + 64.px()).height(10.px()),
                )),
        );
        let also = grove.branch(
            shelf,
            Text::new("From the same aisle")
                .font(faces.roman)
                .font_size(FontSize::new().xs(22).md(24))
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).width(content()),
                    top(0.px()).height(1.letters()),
                )),
        );
        let shelf_rule = label::rule(grove, shelf, also, 8.0, HEAVY);
        let mut rules = vec![hairline, heavy, top_rule, shelf_rule];
        rules.extend(rows_printed);
        let fixed = vec![Print::words(grove, heading), Print::words(grove, also)];
        Self {
            overlay,
            body,
            good: 0,
            window,
            art: None,
            playhead,
            kicker,
            weighs,
            title,
            blurb,
            price,
            per,
            licence,
            formats,
            previews,
            add_row,
            add,
            ping,
            say,
            values,
            shelf,
            related: Vec::new(),
            back,
            close,
            rules,
            fixed,
            flush: 0.0,
            speed: 1.0,
            frozen: false,
            playing: false,
            played: 0.0,
            faces: *faces,
            icons: (icons.play, icons.pause),
        }
    }

    pub fn is_open(&self) -> bool {
        self.overlay.is_open()
    }

    /// Opens the detail on `good`, or turns it to `good` if it is open.
    pub fn open(&mut self, grove: &mut Grove, good: usize, print: &mut Print) {
        self.overlay.open(grove);
        self.show(grove, good, print);
    }

    /// Puts `good` on the sheet, printing it in.
    fn show(&mut self, grove: &mut Grove, good: usize, print: &mut Print) {
        self.good = good;
        let g = &GOODS[good];
        grove.scroll(self.body, ScrollTo::start());
        // The picture, grown afresh for this good, and fed through.
        if let Some(art) = self.art.take() {
            grove.prune(art.leaf());
        }
        let mut art = Art::grow(
            grove,
            self.window,
            Location::new(),
            good,
            Scale::Detail,
            &self.faces,
        );
        art.fit(grove, grove.viewport().width);
        let fed = Print::feed(
            grove,
            art.leaf(),
            Palette::Raised,
            Palette::Accent,
            foliage::Elevation::up(8),
        );
        self.art = Some(art);
        self.speed = 1.0;
        self.frozen = false;
        self.playing = false;
        self.played = 0.0;
        // The words, typed in; the rules, drawn across.
        print.unprint(grove, &self.rules);
        print.now(grove, fed);
        print.retype(
            grove,
            self.kicker,
            &format!("No. {:02} · {}", g.no, g.aisle.single()),
            120,
        );
        print.retype(grove, self.weighs, &format!("net {}", weight(g.bytes)), 160);
        print.retype(grove, self.title, g.name, 240);
        print.retype(grove, self.blurb, g.blurb, 420);
        print.run(grove, &self.rules, 140, 30);
        print.run(grove, &self.fixed, 600, 80);
        // The licence back to personal, and the formats the aisle comes in.
        self.licence.set(grove, 0, Origin::Empty);
        for (n, pick) in self.formats.iter_mut().enumerate() {
            let here = n == g.aisle.index();
            grove.visible(pick.leaf(), here);
            if here {
                pick.set(grove, 0, Origin::Empty);
            }
        }
        self.price.place(grove, &money(g.cents));
        self.reprice(grove, false);
        // The way to try it.
        for (n, &holder) in self.previews.holders.iter().enumerate() {
            grove.visible(holder, n == g.aisle.index());
        }
        grove.anchor(self.add_row, self.previews.holders[g.aisle.index()]);
        match g.look {
            Look::Palette(swatches) => {
                for (n, &swatch) in self.previews.swatches.iter().enumerate() {
                    grove.color(
                        swatch,
                        theme::oklch(swatches[(n + 1).min(swatches.len() - 1)]),
                    );
                }
            }
            Look::Specimen { face, .. } => {
                let k = face_index(face);
                for (n, &input) in self.previews.inputs.iter().enumerate() {
                    grove.visible(input, n == k);
                }
            }
            _ => {}
        }
        grove.at(self.previews.knob, knob_at(speed_t(1.0)));
        grove.text(self.previews.speed_says, "1.0×");
        self.previews.freeze.set(grove, false);
        grove.text(self.previews.clock, clock(0.0, seconds(g)));
        self.previews.progress.set(grove, 0.0, 0);
        self.previews.play.remark(grove, self.icons.0);
        grove.visible(self.playhead, false);
        grove.at(self.playhead, playhead_at(0.0));
        self.say.clear(grove);
        // What is inside.
        let values = [
            g.contents.to_string(),
            g.files.to_string(),
            api::formats_line(g.aisle),
            exact(g.bytes),
            Licence::Personal.terms().to_string(),
            g.credit.unwrap_or("fernwood studio").to_string(),
        ];
        for (leaf, value) in self.values.iter().zip(values) {
            grove.text(*leaf, value);
            grove.opacity(*leaf, 0.0);
            grove.animate(
                *leaf,
                Motion::Opacity(1.0),
                Timing::ms(300).after(700).ease(Ease::Decelerate),
            );
        }
        // The rest of the aisle.
        for related in self.related.drain(..) {
            grove.prune(related.root);
        }
        for (k, other) in related(good).into_iter().enumerate() {
            let shown = self.mini(grove, k, other);
            self.related.push(shown);
        }
    }

    /// Grows related good `good` in place `k` along the foot.
    fn mini(&mut self, grove: &mut Grove, k: usize, good: usize) -> Related {
        let g = &GOODS[good];
        let gap = 16.0;
        let share = (100.pct() - (2.0 * gap).px()) * (1.0 / 3.0);
        let root = grove.branch(
            self.shelf,
            Stem::new().interactive().at(Location::new().xs(
                left(share.clone() * k as f32 + (k as f32 * gap).px()).width(share),
                top(50.px()).height(aspect(1.0) + 74.px()),
            )),
        );
        let framed = grove.branch(
            root,
            Panel::new()
                .color(Palette::Raised)
                .outline(HAIR, Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(0.px()).height(aspect(1.0)),
                )),
        );
        let mut art = Art::grow(
            grove,
            framed,
            Location::new().xs(
                left(6.px()).right(100.pct() - 6.px()),
                top(6.px()).bottom(100.pct() - 6.px()),
            ),
            good,
            Scale::Mini,
            &self.faces,
        );
        grove.elevate(art.leaf(), Elevation::up(1));
        art.fit(grove, grove.viewport().width);
        let name = grove.branch(
            root,
            Text::new(g.name)
                .font(self.faces.roman)
                .font_size(FontSize::new().xs(15).md(17))
                .color(Palette::Ink)
                .intangible()
                .anchored(framed)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + 8.px()).height(content()),
                )),
        );
        grove.branch(
            root,
            Text::new(price(g.cents))
                .font(self.faces.italic)
                .font_size(FontSize::new().xs(15).md(17))
                .color(Palette::Muted)
                .intangible()
                .anchored(name)
                .at(Location::new().xs(
                    left(0.px()).width(content()),
                    top(anchor().bottom() + 2.px()).height(1.letters()),
                )),
        );
        Related { root, good, art }
    }

    /// Takes the detail down, putting its art to sleep. Safe to repeat.
    pub fn close(&mut self, grove: &mut Grove) {
        self.overlay.close(grove);
        if let Some(art) = &mut self.art {
            art.wake(grove, false);
        }
    }

    /// Says a seat went into the bag, which now counts `count`.
    pub fn added(&mut self, grove: &mut Grove, count: u32) {
        self.ping.fire(grove);
        let g = &GOODS[self.good];
        self.say.tell(
            grove,
            &format!(
                "{} is in the bag · {count} seat{} in all",
                g.name,
                if count == 1 { "" } else { "s" }
            ),
            Voice::Hint,
        );
    }

    /// Carries the detail for a frame.
    pub fn frame(
        &mut self,
        grove: &mut Grove,
        pollen: &Pollen,
        print: &mut Print,
        budget: &mut u32,
    ) -> Option<Asked> {
        let escaped = self.overlay.frame(grove, pollen).is_some();
        // Gone: the art it was showing is let go of.
        if !self.overlay.is_shown()
            && let Some(art) = self.art.take()
        {
            grove.prune(art.leaf());
        }
        if !escaped && !self.overlay.is_open() {
            return None;
        }
        if escaped || self.close.pressed(pollen) || self.back.pressed(pollen) {
            self.close(grove);
            return Some(Asked::Closed);
        }
        self.ping.frame(grove, pollen);
        self.price.frame(grove);
        self.flush(grove);
        let g = &GOODS[self.good];
        if self.licence.frame(grove, pollen) {
            self.reprice(grove, true);
        }
        for pick in &mut self.formats {
            pick.frame(grove, pollen);
        }
        // The picture: a shader runs, a sound breathes while it plays, everything else rests.
        let awake = match g.look {
            Look::Shader { .. } => true,
            Look::Sound { .. } => self.playing,
            _ => false,
        };
        if let Some(art) = &mut self.art {
            art.speed = if self.frozen { 0.0 } else { self.speed };
            art.wake(grove, awake);
            art.frame(grove, budget);
        }
        for related in &mut self.related {
            related.art.frame(grove, budget);
        }
        if let Some(n) = self.related.iter().position(|r| pollen.activated(r.root)) {
            let good = self.related[n].good;
            self.show(grove, good, print);
            return None;
        }
        self.try_out(grove, pollen);
        if self.add.pressed(pollen) {
            let from = match grove.tap(self.add.leaf(), Vein::Drawn) {
                Some(Sap::Section(section)) => section,
                _ => Section::default(),
            };
            let format = self.formats[g.aisle.index()].index();
            return Some(Asked::Add {
                good: self.good,
                licence: Licence::ALL[self.licence.index()],
                format,
                from,
            });
        }
        None
    }

    /// Draws the price left by its blank windows, so that it reads from the sheet's edge however
    /// many figures it has.
    fn flush(&mut self, grove: &mut Grove) {
        let Some(Sap::Section(drawn)) = grove.tap(self.price.leaf(), Vein::Placed) else {
            return;
        };
        let blanks = 8usize.saturating_sub(self.price.says().chars().count());
        let to = -(blanks as f32) * drawn.width() / 8.0;
        if (to - self.flush).abs() < 0.5 {
            return;
        }
        let first = self.flush == 0.0;
        self.flush = to;
        match first {
            true => grove.shift(self.price.leaf(), foliage::Position::new(to, 0.0)),
            false => {
                grove.animate(
                    self.price.leaf(),
                    Motion::Shift(foliage::Position::new(to, 0.0)),
                    Timing::ms(420).ease(Ease::Emphasis),
                );
            }
        }
    }

    /// The price, and the line under it, for the licence chosen -- rolled to, or put.
    fn reprice(&mut self, grove: &mut Grove, roll: bool) {
        let g = &GOODS[self.good];
        let licence = Licence::ALL[self.licence.index()];
        let cents = g.cents * licence.times();
        match roll {
            true => self.price.set(grove, &money(cents)),
            false => self.price.place(grove, &money(cents)),
        }
        let times = match licence.times() {
            1 => String::new(),
            n => format!(" · ×{n}"),
        };
        grove.text(
            self.per,
            format!("one seat, {} licence{times}", licence.name()),
        );
        if let Some(&value) = self.values.get(4) {
            grove.text(value, licence.terms());
        }
    }

    /// The ways to try a good: copy a swatch, run a shader faster or slower, type in a face, play
    /// a sound.
    fn try_out(&mut self, grove: &mut Grove, pollen: &Pollen) {
        let g = &GOODS[self.good];
        let p = &mut self.previews;
        match g.look {
            Look::Palette(swatches) => {
                if let Some(n) = p.swatches.iter().position(|&s| pollen.activated(s)) {
                    let hex = api::hex(swatches[(n + 1).min(swatches.len() - 1)]);
                    grove.copy(hex.clone());
                    self.ping.fire(grove);
                    self.say.tell(
                        grove,
                        &format!("copied {hex} · paste it anywhere"),
                        Voice::Hint,
                    );
                }
            }
            Look::Shader { .. } => {
                if let Some(drag) = pollen.dragged(p.knob)
                    && let Some(Sap::Section(track)) = grove.tap(p.track, Vein::Drawn)
                {
                    let t =
                        ((drag.current.x - track.left()) / track.width().max(1.0)).clamp(0.0, 1.0);
                    self.speed = SLOWEST + (FASTEST - SLOWEST) * t * t;
                    grove.at(p.knob, knob_at(t));
                    grove.text(p.speed_says, format!("{:.1}×", self.speed));
                }
                if p.freeze.frame(grove, pollen) {
                    self.frozen = p.freeze.on();
                }
            }
            Look::Sound { .. } => {
                let length = seconds(g) as f32;
                if p.play.pressed(pollen) {
                    self.playing = !self.playing;
                    p.play.remark(
                        grove,
                        if self.playing {
                            self.icons.1
                        } else {
                            self.icons.0
                        },
                    );
                    grove.visible(self.playhead, true);
                }
                if self.playing {
                    grove.again();
                    self.played += grove.frame_time().as_secs_f32();
                    if self.played >= length {
                        self.played = 0.0;
                    }
                    let t = self.played / length;
                    grove.at(self.playhead, playhead_at(t));
                    p.progress.set(grove, t, 0);
                    grove.text(p.clock, clock(self.played, length as u32));
                }
            }
            Look::Specimen { .. } => {}
        }
    }
}

/// Which of the four type inputs is set in `face`.
fn face_index(face: Face) -> usize {
    match face {
        Face::Xanh => 0,
        Face::Major => 1,
        Face::Plex => 2,
        Face::JetBrains => 3,
    }
}

/// Grows the four ways to try a good, each in a holder under `after`, all hidden.
fn previews(grove: &mut Grove, sheet: Leaf, after: Leaf, faces: &Faces, icons: &Icons) -> Previews {
    let m = measure();
    let holder = |grove: &mut Grove, height: f32| {
        grove.branch(
            sheet,
            Stem::new()
                .intangible()
                .anchored(after)
                .visible(false)
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()).at_most(CONTROLS.px()),
                    top(anchor().bottom() + m.gap.px()).height(height.px()),
                )),
        )
    };
    let field_row = |grove: &mut Grove, under: Leaf, y: f32, h: f32| {
        grove.branch(
            under,
            Stem::new()
                .intangible()
                .elevate(Elevation::up(1))
                .font_size(lichen::caption())
                .at(Location::new().xs(left(0.px()).right(100.pct()), top(y.px()).height(h.px()))),
        )
    };
    // Palettes: a swatch to a badge, pressed to copy.
    let palette = holder(grove, m.height);
    let row = field_row(grove, palette, 0.0, m.height);
    let cell = 9.0;
    dress(grove, row, cell, "copy hex");
    let swatches = (0..5)
        .map(|n| {
            let share = 20.0;
            grove.branch(
                row,
                Panel::new()
                    .color(Palette::Muted)
                    .interactive()
                    .rounding(lichen::corner())
                    .elevate(Elevation::up(3))
                    .at(Location::new().xs(
                        left(
                            trunk().letters(cell)
                                + (1.0 + m.gap).px()
                                + (100.pct() - trunk().letters(cell) - (2.0 * m.gap + 1.0).px())
                                    * (n as f32 * share / 100.0),
                        )
                        .width(
                            (100.pct() - trunk().letters(cell) - (2.0 * m.gap + 1.0).px())
                                * (share / 100.0)
                                - 4.px(),
                        ),
                        top(m.notch.px()).height(m.badge().px()),
                    )),
            )
        })
        .collect();
    // Shaders: how fast, and a hold.
    let shader = holder(grove, 2.0 * m.height + m.gap);
    let row = field_row(grove, shader, 0.0, m.height);
    dress(grove, row, 7.0, "speed");
    let rail = grove.branch(
        row,
        Stem::new()
            .intangible()
            .elevate(Elevation::up(3))
            .at(Location::new().xs(
                left(trunk().letters(7.0) + (1.0 + m.gap + 10.0).px())
                    .right(100.pct() - trunk().letters(6.0) - 10.px()),
                top(0.px()).bottom(100.pct()),
            )),
    );
    let track = grove.branch(
        rail,
        Panel::new()
            .color(Palette::Muted)
            .rounding(foliage::Corners::all(foliage::Rounding::Full))
            .intangible()
            .at(Location::new().xs(
                left(0.px()).right(100.pct()),
                center_y(50.pct()).height(2.px()),
            )),
    );
    let knob = grove.branch(
        rail,
        Polygon::circle()
            .color(Palette::Accent)
            .interactive()
            .drags(Axes::Horizontal)
            .elevate(Elevation::up(2))
            .at(knob_at(speed_t(1.0))),
    );
    let speed_says = grove.branch(
        row,
        Text::new("1.0×")
            .font_size(lichen::caption())
            .color(Palette::Ink)
            .intangible()
            .elevate(Elevation::up(3))
            .at(Location::new().xs(
                right(100.pct() - 10.px()).width(content()),
                center_y(50.pct()).height(1.letters()),
            )),
    );
    let freeze = Switch::grow(
        grove,
        shader,
        Location::new().xs(
            left(0.px()).width(Switch::width("freeze the frame")),
            top((m.height + m.gap).px()).height(m.height.px()),
        ),
        Elevation::up(1),
        "freeze the frame",
    );
    // Type: a line to type into, set in the face.
    let kind = holder(grove, 46.0);
    let row = field_row(grove, kind, 0.0, 46.0);
    dress(grove, row, 7.0, "try it");
    let fonts: [Option<Font>; 4] = [Some(faces.roman), Some(faces.major), Some(faces.plex), None];
    let inputs = fonts.map(|font| {
        let input = TextInput::new()
            .placeholder("type to set it")
            .color(Palette::Ink)
            .font_size(FontSize::new().xs(20).md(22))
            .elevate(Elevation::up(3))
            .visible(false)
            .at(Location::new().xs(
                left(trunk().letters(7.0) + (1.0 + m.gap).px()).right(100.pct() - m.gap.px()),
                top(0.px()).bottom(100.pct()),
            ));
        let input = match font {
            Some(font) => input.font(font),
            None => input,
        };
        grove.branch(row, input)
    });
    // Sound: play, and where it has got to.
    let sound = holder(grove, m.height);
    let play = Chip::mark(
        grove,
        sound,
        Location::new().xs(
            left(0.px()).width(Chip::square()),
            top(0.px()).height(m.height.px()),
        ),
        Elevation::up(1),
        icons.play,
    );
    let row = grove.branch(
        sound,
        Stem::new()
            .intangible()
            .elevate(Elevation::up(1))
            .font_size(lichen::caption())
            .at(Location::new().xs(
                left((m.height + m.gap).px()).right(100.pct()),
                top(0.px()).height(m.height.px()),
            )),
    );
    let (_, clock_leaf, _) = dress(grove, row, 12.0, "0:00 / 0:24");
    let progress = Progress::grow(
        grove,
        row,
        Location::new().xs(
            left(trunk().letters(12.0) + (1.0 + m.gap + 6.0).px()).right(100.pct() - 12.px()),
            center_y(50.pct()).height(3.px()),
        ),
        1,
        Palette::Muted,
        Palette::Accent,
    );
    grove.elevate(progress.leaf(), Elevation::up(3));
    Previews {
        holders: [palette, shader, kind, sound],
        swatches,
        knob,
        track,
        speed_says,
        freeze,
        inputs,
        play,
        clock: clock_leaf,
        progress,
    }
}

/// Where along the rail a speed puts the knob.
fn speed_t(speed: f32) -> f32 {
    ((speed - SLOWEST) / (FASTEST - SLOWEST))
        .clamp(0.0, 1.0)
        .sqrt()
}

fn knob_at(t: f32) -> Location {
    Location::new().xs(
        center_x((t * 100.0).pct()).width(16.px()),
        center_y(50.pct()).height(16.px()),
    )
}

/// Where the playhead stands, `t` of the way across the waveform.
fn playhead_at(t: f32) -> Location {
    let x = 9.0 + 82.0 * t.clamp(0.0, 1.0);
    Location::new().xs(
        center_x(x.pct()).width(2.px()),
        top(10.pct()).bottom(90.pct()),
    )
}

/// A clock face: `at` seconds of `of`.
fn clock(at: f32, of: u32) -> String {
    let at = at as u32;
    format!("{}:{:02} / {}:{:02}", at / 60, at % 60, of / 60, of % 60)
}
