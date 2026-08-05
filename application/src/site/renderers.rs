//! The `renderers` section.
//!
//! Panel, Polygon, Text's own glyph mechanics, and Polyline -- the pipelines with a single
//! property worth pressing a button over. Icon and Image are asset-backed and covered on that
//! page instead; full Text (sizing, per-breakpoint scale, registered fonts) gets a section of
//! its own.

use foliage::{
    Canopy, Elevation, FontSize, GridExt, Grows, HorizontalAlignment, Leaf, Location, Panel,
    Polygon, Polyline, Rounding, Sprout, Text, VerticalAlignment,
};

use crate::site::blueprint::{self, Blueprint};
use crate::site::copy::{board, headings, reference, renderers as text};
use crate::site::{Column, Demo, Grow, SCROLL_TAIL, role};

const STAGE_H: (i32, i32, i32) = (150, 165, 190);

pub(crate) fn build(g: &mut Grow, slot: Leaf) {
    let container = crate::site::shell::content_area(g.canopy, slot);
    let mut column = Column::new(g.canopy, container);

    column.display(g.canopy, headings::RENDERERS);
    column.lead(g.canopy, text::LEAD);

    column.heading(g.canopy, headings::RENDERERS_ROUNDING);
    column.prose(g.canopy, text::ROUNDING);
    rounding(g, &mut column);

    column.heading(g.canopy, headings::RENDERERS_SIDES);
    column.prose(g.canopy, text::SIDES);
    sides(g, &mut column);

    column.heading(g.canopy, headings::RENDERERS_GLYPH);
    column.prose(g.canopy, text::GLYPH);
    glyph(g, &mut column);

    column.heading(g.canopy, headings::RENDERERS_DRAW);
    column.prose(g.canopy, text::DRAW);
    draw(g, &mut column);

    column.tail(g.canopy, SCROLL_TAIL);
}

// ---- rounding ----------------------------------------------------------------------------

/// One per [`board::ROUNDING_STEPS`], in order.
const ROUNDING_KINDS: [Rounding; 4] = [Rounding::None, Rounding::Sm, Rounding::Lg, Rounding::Full];

struct RoundingDemo {
    board: Blueprint,
    panel: Leaf,
}

fn rounding(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::ROUNDING_ROWS,
        &board::ROUNDING_STEPS,
        &reference::ROUNDING,
    );
    let panel = g.canopy.branch(
        board.stage,
        Panel::new()
            .color(role::accent())
            .rounding(ROUNDING_KINDS[0])
            .at(Location::new().xs(
                50.pct().as_center_x().with(120.px().as_width()),
                50.pct().as_center_y().with(80.px().as_height()),
            ))
            .elevate(Elevation::up(2)),
    );
    let [round, corners] = board::ROUNDING_VALUES[0];
    board.set(g.canopy, 0, round);
    board.set(g.canopy, 1, corners);
    g.page.demos.push(Box::new(RoundingDemo { board, panel }));
}

impl Demo for RoundingDemo {
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(canopy, step);
        // `Rounding` has no `Motion` variant -- it is a bracket, not a tweenable scalar -- so
        // this is a plain write rather than an animated one, same as every other discrete
        // choice on these boards.
        canopy.rounding(self.panel, ROUNDING_KINDS[step]);
        let [round, corners] = board::ROUNDING_VALUES[step];
        self.board.set(canopy, 0, round);
        self.board.set(canopy, 1, corners);
        true
    }
}

// ---- sides ---------------------------------------------------------------------------------

/// One per [`board::SIDES_STEPS`], in order.
const SIDES_COUNTS: [f32; 3] = [3.0, 6.0, 12.0];
/// Held fixed across every step -- side count is the one thing this board is showing, and a
/// rounding change at the same time would leave two things moving instead of one.
const SIDES_ROUNDING: f32 = 0.12;

struct SidesDemo {
    board: Blueprint,
    shape: Leaf,
}

