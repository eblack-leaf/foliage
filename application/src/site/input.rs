//! The `input` section.
//!
//! Five boards, and the only page on the site where the reader's own pointer is the
//! instrument. Three of them report emissions rather than state: what the engine said about a
//! press you just made, in the words it said it in.
//!
//! Two of the five therefore break the shape the other sections keep. The gesture board has no
//! control row at all -- a press, a drag and a release are not states you pick from a row, so
//! its stage *is* the control ([`blueprint::live`]). And the boards whose steps swap a property
//! fixed at spawn -- propagation, line constraint -- grow both variants and hide the one not in
//! use, the way the renderers page's image board handles `ImageView`.
//!
//! Hiding is not enough on its own here. The hit test consults an element's own enablement and
//! nothing else -- not visibility -- so an invisible variant left enabled still competes for
//! the press and still reports one. Every swap below therefore disables what it hides.

use foliage::{
    Bare, Forest, Color, Elevation, FontSize, Grid, GridExt, Grows, HorizontalAlignment, Leaf,
    LineConstraint, Location, Panel, Polygon, Rounding, Sample, Sap, ScrollTo, Sprout, Text,
    TextInput, TextInputAction, VerticalAlignment,
};

use crate::site::blueprint::{self, Blueprint};
use crate::site::copy::{board, headings, input as text, reference};
use crate::site::{Column, Demo, Grow, Phase, SCROLL_TAIL, role, space, type_scale};

const STAGE_H: (i32, i32, i32) = (150, 165, 190);

pub(crate) fn build(g: &mut Grow, slot: Leaf) {
    let container = crate::site::shell::content_area(g.forest, slot);
    let mut column = Column::new(g.forest, container);

    column.display(g.forest, headings::INPUT);
    column.lead(g.forest, text::LEAD);

    column.heading(g.forest, headings::INPUT_HIT);
    column.prose(g.forest, text::HIT);
    hit(g, &mut column);

    column.heading(g.forest, headings::INPUT_GRAB);
    column.prose(g.forest, text::GRAB);
    grab(g, &mut column);

    column.heading(g.forest, headings::INPUT_GESTURE);
    column.prose(g.forest, text::GESTURE);
    gesture(g, &mut column);

    column.heading(g.forest, headings::INPUT_SCROLL);
    column.prose(g.forest, text::SCROLL);
    scroll(g, &mut column);

    column.heading(g.forest, headings::INPUT_FIELD);
    column.prose(g.forest, text::FIELD);
    field(g, &mut column);

    column.tail(g.forest, SCROLL_TAIL);
}

// ---- hit shape -------------------------------------------------------------------------------

/// One per [`board::HIT_STEPS`], in order. `Full` is the whole board: it is the one bracket that
/// also rewrites the hit test.
const HIT_ROUNDINGS: [Rounding; 2] = [Rounding::None, Rounding::Full];
const HIT_SIZE: i32 = 96;

/// The shape's own box, drawn as an outline behind it and never rounded.
///
/// This is what makes a declined press visible. With the shape at `Full` the drawn circle is
/// inscribed in this square, so the four corners between the two are pixels the element
/// occupies on paper and refuses in practice -- and there is somewhere obvious to aim.
fn hit_box() -> Location {
    Location::new().xs(
        50.pct().as_center_x().with(HIT_SIZE.px().as_width()),
        50.pct().as_center_y().with(HIT_SIZE.px().as_height()),
    )
}

struct Hit {
    board: Blueprint,
    shape: Leaf,
    /// A plain target filling the stage behind the shape. Without it a press the shape declines
    /// lands on nothing and the board has nothing to report -- which reads as the board being
    /// broken rather than as the corner being outside the circle.
    pad: Leaf,
}

fn hit(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::HIT_ROWS,
        &board::HIT_STEPS,
        &reference::HIT,
    );
    let pad = g.forest.branch(
        board.stage,
        Bare::new()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .interactive(),
    );
    g.forest.branch(
        board.stage,
        Panel::new()
            .color(role::outline())
            .outline(1)
            .rounding(Rounding::None)
            .at(hit_box())
            .elevate(Elevation::up(2))
            .pass_through(),
    );
    let shape = g.forest.branch(
        board.stage,
        Panel::new()
            .color(role::accent())
            .rounding(HIT_ROUNDINGS[0])
            .at(hit_box())
            .elevate(Elevation::up(3))
            .interactive(),
    );
    board.set(g.forest, 0, board::HIT_SHAPES[0]);
    board.set(g.forest, 1, board::HIT_NONE);
    g.page.demos.push(Box::new(Hit { board, shape, pad }));
}

