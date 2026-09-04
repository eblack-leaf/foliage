//! The remaining renderers: what each states, what each resolves to, and what reaches the backend.
//!
//! Everything below the batch is the platform layer the suite cannot reach (`harness.md`), so what
//! is proven here is the same thing every other slice proves -- that the declared state becomes the
//! right instance, exactly once, and that an unchanged frame costs nothing.

use crate::coordinate::{Area, Position, Section};
use crate::icon::IconInstance;
use crate::image::ImageInstance;
use crate::line::LineInstance;
use crate::polygon::PolygonInstance;
use crate::tests::{grove, section, tick};
use crate::{
    Boxed, Cap, Color, Fill, Fit, Grove, Grow, Image, Leaf, Line, Location, Motion, Palette, Panel,
    Place, Plate, Point, Polygon, Rounding, Sap, Scheme, Shape, Source, Stem, Text, Timing, Vein,
    anchor, left, top,
};

// -- Lines -----------------------------------------------------------------------------------

fn line(from: (f32, f32), to: (f32, f32), weight: f32) -> Line {
    Line::new().weight(weight).between(
        Point::new(from.0.px(), from.1.px()),
        Point::new(to.0.px(), to.1.px()),
    )
}

fn stroke(grove: &Grove, leaf: Leaf) -> LineInstance {
    grove
        .elm
        .lines
        .holding(leaf)
        .expect("the backend is holding this stroke")
}

/// A stroke has no box of its own, so the resolver gives it one: the rectangle around its ends,
/// grown by half its weight on every side. That is what makes it clippable, rankable and
/// hit-testable like anything else.
#[test]
fn a_strokes_box_is_its_ends_grown_by_half_its_weight() {
    let mut grove = grove();
    let leaf = grove.plant(line((10.0, 20.0), (50.0, 60.0), 4.0));
    tick(&mut grove);
    assert_eq!(section(&grove, leaf), Section::from_edges(8.0, 18.0, 52.0, 62.0));
}

/// The case the weight exists for. Two ends on one line describe a rectangle of no height, and an
/// element with no height is culled before it is ever drawn -- so a rule would be invisible if the
/// weight were decoration rather than placement.
#[test]
fn a_rule_whose_ends_share_a_row_still_has_a_box() {
    let mut grove = grove();
    let leaf = grove.plant(line((0.0, 40.0), (100.0, 40.0), 2.0));
    tick(&mut grove);
    assert_eq!(section(&grove, leaf), Section::from_edges(-1.0, 39.0, 101.0, 41.0));
    assert_eq!(stroke(&grove, leaf).weight, 2.0);
}

/// A rectangle has two diagonals and cannot say which one the stroke runs along, so the ends are
/// settled beside the box rather than recovered from it.
#[test]
fn the_ends_are_settled_and_say_which_diagonal_the_stroke_runs_along() {
    let mut grove = grove();
    let falling = grove.plant(line((10.0, 10.0), (50.0, 50.0), 2.0));
    let rising = grove.plant(line((10.0, 50.0), (50.0, 10.0), 2.0));
    tick(&mut grove);
    // The same box, and not the same stroke.
    assert_eq!(section(&grove, falling), section(&grove, rising));
    assert_eq!(
        grove.tap(falling, Vein::Ends),
        Some(Sap::Ends(Position::new(10.0, 10.0), Position::new(50.0, 50.0)))
    );
    assert_eq!(
        grove.tap(rising, Vein::Ends),
        Some(Sap::Ends(Position::new(10.0, 50.0), Position::new(50.0, 10.0)))
    );
}

/// A point is the ordinary grammar, so every source a box's edges take reads the same way here --
/// and a stroke follows what it reads when that moves.
#[test]
fn a_strokes_ends_read_the_whole_grammar() {
    let mut grove = grove();
    let target = grove.plant(Panel::new().at(Location::new().xs(
        left(100.px()).width(40.px()),
        top(200.px()).height(20.px()),
    )));
    let leaf = grove.plant(
        Line::new()
            .weight(2.0)
            .anchored(target)
            .between(
                Point::new(0.px(), 0.px()),
                Point::new(anchor().left(), anchor().bottom()),
            ),
    );
    tick(&mut grove);
    assert_eq!(
        grove.tap(leaf, Vein::Ends),
        Some(Sap::Ends(Position::new(0.0, 0.0), Position::new(100.0, 220.0)))
    );
}

