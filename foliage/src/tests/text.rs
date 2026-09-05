//! Text: the character cell, wrapping, and the two questions `content()` asks.
//!
//! The first half is pure -- a string and a cell in, a count of lines out -- with the cell fed in
//! rather than read off a font, which is what lets one case stand for a font and the next for a
//! size. The second half runs the whole frame, and is where the cell comes from a real font and the
//! measure reaches a real box.

use crate::coordinate::{Area, Position, Section};
use crate::layout::Layout;
use crate::tests::{grove, resize, section, tick};
use crate::text::font::{Fonts, monospaced};
use crate::text::shape::shape;
use crate::{
    Boxed, Columns, Divide, Font, FontSize, Grid, Grove, Grow, Leaf, Location, Palette, Place, Sap,
    Source, Stem, Text, Vein, anchor, bottom, content, left, top, trunk,
};

/// A cell to stand for one font at one size. Two different ones are two different fonts, which is
/// exactly what the arithmetic under test cannot tell apart.
fn cell(width: f32, height: f32) -> Area {
    Area::new(width, height)
}

/// How many lines `value` takes in a box `columns` cells wide.
fn lines(value: &str, columns: usize) -> usize {
    shape(value, cell(10.0, 20.0)).lines(columns)
}

// Max-content: the widest the run would like to be, unwrapped.

/// A character count times a cell, across fonts and across sizes -- which is the whole of the claim
/// that the down-pass needs no measure pass.
#[test]
fn max_content_is_the_longest_line_times_the_cell() {
    assert_eq!(shape("hello", cell(10.0, 20.0)).max_content(), 50.0);
    // The same run in a narrower font.
    assert_eq!(shape("hello", cell(8.0, 16.0)).max_content(), 40.0);
    // And at a larger size.
    assert_eq!(shape("hello", cell(15.0, 32.0)).max_content(), 75.0);
}

/// Hard lines are what max-content is the widest of, not the whole string.
#[test]
fn max_content_takes_the_widest_hard_line() {
    assert_eq!(
        shape("hi\nthere\nyou", cell(10.0, 20.0)).max_content(),
        50.0
    );
}

/// An element with nothing in it measures to zero rather than to one line of nothing.
#[test]
fn an_empty_run_measures_to_nothing() {
    let shaped = shape("", cell(10.0, 20.0));
    assert_eq!(shaped.max_content(), 0.0);
    assert_eq!(shaped.lines(8), 0);
    assert_eq!(shaped.measure(80.0), 0.0);
}

// Wrapping: exact line counts, at the boundary and past it.

#[test]
fn a_run_that_fits_takes_one_line() {
    assert_eq!(lines("hello world", 20), 1);
}

/// Exactly as many cells as there are characters is one line, and one character more is two. The
/// boundary is where a greedy wrap is most likely to be off by one, so it is stated both ways.
#[test]
fn a_run_wraps_at_the_boundary_and_not_before() {
    assert_eq!(lines("hello world", 11), 1);
    assert_eq!(lines("hello world", 10), 2);
    assert_eq!(lines("hello", 5), 1);
    assert_eq!(lines("hello", 4), 2);
}

/// The break takes the spaces with it. A word starts the new line, and the gap it was separated by
/// does not trail off the end of the old one or indent the new one.
#[test]
fn the_spaces_at_a_break_go_with_it() {
    assert_eq!(lines("aaaa bbbb", 4), 2);
    assert_eq!(lines("aaaa   bbbb", 4), 2);
}

/// Trailing spaces sit on the line they are on and never make one of their own.
#[test]
fn trailing_spaces_make_no_line() {
    assert_eq!(lines("hello   ", 5), 1);
    assert_eq!(lines("hello", 5), 1);
}

/// A word wider than the whole box has no width at which it fits, so it fills what is left and
/// breaks inside itself.
#[test]
fn a_word_wider_than_the_box_breaks_inside_itself() {
    assert_eq!(lines("abcdefghij", 4), 3);
    // It fills what is left of the line it is on rather than starting one of its own: "ab c",
    // "defg", "hij".
    assert_eq!(lines("ab cdefghij", 4), 3);
}

