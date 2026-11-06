//! The viewing room: one work at a time, large, with its spec sheet -- the diffusion replayed, a
//! walk through the latent space toward its neighbour, four variations, and a print to reserve.
//!
//! Beside the picture on a wide window, under it on anything narrower: every part of the sheet is a
//! share of the column it stands in, so nothing in it runs past the window's edge. Every work it is
//! turned to is sampled anew -- the picture out of the dark, the sheet into focus.

use foliage::{
    Axes, Boxed, Corners, Ease, Elevation, Fit, Font, FontSize, Grove, Grow, HAIRLINE, Image, Key,
    Leaf, Location, Motion, Palette, Panel, Place, Plate, Pollen, Polygon, Rounding, Sap, Source,
    Stem, Text, Timing, Tween, Vein, anchor, aspect, center_x, center_y, content, left, right,
    top,
};
use lichen::{Chip, Press, measure};

use arboretum::genart::{Painter, Params, paint};
use arboretum::sprout::{Arrival, Courier, Emerge, Escaped, Stamp};

use crate::api::{self, Certificate};
use crate::develop::{Develop, area};
use crate::frame::PICTURE;
use crate::icons::Icons;
use crate::theme::{self, column};
use crate::works::{ROOMS, WORKS};

/// How large the walk's preview is painted: small, so it keeps up with a drag.
const PREVIEW: u32 = 96;
/// How large a variation's thumbnail is painted.
const THUMB: u32 = 72;
/// How long a replay takes, a work newly turned to, and a picture newly grown here.
const REPLAY_MS: u64 = 2200;
const ARRIVE_MS: u64 = 900;
const REGROW_MS: u64 = 900;
/// The room between two variations.
const BETWEEN: f32 = 10.0;
/// How tall a row of the spec sheet is.
const ROW: f32 = 26.0;
/// How wide the sheet's controls run at most, however wide the sheet.
const CONTROLS: f32 = 440.0;

/// The spec sheet's rows, by name.
const ROWS: [&str; 6] = ["sampler", "steps", "cfg", "seed", "edition", "price"];

/// What the room asks of the page when it closes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Left(pub usize);

struct Variation {
    root: Leaf,
    plate: Plate,
    painter: Option<Painter>,
}

pub struct Viewing {
    page: Leaf,
    backing: Leaf,
    body: Leaf,
    open: bool,
    going: Option<Tween>,
    escape: Escaped,
    work: usize,
    image: Leaf,
    plate: Plate,
    preview: Plate,
    develop: Option<Develop>,
    counter: Leaf,
    eyebrow: Leaf,
    title: Leaf,
    prompt: Leaf,
    values: Vec<Leaf>,
    readout: Leaf,
    close: Chip,
    prev: Chip,
    next: Chip,
    replay: Chip,
    reserve: Chip,
    track: Leaf,
    knob: Leaf,
    toward: Leaf,
    walked: f32,
    previewed: f32,
    dragging: bool,
    variations: Vec<Variation>,
    certificate: Leaf,
    stamp: Stamp,
    courier: Courier<Option<Certificate>>,
    asked: [u32; 12],
    keys: Vec<Leaf>,
    /// The parts of the sheet that are sampled anew with every work, and those that arrive once.
    sampled: Vec<Arrival>,
    controls: Vec<Leaf>,
}

