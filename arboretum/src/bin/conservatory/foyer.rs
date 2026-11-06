//! The foyer: the exhibition's name, set large, and under it the thing the exhibition is about --
//! one work's whole sampling laid out in six steps, out of the dark to finished, left to right.
//!
//! Every few seconds the strip moves on to another work. The next is painted while the last is
//! still up, then the six steps turn over one after another, left to right, as the new work's
//! prompt is typed out under them.

use foliage::{
    Boxed, Color, Elevation, Fit, Font, FontSize, Grove, Grow, HAIRLINE, Image, Leaf, Location,
    Palette, Panel, Place, Plate, Pollen, Source, Stem, Text, Timing, Tween, anchor, aspect,
    content, left, right, top, trunk,
};
use lichen::{Chip, Dashes, Hover, Press, measure};

use arboretum::genart::{Denoise, Painter, Params};
use arboretum::sprout::{Arrival, Emerge, Typewriter};

use crate::develop::{PATCHES, area, blank};
use crate::icons::Icons;
use crate::theme::{self, column};
use crate::works::{ROOMS, WORKS};

/// How many steps the strip shows.
const STEPS: usize = 6;
/// The room between two steps.
const GAP: f32 = 12.0;
/// How large each step is painted.
const SIDE: u32 = 224;
/// How long apart two steps turn over, and how long a work stays up once all six have.
const TURN_MS: f32 = 130.0;
const HOLD_MS: u64 = 7600;
/// How fast a prompt is typed, and taken back.
const TYPE_MS: u64 = 22;
const ERASE_MS: u64 = 6;

/// The works the foyer shows, in order.
const FEATURED: [usize; 8] = [4, 0, 6, 9, 2, 11, 7, 5];

/// What a press in the foyer asks of the page.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Asked {
    Hall,
    Studio,
    /// To see a work in the viewing room.
    View(usize),
}

enum Phase {
    /// The next work is being painted; the last stays up meanwhile.
    Painting(Box<Painter>),
    /// The steps are turning over to it, `next` being the next to turn.
    Turning { next: usize, clock: f32 },
    /// All six are up.
    Holding(Tween),
}

struct Step {
    root: Leaf,
    plate: Plate,
    tag: Leaf,
}

pub struct Foyer {
    tail: Leaf,
    arriving: Vec<Arrival>,
    steps: Vec<Step>,
    dashes: Dashes,
    hovered: Option<usize>,
    caption: Leaf,
    prompt: Typewriter,
    enter: Chip,
    studio: Chip,
    featured: usize,
    denoise: Option<Denoise>,
    phase: Phase,
    started: bool,
}

