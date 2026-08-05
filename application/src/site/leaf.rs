//! The `leaf` section.
//!
//! Every board is a parent with one child declared inside it, and a row of step controls along
//! the bottom that never moves. The row acts on the parent; the parent and child are drawn and
//! nothing else, out of the hit test entirely. Nothing the reader is touching resizes,
//! disappears, or scrolls away between one press and the next.
//!
//! The readout pairs what the child *declares*, written once at spawn and never rewritten, with
//! what that currently *resolves to*. One stays still while the other moves, and that gap is
//! what a stem is.

use foliage::{
    Canopy, Color, Elevation, FontSize, Grid, GridExt, Grows, HorizontalAlignment, Leaf, Location,
    Panel, Polygon, Presence, Rounding, Sprout, Text, VerticalAlignment,
};

use crate::site::blueprint::{self, Blueprint, Frame};
use crate::site::copy::{board, headings, leaf as text, reference};
use crate::site::{Column, Demo, Grow, SCROLL_TAIL, role, type_scale};

const STAGE_H: (i32, i32, i32) = (150, 165, 190);

/// The widths a parent cycles through, as a percentage of the stage. Anchored at the left, so it
/// grows and shrinks against a fixed edge rather than sliding across the field. One per entry in
/// [`board::RESOLVING_STEPS`], which is what the buttons say they are.
const FRAME_WIDTHS: [f32; 3] = [100.0, 66.0, 42.0];

/// The child's declaration, in percentages of its parent. Written once and never rewritten.
const CHILD_LEFT: f32 = 22.0;
const CHILD_RIGHT: f32 = 74.0;

/// The clipping board's child, as a share of the stage's height -- both axes, since `Polygon` is
/// square. Measured off the stage rather than the parent: the parent is the thing being narrowed,
/// and a child that shrank along with it would have nothing to demonstrate.
///
/// [`Clipping::drive`] turns this into px and writes it. One ratio against a measured stage lands
/// on the same proportion at every width, where a table of widths per breakpoint did not.
const CHILD_OF_STAGE: f32 = 0.8;

fn child_tone() -> Color {
    role::accent()
}
fn written_tone() -> Color {
    Color::rose(400)
}
/// What the inheriting board writes to the parent. A neutral rather than another palette hue --
/// the point of the write is only that the parent changed and the child did not, and a second hue
/// behind the orange child reads as a colour scheme instead.
fn parent_write_tone() -> Color {
    Color::stone(500)
}

pub(crate) fn build(g: &mut Grow, slot: Leaf) {
    let container = crate::site::shell::content_area(g.canopy, slot);
    let mut column = Column::new(g.canopy, container);

    column.display(g.canopy, headings::LEAF);
    column.lead(g.canopy, text::LEAD);

    column.heading(g.canopy, headings::LEAF_RESOLVING);
    column.prose(g.canopy, text::RESOLVING);
    resolving(g, &mut column);

    column.heading(g.canopy, headings::LEAF_CLIPPING);
    column.prose(g.canopy, text::CLIPPING);
    clipping(g, &mut column);

    column.heading(g.canopy, headings::LEAF_INHERITING);
    column.prose(g.canopy, text::INHERITING);
    inheriting(g, &mut column);

    column.heading(g.canopy, headings::LEAF_LIFETIME);
    column.prose(g.canopy, text::LIFETIME);
    lifetime(g, &mut column);

    column.tail(g.canopy, SCROLL_TAIL);
}

fn frame_at(width: f32) -> Location {
    Location::new().xs(
        0.pct().as_left().with(width.pct().as_right()),
        0.pct().as_top().with(100.pct().as_bottom()),
    )
}

/// How much of the child each press leaves uncut.
///
/// A fraction rather than a width, because the child's own size is not a constant -- it comes off
/// the stage height. [`Clipping`] measures the child's resolved section and stops the parent at a
/// fraction of it, so the bite is the same at every breakpoint by construction; a width written
/// down here would be a different bite on every screen.
const CLIP_KEPT: [f32; 2] = [0.75, 0.45];

