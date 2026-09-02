//! The placement algebra, against the resolver alone.
//!
//! A struct in and a struct out, with no ECS, no engine and no frame anywhere near it. That is what
//! lets these be exhaustive, and it is only possible because the resolver is a pure function.

use crate::coordinate::{Area, Axis, Section};
use crate::layout::{Layout, Short};
use crate::placement::grid::{Grid, Tracks};
use crate::placement::resolve::{Basis, Context, Span, resolve};
use crate::placement::role::{Horizontal, Vertical};
use crate::{
    Columns, Divide, Rows, Source, anchor, bottom, center_x, center_y, content, left, right, top,
    trunk,
};

/// Everything a placement is read against, with a trunk that is deliberately not at the origin so
/// that anything failing to add the trunk's own position shows up.
///
/// The trunk and the anchor carry the same four readings, because the grammar says the same things
/// about either -- so every case that names one can be written against the other.
struct Given {
    trunk: Section,
    trunk_grid: Grid,
    trunk_cell: Area,
    trunk_intrinsic: Area,
    anchor: Section,
    anchor_grid: Grid,
    anchor_cell: Area,
    anchor_intrinsic: Area,
    /// The element's own, which is all it can read of itself.
    intrinsic: Area,
    cell: Area,
    layout: Layout,
    short: Short,
}

impl Default for Given {
    fn default() -> Self {
        Self {
            trunk: Section::from_edges(10.0, 20.0, 210.0, 120.0),
            trunk_grid: Grid::default(),
            trunk_cell: Area::default(),
            trunk_intrinsic: Area::default(),
            anchor: Section::default(),
            anchor_grid: Grid::default(),
            anchor_cell: Area::default(),
            anchor_intrinsic: Area::default(),
            intrinsic: Area::default(),
            cell: Area::default(),
            layout: Layout::Xs,
            short: Short::No,
        }
    }
}

impl Given {
    fn context(&self, axis: Axis) -> Context {
        Context {
            axis,
            own: Basis {
                section: Section::default(),
                intrinsic: self.intrinsic,
                tracks: Tracks::default(),
                cell: self.cell,
            },
            trunk: Basis {
                section: self.trunk,
                intrinsic: self.trunk_intrinsic,
                tracks: self.trunk_grid.tracks(self.layout, self.short),
                cell: self.trunk_cell,
            },
            anchor: Basis {
                section: self.anchor,
                intrinsic: self.anchor_intrinsic,
                tracks: self.anchor_grid.tracks(self.layout, self.short),
                cell: self.anchor_cell,
            },
        }
    }

    fn across(&self, horizontal: Horizontal) -> Span {
        resolve(&horizontal.0, &self.context(Axis::Horizontal))
    }

    fn down(&self, vertical: Vertical) -> Span {
        resolve(&vertical.0, &self.context(Axis::Vertical))
    }
}

fn span(near: f32, far: f32) -> Span {
    Span { near, far }
}

// The four legal forms, per axis.

#[test]
fn near_and_extent() {
    let given = Given::default();
    assert_eq!(
        given.across(left(20.px()).width(140.px())),
        span(30.0, 170.0)
    );
    assert_eq!(given.down(top(20.px()).height(40.px())), span(40.0, 80.0));
}

#[test]
fn near_and_far() {
    let given = Given::default();
    assert_eq!(
        given.across(left(20.px()).right(100.pct() - 16.px())),
        span(30.0, 194.0)
    );
    assert_eq!(
        given.down(top(20.px()).bottom(100.pct() - 16.px())),
        span(40.0, 104.0)
    );
}

#[test]
fn far_and_extent() {
    let given = Given::default();
    assert_eq!(
        given.across(right(100.pct() - 16.px()).width(140.px())),
        span(54.0, 194.0)
    );
    assert_eq!(
        given.down(bottom(100.pct() - 16.px()).height(40.px())),
        span(64.0, 104.0)
    );
}

#[test]
fn center_and_extent() {
    let given = Given::default();
    assert_eq!(
        given.across(center_x(50.pct()).width(140.px())),
        span(40.0, 180.0)
    );
    assert_eq!(
        given.down(center_y(50.pct()).height(40.px())),
        span(50.0, 90.0)
    );
}