impl Demo for Hit {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        if let Some(step) = self.board.pressed(leaf) {
            self.board.select(forest, step);
            // The write that does it: `Rounding` carries the hit shape with it, so this one
            // call changes both what is drawn and what answers.
            forest.rounding(self.shape, HIT_ROUNDINGS[step]);
            self.board.set(forest, 0, board::HIT_SHAPES[step]);
            self.board.set(forest, 1, board::HIT_NONE);
            return true;
        }
        if leaf == self.shape {
            self.board.set(forest, 1, board::HIT_SHAPE);
            return true;
        }
        if leaf == self.pad {
            self.board.set(forest, 1, board::HIT_PAD);
            return true;
        }
        false
    }
}

// ---- who answers -----------------------------------------------------------------------------

const GRAB_CHILD_SIZE: i32 = 72;

fn grab_child_at() -> Location {
    Location::new().xs(
        50.pct().as_center_x().with(GRAB_CHILD_SIZE.px().as_width()),
        50.pct()
            .as_center_y()
            .with(GRAB_CHILD_SIZE.px().as_height()),
    )
}

struct Grab {
    board: Blueprint,
    parent: Leaf,
    /// One per [`board::GRAB_STEPS`]. Propagation is fixed as an element is grown, so the two
    /// modes are two elements rather than one that is rewritten.
    children: [Leaf; 2],
    /// Who reported the press being handled this frame, in arrival order.
    ///
    /// Collected rather than written straight to the readout because a pass-through press is
    /// *two* emissions, and each arrives as its own call -- writing on each would leave the row
    /// showing whichever happened to land last instead of both.
    heard: Vec<&'static str>,
}

fn grab(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::GRAB_ROWS,
        &board::GRAB_STEPS,
        &reference::GRAB,
    );
    let parent = g.forest.branch(
        board.stage,
        Panel::new()
            .color(role::surface())
            .rounding(Rounding::Xs)
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            .interactive(),
    );
    g.forest.branch(
        parent,
        Text::new(board::GRAB_PARENT)
            .size(FontSize::new(type_scale::LABEL))
            .color(role::on_surface())
            .at(blueprint::frame_label_at())
            .elevate(Elevation::up(1))
            .align(HorizontalAlignment::Left, VerticalAlignment::Middle)
            .pass_through(),
    );
    // Both carry a listener; only their propagation differs. The pass-through one is still told
    // about the gesture -- that is the whole distinction, and why it can still name itself in
    // the readout while never being the one that won.
    let grabbing = g.forest.branch(
        parent,
        Polygon::new()
            .sides(6.0)
            .rounding(0.3)
            .rotation(0.0)
            .color(role::accent())
            .at(grab_child_at())
            .elevate(Elevation::up(2))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            .interactive(),
    );
    blueprint::name(g.forest, grabbing, board::GRAB_CHILD);
    let passing = g.forest.branch(
        parent,
        Polygon::new()
            .sides(6.0)
            .rounding(0.3)
            .rotation(0.0)
            .color(Color::rose(400))
            .at(grab_child_at())
            .elevate(Elevation::up(2))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            .interactive()
            .pass_through(),
    );
    blueprint::name(g.forest, passing, board::GRAB_CHILD);
    g.forest.visible(passing, false);
    g.forest.disable(passing);
    board.set(g.forest, 0, board::GRAB_MODES[0]);
    board.set(g.forest, 1, board::GRAB_NONE);
    g.page.demos.push(Box::new(Grab {
        board,
        parent,
        children: [grabbing, passing],
        heard: Vec::new(),
    }));
}

