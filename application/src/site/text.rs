//! The `text` section.
//!
//! The renderers page already took the glyph itself -- one character, sampled out of an atlas
//! by codepoint. This is the system around it: the size that changes with the window, boxes
//! measured in characters rather than pixels, a box taking its size from its own string, colour
//! per character, and the one face decision that is made before the app runs.
//!
//! Two of the five boards read their own subject back off the tree rather than restating it: a
//! character-sized box and a content-sized box are both *resolutions*, and a board that printed
//! the number it had just written would be showing its own arithmetic.
//!
//! The size board is the one that stands in for something. `FontSize` resolves against the
//! viewport, and no button can move the viewport -- so the board resizes a frame on its own
//! stage and states which breakpoint that width would be. The declaration and the sizes it
//! yields are the real ones; only the window is played by an understudy.

use foliage::{
    Forest, Color, Elevation, FontSize, GlyphColors, Grid, GridExt, Grows, HorizontalAlignment,
    Leaf, Location, Panel, Rounding, Sprout, Text, VerticalAlignment, anchor, text_content,
};

use crate::site::blueprint::{self, Blueprint, Frame};
use crate::site::copy::{board, headings, reference, text as copy};
use crate::site::{Column, Demo, Grow, SCROLL_TAIL, role, space, type_scale};

const STAGE_H: (i32, i32, i32) = (150, 165, 190);

pub(crate) fn build(g: &mut Grow, slot: Leaf) {
    let container = crate::site::shell::content_area(g.forest, slot);
    let mut column = Column::new(g.forest, container);

    column.display(g.forest, headings::TEXT);
    column.lead(g.forest, copy::LEAD);

    column.heading(g.forest, headings::TEXT_SIZE);
    column.prose(g.forest, copy::SIZE);
    size(g, &mut column);

    column.heading(g.forest, headings::TEXT_LETTERS);
    column.prose(g.forest, copy::LETTERS);
    letters(g, &mut column);

    column.heading(g.forest, headings::TEXT_CONTENT);
    column.prose(g.forest, copy::CONTENT);
    content(g, &mut column);

    column.heading(g.forest, headings::TEXT_COLOR);
    column.prose(g.forest, copy::COLOR);
    color(g, &mut column);

    column.heading(g.forest, headings::TEXT_FONT);
    column.prose(g.forest, copy::FONT);
    font(g, &mut column);

    column.tail(g.forest, SCROLL_TAIL);
}

// ---- size per step ---------------------------------------------------------------------------

/// What [`board::SIZE_DECLARATION`] resolves to at each of the three steps, and the width the
/// frame takes to stand in for that breakpoint's window.
///
/// The sizes are the declaration read by hand, and the widths are proportions of the stage
/// rather than the thresholds themselves -- 840 logical pixels does not fit inside a board on a
/// phone, and a frame drawn at true scale would be a demonstration of the stage's width.
const SIZE_SIZES: [u32; 3] = [14, 20, 28];
const SIZE_FRAME_WIDTHS: [f32; 3] = [34.0, 62.0, 100.0];

fn size_frame_at(width: f32) -> Location {
    Location::new().xs(
        0.pct().as_left().with(width.pct().as_right()),
        0.pct().as_top().with(100.pct().as_bottom()),
    )
}

struct Size {
    board: Blueprint,
    frame: Frame,
    specimen: Leaf,
}