/// A far edge and an extent are two independent declarations, and they compose exactly as written:
/// a 140-wide box sitting 16 in from the parent's right. Nothing is counted twice.
#[test]
fn a_far_edge_and_an_extent_do_not_double_count() {
    let given = Given::default();
    let span = given.across(right(100.pct() - 16.px()).width(140.px()));
    assert_eq!(span.far - given.trunk.left(), given.trunk.width() - 16.0);
    assert_eq!(
        span.near - given.trunk.left(),
        given.trunk.width() - 156.0
    );
    assert_eq!(span.extent(), 140.0);
}

// Units.

#[test]
fn a_percentage_is_of_the_parent_on_the_role_s_axis() {
    let given = Given::default();
    assert_eq!(
        given.across(left(0.px()).width(50.pct())),
        span(10.0, 110.0)
    );
    assert_eq!(given.down(top(0.px()).height(50.pct())), span(20.0, 70.0));
}

#[test]
fn letters_are_the_element_s_own_character_cell() {
    let given = Given {
        cell: Area::new(9.0, 18.0),
        ..Given::default()
    };
    assert_eq!(
        given.across(left(0.px()).width(8.letters())),
        span(10.0, 82.0)
    );
    assert_eq!(
        given.down(top(0.px()).height(2.letters())),
        span(20.0, 56.0)
    );
}

// Cells: a one-based index whose meaning is the role's decision.

/// Four columns with an eight pixel gap across two hundred: `(200 - 3 * 8) / 4 = 44`.
fn columns() -> Given {
    Given {
        trunk_grid: Grid::new().xs(4.columns().gap(8.0), 2.rows().gap(10.0)),
        ..Given::default()
    }
}

#[test]
fn a_near_role_gives_a_column_s_near_edge() {
    assert_eq!(
        columns().across(left(2.col()).width(0.px())),
        span(62.0, 62.0)
    );
}

#[test]
fn a_far_role_gives_a_column_s_far_edge() {
    assert_eq!(
        columns().across(right(2.col()).width(0.px())),
        span(106.0, 106.0)
    );
}

#[test]
fn a_centre_role_gives_a_column_s_middle() {
    assert_eq!(
        columns().across(center_x(2.col()).width(0.px())),
        span(84.0, 84.0)
    );
}

#[test]
fn a_size_role_gives_a_span_of_columns_with_its_gaps() {
    assert_eq!(
        columns().across(left(0.px()).width(2.col())),
        span(10.0, 106.0)
    );
}

/// The pair of them is the column itself, which is the pattern a child filling one cell reaches for.
#[test]
fn a_column_addressed_from_both_sides_is_that_column() {
    assert_eq!(
        columns().across(left(1.col()).right(1.col())),
        span(10.0, 54.0)
    );
    assert_eq!(
        columns().across(left(1.col()).right(3.col())),
        span(10.0, 158.0)
    );
}

#[test]
fn rows_read_the_same_way_on_the_other_axis() {
    assert_eq!(
        columns().down(top(2.row()).bottom(2.row())),
        span(75.0, 120.0)
    );
}

/// The horizontal axis resolves first, so a column span is available as a height. The reverse is
/// refused by the type system, which is the resolution order stated as types.
#[test]
fn a_column_span_can_be_a_height() {
    assert_eq!(
        columns().down(top(0.px()).height(2.col())),
        span(20.0, 116.0)
    );
}

#[test]
fn tracks_can_be_pitched_in_pixels() {
    let given = Given {
        trunk_grid: Grid::new().xs(Columns::px(40.0).gap(4.0), 1.rows()),
        ..Given::default()
    };
    assert_eq!(given.across(left(2.col()).right(2.col())), span(54.0, 94.0));
}

/// A letter-pitched track takes its pitch from the element the grid is on, not from the child
/// addressing it -- which is what makes a column a real address into such a grid.
#[test]
fn tracks_can_be_pitched_in_the_parent_s_letters() {
    let given = Given {
        trunk_grid: Grid::new().xs(Columns::letters(1.0), Rows::letters(1.0)),
        trunk_cell: Area::new(9.0, 18.0),
        cell: Area::new(100.0, 100.0),
        ..Given::default()
    };
    assert_eq!(given.across(left(3.col()).right(3.col())), span(28.0, 37.0));
    assert_eq!(given.down(top(2.row()).bottom(2.row())), span(38.0, 56.0));
}

// Anchors.