impl Demo for Grab {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        if let Some(step) = self.board.pressed(leaf) {
            self.board.select(forest, step);
            for (i, child) in self.children.iter().enumerate() {
                forest.visible(*child, i == step);
                if i == step {
                    forest.enable(*child);
                } else {
                    forest.disable(*child);
                }
            }
            self.board.set(forest, 0, board::GRAB_MODES[step]);
            self.board.set(forest, 1, board::GRAB_NONE);
            self.heard.clear();
            return true;
        }
        if self.children.contains(&leaf) {
            self.heard.push(board::GRAB_CHILD);
            return true;
        }
        if leaf == self.parent {
            self.heard.push(board::GRAB_PARENT);
            return true;
        }
        false
    }
    fn drive(&mut self, forest: &mut Forest) {
        if self.heard.is_empty() {
            return;
        }
        let line = self.heard.join(", ");
        self.heard.clear();
        self.board.set(forest, 1, line);
    }
}

// ---- the gesture -----------------------------------------------------------------------------

struct Gesture {
    board: Blueprint,
    pad: Leaf,
    /// Between the press and the release. What the travel row is read for, since the pointer's
    /// own position is only meaningful while a gesture owns it.
    ///
    /// It gates the phase row as well, which is belt and braces rather than a fix for anything:
    /// a pointer move is only reported while the button is down, so a `Dragged` naming this pad
    /// always belongs to a press this board saw begin. What it does buy is the invariant being
    /// stated where the row is written.
    holding: bool,
    /// Whether this gesture has crossed the drag threshold -- which is to say, whether the
    /// release can still be a click.
    ///
    /// Taken from `DragStarted` rather than measured here. The threshold is applied per axis
    /// against a constant the engine owns, so a board that recomputed it would be a second
    /// copy of both -- and a straight-line reading of the travel row, the obvious thing to
    /// write, disagrees with the engine diagonally.
    past_threshold: bool,
}

fn gesture(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::live(g, column, STAGE_H, board::GESTURE_ROWS, &reference::GESTURE);
    let pad = g.forest.branch(
        board.stage,
        Panel::new()
            .color(role::surface())
            .rounding(Rounding::Xs)
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            // No grid on a view of its own, so a drag here walks up to the page's own view and
            // scrolls it as usual. The board reports the gesture; it does not keep it.
            .interactive(),
    );
    g.forest.branch(
        pad,
        Text::new(text::PAD)
            .size(FontSize::new(type_scale::LABEL))
            .color(role::on_surface_variant())
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                50.pct().as_center_y().with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(1))
            .align(HorizontalAlignment::Center, VerticalAlignment::Middle)
            .pass_through(),
    );
    board.set(g.forest, 0, board::GESTURE_IDLE);
    board.set(g.forest, 1, board::travel(0.0, false));
    g.page.demos.push(Box::new(Gesture {
        board,
        pad,
        holding: false,
        past_threshold: false,
    }));
}

impl Demo for Gesture {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        if leaf != self.pad {
            return false;
        }
        // Arrives after the release, and only when the gesture never became a drag -- so this
        // is the last word on the row rather than a state the pad is now in.
        self.board.set(forest, 0, board::GESTURE_CLICKED);
        true
    }
    fn gesture(&mut self, forest: &mut Forest, leaf: Leaf, phase: Phase) -> bool {
        if leaf != self.pad {
            return false;
        }
        let name = match phase {
            Phase::Engaged => {
                self.holding = true;
                self.past_threshold = false;
                board::GESTURE_ENGAGED
            }
            Phase::Dragged if self.holding => board::GESTURE_DRAGGED,
            Phase::Dragged => return true,
            // Not written to the phase row: it arrives on the same move as the `Dragged` that
            // would overwrite it, so a row showing it would show a frame of nothing. Where it
            // shows is the travel row's note, which is what the threshold actually decides.
            Phase::DragStarted => {
                self.past_threshold = true;
                return true;
            }
            Phase::Disengaged => {
                self.holding = false;
                board::GESTURE_DISENGAGED
            }
        };
        self.board.set(forest, 0, name);
        true
    }
    fn drive(&mut self, forest: &mut Forest) {
        if !self.holding {
            return;
        }
        let click = forest.pointer();
        let travelled = click.current.distance(click.start);
        self.board
            .set(forest, 1, board::travel(travelled, self.past_threshold));
    }
}

