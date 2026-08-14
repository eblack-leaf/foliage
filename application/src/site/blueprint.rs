//! A field with a fixed control row, a stage, and a live readout.
//!
//! The controls sit in their own strip along the bottom and never move, whatever they do to the
//! stage above them. That is the entire reason they are there: the thing being demonstrated has
//! to be free to resize and disappear, and a control that *is* that thing walks out from under
//! the reader between one tap and the next.
//!
//! One control per step, rather than one that cycles. A cycling button can only ever say what
//! the *next* press does, which leaves a board whose steps are the whole subject unable to show
//! how many there are or which one you are on; a row shows the sequence, marks your place in it,
//! and lets you go back a step without going round again.
//!
//! The readout is two rows, read back off the tree each frame, set into the field as a recessed
//! panel so the numbers read as an instrument's rather than as two more lines of page.

use foliage::{
    Bare, Canopy, Color, Elevation, FontSize, Grid, GridExt, Grows, HorizontalAlignment, Leaf,
    Location, LocationValue, Panel, Polygon, Rounding, Sprout, Text, VerticalAlignment,
};

use crate::site::{
    Column, Grow, POLY_STEP_ROW_H, StepControl, background, fade_in, motion, poly_step, role,
    space, type_scale,
};

const ROW_H: i32 = 18;
const READOUT_ROWS: usize = 2;
const READOUT_H: i32 = ROW_H * READOUT_ROWS as i32;
/// Air inside the readout's own panel, above the first row and below the last.
const READOUT_PAD: i32 = space::SM;
const STRIP_H: i32 = READOUT_H + READOUT_PAD * 2;
/// The label column, in a strip that is about 240px wide inside its padding at the narrowest
/// breakpoint. Every label here is one word set in caps -- six characters at most -- so the
/// column is sized to the words and the rest of the strip goes to the value, which is the part
/// that moves.
const LABEL_W: i32 = 56;
/// Between the label column and the value, with the divider down the middle of it.
const LABEL_GAP: i32 = space::MD;

/// Bottom up: the control row, the readout above it, then the stage. Every one of these has to
/// be counted here or they land on top of each other.
const CONTROLS_FROM_BOTTOM: i32 = space::MD;
const ABOVE_STAGE: i32 = space::MD;

/// Where the readout's strip sits above the field's bottom edge.
///
/// A board with no step row ([`live`]) reclaims exactly the row plus the gap that separated the
/// two, rather than leaving a band of empty field where the controls would have been -- which
/// is the whole visible difference between the two kinds.
const fn readout_from_bottom(stepped: bool) -> i32 {
    if stepped {
        CONTROLS_FROM_BOTTOM + POLY_STEP_ROW_H + space::MD
    } else {
        CONTROLS_FROM_BOTTOM
    }
}

const fn below_stage(stepped: bool) -> i32 {
    readout_from_bottom(stepped) + STRIP_H + space::SM
}

/// What a region has to be to hold a stage of `stage_h` plus the readout, and the control row
/// if there is one.
pub(crate) fn height(stage_h: (i32, i32, i32), stepped: bool) -> (i32, i32, i32) {
    let extra = ABOVE_STAGE + below_stage(stepped);
    (stage_h.0 + extra, stage_h.1 + extra, stage_h.2 + extra)
}

pub(crate) struct Blueprint {
    /// Where the demo's frame goes. Carries a grid, so its contents place in percentages of it.
    pub(crate) stage: Leaf,
    /// The things on the board that take a tap, in the order they are meant to be pressed.
    steps: Vec<StepControl>,
    /// Which one is lit. The board's own copy of the demo's position, kept here because the
    /// mark is this module's to move.
    at: usize,
    values: [Leaf; READOUT_ROWS],
    /// What each row currently says. A readout is driven from `drive`, which runs every frame,
    /// and re-sending the same string sixty times a second is a write storm for no change --
    /// so a row is only written when its text actually differs.
    shown: [String; READOUT_ROWS],
}