#[test]
fn a_newline_ends_a_line() {
    assert_eq!(lines("a\nb\nc", 40), 3);
    assert_eq!(lines("a\n", 40), 2);
    // And a hard line wraps like any other.
    assert_eq!(lines("aaaa bbbb\ncc", 4), 3);
}

/// Height is a count of lines and a font metric, and nothing else.
#[test]
fn height_is_lines_times_the_cell() {
    let shaped = shape("hello world", cell(10.0, 20.0));
    assert_eq!(shaped.measure(110.0), 20.0);
    assert_eq!(shaped.measure(100.0), 40.0);
    // Five cells: "hello" then "world", and the space between them goes with the break.
    assert_eq!(shaped.measure(50.0), 40.0);
}

/// A line is a whole number of cells: half a cell of room at the end of one is not somewhere a
/// character goes.
#[test]
fn a_line_holds_whole_cells_only() {
    let shaped = shape("abcd", cell(10.0, 20.0));
    assert_eq!(shaped.measure(19.0), shaped.measure(10.0));
    // No room at all still lays the run out one cell at a time rather than dividing by zero.
    assert_eq!(shaped.measure(0.0), 80.0);
}

// The font itself.

/// Advances that agree are monospaced; one that does not names both ends, because the message is the
/// whole of what makes the refusal actionable.
#[test]
fn a_font_whose_advances_disagree_is_refused() {
    assert_eq!(monospaced([('i', 9.6), ('W', 9.6), ('m', 9.601)]), Ok(()));
    assert_eq!(
        monospaced([('i', 4.0), ('W', 12.0)]),
        Err(('i', 4.0, 'W', 12.0))
    );
}

/// The bundled font is the one every element takes without saying anything, and a cell of it is the
/// pitch everything else is measured in.
#[test]
fn the_bundled_font_is_monospaced_and_has_a_cell() {
    let fonts = Fonts::new();
    let cell = fonts.cell(Font::DEFAULT, 16);
    assert_eq!(cell, Area::new(10.0, 22.0));
    // A size is a different cell, which is what makes the shaping key a size as well as a font.
    assert_eq!(fonts.cell(Font::DEFAULT, 24), Area::new(15.0, 32.0));
}

// Through the frame.

/// A box `width` wide, as tall as its own content turns out to be.
fn measured(width: f32) -> Location {
    Location::new().xs(
        left(0.px()).width(width.px()),
        top(0.px()).height(content()),
    )
}

fn text(grove: &Grove, leaf: Leaf) -> String {
    match grove.tap(leaf, Vein::Text) {
        Some(Sap::Text(value)) => value,
        other => panic!("expected a run, got {other:?}"),
    }
}

/// No settling frame. R1 shapes and R2m wraps inside the same resolve the box comes out of, so the
/// frame that plants a run is the frame it is the right size.
#[test]
fn a_run_sized_to_its_content_is_right_on_the_frame_it_is_planted() {
    let mut grove = grove();
    // Fifty logical pixels is five cells of the default font, so "hello world" takes two lines.
    let leaf = grove.plant(Text::new("hello world").at(measured(50.0)));
    tick(&mut grove);
    assert_eq!(
        section(&grove, leaf),
        Section::from_edges(0.0, 0.0, 50.0, 44.0)
    );
}

/// One word, two questions. The axis decides which is being asked, and both are answered in the same
/// frame from the same shaped run.
#[test]
fn content_asks_a_different_question_per_axis() {
    let mut grove = grove();
    let leaf = grove.plant(
        Text::new("hello world")
            .at(Location::new().xs(left(0.px()).width(content()), top(0.px()).height(content()))),
    );
    tick(&mut grove);
    // Across: max-content, eleven cells unwrapped. Down: one line at that width.
    assert_eq!(
        section(&grove, leaf),
        Section::from_edges(0.0, 0.0, 110.0, 22.0)
    );
}

