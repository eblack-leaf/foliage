//! The `motion` section.
//!
//! Named `anim` rather than `motion` only because `site::motion` is already the site's own
//! timing tokens; the page it builds is titled `motion` like every other rail entry.
//!
//! Five boards, one per decision an animation is made of: the curve, the two numbers, which
//! value is moved, how many times, and what the group reports when it is done.
//!
//! Every step here *runs* rather than setting a state, which is the one thing these boards do
//! differently from the rest of the site and is forced by the subject: a motion that has
//! already happened leaves nothing in the tree to look at. So a step has to be pressable twice
//! and mean it twice, and it has to survive being pressed again mid-flight -- a demo of
//! animation that comes apart when you are impatient with it is demonstrating the wrong thing.
//! What makes that work is that nothing here ever *writes* a value a tween owns; see
//! [`far_end`], which is where the one exception tried to live.

use foliage::{
    Canopy, Color, Ease, Elevation, GridExt, Grows, Leaf, Location, Motion, Panel, Polygon, Repeat,
    Rounding, Sprout, Timing,
};

use crate::site::blueprint::{self, Blueprint};
use crate::site::copy::{board, headings, motion as text, reference};
use crate::site::{Column, Demo, Grow, SCROLL_TAIL, role};

const STAGE_H: (i32, i32, i32) = (150, 165, 190);

pub(crate) fn build(g: &mut Grow, slot: Leaf) {
    let container = crate::site::shell::content_area(g.canopy, slot);
    let mut column = Column::new(g.canopy, container);

    column.display(g.canopy, headings::MOTION);
    column.lead(g.canopy, text::LEAD);

    column.heading(g.canopy, headings::MOTION_EASE);
    column.prose(g.canopy, text::EASE);
    ease(g, &mut column);

    column.heading(g.canopy, headings::MOTION_TIMING);
    column.prose(g.canopy, text::TIMING);
    timing(g, &mut column);

    column.heading(g.canopy, headings::MOTION_TWEENS);
    column.prose(g.canopy, text::TWEENS);
    tweens(g, &mut column);

    column.heading(g.canopy, headings::MOTION_REPEAT);
    column.prose(g.canopy, text::REPEAT);
    repeat(g, &mut column);

    column.heading(g.canopy, headings::MOTION_SEQUENCE);
    column.prose(g.canopy, text::SEQUENCE);
    sequence(g, &mut column);

    column.tail(g.canopy, SCROLL_TAIL);
}

// ---- the travelling shape --------------------------------------------------------------------

/// Where a traveller rests, and where it is sent, as centres across the stage. Far enough
/// apart that the trip is plainly a trip and not a nudge, and inside the stage at both ends
/// once the shape's own half-width is counted.
const HOME: f32 = 16.0;
const AWAY: f32 = 84.0;
const TRAVELLER: i32 = 40;

fn travel_at(center_pct: f32) -> Location {
    Location::new().xs(
        center_pct
            .pct()
            .as_center_x()
            .with(TRAVELLER.px().as_width()),
        50.pct()
            .as_center_y()
            .with(TRAVELLER.px().as_height()),
    )
}

/// The line the shape runs along, drawn once behind it.
///
/// Without it a shape crossing an empty field is something wandering; with it the same move is
/// a distance being covered, and the difference between two curves over that distance is what
/// three of these boards are asking you to look at.
fn track(canopy: &mut Canopy, stage: Leaf) {
    canopy.branch(
        stage,
        Panel::new()
            .color(role::outline())
            .rounding(Rounding::None)
            .at(Location::new().xs(
                HOME.pct().as_left().with(AWAY.pct().as_right()),
                50.pct().as_center_y().with(2.px().as_height()),
            ))
            .elevate(Elevation::up(1))
            .pass_through(),
    );
}

/// The shape three of the boards move: resting at [`HOME`], out of the hit test like every
/// other drawn thing on a board.
fn traveller(canopy: &mut Canopy, stage: Leaf) -> Leaf {
    canopy.branch(
        stage,
        Polygon::new()
            .sides(6.0)
            .rounding(0.3)
            .rotation(0.0)
            .color(role::accent())
            .at(travel_at(HOME))
            .elevate(Elevation::up(2))
            .pass_through(),
    )
}

