//! Clashes, marked up the way a checker marks a drawing: a ruled box in red pencil round each of
//! the two meetings someone is due at together, a revision delta on the corner of the one that
//! has to move, and a note beside it saying who and when, with the fix -- the next slot everyone
//! is free -- offered as a press.
//!
//! A box is four straight strokes standing a little outside its card, ruled in one after another
//! from the top left, clockwise, and taken back the other way when the clash is gone. Its ends are
//! fractions of the box's own frame, which is anchored to the meeting's card, so a box follows its
//! card wherever it is dragged and whatever scale the sheet is read at.

use std::f32::consts::PI;

use foliage::{
    Boxed, Cap, Ease, Elevation, Grove, Grow, Leaf, Line, Location, Motion, Panel, Place, Point, Polygon, Pollen,
    Source, Stem, Text, Timing, Trace, anchor, center_x, center_y, content, left, top,
};
use lichen::{Chip, Press};

use crate::board::{Board, Clash, Id, Kind, Slot};
use crate::icons::Icons;
use crate::kit;
use crate::theme::{self, Faces};

/// How far a box stands outside its card, and how heavy its strokes are.
pub const PAD: f32 = 10.0;
const WEIGHT: f32 = 1.5;
/// How large the revision delta on the box's corner is.
const DELTA: f32 = 26.0;
/// How long a box takes to rule, all four sides.
const RULE_MS: u64 = 480;
/// How many clashes are marked up at once. More are counted on the rail, and marked as these are
/// fixed.
pub const SHOWN: usize = 3;
/// How far left of its card a box's markup reaches: the delta, centred on the corner.
pub const REACH: f32 = PAD + DELTA / 2.0;

/// The room a box takes round its card, `(left, top, right, bottom)` in the window's pixels: the
/// strokes stand `PAD + WEIGHT` out, and the delta, where there is one, reaches further up and
/// left from its corner.
pub fn room(delta: bool) -> (f32, f32, f32, f32) {
    match delta {
        true => (REACH, PAD + DELTA * 0.75, SIDE, SIDE),
        false => (SIDE, SIDE, SIDE, SIDE),
    }
}
/// The callout's size, and how far right of the card it moves it stands.
pub const CALLOUT: (f32, f32) = (228.0, 106.0);
pub const CALLOUT_GAP: f32 = 22.0;
/// How far a box's strokes stand outside its card.
pub const SIDE: f32 = PAD + WEIGHT;
/// How far under the card a callout stands where there is no room beside it: hung just under
/// the box, as wide as it, so the two read as one mark.
pub const UNDER_GAP: f32 = SIDE + 6.0;

/// Whether a callout goes under its card rather than beside it, with `clear` of the window's width
/// to stand in: beside it wherever card, gap and callout fit with a margin, and under it where they
/// would run off the window's edge.
pub fn under(clear: f32) -> bool {
    Kind::Meeting.footprint().0 + CALLOUT_GAP + CALLOUT.0 + 32.0 > clear
}

/// Where a callout stands against the card it moves: beside it, or under it.
fn callout_at(under: bool) -> Location {
    match under {
        false => Location::new().xs(
            left(anchor().right() + CALLOUT_GAP.px()).width(CALLOUT.0.px()),
            top(anchor().top()).height(CALLOUT.1.px()),
        ),
        true => Location::new().xs(
            left(anchor().left() - SIDE.px()).right(anchor().right() + SIDE.px()),
            top(anchor().bottom() + UNDER_GAP.px()).height(CALLOUT.1.px()),
        ),
    }
}

/// The four sides of a box, clockwise from the top left, each as the fractions of the box it runs
/// from and to.
pub fn sides() -> [((f32, f32), (f32, f32)); 4] {
    [((0.0, 0.0), (1.0, 0.0)), ((1.0, 0.0), (1.0, 1.0)), ((1.0, 1.0), (0.0, 1.0)), ((0.0, 1.0), (0.0, 0.0))]
}