/// Fit-content: the smaller of what the content wants and what the ceiling allows -- and the wrap
/// follows the clamp, because R2m measures at the width R2a actually produced.
#[test]
fn content_under_a_ceiling_is_fit_content() {
    let mut grove = grove();
    let wide = grove.plant(Text::new("hello world").at(Location::new().xs(
        left(0.px()).width(content()).at_most(300.px()),
        top(0.px()).height(content()),
    )));
    let narrow = grove.plant(Text::new("hello world").at(Location::new().xs(
        left(0.px()).width(content()).at_most(50.px()),
        top(0.px()).height(content()),
    )));
    tick(&mut grove);
    // Under a ceiling it does not reach, it is its own width and one line.
    assert_eq!(section(&grove, wide).width(), 110.0);
    assert_eq!(section(&grove, wide).height(), 22.0);
    // Clamped, and wrapped at the clamp rather than at what it asked for.
    assert_eq!(section(&grove, narrow).width(), 50.0);
    assert_eq!(section(&grove, narrow).height(), 44.0);
}

/// A floor is the other direction, and clamps the same way.
#[test]
fn content_under_a_floor_is_held_open() {
    let mut grove = grove();
    let leaf = grove.plant(Text::new("hi").at(Location::new().xs(
        left(0.px()).width(content()).at_least(200.px()),
        top(0.px()).height(content()),
    )));
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).width(), 200.0);
}

/// Rewriting the string reflows in the frame that wrote it. Measuring is a pass rather than a
/// reaction to the write, so there is nothing to fire and nothing to be a frame late.
#[test]
fn rewriting_a_run_reflows_in_the_same_frame() {
    let mut grove = grove();
    let leaf = grove.plant(Text::new("hello").at(measured(50.0)));
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).height(), 22.0);

    grove.text(leaf, "hello world and everyone in it");
    tick(&mut grove);
    assert_eq!(text(&grove, leaf), "hello world and everyone in it");
    // Thirty characters at five cells a line.
    assert_eq!(section(&grove, leaf).height(), 132.0);
}

/// A breakpoint changes the size, and the size changes both halves of the measure: a cell is wider,
/// so fewer of them fit, so there are more lines and each is taller.
#[test]
fn crossing_a_breakpoint_re_wraps() {
    let mut grove = grove();
    let leaf = grove.plant(
        Text::new("hello world")
            .font_size(FontSize::new().xs(16).md(24))
            .at(measured(60.0)),
    );
    tick(&mut grove);
    assert_eq!(grove.layout(), Layout::Xs);
    // Six cells of ten: "hello" then "world".
    assert_eq!(section(&grove, leaf).height(), 44.0);

    resize(&mut grove, Area::new(700.0, 600.0));
    tick(&mut grove);
    assert_eq!(grove.layout(), Layout::Md);
    // Four cells of fifteen, so both words break inside themselves: "hell", "o wo", "rld" -- and
    // each line is taller as well.
    assert_eq!(section(&grove, leaf).height(), 96.0);
}

/// An anchored element reads a box that a re-measure moved, and follows it in the same frame.
#[test]
fn an_anchored_element_follows_a_run_that_changed_height() {
    let mut grove = grove();
    let run = grove.plant(Text::new("hello").at(measured(50.0)));
    let follower = grove.plant(Stem::new().anchored(run).at(Location::new().xs(
        left(0.px()).width(10.px()),
        top(anchor().bottom() + 8.px()).height(10.px()),
    )));
    tick(&mut grove);
    assert_eq!(section(&grove, follower).top(), 30.0);

    grove.text(run, "hello world");
    tick(&mut grove);
    assert_eq!(section(&grove, run).height(), 44.0);
    assert_eq!(section(&grove, follower).top(), 52.0);
}