fn size(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::SIZE_ROWS,
        &board::SIZE_STEPS,
        &reference::SIZE,
    );
    // The declaration itself, on the stage above the frame. What is being resolved and what it
    // resolved to are the two halves of this board, and leaving the first of them to the
    // reference card underneath put them too far apart to read as a pair.
    g.forest.branch(
        board.stage,
        Text::new(board::SIZE_DECLARATION)
            .size(FontSize::new(type_scale::LABEL))
            .color(role::accent())
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.px().as_top().with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(2))
            .align(HorizontalAlignment::Left, VerticalAlignment::Middle)
            .pass_through(),
    );
    // Below the declaration, so the frame is the lower two thirds of the stage.
    let host = g.forest.branch(
        board.stage,
        foliage::Bare::new()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                (type_scale::LABEL as i32 + space::MD)
                    .px()
                    .as_top()
                    .with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            .pass_through(),
    );
    let frame = blueprint::frame(
        g.forest,
        host,
        size_frame_at(SIZE_FRAME_WIDTHS[0]),
        board::SIZE_FRAMES[0],
        false,
    );
    let specimen = g.forest.branch(
        frame.leaf,
        Text::new(board::SIZE_SPECIMEN)
            .size(FontSize::new(SIZE_SIZES[0]))
            .color(role::accent())
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                60.pct().as_center_y().with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(2))
            .align(HorizontalAlignment::Center, VerticalAlignment::Middle)
            .pass_through(),
    );
    board.set(g.forest, 0, board::SIZE_WINDOWS[0]);
    board.set(g.forest, 1, board::points(SIZE_SIZES[0]));
    g.page.demos.push(Box::new(Size {
        board,
        frame,
        specimen,
    }));
}

impl Demo for Size {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(forest, step);
        forest.location(self.frame.leaf, size_frame_at(SIZE_FRAME_WIDTHS[step]));
        forest.text(self.frame.label, board::SIZE_FRAMES[step]);
        // Written here rather than left to the engine, and that is the one thing on this board
        // being stood in for: `FontSize` resolves against the *viewport*, not against whatever
        // box a run happens to sit in, so a frame cannot put its contents at another
        // breakpoint. The number written is what the declaration above says for this step.
        forest.font_size(self.specimen, FontSize::new(SIZE_SIZES[step]));
        self.board.set(forest, 0, board::SIZE_WINDOWS[step]);
        self.board.set(forest, 1, board::points(SIZE_SIZES[step]));
        true
    }
}

// ---- the character grid ----------------------------------------------------------------------

/// One per [`board::LETTERS_STEPS`], in order.
const LETTERS_SIZES: [u32; 3] = [13, 20, 32];

struct Letters {
    board: Blueprint,
    /// Stated as a number of characters wide and given a size to measure them against. It draws
    /// no text of its own -- what it is doing is being measured.
    cell: Leaf,
    specimen: Leaf,
}

fn letters(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::LETTERS_ROWS,
        &board::LETTERS_STEPS,
        &reference::LETTERS,
    );
    let cell = g.forest.branch(
        board.stage,
        Panel::new()
            .color(role::outline())
            .outline(1)
            .rounding(Rounding::None)
            .at(Location::new().xs(
                50.pct()
                    .as_center_x()
                    .with(board::LETTERS_WIDE.letters().as_width()),
                50.pct().as_center_y().with(2.letters().as_height()),
            ))
            .elevate(Elevation::up(1))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            // the cell's own measure: `.letters()` resolves against the entity's own size, and
            // a box carrying no text has none until it is handed one
            .size(FontSize::new(LETTERS_SIZES[0]))
            .pass_through(),
    );
    let specimen = g.forest.branch(
        cell,
        Text::new(board::LETTERS_SPECIMEN)
            .size(FontSize::new(LETTERS_SIZES[0]))
            .color(role::accent())
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                50.pct().as_center_y().with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(1))
            .align(HorizontalAlignment::Left, VerticalAlignment::Middle)
            .pass_through(),
    );
    board.set(g.forest, 0, board::advance(LETTERS_SIZES[0]));
    g.page.demos.push(Box::new(Letters {
        board,
        cell,
        specimen,
    }));
}

impl Demo for Letters {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(forest, step);
        let size = FontSize::new(LETTERS_SIZES[step]);
        // Both, and for different reasons: one is what is drawn, the other is what the box is
        // measured in. Writing only the text's would leave a twelve-character box that no
        // longer holds twelve of these characters.
        forest.font_size(self.specimen, size);
        forest.font_size(self.cell, size);
        self.board
            .set(forest, 0, board::advance(LETTERS_SIZES[step]));
        true
    }
    fn drive(&mut self, forest: &mut Forest) {
        let width = forest
            .section(self.cell)
            .map(|section| board::box_width(section.width()))
            .unwrap_or_else(|| board::EMPTY_VALUE.to_string());
        self.board.set(forest, 1, width);
    }
}