/// The beat between the press and the motion, and the reason every tween on this page is
/// declared with a start rather than from zero.
///
/// A step that begins moving in the same instant it is pressed gives you nothing to compare
/// against: the eye is still on the button, and the first part of the curve -- which on half of
/// these is the whole difference between two of them -- is already spent. The pause puts the
/// shape's starting position on screen, at rest, before it goes.
///
/// Declared into the tween as a delay rather than staged behind a timer here, so what the boards
/// report as their `start` is the number actually handed to the engine.
pub(crate) const LEAD_IN: u64 = 200;

/// A tween that waits [`LEAD_IN`], then runs for `length`.
fn after_lead(length: u64) -> Timing {
    Timing::over(LEAD_IN + length).after(LEAD_IN)
}

/// Sends the shape to whichever end it is currently further from.
///
/// The alternative -- snap it back to [`HOME`] and tween it out again -- is what these boards
/// did first, and it is broken in a way worth writing down, because nothing about the call
/// looks wrong. A resolved box is `declaration + Diff * animation_percent`
/// (`grid/location.rs:396`); a `Location` tween sets `CreateDiff(true)` once, at its first
/// tick, and then drives `animation_percent` from 1 to 0. A plain `canopy.location` write
/// triggers a resolve but never sets `CreateDiff`, so a reset landing mid-tween resolves the
/// new declaration against the *previous* tween's cached diff at whatever percent it is
/// currently at -- putting the shape a fraction of a full travel off the end of the track.
/// The tween that follows then captures its own diff from that displaced box, so every extra
/// click compounds the error instead of clearing it.
///
/// Reading the current end and tweening to the other one has no reset in it at all. Only
/// animations ever touch the `Location`, which is the arrangement the diff is built for: an
/// interrupted tween is superseded and the new one picks up from where the shape actually is,
/// which is the documented behaviour rather than something worked around here. Pressing the
/// same step twice is a return trip, and pressing one mid-flight turns the shape around.
/// Read off the tree rather than remembered, because the shape does not always finish where
/// the step that sent it said it would: `bounce` ends back at its origin, and `forever` is
/// wherever the loop happens to be. A remembered end would send those two nowhere.
fn far_end(canopy: &Canopy, stage: Leaf, shape: Leaf) -> f32 {
    let (Some(stage), Some(shape)) = (canopy.section(stage), canopy.section(shape)) else {
        // Nothing has resolved yet, so the shape is still at its declared HOME.
        return AWAY;
    };
    if stage.width() <= 0.0 {
        return AWAY;
    }
    // Both are screen boxes, so the shape's centre has to come back into the stage's own
    // percentages before it can be compared with HOME and AWAY.
    let center = (shape.left() + shape.width() / 2.0 - stage.left()) / stage.width() * 100.0;
    if center > (HOME + AWAY) / 2.0 { HOME } else { AWAY }
}

/// Sends the shape to whichever end it is currently further from.
fn travel(canopy: &mut Canopy, stage: Leaf, shape: Leaf, timing: Timing) {
    let to = far_end(canopy, stage, shape);
    canopy.animate(shape, Motion::Location(travel_at(to)), timing);
}

// ---- easing ------------------------------------------------------------------------------

/// One per [`board::EASE_STEPS`], in order.
const EASE_CURVES: [Ease; 4] = [
    Ease::Linear,
    Ease::DECELERATE,
    Ease::ACCELERATE,
    Ease::EMPHASIS,
];
/// Held the same across every step. The curve is the only thing this board changes, and a
/// window that moved with it would leave two things to account for and no way to tell which
/// one you were seeing.
const EASE_MS: u64 = 700;

struct EaseDemo {
    board: Blueprint,
    shape: Leaf,
}

fn ease(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::EASE_ROWS,
        &board::EASE_STEPS,
        &reference::EASE,
    );
    track(g.canopy, board.stage);
    let shape = traveller(g.canopy, board.stage);
    let [curve, reads] = board::EASE_VALUES[0];
    board.set(g.canopy, 0, curve);
    board.set(g.canopy, 1, reads);
    g.page.demos.push(Box::new(EaseDemo { board, shape }));
}