/// Another element's measure, read the way its box is: `content()` is the reader's own, and a named
/// basis is that basis's.
#[test]
fn a_named_basis_measures_itself() {
    let mut grove = grove();
    let run = grove.plant(Text::new("hello world").at(measured(200.0)));
    let follower = grove.plant(Stem::new().anchored(run).at(Location::new().xs(
        left(0.px()).width(anchor().content()),
        top(0.px()).height(anchor().content()),
    )));
    tick(&mut grove);
    // The anchor's max-content across, and the height it wrapped to down -- neither of which is the
    // anchor's box, because it was given more room than it asked for.
    assert_eq!(
        section(&grove, follower),
        Section::from_edges(0.0, 0.0, 110.0, 22.0)
    );
}

// The character cell.

/// `letters` is the reader's own font, always, because that is the only one it composes in.
#[test]
fn letters_read_the_element_s_own_cell() {
    let mut grove = grove();
    let small = grove.plant(
        Stem::new()
            .font_size(FontSize::new().xs(16))
            .at(Location::new().xs(left(0.px()).width(8.letters()), top(0.px()).height(10.px()))),
    );
    let large = grove.plant(
        Stem::new()
            .font_size(FontSize::new().xs(24))
            .at(Location::new().xs(left(0.px()).width(8.letters()), top(0.px()).height(10.px()))),
    );
    tick(&mut grove);
    assert_eq!(section(&grove, small).width(), 80.0);
    assert_eq!(section(&grove, large).width(), 120.0);
}

/// An element with no font and no size has no cell, so a count of letters is a count of nothing.
/// That is what keeps a cell a declaration rather than a default nobody asked for.
#[test]
fn an_element_with_no_typeface_has_no_cell() {
    let mut grove = grove();
    let leaf = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(8.letters()), top(0.px()).height(10.px()))),
    );
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).width(), 0.0);
}

/// A count in another element's font is that element's cell, which is a different number.
#[test]
fn a_named_basis_counts_its_own_letters() {
    let mut grove = grove();
    let holder = grove.plant(
        Stem::new()
            .font_size(FontSize::new().xs(24))
            .at(Location::new().xs(left(0.px()).width(300.px()), top(0.px()).height(100.px()))),
    );
    let branch = grove.branch(
        holder,
        Stem::new()
            .font_size(FontSize::new().xs(16))
            .at(Location::new().xs(
                left(0.px()).width(4.letters()),
                top(0.px()).height(4.letters()),
            )),
    );
    let other = grove.branch(
        holder,
        Stem::new()
            .font_size(FontSize::new().xs(16))
            .at(Location::new().xs(
                left(0.px()).width(trunk().letters(4.0)),
                top(0.px()).height(10.px()),
            )),
    );
    tick(&mut grove);
    // Its own sixteen-pixel cell across, and its own line height down.
    assert_eq!(section(&grove, branch).width(), 40.0);
    assert_eq!(section(&grove, branch).height(), 88.0);
    // The trunk's, which is the larger one.
    assert_eq!(section(&grove, other).width(), 60.0);
}

/// A letter-pitched track takes its pitch from the element the grid is *on*, not from the child
/// addressing it -- which is what makes such a grid addressable by children of any size.
#[test]
fn a_letter_pitched_track_is_in_the_font_of_the_grid_s_own_element() {
    let mut grove = grove();
    let trunk = grove.plant(
        Stem::new()
            .font_size(FontSize::new().xs(24))
            .grid(Grid::new().xs(Columns::letters(4.0), 1.rows()))
            .at(Location::new().xs(left(0.px()).width(400.px()), top(0.px()).height(100.px()))),
    );
    let branch = grove.branch(
        trunk,
        Stem::new()
            .font_size(FontSize::new().xs(16))
            .at(Location::new().xs(left(1.col()).right(1.col()), top(0.px()).height(10.px()))),
    );
    tick(&mut grove);
    // Four cells of the trunk's twenty-four-pixel font, not of the child's sixteen.
    assert_eq!(section(&grove, branch).width(), 60.0);
}

// A container measured from what is in it.

