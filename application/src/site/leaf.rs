//! The `leaf` section.
//!
//! Every board is a parent with one child declared inside it, and a control bar along the bottom
//! that never moves. The bar acts on the parent; the parent and child are drawn and nothing
//! else, out of the hit test entirely. Nothing the reader is touching resizes, disappears, or
//! scrolls away between one press and the next.
//!
//! The readout pairs what the child *declares*, written once at spawn and never rewritten, with
//! what that currently *resolves to*. One stays still while the other moves, and that gap is
//! what a stem is.

use foliage::{
    Canopy, Color, Elevation, FontSize, Grid, GridExt, Grows, HorizontalAlignment, Leaf, Location,
    Panel, Polygon, Presence, Rounding, Sprout, Text, VerticalAlignment,
};

use crate::site::blueprint::{self, Blueprint, Entry};
use crate::site::{Column, Demo, Grow, SCROLL_TAIL, motion, role, space, type_scale};

const STAGE_H: (i32, i32, i32) = (150, 165, 190);

/// The widths a parent cycles through, as a percentage of the stage. Anchored at the left, so it
/// grows and shrinks against a fixed edge rather than sliding across the field.
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

    column.display(g.canopy, "leaf");
    column.lead(
        g.canopy,
        "An element on screen is one entity, and what you keep is a handle to it -- the type is \
         spelled `Leaf`. Growing one under another is what the name means. Each board below is \
         live: press its bar, and read what the parent decided for the child.",
    );

    column.heading(g.canopy, "resolving");
    column.prose(
        g.canopy,
        "A child states its box in percentages of its parent, so one declaration is a different \
         number of pixels in a different parent. The top row never changes. The bottom row is \
         what the tree currently makes of it.",
    );
    resolving(g, &mut column);

    column.heading(g.canopy, "clipping");
    column.prose(
        g.canopy,
        "A parent is a boundary as well as an origin. This child is declared in pixels, so it \
         cannot shrink along with its parent -- narrow the parent and the child is cut at the \
         edge, while the box it asked for stays exactly what it was.",
    );
    clipping(g, &mut column);

    column.heading(g.canopy, "inheriting");
    column.prose(
        g.canopy,
        "Some of what you write to a parent reaches everything beneath it, and some of it stops \
         there. Which is which is not guessable, so the board writes both to the same parent and \
         you read the child.",
    );
    inheriting(g, &mut column);

    column.heading(g.canopy, "lifetime");
    column.prose(
        g.canopy,
        "The stem decides how long a thing lives. One call names the parent, the child is never \
         mentioned, and the dashed outline is the room they occupied.",
    );
    lifetime(g, &mut column);

    column.tail(g.canopy, SCROLL_TAIL);
}

/// A parent and its own label, kept together because the label reports the parent's current
/// width and has to be rewritten whenever that changes.
struct Frame {
    leaf: Leaf,
    label: Leaf,
}

/// `filled` draws it as a solid plane instead of an outlined box. The inheriting board needs
/// that: written to an outline, a colour change moves a two-pixel border and reads as the box
/// being highlighted rather than as the parent itself changing.
fn frame(canopy: &mut Canopy, stage: Leaf, width: f32, filled: bool) -> Frame {
    let panel = Panel::new()
        .color(if filled {
            role::surface()
        } else {
            role::on_surface_variant()
        })
        .rounding(Rounding::Xs)
        .at(frame_at(width))
        .elevate(Elevation::up(1))
        .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
        // `Grid` is `#[require(View)]`, so this is a scrollable view, and an overhanging child
        // gives it real extent to pan. `disable_drag` cannot protect it: `ovrscrl` hands a view's
        // unabsorbed remainder to its parent without consulting propagation, so a drag landing on
        // the child below flows up into this and slides the parent under the reader's thumb.
        // Refusing the gesture outright keeps the whole board out of the running, and the page --
        // which is what a drag here is for -- gets it instead.
        .pass_through();
    let leaf = canopy.branch(stage, if filled { panel } else { panel.outline(2) });
    let label = canopy.branch(
        leaf,
        Text::new(frame_label(width))
            // A filled frame is written a new surface tone mid-demo, and the variant label sits a
            // step from it on the same ramp. The stronger tone holds against both surfaces.
            .size(FontSize::new(type_scale::LABEL))
            .color(if filled {
                role::on_surface()
            } else {
                role::on_surface_variant()
            })
            .at(frame_label_at())
            .elevate(Elevation::up(1))
            .align(HorizontalAlignment::Left, VerticalAlignment::Middle)
            .pass_through(),
    );
    Frame { leaf, label }
}

