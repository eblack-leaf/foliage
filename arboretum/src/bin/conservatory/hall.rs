//! The hall: a corridor of four rooms running sideways, three works to a room.
//!
//! Works come in out of the dark the first time they come into view, and develop as they do -- two
//! at a time at most, so however fast the corridor is run along only two pictures are ever being
//! painted and loaded at once.

use std::collections::VecDeque;

use foliage::{
    Area, Axes, Boxed, Ease, Elevation, Font, FontSize, Grove, Grow, Leaf, Location, Motion,
    Palette, Place, Pollen, Sap, ScrollTo, Source, Stem, Text, Timing, Vein, anchor, content, left,
    right, top,
};
use lichen::{Chip, Dashes, Hover, measure};

use arboretum::sprout::{Arrival, Emerge};

use crate::frame::{Frame, PLACARD, SIDE, Spot};
use crate::icons::Icons;
use crate::theme::{self, column};
use crate::works::{ROOMS, WORKS};

/// How far the corridor's first room stands in from its start, narrow and wide.
const PAD: (f32, f32) = (22.0, 56.0);
/// How wide a room's label is, and the gap between two frames.
const LABEL: (f32, f32) = (150.0, 200.0);
const GAP: (f32, f32) = (26.0, 44.0);
/// The room between the last work of one room and the next room's label.
const BETWEEN: f32 = 72.0;
/// How tall the corridor is, narrow and wide -- a frame, and room round it for the outline and the
/// lean -- and how far below the hall's top it starts.
const CORRIDOR: (f32, f32) = (SIDE.0 + PLACARD + 24.0, SIDE.1 + PLACARD + 24.0);
const HEAD: f32 = 76.0;
/// How many works develop at once.
const AT_ONCE: usize = 2;

/// Where everything in the corridor stands, at one width of frame.
struct Plan {
    labels: [f32; 4],
    frames: [f32; 12],
    stride: f32,
    end: f32,
}

fn plan(pad: f32, label: f32, frame: f32, gap: f32) -> Plan {
    let mut labels = [0.0; 4];
    let mut frames = [0.0; 12];
    let mut x = pad;
    for room in 0..4 {
        labels[room] = x;
        x += label;
        for k in 0..3 {
            frames[room * 3 + k] = x;
            x += frame;
            if k < 2 {
                x += gap;
            }
        }
        if room < 3 {
            x += BETWEEN;
        }
    }
    Plan {
        labels,
        frames,
        stride: label + 3.0 * frame + 2.0 * gap + BETWEEN,
        end: x + pad,
    }
}

pub struct Hall {
    root: Leaf,
    corridor: Leaf,
    frames: Vec<Frame>,
    /// Each room's label, and whether it has come in yet.
    labels: Vec<(Leaf, bool)>,
    head: Vec<Arrival>,
    dashes: Dashes,
    queue: VecDeque<usize>,
    shown: bool,
    brought: Vec<bool>,
    back: Chip,
    on: Chip,
    narrow: Plan,
    wide: Plan,
    hovered: Option<usize>,
}