/// A point of a box's frame, from fractions of it.
fn corner((x, y): (f32, f32)) -> Point {
    Point::new((x * 100.0).pct(), (y * 100.0).pct())
}

/// A red-pencil box, grown once and moved onto whichever card it marks.
struct Mark {
    root: Leaf,
    strokes: [Leaf; 4],
    delta: Leaf,
    letter: Leaf,
    drawn: bool,
}

impl Mark {
    fn grow(grove: &mut Grove, paper: Leaf, faces: &Faces) -> Self {
        let root = grove.branch(
            paper,
            Stem::new()
                .intangible()
                .elevate(Elevation::up(7))
                .visible(false)
                .at(Location::new().xs(
                    left(anchor().left() - PAD.px()).right(anchor().right() + PAD.px()),
                    top(anchor().top() - PAD.px()).bottom(anchor().bottom() + PAD.px()),
                )),
        );
        let strokes = sides().map(|(from, _)| {
            grove.branch(
                root,
                Line::new()
                    .between(corner(from), corner(from))
                    .weight(WEIGHT)
                    .color(theme::pencil())
                    .cap(Cap::Round)
                    .intangible()
                    .opacity(0.0),
            )
        });
        // The delta sits on the top left corner, its base on the top side and point up -- a
        // triangle at rest points down. Drawn in a circle `DELTA` across, its base stands a
        // quarter of that below the middle.
        let delta = grove.branch(
            root,
            Polygon::new()
                .sides(3.0)
                .rotation(PI)
                .rounding(0.08)
                .color(theme::pencil())
                .intangible()
                .elevate(Elevation::up(1))
                .opacity(0.0)
                .at(Location::new().xs(
                    left((-DELTA / 2.0).px()).width(DELTA.px()),
                    top((-DELTA * 0.75).px()).height(DELTA.px()),
                )),
        );
        // Lettered in the paper's colour, a little under the triangle's middle -- which, for an
        // equilateral triangle, is the middle of the circle it is drawn in.
        let letter = grove.branch(
            root,
            Text::new("")
                .font(faces.bold)
                .font_size(theme::size(theme::LABEL))
                .color(theme::paper())
                .intangible()
                .elevate(Elevation::up(2))
                .opacity(0.0)
                .at(Location::new().xs(center_x(0.px()).width(content()), center_y((-DELTA / 4.0 + 1.5).px()).height(1.letters()))),
        );
        Self {
            root,
            strokes,
            delta,
            letter,
            drawn: false,
        }
    }

    /// Rules the box round `card`, a side at a time, starting `after` ms from now; with a delta
    /// lettered `rev` on its corner, if there is one.
    fn draw(&mut self, grove: &mut Grove, card: Leaf, after: u64, rev: Option<&str>) {
        grove.anchor(self.root, card);
        grove.visible(self.root, true);
        self.drawn = true;
        let each = RULE_MS / 4;
        for (n, (&stroke, (from, to))) in self.strokes.iter().zip(sides()).enumerate() {
            let at = after + n as u64 * each;
            grove.between(stroke, corner(from), corner(from));
            grove.opacity(stroke, 0.0);
            grove.animate(stroke, Motion::Opacity(1.0), Timing::ms(40).after(at));
            grove.animate(stroke, Motion::Trace(Trace::new().xs(corner(from), corner(to))), Timing::ms(each).after(at));
        }
        grove.opacity(self.delta, 0.0);
        grove.opacity(self.letter, 0.0);
        if let Some(rev) = rev {
            grove.text(self.letter, rev);
            for leaf in [self.delta, self.letter] {
                grove.animate(leaf, Motion::Opacity(1.0), Timing::ms(180).after(after + RULE_MS));
            }
        }
    }

    /// Letters the delta for the revision a fix would be filed as.
    fn revise(&self, grove: &mut Grove, rev: &str) {
        grove.text(self.letter, rev);
    }