/// Where an element is stated and where it is written are one question, so the two placements are
/// two verbs and each refuses what the other owns.
#[test]
fn a_box_and_a_trace_refuse_each_others_verbs() {
    let mut grove = grove();
    let stroke = grove.plant(line((0.0, 0.0), (10.0, 10.0), 1.0));
    let panel = grove.plant(Panel::new());
    tick(&mut grove);
    let before = (section(&grove, stroke), section(&grove, panel));
    grove.at(
        stroke,
        Location::new().xs(left(0.px()).width(90.px()), top(0.px()).height(90.px())),
    );
    grove.between(
        panel,
        Point::new(0.px(), 0.px()),
        Point::new(90.px(), 90.px()),
    );
    tick(&mut grove);
    assert_eq!((section(&grove, stroke), section(&grove, panel)), before);
}

#[test]
fn between_moves_both_ends_at_once() {
    let mut grove = grove();
    let leaf = grove.plant(line((0.0, 0.0), (10.0, 10.0), 2.0));
    tick(&mut grove);
    grove.between(
        leaf,
        Point::new(20.px(), 30.px()),
        Point::new(60.px(), 70.px()),
    );
    tick(&mut grove);
    assert_eq!(
        grove.tap(leaf, Vein::Ends),
        Some(Sap::Ends(Position::new(20.0, 30.0), Position::new(60.0, 70.0)))
    );
    assert_eq!(stroke(&grove, leaf).from, Position::new(20.0, 30.0));
}

/// Two strokes sharing an end resolve that end to the same point, and each reaches half a weight
/// past it.
///
/// What a chain of strokes needs to cover the wedge at a turn, and the whole of what is checkable
/// headlessly: whether the coverage there actually reads cleanly is the rasteriser's, which is the
/// layer the suite cannot reach.
#[test]
fn two_strokes_sharing_an_end_resolve_it_to_one_point() {
    let mut grove = grove();
    let turn = (60.0, 20.0);
    let first = grove.plant(line((10.0, 40.0), turn, 6.0).cap(Cap::Round));
    let second = grove.plant(line(turn, (90.0, 50.0), 6.0).cap(Cap::Round));
    tick(&mut grove);
    let Some(Sap::Ends(_, ends_here)) = grove.tap(first, Vein::Ends) else {
        panic!("two ends");
    };
    let Some(Sap::Ends(starts_here, _)) = grove.tap(second, Vein::Ends) else {
        panic!("two ends");
    };
    assert_eq!(ends_here, starts_here);
    assert_eq!(grove.tap(first, Vein::Cap), Some(Sap::Cap(Cap::Round)));
    assert_eq!(grove.tap(second, Vein::Cap), Some(Sap::Cap(Cap::Round)));
    // Each stroke's box reaches half a weight past its own ends, so the disc at the turn is inside
    // both of them and no third element is needed to hold it.
    for stroke in [first, second] {
        assert!(section(&grove, stroke).contains(Position::new(turn.0, turn.1)));
    }
}

/// A butt cap is the default, because a rule that reached half a weight past where it was told to
/// stop would not line up with anything.
#[test]
fn a_stroke_is_butt_capped_unless_it_says_otherwise() {
    let mut grove = grove();
    let leaf = grove.plant(line((0.0, 0.0), (10.0, 0.0), 2.0));
    tick(&mut grove);
    assert_eq!(grove.tap(leaf, Vein::Cap), Some(Sap::Cap(Cap::Butt)));
}

/// A weight below a hairline has no drawing to do, so it is clamped once where it is written rather
/// than at every callsite that reads it.
#[test]
fn a_weight_is_clamped_to_a_hairline() {
    let mut grove = grove();
    let leaf = grove.plant(line((0.0, 0.0), (10.0, 0.0), 0.0));
    tick(&mut grove);
    assert_eq!(grove.tap(leaf, Vein::Weight), Some(Sap::Weight(1.0)));
}

// -- Polygons --------------------------------------------------------------------------------

fn shape(grove: &Grove, leaf: Leaf) -> PolygonInstance {
    grove
        .elm
        .polygons
        .holding(leaf)
        .expect("the backend is holding this shape")
}