/// `None` is the full stage; `Some(px)` is the parent's right edge, measured from its own left.
fn clip_frame_at(right: Option<f32>) -> Location {
    match right {
        None => frame_at(100.0),
        Some(right) => Location::new().xs(
            0.pct().as_left().with((right.round() as i32).px().as_right()),
            0.pct().as_top().with(100.pct().as_bottom()),
        ),
    }
}

fn clip_resize(canopy: &mut Canopy, frame: &Frame, right: Option<f32>) {
    canopy.location(frame.leaf, clip_frame_at(right));
    canopy.text(
        frame.label,
        if right.is_none() {
            board::CLIP_FULL
        } else {
            board::CLIP_NARROWED
        },
    );
}

/// Resizes a parent and keeps its label honest.
fn resize(canopy: &mut Canopy, frame: &Frame, width: f32) {
    canopy.location(frame.leaf, frame_at(width));
    canopy.text(frame.label, board::frame(width));
}

/// Below the parent's own label, so a narrow parent never puts the two on top of each other.
fn child_band(left: f32, right: f32) -> Location {
    Location::new().xs(
        left.pct().as_left().with(right.pct().as_right()),
        36.pct().as_top().with(90.pct().as_bottom()),
    )
}

// ---- resolving -----------------------------------------------------------------------------

struct Resolving {
    board: Blueprint,
    frame: Frame,
    child: Leaf,
}

fn resolving(g: &mut Grow, column: &mut Column) {
    // Both boxes, not just the child's. The pair is what the section is about -- one number is
    // half of a ratio -- and it also says plainly which of the two a resize actually moved.
    // The steps are the widths themselves. A board whose presses set a value rather than
    // advancing through one can say so on the buttons, which is half of what the row is for.
    let board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::RESOLVING_ROWS,
        &board::RESOLVING_STEPS,
        &reference::RESOLVING,
    );
    let frame = blueprint::frame(
        g.canopy,
        board.stage,
        frame_at(FRAME_WIDTHS[0]),
        board::frame(FRAME_WIDTHS[0]),
        false,
    );
    let child = blueprint::child_box(
        g.canopy,
        frame.leaf,
        child_band(CHILD_LEFT, CHILD_RIGHT),
        child_tone(),
        board::CHILD,
    );
    g.page.demos.push(Box::new(Resolving {
        board,
        frame,
        child,
    }));
}

impl Demo for Resolving {
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(canopy, step);
        resize(canopy, &self.frame, FRAME_WIDTHS[step]);
        true
    }
    fn drive(&mut self, canopy: &mut Canopy) {
        let parent = blueprint::resolved(canopy, self.frame.leaf);
        let child = blueprint::resolved(canopy, self.child);
        self.board.set(canopy, 0, parent);
        self.board.set(canopy, 1, child);
    }
}

// ---- clipping ------------------------------------------------------------------------------

/// The clipping board's child: the same shape [`blueprint::child`] grows, centred on the *stage*.
///
/// Centred on its parent instead, it would slide left as the parent narrowed from the right and
/// walk out from under the edge doing the cutting, so its position is read off a box the presses
/// do not move. It stays a child of the parent, which still owns and clips it -- `anchored` only
/// redirects the `anchor()` values in the location, and `Anchor` is read per entity
/// (`grid/location.rs:338`), so the name text inside the shape still resolves against the shape.
/// Where the clipping board's child sits: a px box centred in the stage, both axes the same
/// number because `Polygon` is square. Stating the square rather than declaring one axis in px
/// against the other in percent also makes `AspectRatio::constrain` a no-op, so the position is
/// what the location says instead of what the clamp left behind.
///
/// The horizontal is px and not a percentage because the parent narrows from the right: a child
/// centred on it follows that moving centre and walks out from under the edge doing the cutting.
fn clip_child_at(left: f32, size: f32) -> Location {
    Location::new().xs(
        (left.round() as i32)
            .px()
            .as_left()
            .with((size.round() as i32).px().as_width()),
        50.pct()
            .as_center_y()
            .with((size.round() as i32).px().as_height()),
    )
}