fn anchored() -> Given {
    Given {
        anchor: Section::from_edges(300.0, 40.0, 400.0, 90.0),
        ..Given::default()
    }
}

// The anchor as a basis.
//
// An element that leaves its trunk -- to escape a stack, or a clip -- anchors back to it, and this
// is what makes that a move rather than a downgrade: everything it could say against a trunk it can
// say against an anchor.

/// The same grid, addressed by an element that is no longer under it, lands on the same box.
#[test]
fn an_anchor_s_grid_is_addressable() {
    let grid = Grid::new().xs(4.columns().gap(8.0), 2.rows().gap(10.0));
    let under = Given {
        trunk: Section::from_edges(300.0, 40.0, 400.0, 90.0),
        trunk_grid: grid,
        ..Given::default()
    };
    let beside = Given {
        anchor: Section::from_edges(300.0, 40.0, 400.0, 90.0),
        anchor_grid: grid,
        ..Given::default()
    };
    assert_eq!(
        beside.across(left(anchor().col(2)).right(anchor().col(3))),
        under.across(left(2.col()).right(3.col()))
    );
    assert_eq!(
        beside.down(top(anchor().row(2)).bottom(anchor().row(2))),
        under.down(top(2.row()).bottom(2.row()))
    );
}

/// A track of someone else's grid is somewhere on the surface, so it carries that element's origin
/// and not the trunk's.
#[test]
fn an_anchor_s_track_takes_the_anchor_s_origin() {
    let given = Given {
        anchor: Section::from_edges(300.0, 40.0, 400.0, 90.0),
        anchor_grid: Grid::new().xs(2.columns(), 1.rows()),
        ..Given::default()
    };
    assert_eq!(
        given.across(left(anchor().col(1)).right(anchor().col(1))),
        span(300.0, 350.0)
    );
}

/// A letter-pitched grid is measured in the font of the element the grid is on, wherever the
/// element addressing it happens to be grown.
#[test]
fn an_anchor_s_tracks_are_pitched_in_the_anchor_s_letters() {
    let given = Given {
        anchor: Section::from_edges(300.0, 40.0, 400.0, 90.0),
        anchor_grid: Grid::new().xs(Columns::letters(1.0), Rows::letters(1.0)),
        anchor_cell: Area::new(9.0, 18.0),
        cell: Area::new(100.0, 100.0),
        ..Given::default()
    };
    assert_eq!(
        given.across(left(anchor().col(3)).right(anchor().col(3))),
        span(318.0, 327.0)
    );
}

/// What another element measured to, which is a different number from its box whenever it was given
/// more room than it asked for.
#[test]
fn an_anchor_s_content_is_readable_and_is_not_its_box() {
    let given = Given {
        anchor: Section::from_edges(300.0, 40.0, 400.0, 90.0),
        anchor_intrinsic: Area::new(60.0, 12.0),
        ..Given::default()
    };
    assert_eq!(
        given.across(left(anchor().left()).width(anchor().content())),
        span(300.0, 360.0)
    );
    assert_eq!(
        given.across(left(anchor().left()).width(anchor().width())),
        span(300.0, 400.0)
    );
}

#[test]
fn a_trunk_s_content_is_readable() {
    let given = Given {
        trunk_intrinsic: Area::new(60.0, 12.0),
        ..Given::default()
    };
    assert_eq!(
        given.across(left(0.px()).width(trunk().content())),
        span(10.0, 70.0)
    );
}

/// A count of letters is the reader's own font, and another element's is asked for by name -- the
/// two are different numbers and neither stands in for the other.
#[test]
fn letters_are_the_reader_s_own_unless_another_is_named() {
    let given = Given {
        cell: Area::new(7.0, 14.0),
        trunk_cell: Area::new(9.0, 18.0),
        anchor: Section::from_edges(300.0, 40.0, 400.0, 90.0),
        anchor_cell: Area::new(11.0, 22.0),
        ..Given::default()
    };
    assert_eq!(given.across(left(0.px()).width(4.letters())), span(10.0, 38.0));
    assert_eq!(
        given.across(left(0.px()).width(trunk().letters(4.0))),
        span(10.0, 46.0)
    );
    assert_eq!(
        given.across(left(0.px()).width(anchor().letters(4.0))),
        span(10.0, 54.0)
    );
}

