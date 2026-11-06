//! The scale bar along the foot: the working week drawn as a drawing's graphic scale -- alternate
//! days solid and open -- with every meeting laid along it, and a needle for *now*.
//!
//! The needle can be dragged or set playing, and the meetings it stands in are lit on the sheet
//! with the highlighter. A meeting's block can be dragged along the bar to another slot, which is
//! asked of the server; until it answers, the block waits where it was put.

use foliage::{
    Axes, Boxed, Color, Elevation, Grove, Grow, Leaf, Location, Motion, Panel, Place, Pollen, Polygon,
    Source, Stem, Timing, center_x, content, left, right, top,
};
use lichen::{Chip, Say};

use crate::board::{Board, DAYS, HALVES, Id, Kind, Slot};
use crate::icons::Icons;
use crate::kit;
use crate::theme::{self, Faces};

/// Half hours in the week the bar spans.
const WEEK: f32 = 5.0 * HALVES as f32;
/// How fast the needle runs while playing, in half hours a second.
const PLAY: f32 = 3.0;
/// Where the scale stands in the plate: in from the left past the play press, and its height.
const SCALE_LEFT: f32 = 54.0;
const SCALE_TOP: f32 = 34.0;
/// The rows of meeting blocks under the bar.
const LANE_TOP: f32 = 24.0;
const LANE: f32 = 7.0;
const LANES: usize = 3;

/// What the bar asks of the sheet.
#[derive(Clone, Debug, PartialEq)]
pub enum Asked {
    /// A meeting's block was let go at another slot.
    Move(Id, Slot),
}

struct Block {
    id: Id,
    leaf: Leaf,
    slot: Slot,
    lane: usize,
    fill: Color,
}

pub struct Timeline {
    plate: Leaf,
    play: Chip,
    readout: Leaf,
    pub say: Say,
    area: Leaf,
    needle: Leaf,
    blocks: Vec<Block>,
    /// Where the needle stands, in half hours from Monday 08:00.
    pub at: f32,
    playing: bool,
    grabbed: Option<(Id, Slot, Slot)>,
    held: bool,
    /// Blocks let go and waiting on the server, with where they were.
    pub waiting: Option<(Id, Slot)>,
    icons_play: (foliage::Field, foliage::Field),
}

fn pct(halves: f32) -> f32 {
    halves / WEEK * 100.0
}