fn square(size: f32) -> Location {
    Location::new().xs(left(0.px()).width(size.px()), top(0.px()).height(size.px()))
}

#[test]
fn a_polygon_extracts_its_box_and_its_shape() {
    let mut grove = grove();
    let leaf = grove.plant(
        Polygon::new()
            .sides(6.0)
            .rounding(0.25)
            .rotation(0.5)
            .color(Palette::Accent)
            .at(square(48.0)),
    );
    tick(&mut grove);
    let held = shape(&grove, leaf);
    assert_eq!(held.section, Section::from_edges(0.0, 0.0, 48.0, 48.0));
    assert_eq!(held.shape, [6.0, 0.25, 0.5]);
    assert_eq!(held.color, Scheme::default().color(Palette::Accent));
}

/// Fewer than three sides is not a shape, and more than fully round is not rounder. Both are
/// clamped where they are written, so nothing downstream carries a guard for them.
#[test]
fn a_shape_is_clamped_where_it_is_written() {
    let mut grove = grove();
    let leaf = grove.plant(Polygon::new().sides(1.0).rounding(4.0).at(square(20.0)));
    tick(&mut grove);
    assert_eq!(shape(&grove, leaf).shape[0], 3.0);
    assert_eq!(shape(&grove, leaf).shape[1], 1.0);
}

/// A shape blends to a shape, so the blend is written back over the declaration -- which is what
/// makes reading it back report where the motion has reached rather than where it is going.
#[test]
fn a_shape_in_motion_is_written_back_over_its_declaration() {
    let mut grove = grove();
    let leaf = grove.plant(Polygon::new().sides(3.0).at(square(20.0)));
    tick(&mut grove);
    grove.animate(
        leaf,
        Motion::Polygon(Shape {
            sides: 7.0,
            rounding: 1.0,
            rotation: 0.0,
        }),
        Timing::ms(100),
    );
    tick(&mut grove);
    crate::tests::advance(&mut grove, 50);
    tick(&mut grove);
    let Some(Sap::Shape(half)) = grove.tap(leaf, Vein::Shape) else {
        panic!("a shape");
    };
    assert_eq!(half.sides, 5.0);
    assert_eq!(shape(&grove, leaf).shape[0], 5.0);
    crate::tests::advance(&mut grove, 50);
    tick(&mut grove);
    assert_eq!(shape(&grove, leaf).shape[0], 7.0);
}

/// F8, on the property this slice adds: a direct write cancels the motion moving it.
#[test]
fn reshaping_cancels_a_shape_in_motion() {
    let mut grove = grove();
    let leaf = grove.plant(Polygon::new().sides(3.0).at(square(20.0)));
    tick(&mut grove);
    grove.animate(leaf, Motion::Polygon(Shape { sides: 9.0, rounding: 0.0, rotation: 0.0 }), Timing::ms(100));
    tick(&mut grove);
    crate::tests::advance(&mut grove, 50);
    tick(&mut grove);
    grove.reshape(leaf, Shape { sides: 4.0, rounding: 0.0, rotation: 0.0 });
    tick(&mut grove);
    crate::tests::advance(&mut grove, 200);
    tick(&mut grove);
    assert_eq!(shape(&grove, leaf).shape[0], 4.0);
}

/// A polygon has no rectangle to round, and a stroke has no corners at all, so `round` names
/// something that does not apply to them and is dropped like any other such op.
#[test]
fn rounding_a_shape_or_a_stroke_is_dropped() {
    let mut grove = grove();
    let polygon = grove.plant(Polygon::new().at(square(20.0)));
    let stroke = grove.plant(line((0.0, 0.0), (10.0, 10.0), 1.0));
    tick(&mut grove);
    grove.round(polygon, Rounding::Md);
    grove.round(stroke, Rounding::Md);
    tick(&mut grove);
    assert_eq!(grove.tap(polygon, Vein::Rounding), None);
    assert_eq!(grove.tap(stroke, Vein::Rounding), None);
}

// -- Icons -----------------------------------------------------------------------------------

/// A field small enough to state inline. What is on it does not matter here: nothing below the
/// batch is reachable from the suite, so what is proven is the instance rather than the pixels.
fn field(grove: &mut Grove) -> crate::Field {
    grove.fields.register(&[255; 4 * 4 * 4], 4, 2.0)
}