/// Terms in one expression may read different elements. Sitting below the anchor while being sized
/// by the trunk is one measurement, not two frames of reference fighting.
#[test]
fn one_expression_may_read_two_elements() {
    let given = Given {
        anchor: Section::from_edges(300.0, 40.0, 400.0, 90.0),
        ..Given::default()
    };
    assert_eq!(
        given.down(top(anchor().bottom() + 8.px()).height(50.pct())),
        span(98.0, 148.0)
    );
}

#[test]
fn an_anchor_edge_is_a_position_and_takes_no_parent_origin() {
    assert_eq!(
        anchored().across(left(anchor().right()).width(140.px())),
        span(400.0, 540.0)
    );
    assert_eq!(
        anchored().down(top(anchor().bottom()).height(40.px())),
        span(90.0, 130.0)
    );
}

#[test]
fn an_anchor_edge_composes_with_a_length() {
    assert_eq!(
        anchored().across(left(anchor().right() + 8.px()).width(anchor().width())),
        span(408.0, 508.0)
    );
    assert_eq!(
        anchored().down(top(anchor().bottom() + 8.px()).height(anchor().height())),
        span(98.0, 148.0)
    );
}

#[test]
fn an_anchor_centre_is_a_position() {
    assert_eq!(
        anchored().across(center_x(anchor().center_x()).width(20.px())),
        span(340.0, 360.0)
    );
    assert_eq!(
        anchored().down(center_y(anchor().center_y()).height(10.px())),
        span(60.0, 70.0)
    );
}

/// Two positions subtract to the length between them, which is how an anchor's edges are used as a
/// size. An edge on its own is refused, because an edge is not an extent.
#[test]
fn the_distance_between_two_anchor_edges_is_a_size() {
    assert_eq!(
        anchored().across(left(0.px()).width(anchor().right() - anchor().left())),
        span(10.0, 110.0)
    );
}

#[test]
fn an_anchor_extent_scales() {
    assert_eq!(
        anchored().across(left(0.px()).width(anchor().width() * 0.5)),
        span(10.0, 60.0)
    );
}

/// An anchor's width is a length rather than a position, so it is legal on either axis: an element
/// as tall as its anchor is wide.
#[test]
fn an_anchor_width_can_be_a_height() {
    assert_eq!(
        anchored().down(top(0.px()).height(anchor().width())),
        span(20.0, 120.0)
    );
}

#[test]
fn an_element_with_no_anchor_reads_a_zero_box() {
    let given = Given::default();
    assert_eq!(
        given.across(left(anchor().right()).width(10.px())),
        span(0.0, 10.0)
    );
}

// Arithmetic.

#[test]
fn arithmetic_may_resolve_negative() {
    let given = Given::default();
    assert_eq!(
        given.across(left(20.px() - 50.px()).width(10.px())),
        span(-20.0, -10.0)
    );
    assert_eq!(
        given.across(left(-(30.px())).width(10.px())),
        span(-20.0, -10.0)
    );
}

#[test]
fn terms_of_different_units_sum() {
    let given = Given {
        trunk_grid: Grid::new().xs(4.columns().gap(8.0), 1.rows()),
        ..Given::default()
    };
    assert_eq!(
        given.across(left(1.col() + 50.pct() - 4.px()).width(10.px())),
        span(106.0, 116.0)
    );
}

// Clamps.

#[test]
fn at_most_caps_an_extent() {
    let given = Given::default();
    assert_eq!(
        given.across(left(0.px()).width(500.px()).at_most(300.px())),
        span(10.0, 310.0)
    );
}

#[test]
fn at_least_floors_an_extent() {
    let given = Given::default();
    assert_eq!(
        given.across(left(0.px()).width(10.px()).at_least(50.px())),
        span(10.0, 60.0)
    );
}

/// A floor beats a ceiling, so a box asked to be both wider than one and narrower than the other
/// takes the floor rather than an answer neither bound allows.
#[test]
fn a_floor_beats_a_ceiling() {
    let given = Given::default();
    assert_eq!(
        given.across(
            left(0.px())
                .width(10.px())
                .at_least(80.px())
                .at_most(40.px())
        ),
        span(10.0, 90.0)
    );
}