fn sides(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::SIDES_ROWS,
        &board::SIDES_STEPS,
        &reference::SIDES,
    );
    let shape = g.canopy.branch(
        board.stage,
        Polygon::new()
            .sides(SIDES_COUNTS[0])
            .rounding(SIDES_ROUNDING)
            .rotation(0.0)
            .color(role::accent())
            .at(Location::new().xs(
                50.pct().as_center_x().with(100.px().as_width()),
                50.pct().as_center_y().with(100.px().as_height()),
            ))
            .elevate(Elevation::up(2)),
    );
    let [count, name] = board::SIDES_VALUES[0];
    board.set(g.canopy, 0, count);
    board.set(g.canopy, 1, name);
    g.page.demos.push(Box::new(SidesDemo { board, shape }));
}

impl Demo for SidesDemo {
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(canopy, step);
        canopy.polygon(
            self.shape,
            Polygon {
                sides: SIDES_COUNTS[step],
                rounding: SIDES_ROUNDING,
                rotation: 0.0,
            },
        );
        let [count, name] = board::SIDES_VALUES[step];
        self.board.set(canopy, 0, count);
        self.board.set(canopy, 1, name);
        true
    }
}

// ---- glyph -----------------------------------------------------------------------------------

/// Large enough that one character reads as the subject of the board rather than a word on it.
const GLYPH_SIZE: u32 = 80;

struct GlyphDemo {
    board: Blueprint,
    letter: Leaf,
}

fn glyph(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::GLYPH_ROWS,
        &board::GLYPH_STEPS,
        &reference::GLYPH,
    );
    let letter = g.canopy.branch(
        board.stage,
        Text::new(board::GLYPH_STEPS[0])
            .size(FontSize::new(GLYPH_SIZE))
            .color(role::accent())
            .at(Location::new().xs(
                // Wider and taller than the font size itself -- the box clips its own glyphs,
                // and a box matched 1:1 to the size cuts a descender's tail (`g`) and a wide
                // glyph's sides (`@`), both well past what the *advance* alone would suggest.
                50.pct().as_center_x().with(120.px().as_width()),
                50.pct().as_center_y().with(120.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .align(HorizontalAlignment::Center, VerticalAlignment::Middle),
    );
    let [advance, atlas] = board::GLYPH_VALUES[0];
    board.set(g.canopy, 0, advance);
    board.set(g.canopy, 1, atlas);
    g.page.demos.push(Box::new(GlyphDemo { board, letter }));
}

impl Demo for GlyphDemo {
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(canopy, step);
        canopy.text(self.letter, board::GLYPH_STEPS[step]);
        let [advance, atlas] = board::GLYPH_VALUES[step];
        self.board.set(canopy, 0, advance);
        self.board.set(canopy, 1, atlas);
        true
    }
}

// ---- draw --------------------------------------------------------------------------------

/// One per [`board::DRAW_STEPS`], in order.
const DRAW_PROGRESS: [f32; 3] = [0.25, 0.6, 1.0];
const DRAW_WEIGHT: i32 = 4;

struct DrawDemo {
    board: Blueprint,
    line: Leaf,
}

fn draw(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::DRAW_ROWS,
        &board::DRAW_STEPS,
        &reference::DRAW,
    );
    let line = g.canopy.branch(
        board.stage,
        Polyline::new()
            .points(vec![
                (0.0, 60.0),
                (50.0, 10.0),
                (100.0, 60.0),
                (150.0, 10.0),
                (200.0, 60.0),
            ])
            .weight(DRAW_WEIGHT)
            .color(role::accent())
            .draw_progress(DRAW_PROGRESS[0])
            .at(Location::new().xs(
                50.pct().as_center_x().with(200.px().as_width()),
                50.pct().as_center_y().with(70.px().as_height()),
            ))
            .elevate(Elevation::up(2)),
    );
    let [drawn, call] = board::DRAW_VALUES[0];
    board.set(g.canopy, 0, drawn);
    board.set(g.canopy, 1, call);
    g.page.demos.push(Box::new(DrawDemo { board, line }));
}

impl Demo for DrawDemo {
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(canopy, step);
        canopy.draw_progress(self.line, DRAW_PROGRESS[step]);
        let [drawn, call] = board::DRAW_VALUES[step];
        self.board.set(canopy, 0, drawn);
        self.board.set(canopy, 1, call);
        true
    }
}