/// Where a frame's own label sits. Shared so the lifetime board's "no parent" marker can occupy
/// the same slot rather than a hand-matched copy of it that drifts the first time either moves.
fn frame_label_at() -> Location {
    Location::new().xs(
        // Sized in characters, not px: the widest label this holds is "parent 100%", and one
        // line of it. Stated that way the box tracks the type scale instead of being a pair of
        // numbers that happen to fit the size it was written against.
        space::SM.px().as_left().with(11.letters().as_width()),
        space::XS.px().as_top().with(1.letters().as_height()),
    )
}

fn frame_label(width: f32) -> String {
    format!("parent {}%", width as i32)
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
    // Short forms, unlike the other boards' "parent NN%". This label lives inside the frame, and
    // the frame is down to about 134px at the narrowest breakpoint by the last step -- so the
    // words have to fit the narrowed parent, not just their own box. The readout says "parent"
    // on the row above anyway.
    canopy.text(frame.label, if right.is_none() { "full" } else { "narrowed" });
}

/// Resizes a parent and keeps its label honest.
fn resize(canopy: &mut Canopy, frame: &Frame, width: f32) {
    canopy.location(frame.leaf, frame_at(width));
    canopy.text(frame.label, frame_label(width));
}

/// The child: filled, and named on itself, so the two never read as the same kind of thing.
///
/// A shape, which `Polygon` squares to `min(width, height)` via its own `AspectRatio` so the
/// rounded corners stay circular. That is right wherever a cut corner or a vanished shape is what
/// the board is showing, and wrong wherever the readout is two numbers -- see [`child_box`].
fn child(canopy: &mut Canopy, parent: Leaf, at: Location, tone: Color) -> Leaf {
    let child = canopy.branch(
        parent,
        Polygon::new()
            .sides(6.0)
            .rounding(0.3)
            .rotation(0.0)
            .color(tone)
            .at(at)
            .elevate(Elevation::up(2))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            .pass_through(),
    );
    name_child(canopy, child);
    child
}

/// The same child as a plain box, for the one board that reads its size back as a number.
///
/// A squared child cannot show a declaration resolving: its width is `min(width, height)`, its
/// height is a share of a stage that does not move, and on any screen wide enough for the height
/// to be the smaller of the two, the size reads the same in every parent. A `Panel` resolves each
/// axis on its own, so the width the parent actually decided is the width reported.
fn child_box(canopy: &mut Canopy, parent: Leaf, at: Location, tone: Color) -> Leaf {
    let child = canopy.branch(
        parent,
        Panel::new()
            .color(tone)
            .rounding(Rounding::Xs)
            .at(at)
            .elevate(Elevation::up(2))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            .pass_through(),
    );
    name_child(canopy, child);
    child
}

fn name_child(canopy: &mut Canopy, child: Leaf) {
    canopy.branch(
        child,
        Text::new("child")
            .size(FontSize::new(type_scale::LABEL))
            .color(role::on_accent())
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                50.pct().as_center_y().with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(1))
            .align(HorizontalAlignment::Center, VerticalAlignment::Middle)
            .pass_through(),
    );
}

/// Below the parent's own label, so a narrow parent never puts the two on top of each other.
fn child_band(left: f32, right: f32) -> Location {
    Location::new().xs(
        left.pct().as_left().with(right.pct().as_right()),
        36.pct().as_top().with(90.pct().as_bottom()),
    )
}

fn board(
    g: &mut Grow,
    column: &mut Column,
    labels: [&'static str; 2],
    control: &'static str,
    entries: &[Entry],
) -> Blueprint {
    let seq = column.sequence();
    let region = column.region(g.canopy, blueprint::height(STAGE_H), space::LG);
    let board = Blueprint::grow(g, region, labels, control, seq, motion::STAGGER);
    let table = column.region_letters(
        g.canopy,
        blueprint::reference_letters(entries.len()),
        blueprint::reference_extra(entries.len()),
        type_scale::LABEL,
        space::SM,
    );
    blueprint::reference(g, table, entries, seq, motion::STAGGER * 2);
    board
}

fn resolved(canopy: &mut Canopy, leaf: Leaf) -> String {
    match canopy.section(leaf) {
        Some(s) => format!("{} x {}", s.width() as i32, s.height() as i32),
        None => "--".to_string(),
    }
}

// ---- resolving -----------------------------------------------------------------------------

struct Resolving {
    board: Blueprint,
    frame: Frame,
    child: Leaf,
    step: usize,
}

fn resolving(g: &mut Grow, column: &mut Column) {
    let entries = [
        Entry {
            call: "Location::new().xs(h, v)",
            gloss: "The box, per breakpoint. A percentage is of the parent; px is absolute.",
        },
        Entry {
            call: "22.pct().as_left().with(..)",
            gloss: "One edge, then the opposite one. Width and right are interchangeable.",
        },
        Entry {
            call: "canopy.section(leaf)",
            gloss: "What that declaration works out to right now, in real pixels.",
        },
        Entry {
            call: "anchor()",
            gloss: "Resolve against a named element rather than the parent, for stacking.",
        },
    ];
    // Both boxes, not just the child's. The pair is what the section is about -- one number is
    // half of a ratio -- and it also says plainly which of the two a resize actually moved.
    let board = board(
        g,
        column,
        ["parent", "child"],
        "resize the parent",
        &entries,
    );
    let frame = frame(g.canopy, board.stage, FRAME_WIDTHS[0], false);
    let child = child_box(
        g.canopy,
        frame.leaf,
        child_band(CHILD_LEFT, CHILD_RIGHT),
        child_tone(),
    );
    g.page.demos.push(Box::new(Resolving {
        board,
        frame,
        child,
        step: 0,
    }));
}

impl Demo for Resolving {
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        if leaf != self.board.control {
            return false;
        }
        self.step = (self.step + 1) % FRAME_WIDTHS.len();
        resize(canopy, &self.frame, FRAME_WIDTHS[self.step]);
        true
    }
    fn drive(&mut self, canopy: &mut Canopy) {
        let parent = resolved(canopy, self.frame.leaf);
        let child = resolved(canopy, self.child);
        self.board.set(canopy, 0, parent);
        self.board.set(canopy, 1, child);
    }
}

