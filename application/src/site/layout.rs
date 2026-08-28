//! The `layout` section.
//!
//! Three boards, one per noun in the lead: a grid divided into columns and addressed by line
//! number, a box resolving against a named element instead of its parent, and a percentage
//! given a ceiling. Each reuses the board scaffolding `leaf.rs` proved out -- a stage, a step
//! row, a live readout -- for a `Location` idea that percent/px resolving doesn't cover.

use foliage::{
    Forest, Color, Elevation, Grid, GridExt, Grows, Leaf, Location, Panel, Polygon, Rounding,
    Sprout, anchor,
};

use crate::site::blueprint::{self, Blueprint, Frame};
use crate::site::copy::{board, headings, layout as text, reference};
use crate::site::{Column, Demo, Grow, SCROLL_TAIL, role, space};

const STAGE_H: (i32, i32, i32) = (150, 165, 190);

pub(crate) fn build(g: &mut Grow, slot: Leaf) {
    let container = crate::site::shell::content_area(g.forest, slot);
    let mut column = Column::new(g.forest, container);

    column.display(g.forest, headings::LAYOUT);
    column.lead(g.forest, text::LEAD);

    column.heading(g.forest, headings::LAYOUT_GRID);
    column.prose(g.forest, text::GRID);
    grid(g, &mut column);

    column.heading(g.forest, headings::LAYOUT_ANCHOR);
    column.prose(g.forest, text::ANCHOR);
    anchoring(g, &mut column);

    column.heading(g.forest, headings::LAYOUT_MEASURE);
    column.prose(g.forest, text::MEASURE);
    measure(g, &mut column);

    column.tail(g.forest, SCROLL_TAIL);
}

// ---- grid ------------------------------------------------------------------------------------

const GRID_COLS: i32 = 3;

/// Which columns each step's cell spans, `(from, to)` -- both 1-based and inclusive, the same
/// as `N.col()` reads them.
const GRID_SPANS: [(i32, i32); 3] = [(1, 1), (2, 2), (1, 3)];

/// The stage's own division into columns, drawn as an outline so the tracks stay visible
/// wherever the cell currently is -- otherwise a single-column step looks like a box floating
/// in empty space rather than one slot of three.
fn grid_host(forest: &mut Forest, stage: Leaf) -> Leaf {
    forest.branch(
        stage,
        Panel::new()
            .color(role::outline())
            .outline(1)
            .rounding(Rounding::Xs)
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(0))
            .grid(Grid::new(GRID_COLS.col().gap(space::SM), 1.row().gap(0)))
            .pass_through(),
    )
}

fn cell_at(from: i32, to: i32) -> Location {
    Location::new().xs(
        from.col().as_left().with(to.col().as_right()),
        20.pct().as_top().with(80.pct().as_bottom()),
    )
}

struct Tracks {
    board: Blueprint,
    cell: Frame,
}

fn grid(g: &mut Grow, column: &mut Column) {
    let board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::GRID_ROWS,
        &board::GRID_STEPS,
        &reference::GRID,
    );
    let host = grid_host(g.forest, board.stage);
    let (from, to) = GRID_SPANS[0];
    let cell = blueprint::frame(
        g.forest,
        host,
        cell_at(from, to),
        board::GRID_STEPS[0],
        true,
    );
    g.page.demos.push(Box::new(Tracks { board, cell }));
}

impl Demo for Tracks {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(forest, step);
        let (from, to) = GRID_SPANS[step];
        forest.location(self.cell.leaf, cell_at(from, to));
        forest.text(self.cell.label, board::GRID_STEPS[step]);
        true
    }
    fn drive(&mut self, forest: &mut Forest) {
        let stage = blueprint::resolved(forest, self.board.stage);
        let cell = blueprint::resolved(forest, self.cell.leaf);
        self.board.set(forest, 0, stage);
        self.board.set(forest, 1, cell);
    }
}

// ---- anchor ----------------------------------------------------------------------------------

const ANCHOR_TARGET_SIZE: i32 = 64;
const ANCHOR_DEPENDENT_SIZE: i32 = 40;

fn target_tone() -> Color {
    role::accent()
}
fn dependent_tone() -> Color {
    Color::rose(400)
}

/// Fixed in the stage's own corner -- the point of this board is that the dependent moves and
/// the target does not, so nothing here reads its position back.
fn target_at() -> Location {
    Location::new().xs(
        space::MD
            .px()
            .as_left()
            .with(ANCHOR_TARGET_SIZE.px().as_width()),
        space::MD
            .px()
            .as_top()
            .with(ANCHOR_TARGET_SIZE.px().as_height()),
    )
}