    /// Takes the box back the way it was ruled, last side first.
    fn retract(&mut self, grove: &mut Grove) {
        if !self.drawn {
            return;
        }
        self.drawn = false;
        let each = RULE_MS / 5;
        for leaf in [self.delta, self.letter] {
            grove.animate(leaf, Motion::Opacity(0.0), Timing::ms(120));
        }
        for (n, (&stroke, (from, _))) in self.strokes.iter().zip(sides()).rev().enumerate() {
            let at = n as u64 * each;
            grove.animate(stroke, Motion::Trace(Trace::new().xs(corner(from), corner(from))), Timing::ms(each).after(at));
            grove.animate(stroke, Motion::Opacity(0.0), Timing::ms(40).after(at + each - 20));
        }
    }
}

/// Where a clash's fix has got to.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Fix {
    /// A free slot is offered.
    Ready(Option<Slot>),
    /// The server is being asked.
    Asking,
    /// The server said no, and offered this instead.
    Offered(Option<Slot>),
}

struct Markup {
    /// Round the meeting kept, and round the one that moves.
    boxes: [Mark; 2],
    callout: Leaf,
    words: Leaf,
    detail: Leaf,
    fix: Chip,
    clash: Option<Clash>,
    state: Fix,
}

/// What the markup asks of the sheet.
#[derive(Clone, Debug, PartialEq)]
pub enum Asked {
    /// Move `meeting` to `slot`, as clash `n`'s fix.
    Fix { n: usize, meeting: Id, slot: Slot },
}

pub struct Clashes {
    marks: Vec<Markup>,
    pub all: Vec<Clash>,
    /// Whether the sheet is read as an overview, where only the boxes are drawn.
    overview: bool,
    /// Whether the callouts stand under their cards, for want of room beside them.
    under: bool,
    /// How long the next marking waits before it starts: long enough, as the sheet is first
    /// issued, for the cards to be there to mark.
    pub lead: u64,
}

impl Clashes {
    pub fn grow(grove: &mut Grove, paper: Leaf, faces: &Faces, icons: &Icons) -> Self {
        let m = lichen::measure();
        let marks = (0..SHOWN)
            .map(|_| {
                let boxes = [Mark::grow(grove, paper, faces), Mark::grow(grove, paper, faces)];
                let callout = grove.branch(
                    paper,
                    Panel::new()
                        .color(theme::paper())
                        .outline(1.25, theme::pencil())
                        .intangible()
                        .elevate(Elevation::up(14))
                        .font_size(lichen::caption())
                        .visible(false)
                        .at(callout_at(false)),
                );
                kit::letter(
                    grove,
                    callout,
                    "CLASH",
                    faces.bold,
                    theme::LABEL + 1,
                    theme::pencil(),
                    Location::new().xs(left(10.px()).width(content()), top(9.px()).height(1.letters())),
                );
                let words = kit::letter(
                    grove,
                    callout,
                    "",
                    faces.bold,
                    theme::TITLE,
                    theme::pencil(),
                    Location::new().xs(left(10.px()).width(content()), top(24.px()).height(1.letters())),
                );
                let detail = kit::letter(
                    grove,
                    callout,
                    "",
                    faces.regular,
                    theme::SUB,
                    theme::faint(),
                    Location::new().xs(left(10.px()).width(content()), top(41.px()).height(1.letters())),
                );
                let fix = Chip::grow(
                    grove,
                    callout,
                    Location::new().xs(
                        left(10.px()).width(Chip::width("move to 14:00")),
                        top((CALLOUT.1 - m.height - 8.0).px()).height(m.height.px()),
                    ),
                    Elevation::up(1),
                    icons.clock,
                    "move to 14:00",
                );
                Markup {
                    boxes,
                    callout,
                    words,
                    detail,
                    fix,
                    clash: None,
                    state: Fix::Ready(None),
                }
            })
            .collect();
        Self {
            marks,
            all: Vec::new(),
            overview: false,
            under: false,
            lead: 0,
        }
    }