impl Blueprint {
    /// `steps` is the row, left to right. Keep the words short -- four of them share the field's
    /// width, which is about 64px a slot at the narrowest breakpoint.
    ///
    /// An empty `steps` builds the board without a control row at all, for the boards whose
    /// stage *is* the control -- see [`live`].
    pub(crate) fn grow(
        g: &mut Grow,
        region: Leaf,
        labels: [&'static str; READOUT_ROWS],
        steps: &[&'static str],
        seq: Leaf,
        start: u64,
    ) -> Self {
        let stepped = !steps.is_empty();
        let field = g.canopy.branch(
            region,
            Panel::new()
                .color(role::surface_container())
                .rounding(Rounding::Xs)
                .at(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    0.px().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
                .opacity(0.0),
        );
        fade_in(g.canopy, field, seq, start);

        // Across the bottom, in the same shapes and the same three tones the hero's destinations
        // use. The bar this replaced was deliberately chrome-toned so it would not read as part
        // of the drawing above it -- what keeps that true now is the row's place rather than its
        // colour: it is below the readout, in a strip nothing else ever enters, and only one of
        // its shapes is lit at a time.
        let count = steps.len();
        let mut controls_built = Vec::with_capacity(count);
        if stepped {
            let controls = g.canopy.branch(
                field,
                Bare::new()
                    .at(Location::new().xs(
                        space::MD
                            .px()
                            .as_left()
                            .with(100.pct().as_right().adjust(-space::MD)),
                        POLY_STEP_ROW_H
                            .px()
                            .as_height()
                            .with(100.pct().as_bottom().adjust(-CONTROLS_FROM_BOTTOM)),
                    ))
                    .elevate(Elevation::up(3))
                    .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
                    // a full-width box across the foot of every board would otherwise take the
                    // drag meant for the page and hand it nowhere
                    .pass_through(),
            );
            for (i, text) in steps.iter().enumerate() {
                controls_built.push(poly_step(
                    g,
                    controls,
                    text,
                    i,
                    count,
                    seq,
                    start + i as u64 * crate::site::motion::STAGGER,
                ));
            }
        }

        let stage = g.canopy.branch(
            field,
            Bare::new()
                .at(Location::new().xs(
                    space::MD
                        .px()
                        .as_left()
                        .with(100.pct().as_right().adjust(-space::MD)),
                    ABOVE_STAGE
                        .px()
                        .as_top()
                        .with(100.pct().as_bottom().adjust(-below_stage(stepped))),
                ))
                .elevate(Elevation::up(1))
                .grid(Grid::new(1.col().gap(0), 1.row().gap(0))),
        );

        // Recessed rather than written straight onto the field: in the page background, which is
        // a step *darker* than the field it sits in, so the numbers read as being behind glass.
        // Loose text on the field was two more lines of page below a drawing, which is exactly
        // what a live reading is not.
        let strip = g.canopy.branch(
            field,
            Panel::new()
                .color(background())
                .rounding(Rounding::Xs)
                .at(Location::new().xs(
                    space::MD
                        .px()
                        .as_left()
                        .with(100.pct().as_right().adjust(-space::MD)),
                    STRIP_H
                        .px()
                        .as_height()
                        .with(100.pct().as_bottom().adjust(-readout_from_bottom(stepped))),
                ))
                .elevate(Elevation::up(2))
                .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
                .pass_through(),
        );
        // Full height of the strip and one pixel wide, between the names and the numbers. It is
        // what makes two rows a table: without it the labels and the values are four texts that
        // happen to line up, and a value reading "--" leaves its row looking unfinished.
        g.canopy.branch(
            strip,
            Panel::new()
                .color(role::outline())
                .rounding(Rounding::None)
                .at(Location::new().xs(
                    (READOUT_PAD + LABEL_W + LABEL_GAP / 2)
                        .px()
                        .as_left()
                        .with(1.px().as_width()),
                    READOUT_PAD
                        .px()
                        .as_top()
                        .with(100.pct().as_bottom().adjust(-READOUT_PAD)),
                ))
                .elevate(Elevation::up(1))
                .pass_through(),
        );

        let row = |i: usize, canopy: &mut Canopy| {
            let top = READOUT_PAD + i as i32 * ROW_H;
            canopy.branch(
                strip,
                // Capped, like every other structural word on the site. A row's name is a field
                // on an instrument, not a sentence about the value beside it, and caps at the
                // quieter tone say that without spending a second type size on it.
                Text::new(labels[i].to_uppercase())
                    .size(FontSize::new(type_scale::LABEL))
                    .color(role::on_surface_heading())
                    .at(Location::new().xs(
                        READOUT_PAD.px().as_left().with(LABEL_W.px().as_width()),
                        top.px().as_top().with(ROW_H.px().as_height()),
                    ))
                    .elevate(Elevation::up(1))
                    .align(HorizontalAlignment::Left, VerticalAlignment::Middle)
                    .pass_through(),
            );
            canopy.branch(
                strip,
                Text::new(crate::site::copy::board::EMPTY_VALUE)
                    .size(FontSize::new(type_scale::LABEL))
                    .color(role::on_surface())
                    .at(Location::new().xs(
                        (READOUT_PAD + LABEL_W + LABEL_GAP)
                            .px()
                            .as_left()
                            .with(100.pct().as_right().adjust(-READOUT_PAD)),
                        top.px().as_top().with(ROW_H.px().as_height()),
                    ))
                    .elevate(Elevation::up(1))
                    .align(HorizontalAlignment::Left, VerticalAlignment::Middle)
                    .pass_through(),
            )
        };
        let values = [row(0, g.canopy), row(1, g.canopy)];

        let board = Self {
            stage,
            steps: controls_built,
            at: 0,
            values,
            shown: [
                String::from(crate::site::copy::board::EMPTY_VALUE),
                String::from(crate::site::copy::board::EMPTY_VALUE),
            ],
        };
        // The first step is lit from the start, because it is the state the board is already in
        // -- a row with nothing marked would say the demo has not begun, when what it is showing
        // *is* step one.
        for (i, step) in board.steps.iter().enumerate() {
            step.select(g.canopy, i == board.at);
        }
        board
    }

    /// Which step's button this leaf is, if it is one of them at all.
    pub(crate) fn pressed(&self, leaf: Leaf) -> Option<usize> {
        self.steps.iter().position(|s| s.button == leaf)
    }

    /// Moves the mark to `step`. Nothing else about the board changes -- what a step *does* is
    /// the demo's, and this is only the row saying where you are.
    ///
    /// A no-op on a board with no control row, which cannot be told to select anything anyway:
    /// [`pressed`](Self::pressed) never answers, so nothing ever calls this with a step.
    pub(crate) fn select(&mut self, canopy: &mut Canopy, step: usize) {
        if self.steps.is_empty() || self.at == step {
            return;
        }
        self.steps[self.at].select(canopy, false);
        self.steps[step].select(canopy, true);
        self.at = step;
    }

    pub(crate) fn set(&mut self, canopy: &mut Canopy, row: usize, text: impl Into<String>) {
        let text = text.into();
        if self.shown[row] == text {
            return;
        }
        canopy.text(self.values[row], text.clone());
        self.shown[row] = text;
    }
}

/// A region: [`Blueprint::grow`]'s stage and control row, plus the reference table under it.
/// Every demo section on the site is this shape once -- a step-controlled field sized for
/// `stage_h`, and a card of the calls it stands for -- so a section only has to say what fills
/// the stage.
pub(crate) fn board(
    g: &mut Grow,
    column: &mut Column,
    stage_h: (i32, i32, i32),
    labels: [&'static str; 2],
    steps: &[&'static str],
    entries: &[Entry],
) -> Blueprint {
    grown(g, column, stage_h, labels, steps, entries)
}

/// The same region with no control row: a stage, a readout, and the reference table.
///
/// For the boards whose stage *is* the control -- where what is being shown is the reader's own
/// gesture, and a row of buttons would be a second way to do the thing the board is asking you
/// to do directly. Only `input` has any, and only because a press, a drag and a release are not
/// states you can select.
pub(crate) fn live(
    g: &mut Grow,
    column: &mut Column,
    stage_h: (i32, i32, i32),
    labels: [&'static str; 2],
    entries: &[Entry],
) -> Blueprint {
    grown(g, column, stage_h, labels, &[], entries)
}

fn grown(
    g: &mut Grow,
    column: &mut Column,
    stage_h: (i32, i32, i32),
    labels: [&'static str; 2],
    steps: &[&'static str],
    entries: &[Entry],
) -> Blueprint {
    let seq = column.sequence();
    let region = column.region(g.canopy, height(stage_h, !steps.is_empty()), space::LG);
    let board = Blueprint::grow(g, region, labels, steps, seq, motion::STAGGER);
    let table = column.region_letters(
        g.canopy,
        reference_letters(entries.len()),
        reference_extra(entries.len()),
        type_scale::LABEL,
        space::SM,
    );
    reference(g, table, entries, seq, motion::STAGGER * 2);
    board
}

/// A board's readout, read back off the tree: the leaf's resolved box in real pixels, or `--`
/// before anything has been measured.
pub(crate) fn resolved(canopy: &mut Canopy, leaf: Leaf) -> String {
    match canopy.section(leaf) {
        Some(s) => format!("{} x {}", s.width() as i32, s.height() as i32),
        None => "--".to_string(),
    }
}

/// Where a frame's own label sits, in the top-left of every box a board draws. Sized in
/// characters, not px: the widest label held here is "parent 100%" -- 11 letters, one line.
pub(crate) fn frame_label_at() -> Location {
    Location::new().xs(
        space::SM.px().as_left().with(11.letters().as_width()),
        space::XS.px().as_top().with(1.letters().as_height()),
    )
}

/// A labeled box: the shape and its own top-left caption, kept together because the caption
/// reports the shape's current declaration and has to be rewritten whenever that changes.
pub(crate) struct Frame {
    pub(crate) leaf: Leaf,
    pub(crate) label: Leaf,
}

/// A box at `at`, filled or outlined, captioned with `label`. `filled` draws it as a solid
/// plane instead of an outlined box -- a write that colors a filled box is visibly the box's
/// own surface changing, where an outline only moves a two-pixel border.
///
/// What decides `at` and `label` is the caller's: a percent width, a capped percent, a grid
/// line range are all just a [`Location`], and the box drawn for one is drawn the same as any
/// other.
pub(crate) fn frame(
    canopy: &mut Canopy,
    stage: Leaf,
    at: Location,
    label: impl Into<String>,
    filled: bool,
) -> Frame {
    let panel = Panel::new()
        .color(if filled {
            role::surface()
        } else {
            role::on_surface_variant()
        })
        .rounding(Rounding::Xs)
        .at(at)
        .elevate(Elevation::up(1))
        .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
        // `Grid` is `#[require(View)]`, so this is a scrollable view, and an overhanging child
        // gives it real extent to pan. `disable_drag` cannot protect it: `ovrscrl` hands a
        // view's unabsorbed remainder to its parent without consulting propagation, so a drag
        // landing on the child below flows up into this and slides the parent under the
        // reader's thumb. Refusing the gesture outright keeps the whole board out of the
        // running, and the page -- which is what a drag here is for -- gets it instead.
        .pass_through();
    let leaf = canopy.branch(stage, if filled { panel } else { panel.outline(2) });
    let label = canopy.branch(
        leaf,
        Text::new(label.into())
            // A filled frame is written a new surface tone mid-demo, and the label sits a step
            // from it on the same ramp. The stronger tone holds against both surfaces.
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

/// Labels a shape at its own center, pass-through so the name never wins the hit-test over
/// the shape carrying it.
pub(crate) fn name(canopy: &mut Canopy, shape: Leaf, text: impl Into<String>) -> Leaf {
    canopy.branch(
        shape,
        Text::new(text.into())
            .size(FontSize::new(type_scale::LABEL))
            .color(role::on_accent())
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                50.pct().as_center_y().with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(1))
            .align(HorizontalAlignment::Center, VerticalAlignment::Middle)
            .pass_through(),
    )
}

/// A named hexagon: the shape every demo grows for something that behaves like a dependent --
/// a child, a target, whatever else the section is showing settle against something else.
///
/// Squares to `min(width, height)` via `Polygon`'s own `AspectRatio`, which is right wherever a
/// cut corner or a vanished shape is the point, and wrong wherever the readout is two numbers
/// that should track the box's own axes independently -- see [`child_box`].
pub(crate) fn child(
    canopy: &mut Canopy,
    parent: Leaf,
    at: Location,
    tone: Color,
    label: impl Into<String>,
) -> Leaf {
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
    name(canopy, child, label);
    child
}

/// The same named shape as a plain box, for boards that read its size back as a number.
///
/// A squared child cannot show a declaration resolving on both axes independently -- see
/// [`child`]'s doc. A `Panel` resolves each axis on its own, so the size actually declared is
/// the size reported.
pub(crate) fn child_box(
    canopy: &mut Canopy,
    parent: Leaf,
    at: Location,
    tone: Color,
    label: impl Into<String>,
) -> Leaf {
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
    name(canopy, child, label);
    child
}

/// One row of a reference table: the call, and what it does.
pub(crate) struct Entry {
    pub(crate) call: &'static str,
    pub(crate) gloss: &'static str,
}

/// Lines a gloss wraps to, per breakpoint -- (xs, sm, md, lg). The same sentence in a narrower
/// card takes more of them, and the box has to be told, because the rows sit at computed offsets:
/// a gloss that grew its own box would run under the next entry's rule.
///
/// Stated in lines rather than pixels so it tracks the type scale. Written as `3 * 16` against a
/// 12px label, this was a pixel count that quietly stopped fitting the moment the label grew.
///
/// `md` and `lg` do not keep stepping down, because that is exactly where the rail arrives:
/// `shell::capped` starts the wide measure at `RAIL_W + space::XL`, so those breakpoints hand
/// the column a bigger window and take ~188px of it straight back. The reading width barely
/// grows across that step even though the viewport does, and a gloss sized for the window
/// rather than the column wraps into the entry below it.
const GLOSS_LINES: (i32, i32, i32, i32) = (3, 2, 2, 2);

/// An entry is its call -- always one line -- above its gloss.
const fn entry_lines(gloss: i32) -> i32 {
    gloss + 1
}

const ENTRY_GAP: i32 = space::MD;

/// Everything in the card that is not a line of text: the padding above the title and below the
/// last entry, plus one gap per entry.
const fn table_extra(count: i32) -> i32 {
    space::MD + count * ENTRY_GAP + space::MD
}

/// Lines the card is tall: the title, then every entry.
const fn table_lines(count: i32, gloss: i32) -> i32 {
    1 + count * entry_lines(gloss)
}

pub(crate) fn reference_letters(count: usize) -> (i32, i32, i32, i32) {
    let c = count as i32;
    (
        table_lines(c, GLOSS_LINES.0),
        table_lines(c, GLOSS_LINES.1),
        table_lines(c, GLOSS_LINES.2),
        table_lines(c, GLOSS_LINES.3),
    )
}

pub(crate) fn reference_extra(count: usize) -> i32 {
    table_extra(count as i32)
}

/// The detail a field has no room to act out. A rule above every entry but the first, so the
/// rows read as separate things rather than one paragraph.
pub(crate) fn reference(g: &mut Grow, region: Leaf, entries: &[Entry], seq: Leaf, start: u64) {
    let card = g.canopy.branch(
        region,
        Panel::new()
            .color(role::surface_container())
            .rounding(Rounding::Xs)
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.px().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            .opacity(0.0),
    );
    fade_in(g.canopy, card, seq, start);

    g.canopy.branch(
        card,
        Text::new(crate::site::copy::board::REFERENCE)
            .size(FontSize::new(type_scale::LABEL))
            .color(role::on_surface_heading())
            .at(Location::new().xs(
                space::MD
                    .px()
                    .as_left()
                    .with(100.pct().as_right().adjust(-space::MD)),
                space::MD.px().as_top().with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(2))
            .align(HorizontalAlignment::Left, VerticalAlignment::Middle),
    );

    // Each row's offset is part characters and part pixels: the text above it stacks in lines,
    // the gaps between entries do not. `.letters()` carries the first, `.adjust` the second.
    let measure = || {
        space::MD
            .px()
            .as_left()
            .with(100.pct().as_right().adjust(-space::MD))
    };
    let row = |i: i32, gloss: i32, line: i32, px: i32, height: LocationValue| {
        (1 + i * entry_lines(gloss) + line)
            .letters()
            .as_top()
            .adjust(space::MD + i * ENTRY_GAP + px)
            .with(height.as_height())
    };
    // Mid-gap above its entry, so the rule reads as separating two rows rather than belonging
    // to the one under it.
    let rule = |i: i32, gloss: i32| row(i, gloss, 0, ENTRY_GAP / 2, 1.px());
    let call = |i: i32, gloss: i32| row(i, gloss, 0, ENTRY_GAP, 1.letters());
    let gloss_at = |i: i32, gloss: i32| row(i, gloss, 1, ENTRY_GAP, (gloss).letters());

    for (i, entry) in entries.iter().enumerate() {
        let i = i as i32;
        if i > 0 {
            g.canopy.branch(
                card,
                Panel::new()
                    .color(role::outline())
                    .rounding(Rounding::None)
                    .at(Location::new()
                        .xs(measure(), rule(i, GLOSS_LINES.0))
                        .sm(measure(), rule(i, GLOSS_LINES.1))
                        .md(measure(), rule(i, GLOSS_LINES.2))
                        .lg(measure(), rule(i, GLOSS_LINES.3)))
                    .elevate(Elevation::up(2))
                    // Its own offset is stated in characters, so it needs a cell to measure
                    // against even though it draws none of them.
                    .size(FontSize::new(type_scale::LABEL)),
            );
        }
        // The call is code and the gloss is prose about it. On one tone a step apart they read as
        // a single wrapped paragraph, so the call takes the accent the demos draw their children
        // in and the gloss stays chrome.
        g.canopy.branch(
            card,
            Text::new(entry.call)
                .size(FontSize::new(type_scale::LABEL))
                .color(role::accent())
                .at(Location::new()
                    .xs(measure(), call(i, GLOSS_LINES.0))
                    .sm(measure(), call(i, GLOSS_LINES.1))
                    .md(measure(), call(i, GLOSS_LINES.2))
                    .lg(measure(), call(i, GLOSS_LINES.3)))
                .elevate(Elevation::up(2))
                .align(HorizontalAlignment::Left, VerticalAlignment::Middle),
        );
        g.canopy.branch(
            card,
            Text::new(entry.gloss)
                .size(FontSize::new(type_scale::LABEL))
                .color(role::on_surface_variant())
                .at(Location::new()
                    .xs(measure(), gloss_at(i, GLOSS_LINES.0))
                    .sm(measure(), gloss_at(i, GLOSS_LINES.1))
                    .md(measure(), gloss_at(i, GLOSS_LINES.2))
                    .lg(measure(), gloss_at(i, GLOSS_LINES.3)))
                .elevate(Elevation::up(2))
                .align(HorizontalAlignment::Left, VerticalAlignment::Top),
        );
    }
}