impl Viewing {
    pub fn grow(grove: &mut Grove, window: Leaf, page: Leaf, light: Plate, face: Font, icons: &Icons) -> Self {
        let m = measure();
        let backing = grove.branch(
            window,
            Panel::new()
                .color(Palette::Surface)
                .interactive()
                .focus_scope()
                .elevate(Elevation::up(30))
                .visible(false)
                .opacity(0.0)
                .at(Location::new()),
        );
        // The same light on the room's wall as on the page's.
        grove.branch(
            backing,
            Image::new(light).fit(Fit::Stretch).intangible().at(Location::new()),
        );
        let body = grove.branch(
            backing,
            Stem::new()
                .scrolls(Axes::Vertical)
                .elevate(Elevation::up(1))
                .at(Location::new()),
        );
        let col = grove.branch(
            body,
            Stem::new()
                .intangible()
                .font_size(lichen::caption())
                .at(Location::new().xs(column(), top(0.px()).height(10.px()))),
        );
        let bar = |n: f32| Location::new().xs(left((n * (m.height + m.gap)).px()).width(Chip::square()), top(20.px()).height(m.height.px()));
        let prev = Chip::mark(grove, col, bar(0.0), Elevation::up(1), icons.arrow_left);
        let next = Chip::mark(grove, col, bar(1.0), Elevation::up(1), icons.arrow_right);
        let counter = grove.branch(
            col,
            Text::new("01 / 12")
                .font_size(lichen::caption())
                .color(Palette::Muted)
                .intangible()
                .at(Location::new().xs(
                    left((2.0 * (m.height + m.gap) + 8.0).px()).width(content()),
                    center_y((20.0 + m.height / 2.0).px()).height(1.letters()),
                )),
        );
        let close = Chip::mark(
            grove,
            col,
            Location::new().xs(right(100.pct()).width(Chip::square()), top(20.px()).height(m.height.px())),
            Elevation::up(1),
            icons.x,
        );
        // The picture, hung bare, its reading on its corner.
        let plate = grove.plate();
        let preview = grove.plate();
        let image = grove.branch(
            col,
            Image::new(plate)
                .fit(Fit::Crop)
                .intangible()
                .at(Location::new()
                    .xs(left(0.px()).right(100.pct()).at_most(560.px()), top(78.px()).height(aspect(1.0)))
                    .lg(left(0.px()).width(54.pct()), top(78.px()).height(aspect(1.0)))),
        );
        let readout = grove.branch(
            image,
            Text::new("")
                .font_size(lichen::caption())
                .color(Palette::Accent)
                .intangible()
                .elevate(Elevation::up(2))
                .at(Location::new().xs(
                    left(10.px()).width(content()),
                    top(100.pct() - 10.px() - 1.letters()).height(1.letters()),
                )),
        );
        grove.branch(
            image,
            Panel::new()
                .color(Palette::Surface)
                .intangible()
                .opacity(0.84)
                .elevate(Elevation::up(1))
                .anchored(readout)
                .at(Location::new().xs(
                    left(anchor().left() - 6.px()).right(anchor().right() + 6.px()),
                    top(anchor().top() - 4.px()).bottom(anchor().bottom() + 4.px()),
                )),
        );
        // The sheet: beside the picture on a wide window, under it on a narrower one.
        let sheet = grove.branch(
            col,
            Stem::new()
                .intangible()
                .anchored(image)
                .font_size(lichen::caption())
                .at(Location::new()
                    .xs(left(0.px()).right(100.pct()).at_most(560.px()), top(anchor().bottom() + 28.px()).height(10.px()))
                    .lg(left(58.pct()).right(100.pct()), top(anchor().top()).height(10.px()))),
        );
        let eyebrow = grove.branch(
            sheet,
            Text::new("no. 01")
                .font_size(lichen::caption())
                .color(Palette::hue(0))
                .intangible()
                .at(Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(content()))),
        );
        let title = grove.branch(
            sheet,
            Text::new("title")
                .font(face)
                .font_size(FontSize::new().xs(22).md(28))
                .color(Palette::Ink)
                .intangible()
                .anchored(eyebrow)
                .at(Location::new().xs(left(0.px()).right(100.pct()), top(anchor().bottom() + 10.px()).height(content()))),
        );
        let prompt = grove.branch(
            sheet,
            Text::new("prompt")
                .font_size(theme::body())
                .color(Palette::Ink)
                .intangible()
                .anchored(title)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + 14.px()).height(content()),
                )),
        );
        let table = grove.branch(
            sheet,
            Stem::new()
                .intangible()
                .anchored(prompt)
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()).at_most(CONTROLS.px()),
                    top(anchor().bottom() + 22.px()).height((ROWS.len() as f32 * ROW).px()),
                )),
        );
        let values = ROWS
            .iter()
            .enumerate()
            .map(|(n, name)| {
                let y = n as f32 * ROW;
                grove.branch(
                    table,
                    Panel::new()
                        .color(Palette::Raised)
                        .intangible()
                        .at(Location::new().xs(left(0.px()).right(100.pct()), top((y + ROW - 1.0).px()).height(HAIRLINE.px()))),
                );
                grove.branch(
                    table,
                    Text::new(*name)
                        .color(Palette::Muted)
                        .intangible()
                        .at(Location::new().xs(left(0.px()).width(content()), top((y + 5.0).px()).height(1.letters()))),
                );
                grove.branch(
                    table,
                    Text::new("")
                        .color(Palette::Ink)
                        .intangible()
                        .at(Location::new().xs(left(10.letters()).right(100.pct()), top((y + 5.0).px()).height(1.letters()))),
                )
            })
            .collect();
        // The diffusion, replayed on demand.
        let replay_row = grove.branch(
            sheet,
            Stem::new()
                .intangible()
                .anchored(table)
                .font_size(lichen::caption())
                .at(Location::new().xs(left(0.px()).right(100.pct()), top(anchor().bottom() + 24.px()).height(m.height.px()))),
        );
        let replay = Chip::grow(
            grove,
            replay_row,
            Location::new().xs(left(0.px()).width(Chip::width("replay diffusion")), top(0.px()).height(100.pct())),
            Elevation::up(1),
            icons.refresh_cw,
            "replay diffusion",
        );
        // The walk toward the neighbour: a name over a rail.
        let walk = grove.branch(
            sheet,
            Stem::new()
                .intangible()
                .anchored(replay_row)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()).at_most(CONTROLS.px()),
                    top(anchor().bottom() + 26.px()).height(54.px()),
                )),
        );
        let toward = grove.branch(
            walk,
            Text::new("latent walk")
                .color(Palette::Muted)
                .intangible()
                .at(Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(1.letters()))),
        );
        let rail = grove.branch(
            walk,
            Stem::new()
                .intangible()
                .at(Location::new().xs(
                    left(9.px()).right(100.pct() - 9.px()),
                    top(26.px()).height(24.px()),
                )),
        );
        let track = grove.branch(
            rail,
            Panel::new()
                .color(Palette::Signal)
                .rounding(Corners::all(Rounding::Full))
                .intangible()
                .at(Location::new().xs(left(0.px()).right(100.pct()), center_y(50.pct()).height(3.px()))),
        );
        let knob = grove.branch(
            rail,
            Polygon::circle()
                .color(Palette::Accent)
                .interactive()
                .drags(Axes::Horizontal)
                .elevate(Elevation::up(2))
                .at(knob_at(0.0)),
        );
        // Four variations, a seed apart, each a quarter of the row.
        let thumbs = grove.branch(
            sheet,
            Stem::new()
                .intangible()
                .anchored(walk)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()).at_most(CONTROLS.px()),
                    top(anchor().bottom() + 22.px()).height(aspect(4.0) - (BETWEEN * 3.0 / 4.0).px() + 26.px()),
                )),
        );
        grove.branch(
            thumbs,
            Text::new("variations · a seed apart")
                .color(Palette::Muted)
                .intangible()
                .at(Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(1.letters()))),
        );
        let quarter = (100.pct() - (3.0 * BETWEEN).px()) * 0.25;
        let variations = (0..4)
            .map(|n| {
                let root = grove.branch(
                    thumbs,
                    Panel::new()
                        .color(Palette::Raised)
                        .interactive()
                        .at(Location::new().xs(
                            left(quarter.clone() * n as f32 + (n as f32 * BETWEEN).px()).width(quarter.clone()),
                            top(26.px()).height(aspect(1.0)),
                        )),
                );
                let plate = grove.plate();
                grove.branch(
                    root,
                    Image::new(plate)
                        .fit(Fit::Crop)
                        .intangible()
                        .elevate(Elevation::up(1))
                        .at(Location::new()),
                );
                Variation {
                    root,
                    plate,
                    painter: None,
                }
            })
            .collect();
        // A print, reserved through the print desk: the press, what the desk says, and its stamp.
        let print = grove.branch(
            sheet,
            Stem::new()
                .intangible()
                .anchored(thumbs)
                .font_size(lichen::caption())
                .at(Location::new().xs(left(0.px()).right(100.pct()), top(anchor().bottom() + 28.px()).height(m.height.px()))),
        );
        let mut reserve = Chip::grow(
            grove,
            print,
            Location::new().xs(left(0.px()).width(Chip::width("reserve a print · €1800")), top(0.px()).height(100.pct())),
            Elevation::up(1),
            icons.bookmark,
            "reserve a print",
        );
        reserve.arm(grove, Press::Armed);
        let certificate = grove.branch(
            sheet,
            Text::new("")
                .color(Palette::Muted)
                .intangible()
                .anchored(print)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()).at_most(CONTROLS.px()),
                    top(anchor().bottom() + 12.px()).height(content()),
                )),
        );
        let seal = grove.branch(
            sheet,
            Stem::new()
                .intangible()
                .anchored(certificate)
                .at(Location::new().xs(left(4.px()).width(118.px()), top(anchor().bottom() + 16.px()).height(38.px()))),
        );
        let stamp = Stamp::grow(
            grove,
            seal,
            Location::new(),
            "reserved",
            FontSize::new().xs(15),
            Some(face),
            Palette::Accent,
        );
        // A margin under everything, so the last of the sheet is not against the window's edge.
        grove.branch(
            sheet,
            Stem::new()
                .intangible()
                .anchored(seal)
                .at(Location::new().xs(left(0.px()).width(1.px()), top(anchor().bottom()).height(72.px()))),
        );
        let keys = vec![backing, close.leaf(), prev.leaf(), next.leaf(), replay.leaf(), reserve.leaf(), knob];
        Self {
            page,
            backing,
            body,
            open: false,
            going: None,
            escape: Escaped::new(),
            work: 0,
            image,
            plate,
            preview,
            develop: None,
            counter,
            eyebrow,
            title,
            prompt,
            values,
            readout,
            close,
            prev,
            next,
            replay,
            reserve,
            track,
            knob,
            toward,
            walked: 0.0,
            previewed: -1.0,
            dragging: false,
            variations,
            certificate,
            stamp,
            courier: Courier::new(0x000f_1ce5),
            asked: [0; 12],
            keys,
            sampled: vec![
                Arrival::words(eyebrow, None, lichen::caption(), 3.0),
                Arrival::words(title, Some(face), FontSize::new().xs(22).md(28), 6.0),
                Arrival::words(prompt, None, theme::body(), 4.0),
                table.into(),
            ],
            controls: vec![replay_row, walk, thumbs, print, certificate],
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Opens the room on `work`, starting from `picture` where the hall already developed it.
    pub fn open(&mut self, grove: &mut Grove, work: usize, picture: Option<&[u8]>, emerge: &mut Emerge) {
        if !self.open {
            self.open = true;
            if let Some(going) = self.going.take() {
                grove.stop(going);
            }
            grove.disable(self.page);
            grove.visible(self.backing, true);
            grove.animate(self.backing, Motion::Opacity(1.0), Timing::ms(320).ease(Ease::Decelerate));
            grove.scroll(self.body, foliage::ScrollTo::start());
            grove.focus(self.close.leaf());
            self.escape.forget();
            emerge.hide(grove, &self.controls);
            let controls: Vec<Arrival> = self.controls.iter().map(|&leaf| Arrival::from(leaf)).collect();
            emerge.bring(grove, &controls, 380, 90);
        }
        self.show(grove, work, picture, emerge);
    }

    /// Puts `work` on the wall of the room.
    fn show(&mut self, grove: &mut Grove, work: usize, picture: Option<&[u8]>, emerge: &mut Emerge) {
        self.work = work;
        let w = &WORKS[work];
        let room = &ROOMS[w.room];
        grove.text(self.counter, format!("{:02} / {:02}", w.no, WORKS.len()));
        grove.text(self.eyebrow, format!("no. {:02} · room {} · {} · {}", w.no, room.numeral, room.name, w.year));
        grove.color(self.eyebrow, Palette::hue(w.room));
        grove.text(self.title, w.title);
        grove.text(self.prompt, format!("“{}”", w.prompt));
        let values = [
            w.sampler.to_string(),
            w.steps.to_string(),
            format!("{:.1}", w.cfg),
            w.seed_label(),
            format!("{} of {} sold", w.edition.0 + self.asked[work], w.edition.1),
            format!("€{}", w.price),
        ];
        for (leaf, value) in self.values.iter().zip(values) {
            grove.text(*leaf, value);
        }
        let neighbour = &WORKS[(work + 1) % WORKS.len()];
        grove.text(self.toward, format!("latent walk → no. {:02}", neighbour.no));
        grove.text(self.reserve.label(), format!("reserve a print · €{}", w.price));
        let available = api::available(w, self.asked[work]);
        self.reserve.arm(grove, if available { Press::Armed } else { Press::Inert });
        let (taken, of) = w.edition;
        grove.text(
            self.certificate,
            match available {
                true => format!(
                    "holds print {}/{of} of this edition for you, signed and numbered, with a certificate of provenance · nothing is charged",
                    taken + self.asked[work] + 1
                ),
                false => "this edition is sold out".to_string(),
            },
        );
        grove.color(self.certificate, Palette::Muted);
        self.stamp.lift(grove);
        self.courier.cancel_all(grove);
        // The sheet into focus, and the picture out of the dark: what the hall developed, sampled
        // again quickly, or grown here.
        emerge.bring(grove, &self.sampled, 0, 80);
        self.develop = Some(match picture {
            Some(picture) => {
                let mut develop = Develop::ready(grove, self.plate, picture.to_vec(), (PICTURE, PICTURE), w.steps, w.seed());
                develop.replay(grove, ARRIVE_MS);
                develop
            }
            None => Develop::new(grove, self.plate, painter(work, 0.0, None), w.steps, REGROW_MS * 2, w.seed()),
        });
        grove.depict(self.image, self.plate);
        self.walked = 0.0;
        self.previewed = -1.0;
        grove.at(self.knob, knob_at(0.0));
        for (n, variation) in self.variations.iter_mut().enumerate() {
            variation.painter = Some(Painter::new(
                w.style,
                Params::of(w.style, w.seed().fork(n as u64 + 1)),
                &w.ramp(),
                THUMB,
                THUMB,
                0.0,
            ));
        }
        self.readout(grove);
    }

    pub fn close(&mut self, grove: &mut Grove) {
        if !self.open {
            return;
        }
        self.open = false;
        grove.enable(self.page);
        grove.animate(self.backing, Motion::Opacity(0.0), Timing::ms(240).ease(Ease::Accelerate));
        self.going = Some(grove.timer(Timing::ms(240)));
        self.courier.cancel_all(grove);
        if grove.focused().is_some() {
            grove.unfocus();
        }
    }

    /// Carries the room. Hands back the work it was showing, the frame it closes.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen, emerge: &mut Emerge) -> Option<Left> {
        if let Some(going) = self.going
            && pollen.finished(going)
        {
            self.going = None;
            grove.visible(self.backing, false);
        }
        if !self.open {
            return None;
        }
        // Escape: focus gone from somewhere to nowhere while the room is up.
        let escaped = self.escape.frame(grove);
        if escaped || self.close.pressed(pollen) {
            self.close(grove);
            return Some(Left(self.work));
        }
        let mut step = 0i32;
        for &leaf in &self.keys {
            for key in pollen.keys(leaf) {
                match key.key {
                    Key::Left => step -= 1,
                    Key::Right => step += 1,
                    _ => {}
                }
            }
        }
        if self.prev.pressed(pollen) {
            step -= 1;
        }
        if self.next.pressed(pollen) {
            step += 1;
        }
        if step != 0 {
            let to = (self.work as i32 + step).rem_euclid(WORKS.len() as i32) as usize;
            self.show(grove, to, None, emerge);
            return None;
        }
        // The picture on the wall.
        if let Some(develop) = &mut self.develop
            && develop.frame(grove).is_some()
        {
            self.readout(grove);
        }
        if self.replay.pressed(pollen)
            && let Some(develop) = &mut self.develop
        {
            develop.replay(grove, REPLAY_MS);
            self.readout(grove);
        }
        self.walk(grove, pollen);
        // The variations, painted a little each frame, one at a time.
        if let Some(variation) = self.variations.iter_mut().find(|v| v.painter.is_some()) {
            grove.again();
            if let Some(painter) = &mut variation.painter
                && painter.step()
                && let Some(painter) = variation.painter.take()
            {
                grove.load(variation.plate, painter.into_pixels(), area((THUMB, THUMB)));
            }
        }
        if let Some(n) = self.variations.iter().position(|v| pollen.activated(v.root)) {
            let w = &WORKS[self.work];
            self.develop = Some(Develop::new(
                grove,
                self.plate,
                painter(self.work, 0.0, Some(n as u64 + 1)),
                w.steps,
                REGROW_MS,
                w.seed().fork(n as u64 + 1),
            ));
            grove.depict(self.image, self.plate);
            self.readout(grove);
        }
        // The print desk.
        if self.reserve.pressed(pollen) {
            let w = &WORKS[self.work];
            let answer = api::available(w, self.asked[self.work]).then(|| api::reserve(w, self.asked[self.work]));
            self.courier.send(grove, 1500, answer);
            self.reserve.arm(grove, Press::Inert);
            grove.text(self.certificate, "contacting the print desk…");
        }
        for (_, answer) in self.courier.frame(pollen) {
            let w = &WORKS[self.work];
            match answer {
                Some(certificate) => {
                    self.asked[self.work] += 1;
                    grove.text(
                        self.values[4],
                        format!("{} of {} sold", w.edition.0 + self.asked[self.work], w.edition.1),
                    );
                    grove.text(
                        self.certificate,
                        format!("print {} of no. {:02} · provenance {}", certificate.print, w.no, certificate.hash),
                    );
                    grove.color(self.certificate, Palette::Ink);
                    emerge.now(grove, Arrival::words(self.certificate, None, lichen::caption(), 3.0));
                    self.stamp.press(grove, 120);
                    let more = api::available(w, self.asked[self.work]);
                    self.reserve.arm(grove, if more { Press::Rest } else { Press::Inert });
                }
                None => {
                    grove.text(self.certificate, "this edition is sold out");
                    grove.color(self.certificate, Palette::Danger);
                }
            }
        }
        None
    }

    /// The knob along the walk: a preview while it is dragged, and the picture regrown where it is
    /// let go.
    fn walk(&mut self, grove: &mut Grove, pollen: &Pollen) {
        if let Some(drag) = pollen.dragged(self.knob)
            && let Some(Sap::Section(track)) = grove.tap(self.track, Vein::Drawn)
        {
            self.dragging = true;
            let t = ((drag.current.x - track.left()) / track.width().max(1.0)).clamp(0.0, 1.0);
            self.walked = t;
            grove.at(self.knob, knob_at(t));
            if (t - self.previewed).abs() > 0.02 {
                self.previewed = t;
                let (style, params, ramp) = walked(self.work, t);
                let pixels = paint(style, params, &ramp, PREVIEW, PREVIEW, 0.0);
                grove.load(self.preview, pixels, area((PREVIEW, PREVIEW)));
                grove.depict(self.image, self.preview);
            }
            grove.text(self.readout, format!("latent · t {t:.2}"));
        }
        if self.dragging && pollen.disengaged(self.knob) {
            self.dragging = false;
            let w = &WORKS[self.work];
            self.develop = Some(Develop::new(
                grove,
                self.plate,
                painter(self.work, self.walked, None),
                w.steps,
                REGROW_MS,
                w.seed(),
            ));
            grove.depict(self.image, self.plate);
        }
    }

    fn readout(&mut self, grove: &mut Grove) {
        let Some(develop) = &self.develop else {
            return;
        };
        let w = &WORKS[self.work];
        let words = match (develop.painting(), develop.done()) {
            (true, _) => format!("sampling · {:.0}%", develop.progress() * 100.0),
            (false, true) => format!("step {}/{} · done", w.steps, w.steps),
            (false, false) => format!("step {}/{}", develop.model_step(), w.steps),
        };
        grove.text(self.readout, words);
    }
}