    /// The meetings in a clash, and the attendances that double-book someone.
    pub fn involved(&self, board: &Board) -> (Vec<Id>, Vec<(Id, Id)>) {
        let mut meetings = Vec::new();
        let mut wires = Vec::new();
        for clash in &self.all {
            for m in [clash.keeps, clash.moves] {
                if !meetings.contains(&m) {
                    meetings.push(m);
                }
                wires.push((clash.person, m));
            }
        }
        let _ = board;
        (meetings, wires)
    }

    /// Marks up what clashes on `board` now: boxes ruled for new clashes, and taken back for those
    /// that are gone. `card` says where each meeting's card is, and `rev` is the revision a fix
    /// would be filed as, which the deltas are lettered with.
    pub fn sync(&mut self, grove: &mut Grove, board: &Board, rev: &str, card: impl Fn(Id) -> Option<Leaf>) {
        self.all = board.clashes();
        let all = self.all.clone();
        for mark in &self.marks {
            mark.boxes[1].revise(grove, rev);
        }
        // Clashes no longer there are taken back.
        for mark in &mut self.marks {
            let gone = mark.clash.as_ref().is_some_and(|c| !all.contains(c));
            let unseen = mark.clash.as_ref().is_some_and(|c| card(c.moves).is_none() || card(c.keeps).is_none());
            if gone || unseen {
                mark.clash = None;
                for mark in &mut mark.boxes {
                    mark.retract(grove);
                }
                grove.animate(mark.callout, Motion::Opacity(0.0), Timing::ms(240).ease(Ease::Accelerate));
            }
        }
        // And new ones marked up, as far as there are boxes for them.
        let mut delay = std::mem::take(&mut self.lead);
        for clash in &all {
            if self.marks.iter().any(|m| m.clash.as_ref() == Some(clash)) {
                continue;
            }
            let (Some(keeps), Some(moves)) = (card(clash.keeps), card(clash.moves)) else {
                continue;
            };
            let Some(mark) = self.marks.iter_mut().find(|m| m.clash.is_none()) else {
                break;
            };
            mark.clash = Some(clash.clone());
            mark.boxes[0].draw(grove, keeps, delay, None);
            mark.boxes[1].draw(grove, moves, delay + 250, Some(rev));
            grove.anchor(mark.callout, moves);
            grove.visible(mark.callout, !self.overview);
            grove.opacity(mark.callout, 0.0);
            grove.animate(mark.callout, Motion::Opacity(1.0), Timing::ms(260).after(delay + 800));
            grove.text(mark.words, board.clash_words(clash));
            let slot = board.suggest(clash.moves, &[]);
            Self::offer(grove, mark, board, Fix::Ready(slot));
            delay += 400;
        }
    }

    /// Whether the callouts stand under their cards rather than beside them.
    pub fn under(&self) -> bool {
        self.under
    }

    /// Stands the callouts under their cards or beside them. Nothing, if they already do.
    pub fn place(&mut self, grove: &mut Grove, under: bool) {
        if self.under == under {
            return;
        }
        self.under = under;
        for mark in &self.marks {
            grove.at(mark.callout, callout_at(under));
        }
    }

    /// Reads the markup as an overview or not: an overview keeps the boxes and lets the
    /// callouts go, since they would take twice the sheet their meetings do.
    pub fn overview(&mut self, grove: &mut Grove, overview: bool) {
        if self.overview == overview {
            return;
        }
        self.overview = overview;
        for mark in &self.marks {
            if mark.clash.is_some() {
                grove.visible(mark.callout, !overview);
            }
        }
    }