impl Hall {
    pub fn grow(grove: &mut Grove, page: Leaf, after: Leaf, face: Font, hover: &mut Hover, icons: &Icons) -> Self {
        let root = grove.branch(
            page,
            Stem::new()
                .intangible()
                .anchored(after)
                .at(Location::new()
                    .xs(
                        left(0.px()).right(100.pct()),
                        top(anchor().bottom() + 120.px()).height((HEAD + CORRIDOR.0).px()),
                    )
                    .md(
                        left(0.px()).right(100.pct()),
                        top(anchor().bottom() + 150.px()).height((HEAD + CORRIDOR.1).px()),
                    )),
        );
        let head = grove.branch(
            root,
            Stem::new()
                .intangible()
                .font_size(lichen::caption())
                .at(Location::new().xs(column(), top(0.px()).height(HEAD.px()))),
        );
        let heading = grove.branch(
            head,
            Text::new("the hall")
                .font(face)
                .font_size(theme::heading())
                .color(Palette::Ink)
                .intangible()
                .opacity(0.0)
                .at(Location::new().xs(left(0.px()).width(content()), top(0.px()).height(content()))),
        );
        let m = measure();
        let back = Chip::mark(
            grove,
            head,
            Location::new().xs(
                right(100.pct() - Chip::square() - m.gap.px()).width(Chip::square()),
                top(2.px()).height(m.height.px()),
            ),
            Elevation::up(1),
            icons.chevron_left,
        );
        let on = Chip::mark(
            grove,
            head,
            Location::new().xs(right(100.pct()).width(Chip::square()), top(2.px()).height(m.height.px())),
            Elevation::up(1),
            icons.chevron_right,
        );
        grove.opacity(back.leaf(), 0.0);
        grove.opacity(on.leaf(), 0.0);
        let corridor = grove.branch(
            root,
            Stem::new()
                .scrolls(Axes::Horizontal)
                .at(Location::new()
                    .xs(left(0.px()).right(100.pct()), top(HEAD.px()).height(CORRIDOR.0.px()))
                    .md(left(0.px()).right(100.pct()), top(HEAD.px()).height(CORRIDOR.1.px()))),
        );
        let narrow = plan(PAD.0, LABEL.0, SIDE.0, GAP.0);
        let wide = plan(PAD.1, LABEL.1, SIDE.1, GAP.1);
        // How far the corridor runs: its far end, which nothing is drawn at.
        grove.branch(
            corridor,
            Stem::new().intangible().at(Location::new()
                .xs(left(narrow.end.px()).width(1.px()), top(0.px()).height(1.px()))
                .md(left(wide.end.px()).width(1.px()), top(0.px()).height(1.px()))),
        );
        let labels = ROOMS
            .iter()
            .enumerate()
            .map(|(n, room)| {
                let label = grove.branch(
                    corridor,
                    Stem::new()
                        .intangible()
                        .opacity(0.0)
                        .at(Location::new()
                            .xs(
                                left(narrow.labels[n].px()).width((LABEL.0 - 20.0).px()),
                                top(0.px()).height(SIDE.0.px()),
                            )
                            .md(
                                left(wide.labels[n].px()).width((LABEL.1 - 32.0).px()),
                                top(0.px()).height(SIDE.1.px()),
                            )),
                );
                let numeral = grove.branch(
                    label,
                    Text::new(room.numeral)
                        .font(face)
                        .font_size(FontSize::new().xs(30).md(40))
                        .color(Palette::hue(n))
                        .intangible()
                        .at(Location::new().xs(left(0.px()).width(content()), top(0.px()).height(content()))),
                );
                grove.branch(
                    label,
                    Text::new(room.name)
                        .font(face)
                        .font_size(FontSize::new().xs(13).md(17))
                        .color(Palette::Ink)
                        .intangible()
                        .anchored(numeral)
                        .at(Location::new().xs(
                            left(0.px()).right(100.pct()),
                            top(anchor().bottom() + 8.px()).height(content()),
                        )),
                );
                (label, false)
            })
            .collect();
        let frames: Vec<Frame> = (0..WORKS.len())
            .map(|n| {
                let at = Spot {
                    narrow: narrow.frames[n],
                    wide: wide.frames[n],
                };
                let frame = Frame::grow(grove, corridor, n, at);
                hover.watch(frame.leaf());
                frame
            })
            .collect();
        let mut dashes = Dashes::new(Area::new(SIDE.1, SIDE.1))
            .pitch(12.0)
            .color(Palette::Accent)
            .elevate(Elevation::up(5));
        dashes.grow(grove, corridor);
        let brought = vec![false; frames.len()];
        Self {
            root,
            corridor,
            frames,
            labels,
            head: vec![
                Arrival::words(heading, Some(face), theme::heading(), 7.0),
                back.leaf().into(),
                on.leaf().into(),
            ],
            dashes,
            queue: VecDeque::new(),
            shown: false,
            brought,
            back,
            on,
            narrow,
            wide,
            hovered: None,
        }
    }