/// Where the knob stands at `t` along the walk.
fn knob_at(t: f32) -> Location {
    Location::new().xs(
        center_x((t * 100.0).pct()).width(18.px()),
        center_y(50.pct()).height(18.px()),
    )
}

/// What `work` is, `t` of the way toward its neighbour: its own style, the neighbour's numbers.
fn walked(work: usize, t: f32) -> (arboretum::genart::Style, Params, arboretum::genart::Ramp) {
    let w = &WORKS[work];
    let n = &WORKS[(work + 1) % WORKS.len()];
    let from = Params::of(w.style, w.seed());
    let to = Params::of(w.style, n.seed());
    (w.style, from.toward(&to, t), w.ramp().toward(&n.ramp(), t))
}

/// A painter for `work`, walked `t` toward its neighbour, or a variation `fork` seeds on.
fn painter(work: usize, t: f32, fork: Option<u64>) -> Painter {
    let w = &WORKS[work];
    match fork {
        Some(n) => Painter::new(w.style, Params::of(w.style, w.seed().fork(n)), &w.ramp(), PICTURE, PICTURE, 0.0),
        None => {
            let (style, params, ramp) = walked(work, t);
            Painter::new(style, params, &ramp, PICTURE, PICTURE, 0.0)
        }
    }
}