// ---- content size ----------------------------------------------------------------------------

struct Content {
    board: Blueprint,
    specimen: Leaf,
}

fn content(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::CONTENT_ROWS,
        &board::CONTENT_STEPS,
        &reference::CONTENT,
    );
    let specimen = g.forest.branch(
        board.stage,
        // Grows rightward from a fixed left edge rather than being centred, because a box that
        // moved while it resized would be showing two things at once. What keeps it inside the
        // stage is the strings being short: 14 characters at HEADLINE is 185px, against 240px
        // of stage inside this inset at a 320px viewport.
        Text::new(board::CONTENT_STRINGS[0])
            .size(FontSize::new(type_scale::HEADLINE))
            .color(role::accent())
            .at(Location::new().xs(
                // The width is the declaration and the measure both: `text_content()` is what
                // tells the glyph pass to write its measured extent onto this axis, and what
                // resolves to "keep what is there" afterwards so the next resolve doesn't
                // recompute the axis from the parent and throw that measurement away. One
                // value, because they were never separable -- a box measured and then
                // re-resolved from the parent is a box that stayed at the right edge it was
                // given and took its outline off the side of the stage.
                //
                // The spelling is `TextInput`'s own, for its single-line field.
                space::MD.px().as_left().with(text_content().as_width()),
                50.pct().as_center_y().with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(2))
            .align(HorizontalAlignment::Left, VerticalAlignment::Middle)
            .pass_through(),
    );
    // Anchored rather than sized the same way, because a box with no text of its own has no
    // content to be sized by. Reading the specimen's resolved edges is how something else
    // follows a run that measures itself.
    g.forest.branch(
        board.stage,
        Panel::new()
            .color(role::outline())
            .outline(1)
            .rounding(Rounding::None)
            .at(Location::new().xs(
                anchor()
                    .left()
                    .as_left()
                    .adjust(-space::XS)
                    .with(anchor().right().as_right().adjust(space::XS)),
                anchor()
                    .top()
                    .as_top()
                    .adjust(-space::XS)
                    .with(anchor().bottom().as_bottom().adjust(space::XS)),
            ))
            .elevate(Elevation::up(1))
            .anchored(specimen)
            .pass_through(),
    );
    board.set(
        g.forest,
        0,
        board::characters(board::CONTENT_STRINGS[0].len()),
    );
    g.page.demos.push(Box::new(Content { board, specimen }));
}

impl Demo for Content {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(forest, step);
        forest.text(self.specimen, board::CONTENT_STRINGS[step]);
        self.board.set(
            forest,
            0,
            board::characters(board::CONTENT_STRINGS[step].len()),
        );
        true
    }
    fn drive(&mut self, forest: &mut Forest) {
        let width = forest
            .section(self.specimen)
            .map(|section| board::box_width(section.width()))
            .unwrap_or_else(|| board::EMPTY_VALUE.to_string());
        self.board.set(forest, 1, width);
    }
}

// ---- per-glyph color -------------------------------------------------------------------------

/// Where the extension starts in [`board::COLOR_SPECIMEN`] -- the same index the hero's wordmark
/// and the rail's brand both colour from, since it is the same string.
const COLOR_EXTENSION_AT: usize = 7;

/// The ramp: one tone per character of [`board::COLOR_SPECIMEN`], and no repeats.
///
/// It was the site's own three poly-control tones cycled, which on a ten-character string is
/// orange, amber, rose, orange, amber, rose -- and two of those three are close enough that the
/// run read as a single colour someone had failed to apply evenly. A sweep of the palette is
/// the only version of this board that shows what it claims: that every character is addressed
/// separately.
///
/// The one place on the site that leaves its palette, deliberately. Every other page holds to
/// three warm tones so the accent keeps meaning something; here the subject *is* that a run can
/// be coloured per character, and three tones cannot demonstrate it.
const COLOR_RAMP: [fn() -> Color; 10] = [
    || Color::orange(400),
    || Color::amber(400),
    || Color::yellow(400),
    || Color::lime(400),
    || Color::emerald(400),
    || Color::teal(400),
    || Color::cyan(400),
    || Color::sky(400),
    || Color::violet(400),
    || Color::rose(400),
];