// ---- scroll ----------------------------------------------------------------------------------

/// One per [`board::SCROLL_STEPS`], as a fraction of the view's own range.
const SCROLL_STOPS: [f32; 3] = [0.0, 0.5, 1.0];
/// Bars inside the view, and what each is. Enough of them to overflow a stage-height box
/// several times over, which is what gives the view a range to be a fraction of.
const SCROLL_BARS: usize = 8;
const SCROLL_BAR_H: i32 = 28;
const SCROLL_BAR_GAP: i32 = space::SM;

struct Scroll {
    board: Blueprint,
    view: Leaf,
}

fn scroll(g: &mut Grow, column: &mut Column) {
    let board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::SCROLL_ROWS,
        &board::SCROLL_STEPS,
        &reference::SCROLL,
    );
    // The grid is what makes this a view. It is also what makes a drag landing here scroll
    // *this* rather than the page -- until it reaches its own end, at which point the rest is
    // handed outward, which is exactly the behaviour the paragraph above describes.
    let view = g.forest.branch(
        board.stage,
        Panel::new()
            .color(role::surface())
            .rounding(Rounding::Xs)
            .at(Location::new().xs(
                50.pct().as_center_x().with(70.pct().as_width()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0))),
    );
    for i in 0..SCROLL_BARS {
        let top = space::SM + i as i32 * (SCROLL_BAR_H + SCROLL_BAR_GAP);
        g.forest.branch(
            view,
            Panel::new()
                .color(if i == 0 || i + 1 == SCROLL_BARS {
                    role::accent()
                } else {
                    role::surface_container()
                })
                .rounding(Rounding::Xs)
                .at(Location::new().xs(
                    space::SM
                        .px()
                        .as_left()
                        .with(100.pct().as_right().adjust(-space::SM)),
                    top.px().as_top().with(SCROLL_BAR_H.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                // the bars are the content, not targets -- a press on one has to reach the
                // view under them or dragging the middle of the stack would do nothing
                .pass_through(),
        );
    }
    // The same inset the first bar has above it, below the last one.
    //
    // A view's extent grows to cover its contents and stops there, so without this the last bar
    // ends flush with the bottom edge while the first is inset from the top -- which reads as
    // the view still having somewhere to go, at exactly the moment it has arrived.
    g.forest.branch(
        view,
        Bare::new()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                (space::SM + SCROLL_BARS as i32 * (SCROLL_BAR_H + SCROLL_BAR_GAP))
                    .px()
                    .as_top()
                    .with(space::SM.px().as_height()),
            ))
            .elevate(Elevation::up(1))
            .pass_through(),
    );
    g.page.demos.push(Box::new(Scroll { board, view }));
}

impl Demo for Scroll {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(forest, step);
        forest.scroll(self.view, ScrollTo::y(SCROLL_STOPS[step]));
        true
    }
    fn drive(&mut self, forest: &mut Forest) {
        // Read back rather than remembered: the steps are not the only way this view moves, and
        // a row showing the last button pressed would be wrong the moment it is dragged.
        let offset = forest
            .scroll_offset(self.view)
            .map(|position| board::scrolled(position.top()))
            .unwrap_or_else(|| board::EMPTY_VALUE.to_string());
        let at = match forest.sample(self.view, Sap::ScrollProgress) {
            Some(Sample::Pair(_, y)) => board::progress(y),
            _ => board::EMPTY_VALUE.to_string(),
        };
        self.board.set(forest, 0, offset);
        self.board.set(forest, 1, at);
    }
}

// ---- text input ------------------------------------------------------------------------------

/// One per [`board::FIELD_STEPS`], in order.
const FIELD_CONSTRAINTS: [LineConstraint; 2] = [LineConstraint::Single, LineConstraint::Multiple];
const FIELD_H: i32 = 44;

struct Field {
    board: Blueprint,
    /// One per constraint. `LineConstraint` is chosen as the field is grown, so the two are two
    /// elements -- same as the propagation board above.
    fields: [Leaf; 2],
    /// `TextInput` no longer carries its own backdrop (a rounded one needs an inset to not clip
    /// glyphs sitting flush against it, and that inset is exactly what broke `click_at`'s corner
    /// fractions) -- these are the demo's own, one per `fields`, toggled alongside it since
    /// they're siblings rather than a parent that would cascade visibility for free.
    backdrops: [Leaf; 2],
}