/// The clipping board's child: the same shape [`blueprint::child`] grows, placed by [`Clipping::drive`].
///
/// Not `anchored`. Anchoring it to the stage put the horizontal exactly right and left the
/// vertical a full page-scroll high: the anchor path adds the accumulated offset back to state
/// its target in layout space, so a second subtraction cancels for the anchored value and
/// survives for the plain percentage beside it. Measuring the centre reaches the same place
/// without putting this element on that path at all.
fn clip_child(canopy: &mut Canopy, parent: Leaf, tone: Color) -> Leaf {
    let child = canopy.branch(
        parent,
        Polygon::new()
            .sides(6.0)
            .rounding(0.3)
            .rotation(0.0)
            .color(tone)
            // Replaced on the first frame there is a stage to measure. A size here only avoids
            // spawning at zero.
            .at(clip_child_at(0.0, 120.0))
            .elevate(Elevation::up(2))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            .pass_through(),
    );
    blueprint::name(canopy, child, board::CHILD);
    child
}

struct Clipping {
    board: Blueprint,
    frame: Frame,
    child: Leaf,
    step: usize,
    /// The child's box: how far its left edge sits inside the stage, and how wide it resolved to.
    /// `None` until the first frame has resolved a section for it.
    ///
    /// Read every frame, at every step. The child is anchored to the stage, so narrowing the
    /// parent does not move it and this stays a measurement of the same thing throughout -- but a
    /// window resize does move it, and the parent's edge is a px number derived from it.
    full: Option<(f32, f32)>,
}

impl Clipping {
    /// Where the parent's right edge belongs at the current step, in px from its own left --
    /// which is also the stage's left, so the measurement needs no converting. `None` is the full
    /// stage, and is also the answer before anything has been measured.
    fn right(&self) -> Option<f32> {
        match self.step {
            0 => None,
            step => self
                .full
                .map(|(left, width)| left + width * CLIP_KEPT[step - 1]),
        }
    }
}

fn clipping(g: &mut Grow, column: &mut Column) {
    let board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::CLIPPING_ROWS,
        &board::CLIPPING_STEPS,
        &reference::CLIPPING,
    );
    let frame = blueprint::frame(
        g.canopy,
        board.stage,
        frame_at(FRAME_WIDTHS[0]),
        board::frame(FRAME_WIDTHS[0]),
        false,
    );
    let child = clip_child(g.canopy, frame.leaf, written_tone());
    // This board labels its parent full/narrowed rather than by percentage, so it says so from
    // the start instead of after the first press.
    clip_resize(g.canopy, &frame, None);
    g.page.demos.push(Box::new(Clipping {
        board,
        frame,
        child,
        step: 0,
        full: None,
    }));
}

impl Demo for Clipping {
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(canopy, step);
        self.step = step;
        clip_resize(canopy, &self.frame, self.right());
        true
    }
    fn drive(&mut self, canopy: &mut Canopy) {
        if let Some(stage) = canopy.section(self.board.stage) {
            // Derived from the stage, not read back off the child: reading the child would make
            // whatever was written last frame the input to this one, and a resize could never
            // recentre it.
            let size = (stage.height() * CHILD_OF_STAGE).round();
            let left = ((stage.width() - size) / 2.0).max(0.0);
            if self.full != Some((left, size)) {
                self.full = Some((left, size));
                canopy.location(self.child, clip_child_at(left, size));
                // The parent's edge is px derived from the old measurement, so left alone through
                // a resize it means nothing at the new size. Re-applied here rather than waiting
                // for the next press to notice.
                clip_resize(canopy, &self.frame, self.right());
            }
        }
        let parent = blueprint::resolved(canopy, self.frame.leaf);
        let child = blueprint::resolved(canopy, self.child);
        self.board.set(canopy, 0, parent);
        self.board.set(canopy, 1, child);
    }
}

// ---- inheriting ----------------------------------------------------------------------------