/// `content()` on something with elements grown under it is the same question it is on a run: how
/// large is what is inside me. The answer is how far the furthest of them reaches.
#[test]
fn a_container_grows_to_fit_what_is_grown_under_it() {
    let mut grove = grove();
    let container = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(200.px()), top(0.px()).height(content()))),
    );
    grove.branch(
        container,
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(10.px()), top(0.px()).height(20.px()))),
    );
    grove.branch(
        container,
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(10.px()), top(40.px()).height(30.px()))),
    );
    tick(&mut grove);
    assert_eq!(section(&grove, container).height(), 70.0);
}

/// The children may be measured themselves, which is where wrapping and container sizing meet: the
/// sweep is bottom-up, so a run inside a stack is already measured when the stack asks.
#[test]
fn a_container_fits_children_of_differing_wrapped_heights() {
    let mut grove = grove();
    let container =
        grove
            .plant(Stem::new().at(
                Location::new().xs(left(0.px()).width(50.px()), top(0.px()).height(content())),
            ));
    // One line.
    grove.branch(
        container,
        Text::new("hello")
            .at(Location::new().xs(left(0.px()).width(100.pct()), top(0.px()).height(content()))),
    );
    // Three lines, starting below the first.
    grove.branch(
        container,
        Text::new("one two three").at(Location::new().xs(
            left(0.px()).width(100.pct()),
            top(22.px()).height(content()),
        )),
    );
    tick(&mut grove);
    // Twenty-two for the first, then three lines of twenty-two under it.
    assert_eq!(section(&grove, container).height(), 88.0);
}

/// A child sized to its trunk cannot also be what sizes it. It contributes nothing to the measure
/// and is given its real height by the vertical pass like anything else -- which is the whole of why
/// this needs no second pass and nothing has to converge.
#[test]
fn a_child_sized_to_a_measured_trunk_does_not_size_it() {
    let mut grove = grove();
    let container = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(200.px()), top(0.px()).height(content()))),
    );
    let filling = grove.branch(
        container,
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(10.px()), top(0.px()).bottom(100.pct()))),
    );
    grove.branch(
        container,
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(10.px()), top(0.px()).height(60.px()))),
    );
    tick(&mut grove);
    assert_eq!(section(&grove, container).height(), 60.0);
    assert_eq!(section(&grove, filling).height(), 60.0);
}

/// Nothing in it is nothing to measure, on either axis.
#[test]
fn an_empty_container_measures_to_nothing() {
    let mut grove = grove();
    let container = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(200.px()), top(0.px()).height(content()))),
    );
    tick(&mut grove);
    assert_eq!(section(&grove, container).height(), 0.0);
}

// The one exception.

/// Shaping is memoized on the run, its font and its size, and swept back to what the tree states.
/// Everything else recomputes totally, so this is the one thing there is a holding to assert on.
#[test]
fn shaping_is_kept_and_swept() {
    let mut grove = grove();
    let one = grove.plant(Text::new("hello").at(measured(200.0)));
    let two = grove.plant(Text::new("hello").at(measured(200.0)));
    let three = grove.plant(
        Text::new("hello")
            .font_size(FontSize::new().xs(24))
            .at(measured(200.0)),
    );
    tick(&mut grove);
    // Two runs saying the same thing at the same size are one entry; the same words at another size
    // are another, because a shaped run is the run *at that size*.
    assert_eq!(grove.shaping.held(), 2);

    grove.text(two, "goodbye");
    tick(&mut grove);
    assert_eq!(grove.shaping.held(), 3);

    // What nothing states any more is gone, so this is the size of the tree and not of the session.
    grove.prune(three);
    grove.text(two, "hello");
    tick(&mut grove);
    assert_eq!(grove.shaping.held(), 1);

    let _ = one;
}

/// Two ticks with nothing between them produce the same boxes. Measuring is part of the recompute,
/// so it has to be as idempotent as the arithmetic around it.
#[test]
fn measuring_is_idempotent() {
    let mut grove = grove();
    let leaf = grove.plant(Text::new("hello world and everyone").at(measured(50.0)));
    tick(&mut grove);
    let first = section(&grove, leaf);
    tick(&mut grove);
    assert_eq!(section(&grove, leaf), first);
}

// A run is filled like anything else.