impl Demo for EaseDemo {
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(canopy, step);
        travel(
            canopy,
            self.board.stage,
            self.shape,
            after_lead(EASE_MS).eased(EASE_CURVES[step].clone()),
        );
        let [curve, reads] = board::EASE_VALUES[step];
        self.board.set(canopy, 0, curve);
        self.board.set(canopy, 1, reads);
        true
    }
}

// ---- timing ------------------------------------------------------------------------------

/// `(start, finish)` per [`board::TIMING_STEPS`], in milliseconds from the sequence's own
/// beginning. The third is the pair the section is really about: it waits 600 and then runs
/// for 800, because `finish` is an offset and not a length.
///
/// The first two are [`LEAD_IN`] and a length, spelled out rather than built with
/// [`after_lead`] -- this is the one board whose readout is these numbers, so the table it
/// reads from should be the table you can compare against the strings in `copy`.
const TIMING_WINDOWS: [(u64, u64); 3] = [
    (LEAD_IN, LEAD_IN + 220),
    (LEAD_IN, LEAD_IN + 900),
    (600, 1400),
];

struct TimingDemo {
    board: Blueprint,
    shape: Leaf,
}

fn timing(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::TIMING_ROWS,
        &board::TIMING_STEPS,
        &reference::TIMING,
    );
    track(g.canopy, board.stage);
    let shape = traveller(g.canopy, board.stage);
    let [start, finish] = board::TIMING_VALUES[0];
    board.set(g.canopy, 0, start);
    board.set(g.canopy, 1, finish);
    g.page.demos.push(Box::new(TimingDemo { board, shape }));
}

impl Demo for TimingDemo {
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(canopy, step);
        let (start, finish) = TIMING_WINDOWS[step];
        travel(
            canopy,
            self.board.stage,
            self.shape,
            Timing::over(finish).after(start).eased(Ease::DECELERATE),
        );
        let [start, finish] = board::TIMING_VALUES[step];
        self.board.set(canopy, 0, start);
        self.board.set(canopy, 1, finish);
        true
    }
}

// ---- what tweens ---------------------------------------------------------------------------

const TWEENS_MS: u64 = 600;
/// The ends each step alternates its own value between: the shape it starts as, and the tone,
/// alpha and side count the board sends it to. Rounding is held across both, so the shape step
/// moves one number and not two.
const TWEENS_SIDES: f32 = 6.0;
const TWEENS_ROUNDING: f32 = 0.3;
const TWEENS_FADED: f32 = 0.2;

fn tween_tone() -> Color {
    Color::rose(400)
}

struct TweensDemo {
    board: Blueprint,
    shape: Leaf,
    /// Which way each step sends its own value next, one per [`board::TWEENS_STEPS`].
    ///
    /// Per step rather than one flag for the board: the four steps move four unrelated values,
    /// and a shared flag would have a press on `color` answering for what `opacity` did last --
    /// sending a value to the end it is already at, which is a press that does nothing.
    ///
    /// The move step's own entry is unused; where its box currently is is read off the tree
    /// instead, by [`far_end`].
    flipped: [bool; 4],
}

fn tweens(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::TWEENS_ROWS,
        &board::TWEENS_STEPS,
        &reference::TWEENS,
    );
    track(g.canopy, board.stage);
    let shape = traveller(g.canopy, board.stage);
    let [motion, tween] = board::TWEENS_VALUES[0];
    board.set(g.canopy, 0, motion);
    board.set(g.canopy, 1, tween);
    g.page.demos.push(Box::new(TweensDemo {
        board,
        shape,
        flipped: [false; 4],
    }));
}

fn tween_shape(sides: f32) -> Polygon {
    Polygon {
        sides,
        rounding: TWEENS_ROUNDING,
        rotation: 0.0,
    }
}