impl Timeline {
    pub fn grow(grove: &mut Grove, under: Leaf, faces: &Faces, icons: &Icons) -> Self {
        let plate = kit::plate(grove, under, Elevation::up(30));
        let m = lichen::measure();
        let play = Chip::mark(
            grove,
            plate,
            Location::new().xs(left(10.px()).width(m.height.px()), top(((theme::SCALE_BAR - m.height) / 2.0 + 6.0).px()).height(m.height.px())),
            Elevation::up(1),
            icons.play,
        );
        kit::letter(
            grove,
            plate,
            "NOW",
            faces.regular,
            theme::LABEL,
            theme::soft(),
            Location::new().xs(left(SCALE_LEFT.px()).width(content()), top(11.px()).height(1.letters())),
        );
        let readout = kit::letter(
            grove,
            plate,
            "MON 08:00",
            faces.bold,
            theme::TITLE,
            theme::marker(),
            Location::new().xs(left((SCALE_LEFT + 26.0).px()).width(content()), top(9.px()).height(1.letters())),
        );
        let say = Say::grow(
            grove,
            plate,
            Location::new().xs(right(100.pct() - 14.px()).width(content()), top(8.px()).height(1.letters())),
            Some(faces.regular),
        );
        let area = grove.branch(
            plate,
            Stem::new()
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    left(SCALE_LEFT.px()).right(100.pct() - 16.px()),
                    top(SCALE_TOP.px()).bottom(100.pct() - 8.px()),
                )),
        );
        // The graphic scale: a day to a segment, solid and open by turns, ticked every two hours.
        for (d, day) in DAYS.iter().enumerate() {
            let from = pct(d as f32 * HALVES as f32);
            let width = pct(HALVES as f32);
            let segment = Panel::new().intangible();
            let segment = match d % 2 {
                0 => segment.color(theme::ink()),
                _ => segment.color(theme::clear()).outline(1.0, theme::ink()),
            };
            grove.branch(
                area,
                segment.at(Location::new().xs(left(from.pct()).width(width.pct()), top(0.px()).height(6.px()))),
            );
            kit::letter(
                grove,
                area,
                *day,
                faces.bold,
                theme::LABEL + 1,
                theme::soft(),
                Location::new().xs(left(from.pct() + 3.px()).width(content()), top(10.px()).height(1.letters())),
            );
            for hour in [4.0, 8.0, 12.0, 16.0] {
                let x = pct(d as f32 * HALVES as f32 + hour);
                kit::rule(
                    grove,
                    area,
                    Location::new().xs(left(x.pct()).width(1.px()), top(8.px()).height(3.px())),
                    theme::rule(),
                );
            }
        }
        // The needle: a line in highlighter, its head a little triangle over the bar.
        let needle = grove.branch(
            area,
            Stem::new()
                .interactive()
                .drags(Axes::Horizontal)
                .elevate(Elevation::up(4))
                .at(needle_at(0.0)),
        );
        grove.branch(
            needle,
            Panel::new()
                .color(theme::marker())
                .intangible()
                .at(Location::new().xs(center_x(50.pct()).width(1.5.px()), top(6.px()).bottom(100.pct()))),
        );
        grove.branch(
            needle,
            Polygon::new()
                .sides(3.0)
                .color(theme::marker())
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(center_x(50.pct()).width(12.px()), top(0.px()).height(9.px()))),
        );
        Self {
            plate,
            play,
            readout,
            say,
            area,
            needle,
            blocks: Vec::new(),
            at: 0.0,
            playing: false,
            grabbed: None,
            held: false,
            waiting: None,
            icons_play: (icons.play, icons.pause),
        }
    }

    pub fn plate(&self) -> Leaf {
        self.plate
    }

    /// Lays the bar along the foot of the window.
    pub fn place(&mut self, grove: &mut Grove) {
        grove.at(
            self.plate,
            Location::new().xs(
                left(theme::GUTTER.px()).right(100.pct() - theme::GUTTER.px()),
                foliage::bottom(100.pct() - theme::GUTTER.px()).height(theme::SCALE_BAR.px()),
            ),
        );
    }

    /// Brings the blocks into line with the board's meetings: one per meeting, in a lane clear of
    /// the meetings it overlaps, coloured for whether it clashes.
    pub fn sync(&mut self, grove: &mut Grove, board: &Board, clashing: &[Id]) {
        let mut meetings: Vec<(Id, Slot)> = board
            .nodes
            .iter()
            .filter(|n| n.kind == Kind::Meeting)
            .filter_map(|n| Some((n.id, n.slot?)))
            .collect();
        meetings.sort_by_key(|(id, slot)| (slot.week(), *id));
        let mut lanes = [0u32; LANES];
        let mut n = 0;
        while n < self.blocks.len() {
            if meetings.iter().any(|(id, _)| *id == self.blocks[n].id) {
                n += 1;
            } else {
                let block = self.blocks.remove(n);
                grove.prune(block.leaf);
            }
        }
        for (id, slot) in meetings {
            let lane = (0..LANES).find(|&l| lanes[l] <= slot.week()).unwrap_or(LANES - 1);
            lanes[lane] = slot.week() + slot.len as u32;
            let fill = if clashing.contains(&id) { theme::pencil() } else { theme::ink() };
            match self.blocks.iter_mut().find(|b| b.id == id) {
                Some(block) => {
                    if block.slot != slot || block.lane != lane {
                        grove.animate(block.leaf, Motion::Location(block_at(slot, lane)), Timing::ms(420).ease(foliage::Ease::Emphasis));
                    }
                    if block.fill != fill {
                        grove.animate(block.leaf, Motion::Color(fill), Timing::ms(260));
                    }
                    block.slot = slot;
                    block.lane = lane;
                    block.fill = fill;
                }
                None => {
                    let leaf = grove.branch(
                        self.area,
                        Panel::new()
                            .color(fill)
                            .interactive()
                            .drags(Axes::Horizontal)
                            .elevate(Elevation::up(2))
                            .at(block_at(slot, lane)),
                    );
                    self.blocks.push(Block { id, leaf, slot, lane, fill });
                }
            }
        }
    }

    /// The meetings the needle stands in.
    pub fn active(&self, board: &Board) -> Vec<Id> {
        board
            .nodes
            .iter()
            .filter(|n| n.kind == Kind::Meeting)
            .filter(|n| n.slot.is_some_and(|s| (s.week() as f32) <= self.at && self.at < (s.week() + s.len as u32) as f32))
            .map(|n| n.id)
            .collect()
    }

    /// Stands the needle at `at`, in half hours from Monday 08:00.
    pub fn needle(&mut self, grove: &mut Grove, at: f32) {
        self.at = at.rem_euclid(WEEK);
        grove.at(self.needle, needle_at(self.at));
        let day = (self.at / HALVES as f32) as usize % 5;
        let minutes = ((self.at % HALVES as f32) * 30.0 / 5.0).floor() as u32 * 5 + 8 * 60;
        grove.text(self.readout, format!("{} {:02}:{:02}", DAYS[day], minutes / 60, minutes % 60));
    }

    /// Puts a block that was waiting on the server back where it was.
    pub fn snap_back(&mut self, grove: &mut Grove) {
        if let Some((id, slot)) = self.waiting.take()
            && let Some(block) = self.blocks.iter().find(|b| b.id == id)
        {
            grove.animate(block.leaf, Motion::Location(block_at(slot, block.lane)), Timing::ms(320).ease(foliage::Ease::Emphasis));
        }
    }

    /// Carries the bar: the needle dragged or playing, a block dragged and let go. Whether the
    /// needle moved, and what was asked.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> (bool, Vec<Asked>) {
        let mut asked = Vec::new();
        let mut moved = false;
        if self.play.pressed(pollen) {
            self.playing = !self.playing;
            self.play.remark(grove, if self.playing { self.icons_play.1 } else { self.icons_play.0 });
        }
        let area = lichen::drawn(grove, self.area);
        if let (Some(drag), Some(area)) = (pollen.dragged(self.needle), area) {
            self.held = true;
            let at = (drag.current.x - area.left()) / area.width().max(1.0) * WEEK;
            self.needle(grove, at.clamp(0.0, WEEK - 0.01));
            moved = true;
        }
        if self.held && pollen.disengaged(self.needle) {
            self.held = false;
        }
        if self.playing && !self.held {
            let dt = grove.frame_time().as_secs_f32().min(0.1);
            self.needle(grove, self.at + dt * PLAY);
            grove.again();
            moved = true;
        }
        for block in &self.blocks {
            if pollen.drag_started(block.leaf) {
                self.grabbed = Some((block.id, block.slot, block.slot));
            }
            if let (Some(drag), Some(area), Some((id, from, _))) = (pollen.dragged(block.leaf), area, self.grabbed)
                && id == block.id
            {
                let shift = ((drag.current.x - drag.start.x) / area.width().max(1.0) * WEEK).round() as i32;
                let week = (from.week() as i32 + shift).clamp(0, WEEK as i32 - from.len as i32);
                let (mut day, mut start) = ((week / HALVES as i32) as u8, (week % HALVES as i32) as u8);
                if start + from.len > HALVES {
                    if (HALVES - from.len) as i32 >= 0 && start > HALVES / 2 {
                        start = HALVES - from.len;
                    } else {
                        day = (day + 1).min(4);
                        start = 0;
                    }
                }
                let to = Slot::new(day, start, from.len);
                grove.at(block.leaf, block_at(to, block.lane));
                self.grabbed = Some((id, from, to));
            }
            if pollen.disengaged(block.leaf)
                && let Some((id, from, to)) = self.grabbed
                && id == block.id
            {
                self.grabbed = None;
                if to != from {
                    self.waiting = Some((id, from));
                    asked.push(Asked::Move(id, to));
                }
            }
        }
        (moved, asked)
    }
}

fn needle_at(at: f32) -> Location {
    Location::new().xs(center_x(pct(at).pct()).width(18.px()), top((-8.0).px()).bottom(100.pct()))
}

fn block_at(slot: Slot, lane: usize) -> Location {
    Location::new().xs(
        left(pct(slot.week() as f32).pct()).width(pct(slot.len as f32).pct() - 1.px()),
        top((LANE_TOP + lane as f32 * LANE).px()).height((LANE - 2.0).px()),
    )
}