fn field(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::FIELD_ROWS,
        &board::FIELD_STEPS,
        &reference::FIELD,
    );
    let grown = [0usize, 1].map(|i| {
        let outer = Location::new().xs(
            space::SM
                .px()
                .as_left()
                .with(100.pct().as_right().adjust(-space::SM)),
            // The multi-line field takes the rest of the stage, because wrapping is the
            // only thing that tells the two apart and one line of it would not.
            space::SM.px().as_top().with(if i == 0 {
                FIELD_H.px().as_height()
            } else {
                100.pct().as_bottom().adjust(-space::SM)
            }),
        );
        let backdrop = g.forest.branch(
            board.stage,
            Panel::new()
                .color(role::surface())
                .rounding(Rounding::Xs)
                .at(outer)
                .elevate(Elevation::up(1))
                .pass_through(),
        );
        // Inset from the backdrop by `space::XS`: enough that `Rounding::Xs`'s corner doesn't
        // clip a glyph sitting at the field's own edge, without a gap wide enough to look like
        // its own box.
        let leaf = g.forest.branch(
            board.stage,
            TextInput::new()
                .hint_text(board::FIELD_HINT)
                .line_constraint(FIELD_CONSTRAINTS[i])
                .font_size(FontSize::new(type_scale::BODY))
                .foreground(role::on_surface())
                .accent(role::accent())
                .at(Location::new().xs(
                    (space::SM + space::XS)
                        .px()
                        .as_left()
                        .with(100.pct().as_right().adjust(-(space::SM + space::XS))),
                    (space::SM + space::XS).px().as_top().with(if i == 0 {
                        (FIELD_H - space::XS * 2).px().as_height()
                    } else {
                        100.pct().as_bottom().adjust(-(space::SM + space::XS))
                    }),
                ))
                .elevate(Elevation::up(2)),
        );
        (backdrop, leaf)
    });
    let backdrops = [grown[0].0, grown[1].0];
    let fields = [grown[0].1, grown[1].1];
    g.forest.visible(fields[1], false);
    g.forest.disable(fields[1]);
    g.forest.visible(backdrops[1], false);
    board.set(g.forest, 0, board::FIELD_EMPTY);
    board.set(g.forest, 1, board::FIELD_NONE);
    g.page.demos.push(Box::new(Field {
        board,
        fields,
        backdrops,
    }));
}

impl Demo for Field {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(forest, step);
        for (i, field) in self.fields.iter().enumerate() {
            forest.visible(*field, i == step);
            forest.visible(self.backdrops[i], i == step);
            if i == step {
                forest.enable(*field);
            } else {
                forest.disable(*field);
            }
        }
        self.board.set(forest, 0, board::FIELD_EMPTY);
        self.board.set(forest, 1, board::FIELD_NONE);
        true
    }
    fn focus(&mut self, forest: &mut Forest, leaf: Leaf, has: bool) -> bool {
        if !self.fields.contains(&leaf) {
            return false;
        }
        self.board.set(
            forest,
            1,
            if has {
                board::FIELD_FOCUSED
            } else {
                board::FIELD_UNFOCUSED
            },
        );
        true
    }
    fn typed(&mut self, forest: &mut Forest, leaf: Leaf, value: &str) -> bool {
        if !self.fields.contains(&leaf) {
            return false;
        }
        // Cut at the strip rather than written whole: the multi-line field holds far more than
        // one row of readout can show, and a value running off the end of it took the row's
        // own label with it.
        self.board.set(forest, 0, board::typed(value));
        self.board.set(forest, 1, board::FIELD_CHANGED);
        true
    }
    fn acted(&mut self, forest: &mut Forest, leaf: Leaf, action: TextInputAction) -> bool {
        // Every binding the field matches is reported; only this one is a submission, and the
        // rest have already said what they did through the value row.
        if action != TextInputAction::Enter || !self.fields.contains(&leaf) {
            return false;
        }
        self.board.set(forest, 1, board::FIELD_SUBMITTED);
        true
    }
}