    pub fn leaf(&self) -> Leaf {
        self.root
    }

    /// The finished picture of `work`, if it has developed.
    pub fn picture(&self, work: usize) -> Option<&[u8]> {
        self.frames.get(work).and_then(Frame::picture)
    }

    /// Scrolls the corridor so `work` is in view.
    pub fn show(&mut self, grove: &mut Grove, work: usize) {
        if let Some(frame) = self.frames.get(work) {
            grove.animate(
                self.corridor,
                Motion::Scroll(ScrollTo::show(frame.leaf()).on(Axes::Horizontal)),
                Timing::ms(600).ease(Ease::Emphasis),
            );
        }
    }

    /// Carries the hall. `seen` is whether nothing covers it. Hands back the work pressed, if any.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen, hover: &Hover, seen: bool, emerge: &mut Emerge) -> Option<usize> {
        let view = grove.viewport();
        let in_view = matches!(
            grove.tap(self.root, Vein::Drawn),
            Some(Sap::Section(drawn)) if drawn.top() < view.height * 0.9 && drawn.bottom() > 0.0
        );
        if in_view && !self.shown {
            self.shown = true;
            emerge.bring(grove, &self.head, 0, 110);
        }
        // What has come into the window comes in out of the dark, and joins the queue to develop.
        if in_view {
            let near = |drawn: foliage::Section| drawn.right() > -40.0 && drawn.left() < view.width + 40.0;
            for (label, brought) in &mut self.labels {
                if !*brought
                    && let Some(Sap::Section(drawn)) = grove.tap(*label, Vein::Drawn)
                    && near(drawn)
                {
                    *brought = true;
                    emerge.bring(grove, &[Arrival::from(*label)], 60, 0);
                }
            }
            let mut arriving = Vec::new();
            for (n, frame) in self.frames.iter().enumerate() {
                if self.brought[n] {
                    continue;
                }
                if let Some(drawn) = frame.drawn(grove)
                    && near(drawn)
                {
                    self.brought[n] = true;
                    arriving.push(Arrival::from(frame.leaf()));
                    if !frame.started() && !frame.waiting_on_file() {
                        self.queue.push_back(n);
                    }
                }
            }
            emerge.bring(grove, &arriving, 160, 140);
        }
        let busy = self.frames.iter().filter(|frame| frame.busy()).count();
        for _ in busy..AT_ONCE {
            let Some(next) = self.queue.pop_front() else {
                break;
            };
            self.frames[next].develop(grove);
        }
        // The pointer: a lean, the number lit, and the outline drawn round.
        let mut hovered = None;
        for (n, frame) in self.frames.iter_mut().enumerate() {
            let over = seen && hover.is_over(frame.leaf());
            if over {
                hovered = Some(n);
            }
            frame.frame(grove, pollen, over);
        }
        if hovered != self.hovered {
            self.hovered = hovered;
            match hovered {
                Some(n) => {
                    self.dashes.over(grove, self.frames[n].picture_leaf());
                    self.dashes.draw(grove, true);
                }
                None => self.dashes.draw(grove, false),
            }
        }
        // A room at a time.
        let step = |hall: &Hall, grove: &mut Grove, by: f32| {
            let stride = match grove.viewport().width >= foliage::Layout::MD {
                true => hall.wide.stride,
                false => hall.narrow.stride,
            };
            let at = match grove.tap(hall.corridor, Vein::Offset) {
                Some(Sap::Position(offset)) => offset.x,
                _ => 0.0,
            };
            let to = ((at / stride).round() + by).max(0.0) * stride;
            grove.animate(
                hall.corridor,
                Motion::Scroll(ScrollTo::px(to).on(Axes::Horizontal)),
                Timing::ms(700).ease(Ease::Emphasis),
            );
        };
        if self.back.pressed(pollen) {
            step(self, grove, -1.0);
        }
        if self.on.pressed(pollen) {
            step(self, grove, 1.0);
        }
        self.frames
            .iter()
            .find(|frame| pollen.activated(frame.leaf()))
            .map(|frame| frame.work)
    }
}