struct Inheriting {
    board: Blueprint,
    frame: Frame,
}

fn inheriting(g: &mut Grow, column: &mut Column) {
    // "reset" leads rather than trails, because it is step one in the sense the row means: the
    // state the board is in before anything has been written to the parent.
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::INHERITING_ROWS,
        &board::INHERITING_STEPS,
        &reference::INHERITING,
    );
    // Filled, so a colour write is visibly the parent's own surface changing. Full width like the
    // boards above it -- nothing here resizes the parent, so a narrow one is just a smaller stage.
    let frame = blueprint::frame(
        g.canopy,
        board.stage,
        frame_at(FRAME_WIDTHS[0]),
        board::frame(FRAME_WIDTHS[0]),
        true,
    );
    blueprint::child(
        g.canopy,
        frame.leaf,
        child_band(CHILD_LEFT, CHILD_RIGHT),
        child_tone(),
        board::CHILD,
    );
    board.set(g.canopy, 0, board::INHERITING_VALUES[0][0]);
    g.page.demos.push(Box::new(Inheriting { board, frame }));
}

impl Demo for Inheriting {
    /// Every arm writes both properties, not just the one it is named for. The row can be
    /// pressed in any order, so a step has to state the whole parent it means -- written as a
    /// change from whatever came before, jumping from "color" back to "opacity" would leave the
    /// parent wearing the previous step's tone.
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(canopy, step);
        match step {
            1 => {
                canopy.opacity(self.frame.leaf, 0.6);
                canopy.color(self.frame.leaf, role::surface());
            }
            2 => {
                canopy.opacity(self.frame.leaf, 1.0);
                canopy.color(self.frame.leaf, parent_write_tone());
            }
            _ => {
                canopy.opacity(self.frame.leaf, 1.0);
                canopy.color(self.frame.leaf, role::surface());
            }
        }
        let [wrote, child] = board::INHERITING_VALUES[step];
        self.board.set(canopy, 0, wrote);
        self.board.set(canopy, 1, child);
        true
    }
}

// ---- lifetime ------------------------------------------------------------------------------

/// Placed, then filled, then pruned. Building it in two steps is what makes the third one land:
/// you put the parent down and the child in it yourself, so when one call takes both away it is
/// visibly two things going, not one shape vanishing.
///
/// The fourth step is pruned with the handle still held. Its own step because a withered handle
/// is a thing you can still read, and reading it is the only way to see that state: the slot
/// markers stay off there, so the row saying "withered" is never contradicted by a stage saying
/// "no child".
///
/// The step at which that has happened, so the handle the board still holds is withered.
const PRUNED: usize = 3;

struct Lifetime {
    board: Blueprint,
    stage: Leaf,
    frame: Option<Frame>,
    child: Option<Leaf>,
    at: usize,
    /// One marker per slot, each sitting exactly where the thing it stands for will appear -- the
    /// parent's own label position, and the child's band. An outline on its own is a drawing
    /// decision the reader has to interpret; a word in the slot is the board saying that nothing
    /// is there yet, and saying it in the place you are about to watch fill.
    empty: [Leaf; 2],
}

