//! The inspector: a detail callout for the card picked out, tied to it by a leader.
//!
//! Its head is the drafting symbol for a detail -- a circle halved, the detail's number over the
//! sheet's -- and under it the card's record, in lichen's fields: what it is called, a person's
//! role, a meeting's day and time and length, a spec's status, and what it is tied to. Every field
//! holds a record, so a word changed reads in the highlighter until the server has filed it, and
//! every change is written to the card as it is made.
//!
//! Wide, it stands down the right of the window; narrow, it rises from the foot over the scale bar.

use foliage::{
    Boxed, Elevation, Grove, Grow, Layout, Leaf, Location, Motion, Place, Pollen, Source, Stem, Timing, center_x,
    center_y, content, left, right, top,
};
use lichen::{Chip, Confirm, Field, Holds, Origin, Pick, Step};

use crate::board::{Board, Id, Kind, Slot, Status, clock};
use crate::icons::Icons;
use crate::kit;
use crate::theme::{self, Faces};
use crate::undo::History;

/// What the delete says before it is asked.
const ASK: &str = "delete";

/// The width of a field's name cell, in letters.
const CELL: f32 = 7.0;
const ROW: f32 = 42.0;
const HEAD: f32 = 64.0;
const PAD: f32 = 14.0;

const DAY_NAMES: &[&str] = &["MON", "TUE", "WED", "THU", "FRI"];
const LENGTHS: &[&str] = &["30m", "1h", "90m", "2h"];
const STATUSES: &[&str] = &["draft", "review", "approved"];

/// What the inspector asks of the sheet.
#[derive(Clone, Debug, PartialEq)]
pub enum Asked {
    Close,
    /// The record was changed from here.
    Edited,
    Delete(Id),
}

pub struct Inspector {
    plate: Leaf,
    pub bubble: Leaf,
    number: Leaf,
    kind: Leaf,
    heading: Leaf,
    close: Chip,
    title: Field,
    role: Field,
    words: Field,
    day: Pick,
    length: Pick,
    status: Pick,
    time: Leaf,
    time_value: Leaf,
    earlier: Chip,
    later: Chip,
    ties: Leaf,
    delete: Confirm,
    pub id: Option<Id>,
    pub open: bool,
    layout: Layout,
    height: f32,
    /// The field being typed into, whose first edit was kept for undoing.
    typing: Option<(Id, usize)>,
    kind_now: Option<Kind>,
}