/// A fill is a `Fill` whatever holds it, so the same declaration, the same write and the same read
/// answer for a run and for a panel.
#[test]
fn a_run_is_filled_the_way_a_panel_is() {
    let mut grove = grove();
    let leaf = grove.plant(
        Text::new("hello")
            .color(Palette::Accent)
            .at(measured(200.0)),
    );
    tick(&mut grove);
    assert_eq!(
        grove.tap(leaf, Vein::Color),
        Some(Sap::Color(Palette::Accent.into()))
    );

    grove.color(leaf, Palette::Muted);
    tick(&mut grove);
    assert_eq!(
        grove.tap(leaf, Vein::Color),
        Some(Sap::Color(Palette::Muted.into()))
    );
}

/// A run reads against a surface rather than being drawn as one, so that is what it takes when it
/// says nothing.
#[test]
fn a_run_is_ink_by_default() {
    let mut grove = grove();
    let leaf = grove.plant(Text::new("hello").at(measured(200.0)));
    tick(&mut grove);
    assert_eq!(
        grove.tap(leaf, Vein::Color),
        Some(Sap::Color(Palette::Ink.into()))
    );
}

/// A run has no box of its own to round, so the op naming one is dropped like any other that named
/// something it does not apply to.
#[test]
fn a_run_has_no_corners_to_round() {
    let mut grove = grove();
    let leaf = grove.plant(Text::new("hello").at(measured(200.0)));
    tick(&mut grove);
    assert_eq!(grove.tap(leaf, Vein::Rounding), None);
}

/// Rewriting something that says nothing is dropped, on the same terms.
#[test]
fn rewriting_something_that_says_nothing_is_dropped() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    grove.text(leaf, "hello");
    tick(&mut grove);
    assert_eq!(grove.tap(leaf, Vein::Text), None);
}

/// Every live element is in the resolve order, including one anchored to something grown under it.
/// A trunk and an anchor can wait on each other without either chain of anchors closing, and the
/// answer is a box each rather than two elements quietly absent from the frame.
#[test]
fn an_element_anchored_into_its_own_subtree_still_resolves() {
    let mut grove = grove();
    let container =
        grove
            .plant(Stem::new().at(
                Location::new().xs(left(0.px()).width(200.px()), top(0.px()).height(100.px())),
            ));
    let inside = grove.branch(
        container,
        Stem::new()
            .at(Location::new().xs(left(10.px()).width(20.px()), top(10.px()).height(20.px()))),
    );
    tick(&mut grove);
    grove.anchor(container, inside);
    tick(&mut grove);
    assert_eq!(section(&grove, container).width(), 200.0);
    assert_eq!(section(&grove, inside).width(), 20.0);
    assert_eq!(
        section(&grove, container),
        Section::from_edges(0.0, 0.0, 200.0, 100.0)
    );
}

/// The bottom edge of a measured box is where the measure put it, so a run can be pinned by its
/// bottom and grow upward.
#[test]
fn a_measured_box_may_be_pinned_by_either_edge() {
    let mut grove = grove();
    let leaf = grove.plant(Text::new("hello world").at(Location::new().xs(
        left(0.px()).width(50.px()),
        bottom(100.px()).height(content()),
    )));
    tick(&mut grove);
    assert_eq!(
        section(&grove, leaf),
        Section::from_edges(0.0, 56.0, 50.0, 100.0)
    );
}

/// A child positioned against an anchor is asking where something else ended up, and no vertical box
/// has been laid out when the measure is taken. So it does not decide the measure -- and it is
/// placed correctly by the vertical pass regardless, which is where an anchor is answerable.
#[test]
fn an_anchored_child_does_not_size_its_trunk() {
    let mut grove = grove();
    let above = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(10.px()), top(0.px()).height(200.px()))),
    );
    let container = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(200.px()), top(0.px()).height(content()))),
    );
    let hanging = grove.branch(
        container,
        Stem::new().anchored(above).at(Location::new().xs(
            left(0.px()).width(10.px()),
            top(anchor().bottom()).height(30.px()),
        )),
    );
    grove.branch(
        container,
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(10.px()), top(0.px()).height(40.px()))),
    );
    tick(&mut grove);
    // Only the child that describes its own extent decided the measure.
    assert_eq!(section(&grove, container).height(), 40.0);
    // And the anchored one still lands where its anchor put it.
    assert_eq!(section(&grove, hanging).top(), 200.0);
}