#[test]
fn a_clamp_holds_the_edge_the_form_pinned() {
    let given = Given::default();
    assert_eq!(
        given.across(right(100.pct()).width(500.px()).at_most(300.px())),
        span(-90.0, 210.0)
    );
    assert_eq!(
        given.across(center_x(50.pct()).width(500.px()).at_most(300.px())),
        span(-40.0, 260.0)
    );
    assert_eq!(
        given.across(left(0.px()).right(100.pct()).at_most(50.px())),
        span(10.0, 60.0)
    );
}

#[test]
fn an_extent_is_never_negative() {
    let given = Given::default();
    assert_eq!(
        given.across(left(100.px()).right(0.px())),
        span(110.0, 110.0)
    );
}

// Content.

#[test]
fn content_asks_a_different_question_per_axis() {
    let given = Given {
        intrinsic: Area::new(180.0, 60.0),
        ..Given::default()
    };
    assert_eq!(
        given.across(left(0.px()).width(content())),
        span(10.0, 190.0)
    );
    assert_eq!(given.down(top(0.px()).height(content())), span(20.0, 80.0));
}

/// Content under a ceiling is fit-content: whichever of the two is smaller.
#[test]
fn content_under_a_ceiling_is_fit_content() {
    let given = Given {
        intrinsic: Area::new(180.0, 60.0),
        ..Given::default()
    };
    assert_eq!(
        given.across(left(0.px()).width(content()).at_most(100.px())),
        span(10.0, 110.0)
    );
    assert_eq!(
        given.across(left(0.px()).width(content()).at_most(300.px())),
        span(10.0, 190.0)
    );
}

#[test]
fn an_element_with_no_content_is_intrinsically_empty() {
    let given = Given::default();
    assert_eq!(
        given.across(left(0.px()).width(content())),
        span(10.0, 10.0)
    );
}

// Breakpoints.

#[test]
fn a_breakpoint_falls_back_to_the_nearest_smaller_one_given() {
    let location = crate::Location::new()
        .xs(left(0.px()).width(1.px()), top(0.px()).height(1.px()))
        .md(left(0.px()).width(2.px()), top(0.px()).height(2.px()));
    let width = |layout| {
        resolve(
            &location.axes(layout, Short::No).horizontal,
            &Given::default().context(Axis::Horizontal),
        )
        .extent()
    };
    assert_eq!(width(Layout::Xs), 1.0);
    assert_eq!(width(Layout::Sm), 1.0);
    assert_eq!(width(Layout::Md), 2.0);
    assert_eq!(width(Layout::Lg), 2.0);
    assert_eq!(width(Layout::Xl), 2.0);
}

/// Width and height are independent, so a cramped viewport wins outright rather than sitting in the
/// width chain, where only one axis can be ordered.
#[test]
fn a_short_configuration_wins_over_the_width_chain() {
    let location = crate::Location::new()
        .xs(left(0.px()).width(1.px()), top(0.px()).height(1.px()))
        .xl(left(0.px()).width(2.px()), top(0.px()).height(2.px()))
        .short(left(0.px()).width(3.px()), top(0.px()).height(3.px()));
    let width = |layout, short| {
        resolve(
            &location.axes(layout, short).horizontal,
            &Given::default().context(Axis::Horizontal),
        )
        .extent()
    };
    assert_eq!(width(Layout::Xl, Short::No), 2.0);
    assert_eq!(width(Layout::Xl, Short::Yes), 3.0);
    assert_eq!(width(Layout::Xs, Short::Yes), 3.0);
}

#[test]
fn a_grid_falls_back_the_same_way() {
    let grid = Grid::new()
        .xs(2.columns(), 1.rows())
        .lg(4.columns(), 1.rows());
    let column = |layout| {
        Given {
            trunk_grid: grid,
            layout,
            ..Given::default()
        }
        .across(left(0.px()).width(1.col()))
        .extent()
    };
    assert_eq!(column(Layout::Md), 100.0);
    assert_eq!(column(Layout::Lg), 50.0);
}

#[test]
fn an_element_that_says_nothing_fills_its_parent() {
    let given = Given::default();
    let location = crate::Location::default();
    let axes = location.axes(Layout::Xs, Short::No);
    assert_eq!(
        resolve(&axes.horizontal, &given.context(Axis::Horizontal)),
        span(10.0, 210.0)
    );
    assert_eq!(
        resolve(&axes.vertical, &given.context(Axis::Vertical)),
        span(20.0, 120.0)
    );
}