fn mark(grove: &Grove, leaf: Leaf) -> IconInstance {
    grove
        .elm
        .icons
        .holding(leaf)
        .expect("the backend is holding this mark")
}

/// A distance field is square, so the mark sits in the largest square its box holds rather than
/// stretching to the box's own ratio -- which is what lets a composite size an icon's box loosely.
#[test]
fn a_mark_is_squared_inside_its_box() {
    let mut grove = grove();
    let art = field(&mut grove);
    let leaf = grove.plant(crate::Icon::new(art).at(Location::new().xs(
        left(0.px()).width(80.px()),
        top(0.px()).height(20.px()),
    )));
    tick(&mut grove);
    assert_eq!(
        mark(&grove, leaf).section,
        Section::from_edges(30.0, 0.0, 50.0, 20.0)
    );
}

/// The field carries shape and no colour, so a mark is filled, repainted and animated by exactly
/// the writes a panel and a run take.
#[test]
fn a_mark_is_filled_like_anything_else() {
    let mut grove = grove();
    let art = field(&mut grove);
    let leaf = grove.plant(crate::Icon::new(art).color(Palette::Accent).at(square(24.0)));
    tick(&mut grove);
    assert_eq!(mark(&grove, leaf).color, Scheme::default().color(Palette::Accent));
    assert_eq!(grove.tap(leaf, Vein::Color), Some(Sap::Color(Fill::Role(Palette::Accent))));
    grove.color(leaf, Color::rgb(1.0, 0.0, 0.0));
    tick(&mut grove);
    assert_eq!(mark(&grove, leaf).color, Color::rgb(1.0, 0.0, 0.0));
}

// -- Images ----------------------------------------------------------------------------------

/// Four texels of RGBA, two by two.
fn pixels() -> Vec<u8> {
    vec![255; 2 * 2 * 4]
}

fn picture(grove: &Grove, leaf: Leaf) -> ImageInstance {
    grove
        .elm
        .images
        .holding(leaf)
        .expect("the backend is holding this picture")
}

/// A name is valid the moment it is handed out, so an element can be grown against a picture whose
/// pixels are still on their way -- which is the whole reason registration is an op rather than a
/// call that has to happen at boot.
#[test]
fn a_plate_is_usable_in_the_frame_it_is_named() {
    let mut grove = grove();
    let plate = grove.image(pixels(), Area::new(2.0, 2.0));
    let leaf = grove.plant(Image::new(plate).at(square(40.0)));
    tick(&mut grove);
    assert_eq!(picture(&grove, leaf).section, Section::from_edges(0.0, 0.0, 40.0, 40.0));
    assert_eq!(grove.tap(leaf, Vein::Picture), Some(Sap::Picture(plate)));
}

/// A picture with nothing behind it yet occupies its box and draws nothing. Absent from the batch
/// rather than held as blank, so the frame its pixels arrive is the frame it appears and there is
/// nothing to undo.
#[test]
fn a_picture_with_no_pixels_yet_is_absent_from_the_batch() {
    let mut grove = grove();
    let plate = grove.plate();
    let leaf = grove.plant(Image::new(plate).at(square(40.0)));
    tick(&mut grove);
    assert_eq!(grove.elm.images.len(), 0);
    // The element is still there, still placed, still in the stack.
    assert_eq!(section(&grove, leaf), Section::from_edges(0.0, 0.0, 40.0, 40.0));
}