// ---- clipping ------------------------------------------------------------------------------

/// The clipping board's child: the same shape [`child`] grows, centred on the *stage*.
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

/// The clipping board's child: the same shape [`child`] grows, placed by [`Clipping::drive`].
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
    name_child(canopy, child);
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
    let entries = [
        Entry {
            call: "clip = every ancestor's box",
            gloss: "An element draws only where its own box and all of its parents' overlap.",
        },
        Entry {
            call: "canopy.section(leaf)",
            gloss: "Reports the box it asked for, whether or not all of it is drawn.",
        },
        Entry {
            call: "ClipToViewport",
            gloss: "Opts out, and is bounded by the window instead of by the parent chain.",
        },
    ];
    let board = board(
        g,
        column,
        ["parent", "child"],
        "resize the parent",
        &entries,
    );
    let frame = frame(g.canopy, board.stage, FRAME_WIDTHS[0], false);
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
        if leaf != self.board.control {
            return false;
        }
        self.step = (self.step + 1) % (CLIP_KEPT.len() + 1);
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
        let parent = resolved(canopy, self.frame.leaf);
        let child = resolved(canopy, self.child);
        self.board.set(canopy, 0, parent);
        self.board.set(canopy, 1, child);
    }
}

// ---- inheriting ----------------------------------------------------------------------------

struct Inheriting {
    board: Blueprint,
    frame: Frame,
    step: usize,
}

fn inheriting(g: &mut Grow, column: &mut Column) {
    let entries = [
        Entry {
            call: "canopy.opacity(leaf, to)",
            gloss: "Inherited. Everything beneath is drawn through the parent's value.",
        },
        Entry {
            call: "canopy.color(leaf, to)",
            gloss: "Not inherited. It is one element's own component and stops there.",
        },
        Entry {
            call: "canopy.visible(leaf, yes)",
            gloss: "Inherited, and the subtree keeps its state while it is hidden.",
        },
        Entry {
            call: "canopy.disable(leaf)",
            gloss: "Inherited. The subtree still draws, and stops taking input.",
        },
    ];
    let mut board = board(
        g,
        column,
        ["wrote", "child"],
        "write to the parent",
        &entries,
    );
    // Filled, so a colour write is visibly the parent's own surface changing. Full width like the
    // boards above it -- nothing here resizes the parent, so a narrow one is just a smaller stage.
    let frame = frame(g.canopy, board.stage, FRAME_WIDTHS[0], true);
    child(
        g.canopy,
        frame.leaf,
        child_band(CHILD_LEFT, CHILD_RIGHT),
        child_tone(),
    );
    board.set(g.canopy, 0, "nothing yet");
    g.page.demos.push(Box::new(Inheriting {
        board,
        frame,
        step: 0,
    }));
}

impl Demo for Inheriting {
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        if leaf != self.board.control {
            return false;
        }
        self.step = (self.step + 1) % 3;
        match self.step {
            1 => {
                canopy.opacity(self.frame.leaf, 0.6);
                self.board.set(canopy, 0, "opacity 0.6");
                self.board.set(canopy, 1, "faded as well");
            }
            2 => {
                canopy.opacity(self.frame.leaf, 1.0);
                canopy.color(self.frame.leaf, parent_write_tone());
                self.board.set(canopy, 0, "a new color");
                self.board.set(canopy, 1, "kept its own");
            }
            _ => {
                canopy.color(self.frame.leaf, role::surface());
                self.board.set(canopy, 0, "nothing yet");
                self.board.set(canopy, 1, "--");
            }
        }
        true
    }
}