    fn offer(grove: &mut Grove, mark: &mut Markup, board: &Board, state: Fix) {
        mark.state = state;
        let title = mark
            .clash
            .as_ref()
            .and_then(|c| board.get(c.moves))
            .map(|n| n.title.clone())
            .unwrap_or_default();
        match state {
            Fix::Ready(Some(slot)) | Fix::Offered(Some(slot)) => {
                grove.text(mark.fix.label(), format!("move to {}", crate::board::clock(slot.start)));
                let words = match state {
                    Fix::Offered(_) => String::new(),
                    _ => format!("moves {}", theme::clip(&title, 22)),
                };
                if !words.is_empty() {
                    grove.text(mark.detail, words);
                    grove.color(mark.detail, theme::faint());
                }
                mark.fix.arm(grove, if matches!(state, Fix::Offered(_)) { Press::Armed } else { Press::Rest });
            }
            Fix::Ready(None) | Fix::Offered(None) => {
                grove.text(mark.fix.label(), "no free slot");
                grove.text(mark.detail, "the week is full for them");
                mark.fix.arm(grove, Press::Inert);
            }
            Fix::Asking => {
                grove.text(mark.fix.label(), "checking…");
                grove.text(mark.detail, "asking everyone's calendars…");
                grove.color(mark.detail, theme::faint());
                mark.fix.arm(grove, Press::Inert);
            }
        }
    }

    /// The server's answer to clash `n`'s fix: done -- the clash goes once the board is moved --
    /// or refused, with what it offers instead.
    pub fn answered(&mut self, grove: &mut Grove, board: &Board, n: usize, refused: Option<(String, Option<Slot>)>) {
        let Some(mark) = self.marks.get_mut(n) else {
            return;
        };
        match refused {
            None => {
                grove.text(mark.fix.label(), "moved");
                grove.text(mark.detail, "rescheduled · filing revision…");
            }
            Some((why, instead)) => {
                Self::offer(grove, mark, board, Fix::Offered(instead));
                grove.text(mark.detail, theme::clip(&why, 30));
                grove.color(mark.detail, theme::marked());
            }
        }
    }

    /// Says clash `n`'s fix is being asked for.
    pub fn asking(&mut self, grove: &mut Grove, board: &Board, n: usize) {
        if let Some(mark) = self.marks.get_mut(n) {
            Self::offer(grove, mark, board, Fix::Asking);
        }
    }

    pub fn frame(&mut self, pollen: &Pollen) -> Vec<Asked> {
        let mut asked = Vec::new();
        for (n, mark) in self.marks.iter().enumerate() {
            if mark.fix.pressed(pollen)
                && let Some(clash) = &mark.clash
                && let Fix::Ready(Some(slot)) | Fix::Offered(Some(slot)) = mark.state
            {
                asked.push(Asked::Fix {
                    n,
                    meeting: clash.moves,
                    slot,
                });
            }
        }
        asked
    }
}

#[cfg(test)]
mod tests {
    use super::sides;

    #[test]
    fn a_box_goes_once_round_clockwise_without_a_gap() {
        let sides = sides();
        for n in 0..4 {
            let (_, end) = sides[n];
            let (start, _) = sides[(n + 1) % 4];
            assert_eq!(end, start, "a gap after side {n}");
        }
        assert_eq!(sides[0].0, (0.0, 0.0), "it starts at the top left");
        // Clockwise, on a frame whose y runs down: right, down, left, up.
        let headings: Vec<(f32, f32)> = sides.iter().map(|(a, b)| (b.0 - a.0, b.1 - a.1)).collect();
        assert_eq!(headings, vec![(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)]);
    }

    #[test]
    fn every_side_is_straight_along_an_edge() {
        for (a, b) in sides() {
            assert!(a.0 == b.0 || a.1 == b.1, "{a:?} to {b:?} is not straight");
            for (x, y) in [a, b] {
                assert!((x == 0.0 || x == 1.0) && (y == 0.0 || y == 1.0), "{x},{y} is not a corner");
            }
        }
    }
}