impl Demo for TweensDemo {
    /// Each step sends its own value to the end it is not at, and nothing is written back
    /// first. A press is a one-way trip, so the readout's `1.0 <-> 0.2` is what actually
    /// happens: press once and the shape fades, press again and it comes back.
    ///
    /// The reason it is not a reset plus a round trip -- which is what a demo of a single
    /// motion wants to be -- is that a reset is a write to a value a tween owns, and pressing
    /// again before the last one landed is exactly when that goes wrong. See [`far_end`].
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(canopy, step);
        let timing = after_lead(TWEENS_MS).eased(Ease::DECELERATE);
        let back = self.flipped[step];
        self.flipped[step] = !back;
        let to = match step {
            1 => Motion::Color(if back { role::accent() } else { tween_tone() }),
            2 => Motion::Location(travel_at(far_end(canopy, self.board.stage, self.shape))),
            3 => Motion::Polygon(tween_shape(if back { TWEENS_SIDES } else { 3.0 })),
            _ => Motion::Opacity(if back { 1.0 } else { TWEENS_FADED }),
        };
        canopy.animate(self.shape, to, timing);
        let [motion, tween] = board::TWEENS_VALUES[step];
        self.board.set(canopy, 0, motion);
        self.board.set(canopy, 1, tween);
        true
    }
}

// ---- repeat --------------------------------------------------------------------------------

/// One pass, so four of them is a little over two seconds -- long enough that `forever` reads
/// as a loop rather than as a stutter.
const REPEAT_MS: u64 = 600;

/// `(repeat, backtrack)` per [`board::REPEAT_STEPS`], in order.
///
/// `Times(1)` is two passes: the count is replays *after* the first. `forever` backtracks for
/// the same reason the hero's chevron does -- a loop that snaps back to the left edge every
/// cycle is a jerk the eye catches, and there is nothing to catch when the ends look alike.
const REPEAT_MODES: [(Repeat, bool); 4] = [
    (Repeat::Once, false),
    (Repeat::Times(1), false),
    (Repeat::Times(1), true),
    (Repeat::Forever, true),
];

struct RepeatDemo {
    board: Blueprint,
    shape: Leaf,
}

fn repeat(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::REPEAT_ROWS,
        &board::REPEAT_STEPS,
        &reference::REPEAT,
    );
    track(g.canopy, board.stage);
    let shape = traveller(g.canopy, board.stage);
    let [repeat, passes] = board::REPEAT_VALUES[0];
    board.set(g.canopy, 0, repeat);
    board.set(g.canopy, 1, passes);
    g.page.demos.push(Box::new(RepeatDemo { board, shape }));
}

impl Demo for RepeatDemo {
    /// Pressing another step is also how the running loop is stopped, and the row says so: the
    /// new animation supersedes the old one on the same value rather than joining it. There is
    /// no stop button here because there is no stop call -- that is the point.
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(canopy, step);
        let (mode, backtrack) = REPEAT_MODES[step];
        let timing = after_lead(REPEAT_MS).eased(Ease::DECELERATE).repeat(mode);
        travel(
            canopy,
            self.board.stage,
            self.shape,
            if backtrack { timing.backtrack() } else { timing },
        );
        let [repeat, passes] = board::REPEAT_VALUES[step];
        self.board.set(canopy, 0, repeat);
        self.board.set(canopy, 1, passes);
        true
    }
}

// ---- sequence ------------------------------------------------------------------------------

/// Three of them, so the stagger is a wave rather than a pair of things that happen to differ.
const SEQUENCE_COUNT: usize = 3;
/// Slower than the site's own entrance fade, and slower than the boards above.
///
/// Everywhere else on the site a fade is over as soon as you have registered it, which is what
/// you want of an entrance. Here the fade *is* the subject: the readout has to be caught saying
/// `running` before it says `finished`, and three of these have to be seen not landing together.
const SEQUENCE_MS: u64 = 700;
/// Between one shape's start and the next, on the staggered step.
///
/// Much wider than the site's own [`STAGGER`](crate::site::motion::STAGGER), which is tuned so a
/// page arrives as one gesture -- exactly the reading this board needs to break. At 90ms the two
/// steps were the same press twice.
const SEQUENCE_STAGGER: u64 = 260;
/// The two alphas a press moves the three shapes between. Dim rather than nearly gone: the
/// wave has to be legible on a shape that is still plainly there.
const SEQUENCE_FADED: f32 = 0.25;
const SEQUENCE_FULL: f32 = 1.0;