/// A height stated in a horizontal reading counts, because the horizontal axis resolved first. That
/// is the same asymmetry the grammar already states as types, showing up in the measure.
#[test]
fn a_child_sized_from_the_horizontal_axis_counts() {
    let mut grove = grove();
    let container = grove.plant(
        Stem::new()
            .at(Location::new().xs(left(0.px()).width(200.px()), top(0.px()).height(content()))),
    );
    grove.branch(
        container,
        Stem::new().at(Location::new().xs(
            left(0.px()).width(60.px()),
            // As tall as it is wide, which the down-pass already answered.
            top(0.px()).height(trunk().width() * 0.25),
        )),
    );
    tick(&mut grove);
    assert_eq!(section(&grove, container).height(), 50.0);
}

// -- Where the glyphs land -----------------------------------------------------------------------

/// Where each of `value`'s characters lands in a box `columns` cells wide, in cells rather than in
/// pixels, so a case says which cell it means rather than restating the cell size.
fn placed(value: &str, columns: usize) -> Vec<(char, usize, usize)> {
    let cell = cell(10.0, 20.0);
    let mut placed = Vec::new();
    shape(value, cell).place(columns as f32 * cell.width, |character, _, at| {
        placed.push((
            character,
            (at.x / cell.width) as usize,
            (at.y / cell.height) as usize,
        ));
    });
    placed
}

/// Every character that leaves ink is placed, at its own cell, and nothing else is. A space is an
/// advance and a newline is a break; handing either to a renderer would leave it deciding what is
/// worth drawing.
#[test]
fn every_inked_character_is_placed_at_its_cell_and_no_other_is() {
    assert_eq!(
        placed("hi there", 20),
        vec![
            ('h', 0, 0),
            ('i', 1, 0),
            ('t', 3, 0),
            ('h', 4, 0),
            ('e', 5, 0),
            ('r', 6, 0),
            ('e', 7, 0),
        ]
    );
}

/// A word carried to the next line starts at that line's first cell, and the spaces it broke on go
/// with the break rather than indenting it.
#[test]
fn a_wrapped_word_starts_the_next_line_at_its_first_cell() {
    assert_eq!(
        placed("aaaa   bbbb", 4),
        vec![
            ('a', 0, 0),
            ('a', 1, 0),
            ('a', 2, 0),
            ('a', 3, 0),
            ('b', 0, 1),
            ('b', 1, 1),
            ('b', 2, 1),
            ('b', 3, 1),
        ]
    );
}

/// A word too long for any line fills what is left and breaks inside itself, and the break is in the
/// same place the measure counted it.
#[test]
fn a_word_longer_than_a_line_breaks_inside_itself() {
    assert_eq!(
        placed("ab cdefghij", 4),
        vec![
            ('a', 0, 0),
            ('b', 1, 0),
            ('c', 3, 0),
            ('d', 0, 1),
            ('e', 1, 1),
            ('f', 2, 1),
            ('g', 3, 1),
            ('h', 0, 2),
            ('i', 1, 2),
            ('j', 2, 2),
        ]
    );
}

/// The one walk. A glyph placed on a line the measure never counted is a run drawn taller than the
/// box it was measured into, which is the bug having two walks would produce and could not be seen
/// in either of them alone.
#[test]
fn no_glyph_lands_past_the_height_the_run_measured() {
    for (value, columns) in [
        ("hello world", 10),
        ("hello world", 11),
        ("aaaa   bbbb", 4),
        ("ab cdefghij", 4),
        ("abcdefghij", 4),
        ("a\nb\nc", 40),
        ("aaaa bbbb\ncc", 4),
        ("hello   ", 5),
        ("a\n", 40),
        ("", 8),
    ] {
        let counted = lines(value, columns);
        let reached = placed(value, columns)
            .iter()
            .map(|(_, _, line)| line + 1)
            .max()
            .unwrap_or(0);
        assert!(
            reached <= counted,
            "{value:?} at {columns}: drawn on {reached} lines, measured at {counted}"
        );
    }
}