/// One tone per character, and the ramp is written out rather than generated -- so the pairing
/// is worth holding to at build time instead of silently leaving the tail of a longer specimen
/// uncoloured.
const _: () = assert!(COLOR_RAMP.len() == board::COLOR_SPECIMEN.len());

fn color_run(step: usize) -> GlyphColors {
    let mut colors = GlyphColors::new();
    match step {
        0 => {}
        1 => {
            colors = colors.add(
                COLOR_EXTENSION_AT..board::COLOR_SPECIMEN.len(),
                role::accent(),
            );
        }
        _ => {
            for (i, tone) in COLOR_RAMP.iter().enumerate() {
                colors = colors.add(i..i + 1, tone());
            }
        }
    }
    colors
}

struct Colored {
    board: Blueprint,
    specimen: Leaf,
}

fn color(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::COLOR_ROWS,
        &board::COLOR_STEPS,
        &reference::COLOR,
    );
    let specimen = g.forest.branch(
        board.stage,
        Text::new(board::COLOR_SPECIMEN)
            .size(FontSize::new(type_scale::DISPLAY))
            .color(role::on_surface())
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                50.pct().as_center_y().with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(2))
            .align(HorizontalAlignment::Center, VerticalAlignment::Middle)
            .pass_through(),
    );
    board.set(g.forest, 0, board::COLOR_RUNS[0]);
    board.set(g.forest, 1, board::COLOR_OVERRIDES[0]);
    g.page.demos.push(Box::new(Colored { board, specimen }));
}

impl Demo for Colored {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(forest, step);
        forest.glyph_colors(self.specimen, color_run(step));
        self.board.set(forest, 0, board::COLOR_RUNS[step]);
        self.board.set(forest, 1, board::COLOR_OVERRIDES[step]);
        true
    }
}

// ---- registered fonts ------------------------------------------------------------------------

struct Faces {
    board: Blueprint,
    /// One per [`board::FONT_STEPS`]. A face is named as an element is grown, so these are two
    /// elements rather than one that is rewritten -- the same shape the input page's two
    /// propagation modes take, for the same reason.
    specimens: [Leaf; 2],
}

fn font(g: &mut Grow, column: &mut Column) {
    let mut board = blueprint::board(
        g,
        column,
        STAGE_H,
        board::FONT_ROWS,
        &board::FONT_STEPS,
        &reference::FONT,
    );
    let faces = [foliage::FontId::default(), crate::site::italic()];
    let specimens = [0usize, 1].map(|i| {
        g.forest.branch(
            board.stage,
            Text::new(board::FONT_SPECIMEN)
                .size(FontSize::new(type_scale::HEADLINE))
                .color(role::accent())
                .at(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    50.pct().as_center_y().with(1.letters().as_height()),
                ))
                .elevate(Elevation::up(2))
                .align(HorizontalAlignment::Center, VerticalAlignment::Middle)
                .font(faces[i])
                .pass_through(),
        )
    });
    g.forest.visible(specimens[1], false);
    board.set(g.forest, 0, board::FONT_IDS[0]);
    board.set(g.forest, 1, board::FONT_FACES[0]);
    g.page.demos.push(Box::new(Faces { board, specimens }));
}

impl Demo for Faces {
    fn clicked(&mut self, forest: &mut Forest, leaf: Leaf) -> bool {
        let Some(step) = self.board.pressed(leaf) else {
            return false;
        };
        self.board.select(forest, step);
        // Nothing to disable alongside the hide, unlike the input page's swaps: neither of
        // these takes input in the first place.
        for (i, specimen) in self.specimens.iter().enumerate() {
            forest.visible(*specimen, i == step);
        }
        self.board.set(forest, 0, board::FONT_IDS[step]);
        self.board.set(forest, 1, board::FONT_FACES[step]);
        true
    }
}