/// Fitting inside the box changes the box and shows the whole picture; filling it keeps the box and
/// shows part of the picture. One of the two moves, never both.
#[test]
fn a_fit_moves_the_box_or_the_crop_and_never_both() {
    let mut grove = grove();
    let wide = grove.image(vec![255; 4 * 1 * 4], Area::new(4.0, 1.0));
    let across = Location::new().xs(left(0.px()).width(40.px()), top(0.px()).height(40.px()));
    let fitted = grove.plant(Image::new(wide).fit(Fit::Aspect).at(across.clone()));
    let cropped = grove.plant(Image::new(wide).fit(Fit::Crop).at(across.clone()));
    let stretched = grove.plant(Image::new(wide).fit(Fit::Stretch).at(across));
    tick(&mut grove);
    // Four to one inside a square: forty across, ten tall, centred.
    assert_eq!(
        picture(&grove, fitted).section,
        Section::from_edges(0.0, 15.0, 40.0, 25.0)
    );
    assert_eq!(picture(&grove, fitted).crop, [0.0, 0.0, 1.0, 1.0]);
    // The box is kept and a quarter of the picture's width fills it.
    assert_eq!(
        picture(&grove, cropped).section,
        Section::from_edges(0.0, 0.0, 40.0, 40.0)
    );
    assert_eq!(picture(&grove, cropped).crop, [0.375, 0.0, 0.25, 1.0]);
    assert_eq!(
        picture(&grove, stretched).section,
        Section::from_edges(0.0, 0.0, 40.0, 40.0)
    );
    assert_eq!(picture(&grove, stretched).crop, [0.0, 0.0, 1.0, 1.0]);
}

/// The radii are measured against the box the pixels are actually drawn into, which under
/// [`Fit::Aspect`] is not the box the element resolved to.
#[test]
fn a_pictures_corners_round_against_the_box_its_pixels_fill() {
    let mut grove = grove();
    let wide = grove.image(vec![255; 4 * 1 * 4], Area::new(4.0, 1.0));
    let leaf = grove.plant(
        Image::new(wide)
            .fit(Fit::Aspect)
            .rounding(Rounding::Full)
            .at(square(40.0)),
    );
    tick(&mut grove);
    // Half the shorter side of the drawn box -- ten tall, so five -- and not half of forty.
    assert_eq!(picture(&grove, leaf).radii, [5.0; 4]);
}

/// Writing the same name again replaces what it holds, so every element drawing it follows without
/// any of them being named.
#[test]
fn loading_a_plate_again_reaches_every_element_drawing_it() {
    let mut grove = grove();
    let plate = grove.image(vec![255; 4 * 4], Area::new(2.0, 2.0));
    let leaf = grove.plant(Image::new(plate).fit(Fit::Aspect).at(square(40.0)));
    tick(&mut grove);
    assert_eq!(picture(&grove, leaf).section.height(), 40.0);
    // The same name, a picture of a different shape.
    grove.load(plate, vec![255; 4 * 1 * 4], Area::new(4.0, 1.0));
    tick(&mut grove);
    assert_eq!(picture(&grove, leaf).section.height(), 10.0);
}

// -- Per-character tints ---------------------------------------------------------------------

fn colors(grove: &Grove, leaf: Leaf) -> Vec<Color> {
    grove
        .elm
        .texts
        .run(leaf.into())
        .expect("the backend is holding this run")
        .glyphs
        .iter()
        .map(|glyph| glyph.color)
        .collect()
}

fn wide(value: &str) -> Text {
    Text::new(value).at(Location::new().xs(
        left(0.px()).width(400.px()),
        top(0.px()).height(40.px()),
    ))
}

/// A tint fills part of a run, and everything untinted stays the run's own -- so a run with tints
/// is the run it would otherwise be, with some of its characters saying otherwise.
#[test]
fn a_tint_fills_its_range_and_nothing_else() {
    let mut grove = grove();
    let leaf = grove.plant(wide("abcd").color(Palette::Ink).tint(1..3, Palette::Accent));
    tick(&mut grove);
    let scheme = Scheme::default();
    assert_eq!(
        colors(&grove, leaf),
        vec![
            scheme.color(Palette::Ink),
            scheme.color(Palette::Accent),
            scheme.color(Palette::Accent),
            scheme.color(Palette::Ink),
        ]
    );
}

/// Ranges are in characters of the value, spaces included. Counting drawn glyphs instead would make
/// every index after a space mean something other than what was written.
#[test]
fn a_range_is_in_characters_of_the_value_and_not_in_glyphs() {
    let mut grove = grove();
    // "ab cd": the space is character two, so character three is "c". Counting the four drawn
    // glyphs instead would put "d" at three, which is what this range would then have hit.
    let leaf = grove.plant(wide("ab cd").color(Palette::Ink).tint(3..4, Palette::Accent));
    tick(&mut grove);
    let scheme = Scheme::default();
    assert_eq!(
        colors(&grove, leaf),
        vec![
            scheme.color(Palette::Ink),
            scheme.color(Palette::Ink),
            // "c", and not "d".
            scheme.color(Palette::Accent),
            scheme.color(Palette::Ink),
        ]
    );
}