// -- What extraction hands the backend -----------------------------------------------------------

/// A run's glyphs, as the backend is handed them: the cell each one occupies, in logical pixels on
/// the surface.
fn extracted(grove: &Grove, leaf: Leaf) -> Vec<(char, Section)> {
    grove
        .elm
        .texts
        .run(leaf.into())
        .expect("a run the backend is holding")
        .glyphs
        .iter()
        .map(|glyph| (glyph.character, glyph.cell))
        .collect()
}

/// One entry per character that leaves ink, each at the cell wrapping put it in, offset by the run's
/// own box. The default font's cell is ten by twenty-two at the default size.
#[test]
fn a_run_extracts_one_glyph_per_inked_character() {
    let mut grove = grove();
    let leaf = grove.plant(Text::new("hi there").at(measured(200.0)));
    tick(&mut grove);

    let glyphs = extracted(&grove, leaf);
    assert_eq!(glyphs.len(), 7, "the space is an advance and not a glyph");
    assert_eq!(
        glyphs[0],
        (
            'h',
            Section::new(Position::default(), Area::new(10.0, 22.0))
        )
    );
    // Three cells along, because the space between the words advanced one.
    assert_eq!(glyphs[2].0, 't');
    assert_eq!(glyphs[2].1.position.x, 30.0);
}

/// A wrapped run puts its second line one cell-height down, and every glyph of it is inside the box
/// the run measured to.
#[test]
fn a_wrapped_run_extracts_its_second_line_below_the_first() {
    let mut grove = grove();
    let leaf = grove.plant(Text::new("hello world").at(measured(50.0)));
    tick(&mut grove);

    let glyphs = extracted(&grove, leaf);
    assert_eq!(glyphs.len(), 10);
    assert_eq!(glyphs[0].1.position.y, 0.0);
    assert_eq!(glyphs[5].0, 'w');
    assert_eq!(glyphs[5].1.position, Position::new(0.0, 22.0));
    let box_of = section(&grove, leaf);
    for (character, cell) in glyphs {
        assert!(
            cell.bottom() <= box_of.bottom() && cell.right() <= box_of.right(),
            "{character:?} at {cell:?} is outside {box_of:?}"
        );
    }
}

/// The whole run is the unit of change. A frame that moved nothing writes nothing, and one that
/// rewrote the string writes the run once however many of its glyphs differ.
#[test]
fn a_run_is_written_whole_or_not_at_all() {
    let mut grove = grove();
    let leaf = grove.plant(Text::new("hello").at(measured(200.0)));
    tick(&mut grove);
    assert_eq!(grove.elm.texts.written, vec![leaf.into()]);

    tick(&mut grove);
    assert!(
        grove.elm.texts.written.is_empty(),
        "an unchanged frame wrote"
    );

    grove.text(leaf, "goodbye");
    tick(&mut grove);
    assert_eq!(grove.elm.texts.written, vec![leaf.into()]);
    assert_eq!(extracted(&grove, leaf).len(), 7);
}

/// A run that is no longer painted is withdrawn, and holding nothing for it is what makes coming
/// back cost nothing to undo.
#[test]
fn a_run_that_stops_being_painted_is_withdrawn() {
    let mut grove = grove();
    let leaf = grove.plant(Text::new("hello").at(measured(200.0)));
    tick(&mut grove);

    grove.visible(leaf, false);
    tick(&mut grove);
    assert_eq!(grove.elm.texts.withdrawn, vec![leaf.into()]);
    assert!(grove.elm.texts.run(leaf.into()).is_none());

    grove.visible(leaf, true);
    tick(&mut grove);
    assert_eq!(extracted(&grove, leaf).len(), 5);
}