fn lifetime(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::LIFETIME_ROWS,
        &board::LIFETIME_STEPS,
        &reference::LIFETIME,
    );
    // The room the pair occupies, drawn once and never pruned. Without it the board empties to
    // nothing and there is no telling what left or where it was.
    let room = g.canopy.branch(
        board.stage,
        Panel::new()
            .color(role::outline())
            .outline(1)
            .rounding(Rounding::Xs)
            .at(frame_at(FRAME_WIDTHS[0]))
            .elevate(Elevation::up(0))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            .pass_through(),
    );
    // Each marker takes its slot's own alignment as well as its box, or the words sit in the
    // right rectangle and still not where the thing they stand for will be: a frame's label is
    // left-aligned in a box a fixed number of characters wide, a child's name is centred across
    // its whole band.
    let marker = |canopy: &mut Canopy, text: &'static str, at: Location, h: HorizontalAlignment| {
        canopy.branch(
            room,
            Text::new(text)
                .size(FontSize::new(type_scale::LABEL))
                .color(role::on_surface_variant())
                .at(at)
                .elevate(Elevation::up(1))
                .align(h, VerticalAlignment::Middle)
                .pass_through(),
        )
    };
    // Hidden one at a time as the presses fill the slots, and both back on the prune.
    let empty = [
        marker(
            g.canopy,
            board::NO_PARENT,
            blueprint::frame_label_at(),
            HorizontalAlignment::Left,
        ),
        marker(
            g.canopy,
            board::NO_CHILD,
            child_band(CHILD_LEFT, CHILD_RIGHT),
            HorizontalAlignment::Center,
        ),
    ];
    let stage = board.stage;
    board.set(g.canopy, 0, board::LIFETIME_CALLS[0]);
    board.set(g.canopy, 1, board::CHILD_NONE);
    g.page.demos.push(Box::new(Lifetime {
        board,
        stage,
        frame: None,
        child: None,
        at: 0,
        empty,
    }));
}

impl Lifetime {
    /// Puts the board in the state `step` names, from wherever it currently is.
    ///
    /// Torn down and replayed rather than nudged along. This is the one board whose steps are a
    /// sequence -- there is no parent to prune until one has been placed -- and the row lets a
    /// reader press them in any order, so a step that only knew how to advance from the one
    /// before it would either wedge or lie. Replaying from empty is a handful of spawns, and it
    /// is the only thing that makes every button land on the state its word names.
    fn goto(&mut self, canopy: &mut Canopy, step: usize) {
        if let Some(frame) = self.frame.take() {
            canopy.prune(frame.leaf);
        }
        // Dropping the handle is what takes the child row back to "none", so an empty board reads
        // the same on every pass through.
        self.child = None;
        // Each marker says "nothing is here yet", which is true of an empty slot and not of one
        // that was just pruned -- so both go off from the moment the child exists and stay off.
        canopy.visible(self.empty[0], step == 0);
        canopy.visible(self.empty[1], step <= 1);
        if step >= 1 {
            // Filled, not outlined: the room is already an outline, and placing a second one over
            // it is a press that appears to do nothing.
            self.frame = Some(blueprint::frame(
                canopy,
                self.stage,
                frame_at(FRAME_WIDTHS[0]),
                board::frame(FRAME_WIDTHS[0]),
                true,
            ));
        }
        if step >= 2 {
            let parent = self.frame.as_ref().unwrap().leaf;
            self.child = Some(blueprint::child(
                canopy,
                parent,
                child_band(CHILD_LEFT, CHILD_RIGHT),
                child_tone(),
                board::CHILD,
            ));
        }
        if step == PRUNED {
            // Only the parent is named. Both go -- and the child's handle is kept, because this
            // step exists to read it.
            canopy.prune(self.frame.take().unwrap().leaf);
        }
        self.at = step;
        self.board.set(canopy, 0, board::LIFETIME_CALLS[step]);
    }
}

impl Demo for Lifetime {
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(canopy, step);
        self.goto(canopy, step);
        true
    }

    /// The child's row is the tree's answer, not a copy kept here: while a child exists the row
    /// is whatever `presence` says about it that frame, and each state is reported as itself
    /// rather than folded together.
    fn drive(&mut self, canopy: &mut Canopy) {
        let text = match (self.at, self.child) {
            // The prune is a command, not an edit: for the frames before it lands the handle still
            // reads `Planted`, which would put "growing" on a board that just emptied. The step is
            // the authority on having pruned; `presence` is the authority on everything else.
            (PRUNED, _) => board::CHILD_WITHERED,
            (_, None) => board::CHILD_NONE,
            (_, Some(child)) => match canopy.presence(child) {
                Presence::Planted => board::CHILD_GROWING,
                Presence::Live => board::CHILD_LIVE,
                Presence::Withered => board::CHILD_WITHERED,
            },
        };
        self.board.set(canopy, 1, text);
    }
}