/// The rule a reader can predict without knowing what else was written.
#[test]
fn the_later_tint_wins_where_two_overlap() {
    let mut grove = grove();
    let leaf = grove.plant(
        wide("abcd")
            .tint(0..4, Palette::Muted)
            .tint(2..4, Palette::Accent),
    );
    tick(&mut grove);
    let scheme = Scheme::default();
    assert_eq!(
        colors(&grove, leaf)[1..],
        [
            scheme.color(Palette::Muted),
            scheme.color(Palette::Accent),
            scheme.color(Palette::Accent),
        ]
    );
}

/// A tint is a fill like any other, so a role follows the scheme and a literal does not.
#[test]
fn a_tinted_role_follows_a_repaint_and_a_literal_does_not() {
    let mut grove = grove();
    let literal = Color::rgb(1.0, 0.0, 0.0);
    let leaf = grove.plant(
        wide("abcd")
            .tint(0..2, Palette::Accent)
            .tint(2..4, literal),
    );
    tick(&mut grove);
    let moved = Color::rgb(0.0, 1.0, 0.0);
    grove.repaint(Scheme::default().set(Palette::Accent, moved));
    tick(&mut grove);
    assert_eq!(colors(&grove, leaf), vec![moved, moved, literal, literal]);
}

#[test]
fn tinting_replaces_every_tint_and_untinting_takes_them_off() {
    let mut grove = grove();
    let leaf = grove.plant(wide("abcd").color(Palette::Ink).tint(0..4, Palette::Accent));
    tick(&mut grove);
    grove.tint(leaf, [(0..1, Palette::Muted)]);
    tick(&mut grove);
    let scheme = Scheme::default();
    assert_eq!(colors(&grove, leaf)[0], scheme.color(Palette::Muted));
    assert_eq!(colors(&grove, leaf)[1], scheme.color(Palette::Ink));
    grove.untint(leaf);
    tick(&mut grove);
    assert_eq!(colors(&grove, leaf), vec![scheme.color(Palette::Ink); 4]);
}

/// Naming something that has no run to tint is dropped like any other op that does not apply.
#[test]
fn tinting_something_that_is_not_a_run_is_dropped() {
    let mut grove = grove();
    let leaf = grove.plant(Panel::new().at(square(20.0)));
    tick(&mut grove);
    grove.tint(leaf, [(0..2, Palette::Accent)]);
    tick(&mut grove);
    assert_eq!(grove.elm.panels.written.len(), 0);
}

// -- What an unchanged frame costs -------------------------------------------------------------

/// The claim the whole phase rests on, over the five renderers this slice adds to it.
#[test]
fn an_unchanged_frame_writes_nothing_for_any_renderer() {
    let mut grove = grove();
    let art = field(&mut grove);
    let plate = grove.image(pixels(), Area::new(2.0, 2.0));
    grove.plant(Polygon::new().sides(5.0).at(square(20.0)));
    grove.plant(line((0.0, 0.0), (30.0, 30.0), 2.0));
    grove.plant(crate::Icon::new(art).at(square(20.0)));
    grove.plant(Image::new(plate).at(square(20.0)));
    grove.plant(wide("abc").tint(0..1, Palette::Accent));
    grove.plant(Stem::new());
    for _ in 0..8 {
        tick(&mut grove);
    }
    assert_eq!(grove.elm.moved(), (0, 0));
}

/// The backend's copy is in device pixels and this one is not, so a display that changed density
/// leaves every derived instance correct against a density that is gone -- while the logical values
/// they came from compare equal forever.
#[test]
fn a_recut_makes_the_next_frame_write_everything_again() {
    let mut grove = grove();
    grove.plant(Polygon::new().at(square(20.0)));
    grove.plant(line((0.0, 0.0), (30.0, 30.0), 2.0));
    grove.plant(wide("abc"));
    tick(&mut grove);
    tick(&mut grove);
    assert_eq!(grove.elm.moved(), (0, 0));
    grove.elm.recut();
    tick(&mut grove);
    assert_eq!(grove.elm.moved(), (3, 0));
}