impl Foyer {
    pub fn grow(grove: &mut Grove, page: Leaf, face: Font, hover: &mut Hover, icons: &Icons) -> Self {
        let m = measure();
        let col = grove.branch(
            page,
            Stem::new()
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(column(), top(0.px()).height(10.px()))),
        );
        // The masthead: who, and when, over a rule. On a narrow window the dates go under the name.
        let name = grove.branch(
            col,
            Text::new("ilex")
                .font(face)
                .font_size(FontSize::new().xs(16).md(18))
                .color(Palette::Ink)
                .intangible()
                .opacity(0.0)
                .at(Location::new().xs(left(0.px()).width(content()), top(26.px()).height(content()))),
        );
        let dates = grove.branch(
            col,
            Text::new("12 works · 4 rooms · until 31.01.2027")
                .font_size(lichen::caption())
                .color(Palette::Muted)
                .intangible()
                .opacity(0.0)
                .at(Location::new()
                    .xs(left(0.px()).width(content()), top(54.px()).height(1.letters()))
                    .sm(right(100.pct()).width(content()), top(30.px()).height(1.letters()))),
        );
        let rule = grove.branch(
            col,
            Panel::new()
                .color(Palette::Signal)
                .intangible()
                .opacity(0.0)
                .at(Location::new()
                    .xs(left(0.px()).right(100.pct()), top(84.px()).height(HAIRLINE.px()))
                    .sm(left(0.px()).right(100.pct()), top(62.px()).height(HAIRLINE.px()))),
        );
        // The name of the show, two lines wherever one will not hold it.
        let title = grove.branch(
            col,
            Text::new("latent flora")
                .font(face)
                .font_size(theme::title())
                .color(Palette::Ink)
                .intangible()
                .opacity(0.0)
                .at(Location::new()
                    .xs(left((-3).px()).right(100.pct()), top(104.px()).height(content()))
                    .sm(left((-4).px()).right(100.pct()), top(86.px()).height(content()))),
        );
        // The strip: six across on a wide window, two rows of three on a narrow one.
        let strip = grove.branch(
            col,
            Stem::new()
                .intangible()
                .anchored(title)
                .at(Location::new()
                    .xs(
                        left(0.px()).right(100.pct()),
                        top(anchor().bottom() + 24.px()).height(aspect(1.5) - (GAP / 3.0).px()),
                    )
                    .md(
                        left(0.px()).right(100.pct()),
                        top(anchor().bottom() + 34.px()).height(aspect(6.0) - (GAP * 5.0 / 6.0).px()),
                    )),
        );
        let steps: Vec<Step> = (0..STEPS)
            .map(|k| {
                let (across, down) = ((k % 3) as f32, (k / 3) as f32);
                let side = |of: f32| (100.pct() - ((of - 1.0) * GAP).px()) * (1.0 / of);
                let root = grove.branch(
                    strip,
                    Stem::new()
                        .interactive()
                        .opacity(0.0)
                        .at(Location::new()
                            .xs(
                                left(side(3.0) * across + (across * GAP).px()).width(side(3.0)),
                                top((trunk().width() - (2.0 * GAP).px()) * (down / 3.0) + (down * GAP).px())
                                    .height(aspect(1.0)),
                            )
                            .md(
                                left(side(6.0) * k as f32 + (k as f32 * GAP).px()).width(side(6.0)),
                                top(0.px()).height(aspect(1.0)),
                            )),
                );
                let plate = grove.plate();
                grove.load(plate, blank((SIDE, SIDE)), area((SIDE, SIDE)));
                grove.branch(
                    root,
                    Image::new(plate)
                        .fit(Fit::Crop)
                        .intangible()
                        .at(Location::new()),
                );
                // A hairline round the step, so the first, still nearly the wall's own dark, has
                // an edge.
                grove.branch(
                    root,
                    Panel::new()
                        .color(Color::rgba(0.0, 0.0, 0.0, 0.0))
                        .outline(HAIRLINE, Palette::Raised)
                        .intangible()
                        .elevate(Elevation::up(1))
                        .at(Location::new()),
                );
                // Which step of the sampler this is, on the step itself.
                let tag = grove.branch(
                    root,
                    Text::new(step_label(k, 50))
                        .font_size(lichen::caption())
                        .color(if k == 0 { Palette::Accent } else { Palette::Ink })
                        .intangible()
                        .elevate(Elevation::up(3))
                        .at(Location::new().xs(
                            left(8.px()).width(content()),
                            top(100.pct() - 8.px() - 1.letters()).height(1.letters()),
                        )),
                );
                grove.branch(
                    root,
                    Panel::new()
                        .color(Palette::Surface)
                        .intangible()
                        .opacity(0.82)
                        .elevate(Elevation::up(2))
                        .anchored(tag)
                        .at(Location::new().xs(
                            left(anchor().left() - 5.px()).right(anchor().right() + 5.px()),
                            top(anchor().top() - 3.px()).bottom(anchor().bottom() + 3.px()),
                        )),
                );
                hover.watch(root);
                Step { root, plate, tag }
            })
            .collect();
        let mut dashes = Dashes::new(foliage::Area::new(190.0, 190.0))
            .color(Palette::Accent)
            .elevate(Elevation::up(3));
        dashes.grow(grove, strip);
        // The work the strip is showing: its number and title, and its prompt being typed.
        let caption = grove.branch(
            col,
            Text::new("")
                .font_size(lichen::caption())
                .color(Palette::Muted)
                .intangible()
                .opacity(0.0)
                .anchored(strip)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + 24.px()).height(content()),
                )),
        );
        let typed = grove.branch(
            col,
            Text::new("")
                .font_size(theme::lead())
                .color(Palette::Ink)
                .intangible()
                .opacity(0.0)
                .anchored(caption)
                .at(Location::new()
                    .xs(left(0.px()).right(100.pct()), top(anchor().bottom() + 10.px()).height(3.letters()))
                    .md(left(0.px()).right(100.pct()).at_most(980.px()), top(anchor().bottom() + 10.px()).height(2.letters()))),
        );
        let prompt = Typewriter::new(grove, typed, "");
        let row = grove.branch(
            col,
            Stem::new()
                .intangible()
                .anchored(typed)
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + 22.px()).height(m.height.px()),
                )),
        );
        let at = |from: foliage::Length, name: &str| {
            Location::new().xs(left(from).width(Chip::width(name)), top(0.px()).height(100.pct()))
        };
        let mut enter = Chip::grow(grove, row, at(0.px(), "enter the hall"), Elevation::up(1), icons.arrow_right, "enter the hall");
        enter.arm(grove, Press::Armed);
        let after_enter = Chip::width("enter the hall") + m.gap.px();
        let studio = Chip::grow(grove, row, at(after_enter, "grow your own"), Elevation::up(1), icons.feather, "grow your own");
        grove.opacity(enter.leaf(), 0.0);
        grove.opacity(studio.leaf(), 0.0);
        let mut arriving = vec![
            Arrival::words(name, Some(face), FontSize::new().xs(16).md(18), 4.0),
            Arrival::words(dates, None, lichen::caption(), 3.0),
            rule.into(),
            Arrival::words(title, Some(face), theme::title(), 16.0),
        ];
        arriving.extend(steps.iter().map(|step| Arrival::from(step.root)));
        arriving.extend([
            Arrival::words(caption, None, lichen::caption(), 3.0),
            typed.into(),
            enter.leaf().into(),
            studio.leaf().into(),
        ]);
        Self {
            tail: row,
            arriving,
            steps,
            dashes,
            hovered: None,
            caption,
            prompt,
            enter,
            studio,
            featured: 0,
            denoise: None,
            phase: Phase::Painting(Box::new(painter(FEATURED[0]))),
            started: false,
        }
    }

    /// The last thing in the foyer, which the wall text stands below.
    pub fn tail(&self) -> Leaf {
        self.tail
    }

    /// Carries the foyer for a frame. `seen` is whether nothing covers it.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen, hover: &Hover, seen: bool, emerge: &mut Emerge) -> Option<Asked> {
        if !self.started {
            self.started = true;
            // Everything comes in out of the dark, the masthead first and the presses last.
            emerge.bring(grove, &self.arriving[..4], 150, 90);
            emerge.bring(grove, &self.arriving[4..4 + STEPS], 520, 70);
            emerge.bring(grove, &self.arriving[4 + STEPS..], 900, 110);
        }
        self.prompt.frame(grove, pollen);
        self.turn(grove, pollen, seen, emerge);
        // A step under the pointer is outlined.
        let hovered = self.steps.iter().position(|step| seen && hover.is_over(step.root));
        if hovered != self.hovered {
            self.hovered = hovered;
            match hovered {
                Some(n) => {
                    self.dashes.over(grove, self.steps[n].root);
                    self.dashes.draw(grove, true);
                }
                None => self.dashes.draw(grove, false),
            }
        }
        if self.enter.pressed(pollen) {
            return Some(Asked::Hall);
        }
        if self.studio.pressed(pollen) {
            return Some(Asked::Studio);
        }
        if self.steps.iter().any(|step| pollen.activated(step.root)) {
            return Some(Asked::View(FEATURED[self.featured]));
        }
        None
    }

    /// Moves the strip along: paints the next work, turns the steps over to it, holds it.
    fn turn(&mut self, grove: &mut Grove, pollen: &Pollen, seen: bool, emerge: &mut Emerge) {
        match &mut self.phase {
            Phase::Painting(painter) => {
                if !seen {
                    return;
                }
                grove.again();
                if !painter.step() {
                    return;
                }
                let Phase::Painting(painter) = std::mem::replace(&mut self.phase, Phase::Turning { next: 0, clock: 0.0 }) else {
                    unreachable!("matched as painting above");
                };
                let w = &WORKS[FEATURED[self.featured]];
                self.denoise = Some(Denoise::new(painter.into_pixels(), SIDE, SIDE, w.seed()));
                let room = &ROOMS[w.room];
                let number = format!("no. {:02}", w.no);
                grove.text(self.caption, format!("{number} · {} · room {} · {}", w.title, room.numeral, room.name));
                grove.tint(self.caption, [(0..number.len(), Palette::Accent)]);
                // The caption is sampled anew with the work, unless it is still on its way in.
                if !emerge.busy(self.caption) {
                    emerge.now(grove, Arrival::words(self.caption, None, lichen::caption(), 3.0));
                }
                self.prompt.write(grove, &format!("“{}”", w.prompt));
                self.prompt.type_out(grove, TYPE_MS, 0);
            }
            Phase::Turning { next, clock } => {
                grove.again();
                *clock += grove.frame_time().as_secs_f32() * 1000.0;
                while *clock >= TURN_MS && *next < STEPS {
                    *clock -= TURN_MS;
                    let k = *next as u32;
                    let of = (STEPS - 1) as u32;
                    if let Some(denoise) = &self.denoise {
                        grove.load(self.steps[*next].plate, denoise.patches(k, of, SIDE / PATCHES), area((SIDE, SIDE)));
                    }
                    let steps = WORKS[FEATURED[self.featured]].steps;
                    grove.text(self.steps[*next].tag, step_label(*next, steps));
                    *next += 1;
                }
                if *next >= STEPS {
                    self.phase = Phase::Holding(grove.timer(Timing::ms(HOLD_MS)));
                }
            }
            Phase::Holding(hold) => {
                if pollen.finished(*hold) {
                    if !seen {
                        // Out of sight: wait for the visitor to come back rather than turn unseen.
                        self.phase = Phase::Holding(grove.timer(Timing::ms(1200)));
                        return;
                    }
                    self.featured = (self.featured + 1) % FEATURED.len();
                    self.prompt.erase(grove, ERASE_MS, 0);
                    self.phase = Phase::Painting(Box::new(painter(FEATURED[self.featured])));
                }
            }
        }
    }
}

/// What step `k` of the strip stands for, of a sampler running `steps`.
fn step_label(k: usize, steps: u32) -> String {
    let at = k as u32 * steps / (STEPS - 1) as u32;
    format!("step {at}/{steps}")
}

/// A painter for `work`, at the strip's size.
fn painter(work: usize) -> Painter {
    let w = &WORKS[work];
    Painter::new(w.style, Params::of(w.style, w.seed()), &w.ramp(), SIDE, SIDE, 0.0)
}