/// Every axis here is an `anchor()` value, not a percentage of the parent -- this box's
/// `Location` is read against whatever entity its `Anchor` names, which is `target`, not the
/// stage it happens to be parented under.
fn dependent_at(step: usize) -> Location {
    let w = ANCHOR_DEPENDENT_SIZE.px().as_width();
    let h = ANCHOR_DEPENDENT_SIZE.px().as_height();
    match step {
        0 => Location::new().xs(
            anchor().center_x().as_center_x().with(w),
            anchor().bottom().as_top().adjust(space::SM).with(h),
        ),
        1 => Location::new().xs(
            anchor().right().as_left().adjust(space::SM).with(w),
            anchor().center_y().as_center_y().with(h),
        ),
        // The same two offsets "right" and "below" each use on their own axis, combined on
        // both at once -- sits past the target's corner instead of opening a third direction
        // of its own, so the target doesn't need to move to make room for it. Centering on
        // the corner point instead (half in, half out) was tried first and sat too close:
        // the two shapes overlapped enough to read as one merged one.
        _ => Location::new().xs(
            anchor().right().as_left().adjust(space::SM).with(w),
            anchor().bottom().as_top().adjust(space::SM).with(h),
        ),
    }
}

fn dependent(forest: &mut Forest, stage: Leaf, target: Leaf) -> Leaf {
    let dep = forest.branch(
        stage,
        Polygon::new()
            .sides(5.0)
            .rounding(0.3)
            .rotation(0.0)
            .color(dependent_tone())
            .at(dependent_at(0))
            .elevate(Elevation::up(2))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            // The point of the board: parented to the stage like any other child, but its
            // `Location` reads against `target` instead -- the two are unrelated.
            .anchored(target)
            .pass_through(),
    );
    blueprint::name(forest, dep, board::DEPENDENT);
    dep
}

struct Anchoring {
    board: Blueprint,
    dependent: Leaf,
}

fn anchoring(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::ANCHOR_ROWS,
        &board::ANCHOR_STEPS,
        &reference::ANCHOR,
    );
    let target = blueprint::child(
        g.forest,
        board.stage,
        target_at(),
        target_tone(),
        board::TARGET,
    );
    let dependent = dependent(g.forest, board.stage, target);
    let [call, sits] = board::ANCHOR_VALUES[0];
    board.set(g.forest, 0, call);
    board.set(g.forest, 1, sits);
    g.page.demos.push(Box::new(Anchoring { board, dependent }));
}

impl Demo for Anchoring {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(forest, step);
        forest.location(self.dependent, dependent_at(step));
        let [call, sits] = board::ANCHOR_VALUES[step];
        self.board.set(forest, 0, call);
        self.board.set(forest, 1, sits);
        true
    }
}

// ---- measure ---------------------------------------------------------------------------------

/// The widths a frame cycles through, as a percentage of the stage -- past [`MEASURE_CAP`] the
/// cell inside it stops following.
const MEASURE_WIDTHS: [f32; 3] = [40.0, 70.0, 100.0];
/// The cell's ceiling, in logical pixels. Comfortably past what [`MEASURE_WIDTHS`]'s first step
/// resolves to at the narrowest breakpoint, and comfortably short of its last, so both sides of
/// the cap are visible across the row.
const MEASURE_CAP: f32 = 160.0;

fn frame_at(width: f32) -> Location {
    Location::new().xs(
        0.pct().as_left().with(width.pct().as_right()),
        0.pct().as_top().with(100.pct().as_bottom()),
    )
}

/// Declared once and never rewritten -- the whole point is that this box's own `Location`
/// never changes across the row, only what the frame around it does.
fn cell_cap_at(cap: f32) -> Location {
    Location::new().xs(
        0.pct().as_left().with(100.pct().as_right()).max(cap),
        20.pct().as_top().with(80.pct().as_bottom()),
    )
}

struct Measure {
    board: Blueprint,
    frame: Frame,
    cell: Frame,
}

fn measure(g: &mut Grow, column: &mut Column) {
    let board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::MEASURE_ROWS,
        &board::MEASURE_STEPS,
        &reference::MEASURE,
    );
    let frame = blueprint::frame(
        g.forest,
        board.stage,
        frame_at(MEASURE_WIDTHS[0]),
        board::frame(MEASURE_WIDTHS[0]),
        false,
    );
    let cell = blueprint::frame(
        g.forest,
        frame.leaf,
        cell_cap_at(MEASURE_CAP),
        format!("max {}px", MEASURE_CAP as i32),
        true,
    );
    g.page.demos.push(Box::new(Measure { board, frame, cell }));
}

impl Demo for Measure {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(forest, step);
        forest.location(self.frame.leaf, frame_at(MEASURE_WIDTHS[step]));
        forest.text(self.frame.label, board::frame(MEASURE_WIDTHS[step]));
        true
    }
    fn drive(&mut self, forest: &mut Forest) {
        let frame = blueprint::resolved(forest, self.frame.leaf);
        let cell = blueprint::resolved(forest, self.cell.leaf);
        self.board.set(forest, 0, frame);
        self.board.set(forest, 1, cell);
    }
}