// ---- lifetime ------------------------------------------------------------------------------

/// Placed, then filled, then pruned. Building it in two steps is what makes the third one land:
/// you put the parent down and the child in it yourself, so when one call takes both away it is
/// visibly two things going, not one shape vanishing.
#[derive(Copy, Clone, PartialEq)]
enum Stage {
    Empty,
    ParentOnly,
    Both,
    /// Pruned, with the handle still held. Its own step because a withered handle is a thing you
    /// can still read, and reading it is the only way to see that state: the slot markers stay
    /// off here, so the row saying "withered" is never contradicted by a stage saying "no child".
    Pruned,
}

struct Lifetime {
    board: Blueprint,
    stage: Leaf,
    frame: Option<Frame>,
    child: Option<Leaf>,
    at: Stage,
    /// One marker per slot, each sitting exactly where the thing it stands for will appear -- the
    /// parent's own label position, and the child's band. An outline on its own is a drawing
    /// decision the reader has to interpret; a word in the slot is the board saying that nothing
    /// is there yet, and saying it in the place you are about to watch fill.
    empty: [Leaf; 2],
}

fn lifetime(g: &mut Grow, column: &mut Column) {
    let entries = [
        Entry {
            call: "canopy.prune(leaf)",
            gloss: "Removes it and everything beneath it, in the one call.",
        },
        Entry {
            call: "canopy.presence(leaf)",
            gloss: "Planted while it is still being grown, then Live, then Withered.",
        },
        Entry {
            call: "Bloom::Withered(leaf)",
            gloss: "Reported once per element that went, after the frame applies it.",
        },
        Entry {
            call: "a withered handle",
            gloss: "Takes writes and drops them. Nothing panics, and it is never reused.",
        },
    ];
    let mut board = board(
        g,
        column,
        ["called", "child"],
        "place the parent",
        &entries,
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
            "no parent",
            frame_label_at(),
            HorizontalAlignment::Left,
        ),
        marker(
            g.canopy,
            "no child",
            child_band(CHILD_LEFT, CHILD_RIGHT),
            HorizontalAlignment::Center,
        ),
    ];
    let stage = board.stage;
    board.set(g.canopy, 0, "nothing yet");
    board.set(g.canopy, 1, "none");
    g.page.demos.push(Box::new(Lifetime {
        board,
        stage,
        frame: None,
        child: None,
        at: Stage::Empty,
        empty,
    }));
}

impl Demo for Lifetime {
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool {
        if leaf != self.board.control {
            return false;
        }
        match self.at {
            Stage::Empty => {
                // Filled, not outlined: the room is already an outline, and placing a second one
                // over it is a press that appears to do nothing.
                self.frame = Some(frame(canopy, self.stage, FRAME_WIDTHS[0], true));
                canopy.visible(self.empty[0], false);
                self.at = Stage::ParentOnly;
                self.board.set(canopy, 0, "canopy.leaf(..)");
                self.board.label(canopy, "add the child");
            }
            Stage::ParentOnly => {
                let parent = self.frame.as_ref().unwrap().leaf;
                self.child = Some(child(
                    canopy,
                    parent,
                    child_band(CHILD_LEFT, CHILD_RIGHT),
                    child_tone(),
                ));
                canopy.visible(self.empty[1], false);
                self.at = Stage::Both;
                self.board.set(canopy, 0, "branch(parent)");
                self.board.label(canopy, "prune the parent");
            }
            Stage::Both => {
                // Only the parent is named. Both go -- and the handle is kept, because the next
                // step exists to read it. The slot markers stay off: they say "nothing placed
                // yet", which is not what just happened here.
                canopy.prune(self.frame.take().unwrap().leaf);
                self.at = Stage::Pruned;
                self.board.set(canopy, 0, "prune(parent)");
                self.board.label(canopy, "clear the handle");
            }
            Stage::Pruned => {
                // Dropping the handle is what takes the row back to "none", so an empty board
                // reads the same on every pass through.
                self.child = None;
                for slot in self.empty {
                    canopy.visible(slot, true);
                }
                self.at = Stage::Empty;
                self.board.set(canopy, 0, "nothing yet");
                self.board.label(canopy, "place the parent");
            }
        }
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
            (Stage::Pruned, _) => "withered",
            (_, None) => "none",
            (_, Some(child)) => match canopy.presence(child) {
                Presence::Planted => "growing",
                Presence::Live => "live",
                Presence::Withered => "withered",
            },
        };
        self.board.set(canopy, 1, text);
    }
}