impl Inspector {
    pub fn grow(grove: &mut Grove, under: Leaf, faces: &Faces, icons: &Icons) -> Self {
        let plate = kit::plate(grove, under, Elevation::up(32));
        grove.visible(plate, false);
        let body = grove.branch(
            plate,
            Stem::new()
                .intangible()
                .elevate(Elevation::up(1))
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(PAD.px()).right(100.pct() - PAD.px()),
                    top(PAD.px()).bottom(100.pct() - PAD.px()),
                )),
        );
        // The detail bubble.
        let bubble = grove.branch(
            body,
            kit::ring(1.25, theme::ink())
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(left(0.px()).width(46.px()), top(0.px()).height(46.px()))),
        );
        kit::rule(
            grove,
            bubble,
            Location::new().xs(left(0.px()).right(100.pct()), center_y(50.pct()).height(1.px())),
            theme::ink(),
        );
        let number = kit::letter(
            grove,
            bubble,
            "1",
            faces.bold,
            theme::TITLE,
            theme::ink(),
            Location::new().xs(center_x(50.pct()).width(content()), top(7.px()).height(1.letters())),
        );
        kit::letter(
            grove,
            bubble,
            "TR",
            faces.regular,
            theme::LABEL,
            theme::soft(),
            Location::new().xs(center_x(50.pct()).width(content()), top(27.px()).height(1.letters())),
        );
        let kind = kit::letter(
            grove,
            body,
            "",
            faces.bold,
            theme::HEAD,
            theme::soft(),
            Location::new().xs(left(58.px()).width(content()), top(5.px()).height(1.letters())),
        );
        let heading = kit::letter(
            grove,
            body,
            "",
            faces.bold,
            theme::TITLE + 1,
            theme::ink(),
            Location::new().xs(left(58.px()).right(100.pct() - 44.px()), top(22.px()).height(1.letters())),
        );
        let m = lichen::measure();
        let close = Chip::mark(
            grove,
            body,
            Location::new().xs(right(100.pct()).width(m.height.px()), top(0.px()).height(m.height.px())),
            Elevation::up(1),
            icons.x,
        );
        kit::rule(
            grove,
            body,
            Location::new().xs(left(0.px()).right(100.pct()), top((HEAD - 10.0).px()).height(1.px())),
            theme::rule(),
        );
        let row = |y: f32, h: f32| Location::new().xs(left(0.px()).right(100.pct()), top(y.px()).height(h.px()));
        let mut title = Field::grow(grove, body, row(HEAD, m.height), CELL, "name", false);
        title.holds(Holds::Record);
        let mut role = Field::grow(grove, body, row(HEAD, m.height), CELL, "role", false);
        role.holds(Holds::Record);
        let mut words = Field::grow(grove, body, row(HEAD, 74.0), CELL, "words", true);
        words.holds(Holds::Record);
        let day = Pick::grow(grove, body, row(HEAD, Pick::height(1)), CELL, "day", DAY_NAMES, 1);
        let length = Pick::grow(grove, body, row(HEAD, Pick::height(1)), CELL, "length", LENGTHS, 1);
        let status = Pick::grow(grove, body, row(HEAD, Pick::height(1)), CELL, "status", STATUSES, 1);
        // A meeting's time: its name in a field's cell, and a step either way.
        let time = grove.branch(body, Stem::new().intangible().elevate(Elevation::up(1)).at(row(HEAD, m.height)));
        lichen::dress(grove, time, CELL, "time");
        let time_value = kit::letter(
            grove,
            time,
            "",
            faces.bold,
            theme::TITLE,
            theme::ink(),
            Location::new().xs(left(CELL.letters() + 10.px()).width(content()), center_y(50.pct()).height(1.letters())),
        );
        let square = |n: f32| {
            Location::new().xs(
                right(100.pct() - (n * (m.height - 6.0 + 4.0) + 3.0).px()).width((m.height - 6.0).px()),
                center_y(50.pct()).height((m.height - 6.0).px()),
            )
        };
        let mut earlier = Chip::mark(grove, time, square(1.0), Elevation::up(2), icons.minus);
        let mut later = Chip::mark(grove, time, square(0.0), Elevation::up(2), icons.plus);
        earlier.wear(grove, lichen::WELL);
        later.wear(grove, lichen::WELL);
        let ties = kit::letter(
            grove,
            body,
            "",
            faces.regular,
            theme::SUB,
            theme::soft(),
            Location::new().xs(left(0.px()).right(100.pct()), top(HEAD.px()).height(content())),
        );
        // Named, all three: the press that stands in the delete's place once it is asked must not
        // read as the close beside it, as a bare cross would.
        let delete = Confirm::grow(
            grove,
            body,
            Location::new().xs(left(0.px()).width(Chip::width(ASK)), top(HEAD.px()).height(m.height.px())),
            Location::new().xs(right(100.pct()).width(Chip::width("delete for good")), top(HEAD.px()).height(m.height.px())),
            (icons.trash_2, ASK),
            (icons.arrow_left, "keep"),
            (icons.trash_2, "delete for good"),
        );
        Self {
            plate,
            bubble,
            number,
            kind,
            heading,
            close,
            title,
            role,
            words,
            day,
            length,
            status,
            time,
            time_value,
            earlier,
            later,
            ties,
            delete,
            id: None,
            open: false,
            layout: Layout::Xl,
            height: 300.0,
            typing: None,
            kind_now: None,
        }
    }

    pub fn plate(&self) -> Leaf {
        self.plate
    }

    /// Every element of the inspector that takes keys, for the sheet's own shortcuts -- not its
    /// fields, whose keys are words.
    pub fn keyed(&self) -> Vec<Leaf> {
        vec![self.plate, self.close.leaf()]
    }

    /// Where it stands open, and where it waits closed, on `layout`.
    fn spots(&self) -> (Location, Location) {
        match theme::docked(self.layout) {
            Some(width) => {
                // As tall as the record it holds, never past the scale bar.
                let h = self.extent();
                let room = 100.pct() - (theme::TOP + theme::BOTTOM).px();
                (
                    Location::new().xs(
                        right(100.pct() - theme::GUTTER.px()).width(width.px()),
                        top(theme::TOP.px()).height(h.px()).at_most(room.clone()),
                    ),
                    Location::new().xs(
                        left(100.pct() + 24.px()).width(width.px()),
                        top(theme::TOP.px()).height(h.px()).at_most(room),
                    ),
                )
            }
            None => {
                // Risen from the foot to stand on the scale bar, right of the dock where there is
                // one.
                let h = self.extent();
                let from = match theme::dock(self.layout) {
                    0.0 => theme::GUTTER,
                    dock => theme::GUTTER * 2.0 + dock,
                };
                (
                    Location::new().xs(
                        left(from.px()).right(100.pct() - theme::GUTTER.px()).at_most(560.px()),
                        foliage::bottom(100.pct() - theme::BOTTOM.px()).height(h.px()),
                    ),
                    Location::new().xs(
                        left(from.px()).right(100.pct() - theme::GUTTER.px()).at_most(560.px()),
                        top(100.pct() + 24.px()).height(h.px()),
                    ),
                )
            }
        }
    }

    /// How tall the plate stands for the record it holds.
    pub fn extent(&self) -> f32 {
        (self.height + 2.0 * PAD).round()
    }

    /// Lays the inspector out for `layout`.
    pub fn place(&mut self, grove: &mut Grove, layout: Layout) {
        self.layout = layout;
        let (here, away) = self.spots();
        grove.at(self.plate, if self.open { here } else { away });
    }

    /// Whether it stands docked down the right.
    pub fn docked(&self) -> bool {
        theme::docked(self.layout).is_some()
    }

    /// Opens on `id`'s record.
    pub fn show(&mut self, grove: &mut Grove, board: &Board, id: Id) {
        self.typing = None;
        self.id = Some(id);
        self.fill(grove, board);
        if !self.open {
            self.open = true;
            let (here, away) = self.spots();
            grove.visible(self.plate, true);
            grove.at(self.plate, away);
            grove.animate(self.plate, Motion::Location(here), Timing::ms(360).ease(foliage::Ease::Emphasis));
        } else {
            let (here, _) = self.spots();
            grove.at(self.plate, here);
        }
    }

    pub fn close(&mut self, grove: &mut Grove) {
        self.delete.disarm(grove);
        self.typing = None;
        if !self.open {
            return;
        }
        self.open = false;
        let (_, away) = self.spots();
        grove.animate(self.plate, Motion::Location(away), Timing::ms(260).ease(foliage::Ease::Accelerate));
    }

    /// Puts focus in the first field, as a second tap on a card asks.
    pub fn focus(&self, grove: &mut Grove) {
        match self.id_kind() {
            Some(Kind::Note) => self.words.focus(grove),
            Some(_) => self.title.focus(grove),
            None => {}
        }
    }

    fn id_kind(&self) -> Option<Kind> {
        self.kind_now
    }

    /// Writes the record into the fields, as the board has it.
    pub fn fill(&mut self, grove: &mut Grove, board: &Board) {
        let Some(node) = self.id.and_then(|id| board.get(id)) else {
            return;
        };
        let kind = node.kind;
        self.kind_now = Some(kind);
        let ordinal = board.nodes.iter().position(|n| n.id == node.id).unwrap_or(0) + 1;
        grove.text(self.number, ordinal.to_string());
        let what = match kind {
            Kind::Person if node.external => "PERSON · EXTERNAL",
            Kind::Person => "PERSON",
            Kind::Meeting => "MEETING",
            Kind::Spec => "SPEC",
            Kind::Milestone => "MILESTONE",
            Kind::Note => "NOTE",
        };
        grove.text(self.kind, format!("{what} · {}", node.code));
        grove.text(
            self.heading,
            theme::clip(
                &match kind {
                    Kind::Note => node.code.clone(),
                    _ => node.title.to_uppercase(),
                },
                24,
            ),
        );
        grove.text(self.title.label(), if kind == Kind::Person { "name" } else { "title" });
        // Only the fields this kind has stand, one under another.
        let m = lichen::measure();
        let mut y = HEAD;
        let mut rows: Vec<(Leaf, f32)> = Vec::new();
        match kind {
            Kind::Note => rows.push((self.words.leaf(), 74.0)),
            _ => rows.push((self.title.leaf(), m.height)),
        }
        if kind == Kind::Person {
            rows.push((self.role.leaf(), m.height));
        }
        if matches!(kind, Kind::Meeting | Kind::Milestone) {
            rows.push((self.day.leaf(), Pick::height(1)));
        }
        if kind == Kind::Meeting {
            rows.push((self.time, m.height));
            rows.push((self.length.leaf(), Pick::height(1)));
        }
        if kind == Kind::Spec {
            rows.push((self.status.leaf(), Pick::height(1)));
        }
        let all = [
            self.title.leaf(),
            self.role.leaf(),
            self.words.leaf(),
            self.day.leaf(),
            self.length.leaf(),
            self.status.leaf(),
            self.time,
        ];
        for leaf in all {
            grove.visible(leaf, rows.iter().any(|(l, _)| *l == leaf));
        }
        for (leaf, h) in &rows {
            grove.at(*leaf, Location::new().xs(left(0.px()).right(100.pct()), top(y.px()).height(h.px())));
            y += h + (ROW - m.height);
        }
        grove.at(
            self.ties,
            Location::new().xs(left(2.px()).right(100.pct()), top((y + 2.0).px()).height(content())),
        );
        grove.text(self.ties, board.ties(node.id));
        let delete_y = y + 40.0;
        let ask = self.delete.ask().leaf();
        let abort = self.delete.abort().leaf();
        let commit = self.delete.commit().leaf();
        for leaf in [ask, abort] {
            grove.at(leaf, Location::new().xs(left(0.px()).width(Chip::width(ASK)), top(delete_y.px()).height(m.height.px())));
        }
        grove.at(
            commit,
            Location::new().xs(right(100.pct()).width(Chip::width("delete for good")), top(delete_y.px()).height(m.height.px())),
        );
        self.delete.open(grove, true);
        self.height = delete_y + m.height;
        // The record itself.
        self.title.put(grove, &node.title);
        self.role.put(grove, &node.role);
        self.words.put(grove, &node.title);
        let day = match kind {
            Kind::Milestone => node.day,
            _ => node.slot.map(|s| s.day).unwrap_or(0),
        };
        self.day.set(grove, day as usize, Origin::Imported);
        if let Some(slot) = node.slot {
            self.length.set(grove, (slot.len as usize).clamp(1, 4) - 1, Origin::Imported);
            grove.text(self.time_value, format!("{}–{}", clock(slot.start), clock(slot.end())));
        }
        self.status.set(grove, node.status.index(), Origin::Imported);
        if self.open && !self.docked() {
            let (here, _) = self.spots();
            grove.at(self.plate, here);
        }
    }

    /// Says the record is filed: what was changed reads in ink again.
    pub fn filed(&mut self, grove: &mut Grove, board: &Board) {
        if self.typing.is_some() {
            return;
        }
        let Some(node) = self.id.and_then(|id| board.get(id)) else {
            return;
        };
        for (field, value) in [(&mut self.title, &node.title), (&mut self.role, &node.role), (&mut self.words, &node.title)] {
            if field.origin() == Origin::Typed {
                field.put(grove, value);
            }
        }
    }

    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen, board: &mut Board, history: &mut History) -> Vec<Asked> {
        let mut asked = Vec::new();
        if !self.open {
            return asked;
        }
        if self.close.pressed(pollen) {
            asked.push(Asked::Close);
            return asked;
        }
        let Some(id) = self.id else {
            return asked;
        };
        let Some(code) = board.get(id).map(|n| n.code.clone()) else {
            return asked;
        };
        // Words, written to the card as they are typed; the first edit of a stretch of typing is
        // kept for undoing.
        let fields: [(usize, bool); 3] = [
            (0, self.title.frame(grove, pollen)),
            (1, self.role.frame(grove, pollen)),
            (2, self.words.frame(grove, pollen)),
        ];
        for (n, edited) in fields {
            if !edited {
                continue;
            }
            if self.typing != Some((id, n)) {
                let what = match n {
                    1 => format!("re-roled {code}"),
                    _ => format!("renamed {code}"),
                };
                history.keep(board, what);
                self.typing = Some((id, n));
            }
            let value = match n {
                0 => self.title.value(grove),
                1 => self.role.value(grove),
                _ => self.words.value(grove),
            };
            if let Some(node) = board.get_mut(id) {
                match n {
                    1 => node.role = value,
                    _ => node.title = value,
                }
            }
            asked.push(Asked::Edited);
        }
        for field in [self.title.input(), self.role.input(), self.words.input()] {
            if pollen.unfocused(field) {
                self.typing = None;
            }
        }
        // Choices.
        let kind = board.get(id).map(|n| n.kind);
        if self.day.frame(grove, pollen) {
            history.keep(board, format!("re-dated {code}"));
            let day = self.day.index() as u8;
            if let Some(node) = board.get_mut(id) {
                match kind {
                    Some(Kind::Milestone) => node.day = day,
                    _ => {
                        if let Some(slot) = &mut node.slot {
                            slot.day = day;
                        }
                    }
                }
            }
            asked.push(Asked::Edited);
        }
        if self.length.frame(grove, pollen) {
            history.keep(board, format!("re-timed {code}"));
            let len = self.length.index() as u8 + 1;
            if let Some(slot) = board.get_mut(id).and_then(|n| n.slot.as_mut()) {
                slot.len = len;
                if slot.end() > crate::board::HALVES {
                    slot.start = crate::board::HALVES - len;
                }
            }
            asked.push(Asked::Edited);
        }
        if self.status.frame(grove, pollen) {
            history.keep(board, format!("{code} to {}", STATUSES[self.status.index()]));
            if let Some(node) = board.get_mut(id) {
                node.status = Status::ALL[self.status.index()];
            }
            asked.push(Asked::Edited);
        }
        for (chip, step) in [(&self.earlier, -1i32), (&self.later, 1)] {
            if !chip.pressed(pollen) {
                continue;
            }
            let Some(slot) = board.get(id).and_then(|n| n.slot) else {
                continue;
            };
            let start = (slot.start as i32 + step).clamp(0, (crate::board::HALVES - slot.len) as i32) as u8;
            if start != slot.start {
                history.keep(board, format!("re-timed {code}"));
                if let Some(node) = board.get_mut(id) {
                    node.slot = Some(Slot::new(slot.day, start, slot.len));
                }
                asked.push(Asked::Edited);
            }
        }
        if asked.contains(&Asked::Edited) {
            if let Some(slot) = board.get(id).and_then(|n| n.slot) {
                grove.text(self.time_value, format!("{}–{}", clock(slot.start), clock(slot.end())));
            }
            grove.text(self.ties, board.ties(id));
            if let Some(node) = board.get(id) {
                let heading = match node.kind {
                    Kind::Note => node.code.clone(),
                    _ => node.title.to_uppercase(),
                };
                grove.text(self.heading, theme::clip(&heading, 24));
            }
        }
        if let Some(Step::Confirmed) = self.delete.frame(grove, pollen) {
            asked.push(Asked::Delete(id));
        }
        asked
    }
}