struct SequenceDemo {
    board: Blueprint,
    shapes: [Leaf; SEQUENCE_COUNT],
    /// The sequence the last press opened, while it is still running.
    ///
    /// Held so the emission can be told from any other sequence's -- the page's own entrance is
    /// one of these too, and it reports itself finished on the same channel. `None` once the
    /// group has landed, so a stale report cannot rewrite a row that has moved on.
    running: Option<Leaf>,
    /// Which alpha the next press fades to. A press alternates rather than resetting and
    /// running one way, for the reason [`far_end`] gives: writing a value a tween owns is what
    /// breaks under a second press. Here the write would not be displaced, it would be
    /// *overwritten* -- the two staggered shapes' new tweens are delayed, so they do not
    /// supersede the running ones until their delay is up, and the old tween spends that time
    /// undoing the reset. Alternating means there is nothing to reset: each tween picks up
    /// whatever alpha it finds and takes it to the other one.
    to: f32,
}

/// Three shapes across the stage, evenly spaced. Placed rather than travelling: this board is
/// about *when* each tween runs, and three things moving at once would make the answer a
/// question of which one you happened to be watching.
fn sequence_shape_at(i: usize) -> Location {
    let slot = 100.0 / SEQUENCE_COUNT as f32;
    let center = slot * i as f32 + slot / 2.0;
    Location::new().xs(
        center.pct().as_center_x().with(TRAVELLER.px().as_width()),
        50.pct().as_center_y().with(TRAVELLER.px().as_height()),
    )
}

fn sequence(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::SEQUENCE_ROWS,
        &board::SEQUENCE_STEPS,
        &reference::SEQUENCE,
    );
    let shapes = std::array::from_fn(|i| {
        g.canopy.branch(
            board.stage,
            Polygon::new()
                .sides(6.0)
                .rounding(0.3)
                .rotation(0.0)
                .color(role::accent())
                .at(sequence_shape_at(i))
                .elevate(Elevation::up(2))
                .pass_through(),
        )
    });
    board.set(g.canopy, 0, board::SEQUENCE_STARTS[0]);
    board.set(g.canopy, 1, board::SEQUENCE_IDLE);
    g.page.demos.push(Box::new(SequenceDemo {
        board,
        shapes,
        running: None,
        to: SEQUENCE_FADED,
    }));
}

impl Demo for SequenceDemo {
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(canopy, step);
        // A fresh one per press. A sequence is spent when its last animation settles -- it
        // reports and is despawned -- so there is nothing to reuse, and holding the old handle
        // would only be holding a withered one.
        let seq = canopy.sequence();
        let apart = if step == 0 { SEQUENCE_STAGGER } else { 0 };
        for (i, shape) in self.shapes.iter().enumerate() {
            // The same beat before the first one moves that every other board on this page
            // takes, and the thing the `starts` row is reporting.
            let start = LEAD_IN + i as u64 * apart;
            canopy.animate_during(
                *shape,
                Motion::Opacity(self.to),
                Timing::over(start + SEQUENCE_MS)
                    .after(start)
                    .eased(Ease::DECELERATE),
                seq,
            );
        }
        self.to = if self.to == SEQUENCE_FADED {
            SEQUENCE_FULL
        } else {
            SEQUENCE_FADED
        };
        self.running = Some(seq);
        self.board.set(canopy, 0, board::SEQUENCE_STARTS[step]);
        self.board.set(canopy, 1, board::SEQUENCE_RUNNING);
        true
    }

    /// The group's own report, and the reason this board exists: the last tween to land is what
    /// ends the sequence, so on the staggered step this arrives a stagger later than the same
    /// three tweens fired together -- with nothing here counting milliseconds to work that out.
    fn finished(&mut self, canopy: &mut Canopy, seq: Leaf) -> bool {
        if self.running != Some(seq) {
            return false;
        }
        self.running = None;
        self.board.set(canopy, 1, board::SEQUENCE_FINISHED);
        true
    }
}
