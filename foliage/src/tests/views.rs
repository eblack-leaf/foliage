//! Views: extent, pinning, chain against contain, `ScrollTo`, and momentum.
//!
//! What a region *does* with a gesture was proven in `interaction` when the structure landed --
//! that a drag anywhere inside a region scrolls it, that a region at its end hands a continuing
//! drag outward, that nested regions hand off in order, that a hidden child leaves the extent, and
//! that a disabled region neither scrolls nor chains. Those stay where they are, because they are
//! statements about the gesture. What is here is everything about the region itself.

use crate::coordinate::{Area, Axes, Position, Section};
use crate::tests::{advance, drag, grove, press, release, section, tick, wheel};
use crate::view::coasted;
use crate::{
    Boxed,
    Divide, Elevation, Escape, Grid, Grove, Grow, Leaf, Location, Motion, Panel, Place, Sap, Scroll,
    ScrollTo, Source, Stem, Text, Timing, Vein, anchor, content, left, top,
};

/// A box at a stated place, so a point can be aimed at it and a reach can be counted.
fn at(x: f32, y: f32, width: f32, height: f32) -> Location {
    Location::new().xs(
        left(x.px()).width(width.px()),
        top(y.px()).height(height.px()),
    )
}

/// How far a region has been scrolled, as an app would read it.
fn offset(grove: &Grove, leaf: Leaf) -> Position {
    match grove.tap(leaf, Vein::Offset) {
        Some(Sap::Position(offset)) => offset,
        other => panic!("expected an offset, got {other:?}"),
    }
}

/// How far a region's content reaches, as an app would read it.
fn extent(grove: &Grove, leaf: Leaf) -> Area {
    match grove.tap(leaf, Vein::Extent) {
        Some(Sap::Area(extent)) => extent,
        other => panic!("expected an extent, got {other:?}"),
    }
}

/// A column two hundred by one hundred, with four hundred of content in it: a range of three
/// hundred.
fn column(grove: &mut Grove) -> (Leaf, Leaf) {
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    let content = grove.branch(region, Panel::new().at(at(0.0, 0.0, 200.0, 400.0)));
    (region, content)
}

/// Runs frames until nothing is coasting, or gives up and says so.
fn settle(grove: &mut Grove) {
    for _ in 0..600 {
        if grove.coasting.idle() {
            return;
        }
        advance(grove, 16);
        tick(grove);
    }
    panic!("a coast that never settled");
}

// -- A grid is not a view ------------------------------------------------------------------------

/// Laying children out on a grid says nothing about scrolling. An element scrolls because it said
/// so, and for no other reason -- so none of the three ways to move one moves this.
#[test]
fn a_grid_alone_does_not_scroll() {
    let mut grove = grove();
    let leaf = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .grid(Grid::new().xs(2.columns(), 2.rows())),
    );
    let content = grove.branch(leaf, Panel::new().at(at(0.0, 0.0, 200.0, 400.0)));
    tick(&mut grove);

    // It has no offset at all, rather than an offset of zero it can never leave.
    assert_eq!(grove.tap(leaf, Vein::Offset), None);
    assert_eq!(grove.tap(leaf, Vein::Extent), None);

    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 10.0);
    wheel(&mut grove, (50.0, 50.0), (0.0, -80.0));
    grove.scroll(leaf, ScrollTo::end());
    tick(&mut grove);

    assert_eq!(grove.tap(leaf, Vein::Offset), None);
    assert_eq!(section(&grove, content).top(), 0.0);
}

// -- Extent --------------------------------------------------------------------------------------

/// An axis that was not declared is not a scrolling axis with a range of zero: no extent is
/// computed along it at all, so the region reads its own box there however far a child reaches
/// sideways. Most accidental extent came from the axis nobody was scrolling.
#[test]
fn a_vertical_view_computes_no_horizontal_extent() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(region, Panel::new().at(at(0.0, 0.0, 600.0, 400.0)));
    tick(&mut grove);

    assert_eq!(extent(&grove, region), Area::new(200.0, 400.0));
}

/// The same region, and the same drag it has no axis to answer with.
#[test]
fn a_vertical_view_ignores_a_horizontal_drag() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(region, Panel::new().at(at(0.0, 0.0, 600.0, 400.0)));
    tick(&mut grove);

    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 150.0, 50.0);
    tick(&mut grove);

    assert_eq!(offset(&grove, region), Position::default());
}

/// A visible child that is simply far away is content, and is counted. That is the answer to
/// `rowan.md`'s question, and it is deliberately not a guess about intent: if it is out there and
/// visible it is reachable, and scrolling to it works.
#[test]
fn a_child_parked_outside_its_region_grows_the_extent_and_can_be_reached() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    let parked = grove.branch(region, Panel::new().at(at(0.0, 900.0, 200.0, 40.0)));
    tick(&mut grove);
    assert_eq!(extent(&grove, region).height, 940.0);

    grove.scroll(region, ScrollTo::end());
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 840.0);
    // Where it was parked, brought to the bottom of the region it was parked in.
    assert_eq!(section(&grove, parked).top(), 60.0);
}

/// And the fix for parking is the app saying so, which takes the child and its whole subtree out.
/// It is the whole answer rather than half of one, because nothing else in the engine writes it:
/// culling is a decision extraction makes and is never recorded on an element.
#[test]
fn hiding_the_parked_child_is_what_takes_it_out() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    let parked = grove.branch(region, Stem::new().at(at(0.0, 900.0, 200.0, 40.0)));
    grove.branch(parked, Panel::new().at(at(0.0, 0.0, 200.0, 200.0)));
    tick(&mut grove);
    assert_eq!(extent(&grove, region).height, 1100.0);

    grove.visible(parked, false);
    tick(&mut grove);
    assert_eq!(extent(&grove, region).height, 100.0);
}

/// Content scrolled past is not hidden. It still counts, which is exactly what makes scrolling back
/// to it work -- and what the old overload of `Visible` made impossible to say.
#[test]
fn a_child_scrolled_fully_out_of_sight_still_counts() {
    let mut grove = grove();
    let (region, content) = column(&mut grove);
    tick(&mut grove);

    wheel(&mut grove, (50.0, 50.0), (0.0, -300.0));
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 300.0);
    assert_eq!(extent(&grove, region).height, 400.0);
    assert_eq!(section(&grove, content).top(), -300.0);

    grove.scroll(region, ScrollTo::start());
    tick(&mut grove);
    assert_eq!(section(&grove, content).top(), 0.0);
}

/// Measured outward from the content origin and clamped at the near side: content above the origin
/// is a layout mistake, and the previous behaviour turned it into a feature.
#[test]
fn a_child_behind_the_origin_creates_no_backward_range() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(region, Panel::new().at(at(0.0, -400.0, 200.0, 100.0)));
    tick(&mut grove);

    assert_eq!(extent(&grove, region), Area::new(200.0, 100.0));
    // A drag the other way has nothing to move into either.
    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 90.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 0.0);
}

/// Never smaller than the region's own box, so a near-empty region has a range of zero rather than
/// a negative one.
#[test]
fn an_empty_view_has_zero_range() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    tick(&mut grove);

    assert_eq!(extent(&grove, region), Area::new(200.0, 100.0));
    grove.scroll(region, ScrollTo::end());
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 0.0);
}

/// Extent is recomputed from where the children landed, every frame, so a child that grew is a
/// region that reaches further with nothing written to say so.
#[test]
fn extent_tracks_a_child_that_grows() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    let child = grove.branch(region, Panel::new().at(at(0.0, 0.0, 200.0, 150.0)));
    tick(&mut grove);
    assert_eq!(extent(&grove, region).height, 150.0);

    grove.at(child, at(0.0, 0.0, 200.0, 500.0));
    tick(&mut grove);
    assert_eq!(extent(&grove, region).height, 500.0);
}

/// Including one that grew by wrapping. R2m measures the run before R3 asks how far the content
/// reaches, so the region is the right size on the frame the string changed.
#[test]
fn extent_tracks_a_child_that_grew_by_wrapping() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(
        region,
        // Five cells to a line, and a cell is twenty-two tall.
        Text::new("hello").at(Location::new().xs(
            left(0.px()).width(50.px()),
            top(0.px()).height(content()),
        )),
    );
    tick(&mut grove);
    assert_eq!(extent(&grove, region).height, 100.0);

    let run = grove.tap(region, Vein::Branches);
    let Some(Sap::Leaves(branches)) = run else {
        panic!("the region has a branch")
    };
    grove.text(branches[0], "hello world and everyone in it");
    tick(&mut grove);
    // Thirty characters at five cells a line, twenty-two tall each.
    assert_eq!(extent(&grove, region).height, 132.0);
}



// -- Pinned --------------------------------------------------------------------------------------

/// A header that stays at the top while the content slides under it. One declaration answers both
/// halves, so they cannot disagree.
#[test]
fn a_pinned_child_neither_moves_with_the_content_nor_counts_toward_it() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    let content = grove.branch(region, Panel::new().at(at(0.0, 0.0, 200.0, 400.0)));
    let header = grove.branch(region, Panel::new().at(at(0.0, 0.0, 200.0, 20.0)).pinned());
    tick(&mut grove);

    wheel(&mut grove, (50.0, 50.0), (0.0, -60.0));
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 60.0);
    assert_eq!(section(&grove, content).top(), -60.0);
    // It did not travel with the content.
    assert_eq!(section(&grove, header).top(), 0.0);
}

/// The other half of the one declaration, on its own: a pinned child that reaches far past the
/// region gives it nothing to scroll to.
#[test]
fn a_pinned_child_contributes_nothing_to_extent() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    let tall = grove.branch(region, Stem::new().at(at(0.0, 0.0, 200.0, 400.0)).pinned());
    // Its subtree goes with it, because a pinned element's children travel with the element and not
    // with the content.
    grove.branch(tall, Panel::new().at(at(0.0, 0.0, 200.0, 900.0)));
    tick(&mut grove);

    assert_eq!(extent(&grove, region).height, 100.0);
}

/// Pinning is relative to the region the element sits in, and says nothing about what contains that
/// region -- so a pinned header inside an inner region still travels when the page under it moves.
#[test]
fn a_pinned_child_still_travels_with_a_region_outside_its_own() {
    let mut grove = grove();
    let outer = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 200.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(outer, Panel::new().at(at(0.0, 0.0, 200.0, 600.0)));
    let inner = grove.branch(
        outer,
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    let content = grove.branch(inner, Panel::new().at(at(0.0, 0.0, 200.0, 300.0)));
    let header = grove.branch(inner, Panel::new().at(at(0.0, 0.0, 200.0, 20.0)).pinned());
    tick(&mut grove);

    grove.scroll(outer, ScrollTo::px(40.0));
    grove.scroll(inner, ScrollTo::px(30.0));
    tick(&mut grove);

    // Both offsets for what travels with both.
    assert_eq!(section(&grove, content).top(), -70.0);
    // The inner region's own is left out, and the outer one's is not.
    assert_eq!(section(&grove, header).top(), -40.0);
    assert_eq!(section(&grove, inner).top(), -40.0);
}

// -- Chain against contain -----------------------------------------------------------------------

/// Two regions, one inside the other, and the inner one owns its gesture outright. Reaching its
/// bottom and having the whole page lurch is the bug the declaration exists to prevent.
#[test]
fn a_contained_region_at_its_end_absorbs_the_drag() {
    let mut grove = grove();
    let outer = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 200.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(outer, Panel::new().at(at(0.0, 0.0, 200.0, 600.0)));
    let inner = grove.branch(
        outer,
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Scroll::new(Axes::Vertical).contain(Axes::Vertical)),
    );
    grove.branch(inner, Panel::new().at(at(0.0, 0.0, 200.0, 150.0)));
    tick(&mut grove);

    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 20.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, inner).y, 30.0);

    // Past its end, and on past it again. Nothing outside moves.
    drag(&mut grove, 50.0, -100.0);
    drag(&mut grove, 50.0, -300.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, inner).y, 50.0);
    assert_eq!(offset(&grove, outer).y, 0.0);
}

/// A wheel notch is the same question asked without a gesture, and gets the same answer.
#[test]
fn a_contained_region_absorbs_a_wheel_notch_too() {
    let mut grove = grove();
    let outer = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 200.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(outer, Panel::new().at(at(0.0, 0.0, 200.0, 600.0)));
    let inner = grove.branch(
        outer,
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Scroll::new(Axes::Vertical).contain(Axes::Vertical)),
    );
    grove.branch(inner, Panel::new().at(at(0.0, 0.0, 200.0, 150.0)));
    tick(&mut grove);

    wheel(&mut grove, (50.0, 50.0), (0.0, -400.0));
    wheel(&mut grove, (50.0, 50.0), (0.0, -400.0));
    tick(&mut grove);
    assert_eq!(offset(&grove, inner).y, 50.0);
    assert_eq!(offset(&grove, outer).y, 0.0);
}

/// The policy is per axis, so containing one leaves the other chaining. The two axes almost never
/// want the same answer, which is the whole reason the split is there.
#[test]
fn containing_one_axis_leaves_the_other_chaining() {
    let mut grove = grove();
    let outer = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 200.0))
            .scrolls(Axes::Both),
    );
    grove.branch(outer, Panel::new().at(at(0.0, 0.0, 600.0, 600.0)));
    let inner = grove.branch(
        outer,
        Stem::new()
            .at(at(0.0, 0.0, 100.0, 100.0))
            .scrolls(Scroll::new(Axes::Both).contain(Axes::Vertical)),
    );
    grove.branch(inner, Panel::new().at(at(0.0, 0.0, 150.0, 150.0)));
    tick(&mut grove);

    // Down: the inner one takes what it can, and then absorbs what it cannot.
    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, -100.0);
    drag(&mut grove, 50.0, -400.0);
    release(&mut grove, 50.0, -400.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, inner).y, 50.0);
    assert_eq!(offset(&grove, outer).y, 0.0);

    // Across: it hands the rest outward, because that is the axis it said nothing about.
    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, -100.0, 50.0);
    drag(&mut grove, -400.0, 50.0);
    release(&mut grove, -400.0, 50.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, inner).x, 50.0);
    assert!(offset(&grove, outer).x > 0.0);
}

// -- ScrollTo ------------------------------------------------------------------------------------

/// Every framing lands in the same unit, so two ways of saying the same distance are the same
/// distance. That was the thing the fraction-only write made impossible.
#[test]
fn px_and_fraction_land_on_the_same_place() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    tick(&mut grove);

    grove.scroll(region, ScrollTo::px(150.0));
    tick(&mut grove);
    let written = offset(&grove, region);
    assert_eq!(written.y, 150.0);

    grove.scroll(region, ScrollTo::start());
    tick(&mut grove);
    grove.scroll(region, ScrollTo::fraction(0.5));
    tick(&mut grove);
    assert_eq!(offset(&grove, region), written);
}

/// The two ends, named rather than computed.
#[test]
fn start_and_end_are_the_two_ends_of_the_range() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    tick(&mut grove);

    grove.scroll(region, ScrollTo::end());
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 300.0);

    grove.scroll(region, ScrollTo::start());
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 0.0);
}

/// Offset is logical pixels, read and written. Reading it back after writing it returns the pixels
/// it was written in, which is the whole of what one unit buys.
#[test]
fn the_offset_reads_back_as_the_pixels_it_was_written_in() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    tick(&mut grove);

    grove.scroll(region, ScrollTo::px(123.0));
    tick(&mut grove);
    assert_eq!(
        grove.tap(region, Vein::Offset),
        Some(Sap::Position(Position::new(0.0, 123.0)))
    );
    // And the derived reading a scrollbar takes, which is the one thing that is not in pixels.
    assert_eq!(
        grove.tap(region, Vein::Progress),
        Some(Sap::Progress(Position::new(0.0, 123.0 / 300.0)))
    );
}

/// The least distance that brings a descendant in, and no more than that.
#[test]
fn show_brings_a_descendant_just_into_view_and_no_further() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(region, Panel::new().at(at(0.0, 0.0, 200.0, 400.0)));
    let section = grove.branch(region, Panel::new().at(at(0.0, 250.0, 200.0, 50.0)));
    tick(&mut grove);

    grove.scroll(region, ScrollTo::show(section));
    tick(&mut grove);
    // Its far edge against the region's far edge, and not one pixel past.
    assert_eq!(offset(&grove, region).y, 200.0);

    // Coming back the other way stops with its near edge against the near edge.
    grove.scroll(region, ScrollTo::end());
    tick(&mut grove);
    grove.scroll(region, ScrollTo::show(section));
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 250.0);
}

/// A descendant already in view is a region that moves nowhere.
#[test]
fn show_moves_nothing_when_the_descendant_is_already_in_view() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(region, Panel::new().at(at(0.0, 0.0, 200.0, 400.0)));
    let near = grove.branch(region, Panel::new().at(at(0.0, 20.0, 200.0, 30.0)));
    tick(&mut grove);

    grove.scroll(region, ScrollTo::show(near));
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 0.0);
}

/// Bringing something into view means nothing unless the region is what it is inside, so an element
/// grown elsewhere is an op naming something it does not apply to.
#[test]
fn showing_an_element_grown_elsewhere_is_dropped() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    let elsewhere = grove.plant(Panel::new().at(at(0.0, 900.0, 200.0, 40.0)));
    tick(&mut grove);

    grove.scroll(region, ScrollTo::show(elsewhere));
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 0.0);
}

/// A distance along one axis is not a distance along the other, so a region scrolling both is asked
/// to say which -- and one that does not is refused rather than guessed at.
#[test]
fn a_distance_naming_no_axis_on_a_region_scrolling_both_is_dropped() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Both),
    );
    grove.branch(region, Panel::new().at(at(0.0, 0.0, 600.0, 400.0)));
    tick(&mut grove);

    grove.scroll(region, ScrollTo::px(120.0));
    tick(&mut grove);
    assert_eq!(offset(&grove, region), Position::default());

    // Named, and it moves the one axis it named.
    grove.scroll(region, ScrollTo::px(120.0).on(Axes::Vertical));
    tick(&mut grove);
    assert_eq!(offset(&grove, region), Position::new(0.0, 120.0));

    // A place rather than a distance needs no axis on any region, because it means both.
    grove.scroll(region, ScrollTo::end());
    tick(&mut grove);
    assert_eq!(offset(&grove, region), Position::new(400.0, 300.0));
}

/// Naming an axis the region does not scroll leaves the destination nothing to move, and the op
/// that carried it is dropped rather than doing nothing quietly.
#[test]
fn a_destination_on_an_axis_the_region_does_not_scroll_is_dropped() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    tick(&mut grove);

    grove.scroll(region, ScrollTo::end().on(Axes::Horizontal));
    tick(&mut grove);
    assert_eq!(offset(&grove, region), Position::default());
}

/// The destination is answered against the extent of the frame it lands in, not the one the last
/// frame left. Growing the content and asking for its end in one turn lands at the new end.
#[test]
fn a_destination_is_answered_against_the_extent_of_its_own_frame() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    tick(&mut grove);

    grove.branch(region, Panel::new().at(at(0.0, 0.0, 200.0, 900.0)));
    grove.scroll(region, ScrollTo::end());
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 800.0);
}

/// A region whose content shrank under it comes back into range rather than staying somewhere it
/// can no longer reach.
#[test]
fn a_region_whose_content_shrank_is_brought_back_into_range() {
    let mut grove = grove();
    let (region, content) = column(&mut grove);
    tick(&mut grove);

    grove.scroll(region, ScrollTo::end());
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 300.0);

    grove.at(content, at(0.0, 0.0, 200.0, 180.0));
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 80.0);
}

// -- Motion::Scroll ------------------------------------------------------------------------------

/// A smooth scroll is a motion like any other: it takes no time on the frame it starts, and it
/// lands exactly on its target rather than near it.
#[test]
fn a_scroll_motion_lands_exactly_on_its_destination() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    tick(&mut grove);

    grove.animate(region, Motion::Scroll(ScrollTo::end()), Timing::ms(100));
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 0.0);

    advance(&mut grove, 50);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 150.0);

    advance(&mut grove, 50);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 300.0);

    // And the frame after is identical, because the destination was written out where the motion
    // ended rather than left for something to settle.
    advance(&mut grove, 50);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 300.0);
}

/// Both ends are answered every frame, in that frame's context. A list that grows under a motion
/// toward its end still lands on the end.
#[test]
fn a_scroll_motion_re_resolves_its_destination_every_frame() {
    let mut grove = grove();
    let (region, content) = column(&mut grove);
    tick(&mut grove);

    grove.animate(region, Motion::Scroll(ScrollTo::end()), Timing::ms(100));
    tick(&mut grove);

    advance(&mut grove, 50);
    grove.at(content, at(0.0, 0.0, 200.0, 900.0));
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 400.0);

    advance(&mut grove, 50);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 800.0);
}

/// A drag is a write, so the reader taking hold of a region cancels the animation moving it. The
/// person wins, and the region is where the drag put it with nothing left over.
#[test]
fn a_drag_cancels_a_scroll_motion() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    tick(&mut grove);

    grove.animate(region, Motion::Scroll(ScrollTo::end()), Timing::ms(400));
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 75.0);

    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 30.0);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 95.0);

    // Nothing is still moving it.
    advance(&mut grove, 200);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 95.0);
}

/// And so is a `scroll` written directly, on the same terms as every other property.
#[test]
fn a_direct_scroll_cancels_a_scroll_motion() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    tick(&mut grove);

    grove.animate(region, Motion::Scroll(ScrollTo::end()), Timing::ms(400));
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 75.0);

    grove.scroll(region, ScrollTo::px(20.0));
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 20.0);

    advance(&mut grove, 400);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 20.0);
}

/// The one motion whose target is a statement about the element it names, so it is refused on the
/// same terms the verb is.
#[test]
fn a_scroll_motion_on_something_that_does_not_scroll_is_dropped() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new().at(at(0.0, 0.0, 200.0, 100.0)));
    grove.animate(leaf, Motion::Scroll(ScrollTo::end()), Timing::ms(100));
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(grove.tap(leaf, Vein::Offset), None);
}

// -- Momentum ------------------------------------------------------------------------------------

/// The decay is continuous, so a coast covers the same ground however often frames happen to run.
/// A fling that travelled further on a fast machine would be a different gesture on each one.
#[test]
fn a_coast_covers_the_same_ground_at_any_frame_rate() {
    let (once, left) = coasted(1000.0, 0.35, 0.1);
    let (mut stepped, mut speed) = (0.0, 1000.0);
    for _ in 0..10 {
        let (travelled, remaining) = coasted(speed, 0.35, 0.01);
        stepped += travelled;
        speed = remaining;
    }
    assert!((once - stepped).abs() < 0.001, "{once} against {stepped}");
    assert!((left - speed).abs() < 0.001, "{left} against {speed}");
}

/// A release with speed keeps the region moving, decaying until it settles -- and it settles, which
/// is what stops the loop being asked for frames forever.
#[test]
fn a_release_with_speed_coasts_on_and_settles() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    tick(&mut grove);

    advance(&mut grove, 100);
    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 40.0);
    release(&mut grove, 50.0, 40.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 10.0);
    assert!(!grove.coasting.idle());

    settle(&mut grove);
    let settled = offset(&grove, region).y;
    assert!(settled > 10.0 && settled < 300.0, "settled at {settled}");

    // And it stays there, with nothing left running.
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, settled);
}

/// A hand held still before it lifts is a hand that meant to stop. It rested for longer than the
/// window, so the movement before it is out of the measurement and there is no speed to carry.
#[test]
fn a_release_that_rested_before_lifting_starts_no_coast() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    tick(&mut grove);

    advance(&mut grove, 100);
    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 20.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 30.0);

    advance(&mut grove, 150);
    release(&mut grove, 50.0, 20.0);
    tick(&mut grove);
    assert!(grove.coasting.idle());
    assert_eq!(offset(&grove, region).y, 30.0);
}

/// The frame a release lands in carries almost nothing: a pointer reports no movement at all in the
/// frame its button came up. Measured from that frame alone the fling reads as a hand that stopped
/// and is thrown away entirely, so it is measured over the window instead and the fling carries.
#[test]
fn a_fling_survives_a_release_frame_that_moved_nothing() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    tick(&mut grove);

    advance(&mut grove, 16);
    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 20.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 30.0);

    advance(&mut grove, 16);
    release(&mut grove, 50.0, 20.0);
    tick(&mut grove);
    assert!(!grove.coasting.idle());

    settle(&mut grove);
    assert!(offset(&grove, region).y > 30.0);
}

/// The window is a mean over time and not a total, so a hand that hesitated before it lifted flings
/// at what it was doing across the window rather than at the fastest part of it. Both of these are
/// the same movement and both are inside the window; the one that hesitated is the slower gesture.
#[test]
fn a_fling_carries_the_mean_of_the_window() {
    let mut flicked = grove();
    let region = runway(&mut flicked);
    tick(&mut flicked);
    advance(&mut flicked, 16);
    press(&mut flicked, 50.0, 50.0);
    drag(&mut flicked, 50.0, 20.0);
    release(&mut flicked, 50.0, 20.0);
    tick(&mut flicked);
    settle(&mut flicked);

    let mut hesitated = grove();
    let same = runway(&mut hesitated);
    tick(&mut hesitated);
    advance(&mut hesitated, 16);
    press(&mut hesitated, 50.0, 50.0);
    drag(&mut hesitated, 50.0, 20.0);
    tick(&mut hesitated);
    for _ in 0..2 {
        advance(&mut hesitated, 16);
        tick(&mut hesitated);
    }
    advance(&mut hesitated, 16);
    release(&mut hesitated, 50.0, 20.0);
    tick(&mut hesitated);
    settle(&mut hesitated);

    // Three times the time for the same distance, so a third of the speed and a third of the ground
    // -- but still ground, because hesitating inside the window is not stopping.
    let (flicked, hesitated) = (offset(&flicked, region).y, offset(&hesitated, same).y);
    assert!(hesitated > 30.0, "the hesitant fling carried nothing");
    assert!(flicked > hesitated * 2.0, "{flicked} against {hesitated}");
}

/// A column with far more content than a fling can cross, so how far one was going is readable from
/// where it stopped rather than from the end it ran into.
fn runway(grove: &mut Grove) -> Leaf {
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(region, Panel::new().at(at(0.0, 0.0, 200.0, 4000.0)));
    region
}

/// A coast is clamped against the extent exactly as a drag is, and a region with nothing outside it
/// keeps what it could not use.
#[test]
fn a_coast_stops_at_the_end_of_the_region() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    tick(&mut grove);

    advance(&mut grove, 16);
    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 20.0);
    release(&mut grove, 50.0, 20.0);
    tick(&mut grove);

    settle(&mut grove);
    assert_eq!(offset(&grove, region).y, 300.0);
}

/// Reaching an end while coasting chains outward exactly as a drag would, because it is the same
/// question asked of the same region at the same place in its own extent.
#[test]
fn a_coast_at_an_end_hands_outward() {
    let mut grove = grove();
    let (outer, inner) = nested(&mut grove, Axes::Vertical.into());
    tick(&mut grove);

    advance(&mut grove, 16);
    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 20.0);
    release(&mut grove, 50.0, 20.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, inner).y, 30.0);
    assert_eq!(offset(&grove, outer).y, 0.0);

    settle(&mut grove);
    assert_eq!(offset(&grove, inner).y, 100.0);
    assert!(offset(&grove, outer).y > 0.0);
}

/// And a region that contains absorbs it, on the same terms.
#[test]
fn a_coast_in_a_contained_region_is_absorbed() {
    let mut grove = grove();
    let (outer, inner) = nested(
        &mut grove,
        Scroll::new(Axes::Vertical).contain(Axes::Vertical),
    );
    tick(&mut grove);

    advance(&mut grove, 16);
    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 20.0);
    release(&mut grove, 50.0, 20.0);
    tick(&mut grove);

    settle(&mut grove);
    assert_eq!(offset(&grove, inner).y, 100.0);
    assert_eq!(offset(&grove, outer).y, 0.0);
}

/// Catching a coasting list stops it where the hand met it. A coast is the reader's own last
/// gesture carrying on, so taking hold of it is how it is meant to end.
#[test]
fn taking_hold_of_a_coasting_region_stops_it() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    tick(&mut grove);

    advance(&mut grove, 16);
    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 20.0);
    release(&mut grove, 50.0, 20.0);
    tick(&mut grove);

    advance(&mut grove, 16);
    tick(&mut grove);
    let running = offset(&grove, region).y;
    assert!(running > 30.0);

    advance(&mut grove, 16);
    press(&mut grove, 50.0, 50.0);
    tick(&mut grove);
    assert!(grove.coasting.idle());
    assert_eq!(offset(&grove, region).y, running);
}

/// An outer region two hundred tall over an inner one a hundred tall, each with content of its own:
/// ranges of four hundred outside and a hundred inside.
fn nested(grove: &mut Grove, inner: Scroll) -> (Leaf, Leaf) {
    let outer = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 200.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(outer, Panel::new().at(at(0.0, 0.0, 200.0, 600.0)));
    let region = grove.branch(
        outer,
        Stem::new().at(at(0.0, 0.0, 200.0, 100.0)).scrolls(inner),
    );
    grove.branch(region, Panel::new().at(at(0.0, 0.0, 200.0, 200.0)));
    (outer, region)
}

/// A region reads its own three values and nothing else reads them, which is what keeps a
/// scrollbar's arithmetic on the app's side of the boundary in the one unit everything is in.
#[test]
fn only_a_region_has_an_offset_an_extent_or_a_progress() {
    let mut grove = grove();
    let (region, content) = column(&mut grove);
    tick(&mut grove);

    for vein in [Vein::Offset, Vein::Extent, Vein::Progress] {
        assert!(grove.tap(region, vein).is_some(), "{vein:?} on a region");
        assert_eq!(grove.tap(content, vein), None, "{vein:?} on plain content");
    }
    assert_eq!(
        grove.tap(region, Vein::Extent),
        Some(Sap::Area(Area::new(200.0, 400.0)))
    );
}

/// A region with nowhere to go reads no progress, which is the honest answer rather than a division
/// nobody can use.
#[test]
fn a_region_with_no_range_reads_no_progress() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    tick(&mut grove);
    assert_eq!(
        grove.tap(region, Vein::Progress),
        Some(Sap::Progress(Position::default()))
    );
}

/// A frame that took no time costs a coast neither distance nor speed. Without that, a run of them
/// -- which is what a suite advancing the clock by hand produces -- would settle a coast that had
/// not moved at all.
#[test]
fn a_frame_that_took_no_time_neither_moves_a_coast_nor_settles_it() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    tick(&mut grove);

    advance(&mut grove, 16);
    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 20.0);
    release(&mut grove, 50.0, 20.0);
    tick(&mut grove);

    tick(&mut grove);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 30.0);
    assert!(!grove.coasting.idle());

    advance(&mut grove, 16);
    tick(&mut grove);
    assert!(offset(&grove, region).y > 30.0);
}



// -- Floating over a region ----------------------------------------------------------------------

/// A region, a row inside it, and a menu the row opens. The menu is grown under the region and
/// positioned by the row, which is what an anchor is for.
fn menu(grove: &mut Grove, floats: bool) -> (Leaf, Leaf, Leaf) {
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(region, Panel::new().at(at(0.0, 0.0, 200.0, 300.0)));
    let row = grove.branch(region, Panel::new().at(at(0.0, 60.0, 100.0, 20.0)));
    let options = Panel::new()
        .anchored(row)
        .elevate(Elevation::up(10))
        .at(Location::new().xs(
            left(anchor().left()).width(120.px()),
            top(anchor().bottom()).height(120.px()),
        ));
    let options = match floats {
        true => grove.branch(region, options.floats(Escape::Region)),
        false => grove.branch(region, options),
    };
    (region, row, options)
}

/// Without the mark, the region cuts the menu off at its own edge -- which undoes the placement
/// that put the menu out there in the first place.
#[test]
fn an_element_positioned_outside_its_region_is_cut_off_at_the_region_s_edge() {
    let mut grove = grove();
    let (_, _, options) = menu(&mut grove, false);
    tick(&mut grove);

    let seen = section(&grove, options).intersect(grove.tree.clip(options));
    assert_eq!(seen, Section::from_edges(0.0, 80.0, 120.0, 100.0));
}

/// With it, the region clips it no further than whatever clips the region -- so the menu is whole.
#[test]
fn a_floating_element_is_not_clipped_by_the_region_it_is_grown_in() {
    let mut grove = grove();
    let (_, _, options) = menu(&mut grove, true);
    tick(&mut grove);

    let drawn = section(&grove, options);
    assert_eq!(drawn.intersect(grove.tree.clip(options)), drawn);
}

/// The other half of the one declaration: an overlay is not content, so it invents no room to
/// scroll to. Nothing is written to say so twice.
#[test]
fn a_floating_element_contributes_nothing_to_the_extent() {
    // A region with nothing in it but a row and the menu that row opens, so the menu is the only
    // thing that could be reaching anywhere.
    let opened = |floats: bool| {
        let mut grove = grove();
        let region = grove.plant(
            Stem::new()
                .at(at(0.0, 0.0, 200.0, 100.0))
                .scrolls(Axes::Vertical),
        );
        let row = grove.branch(region, Panel::new().at(at(0.0, 60.0, 100.0, 20.0)));
        let options = Panel::new().anchored(row).at(Location::new().xs(
            left(anchor().left()).width(120.px()),
            top(anchor().bottom()).height(400.px()),
        ));
        match floats {
            true => grove.branch(region, options.floats(Escape::Region)),
            false => grove.branch(region, options),
        };
        tick(&mut grove);
        extent(&grove, region).height
    };
    // Unmarked, the menu is read as content and invents four hundred pixels of room to scroll to.
    assert_eq!(opened(false), 480.0);
    // Marked, the region is as deep as it looks.
    assert_eq!(opened(true), 100.0);
}

/// What it keeps is the movement, which is the whole difference from `pinned`: a menu travels with
/// the row that opened it, and a header does not travel at all.
#[test]
fn a_floating_element_still_travels_with_the_content() {
    let mut grove = grove();
    let (region, row, options) = menu(&mut grove, true);
    tick(&mut grove);
    let (before_row, before_menu) = (section(&grove, row).top(), section(&grove, options).top());

    grove.scroll(region, ScrollTo::px(40.0));
    tick(&mut grove);
    assert_eq!(section(&grove, row).top(), before_row - 40.0);
    assert_eq!(section(&grove, options).top(), before_menu - 40.0);
}

/// Escaping is one region deep, never all of them. A menu inside a pane inside a sheet leaves the
/// pane and is still held by the sheet, which is what stops an overlay in a dialog painting over
/// the page behind it.
#[test]
fn a_floating_element_is_still_held_by_the_region_outside_its_own() {
    let mut grove = grove();
    let sheet = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 150.0))
            .scrolls(Axes::Vertical),
    );
    let pane = grove.branch(
        sheet,
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 60.0))
            .scrolls(Axes::Vertical),
    );
    let options = grove.branch(
        pane,
        Panel::new().floats(Escape::Region).at(at(0.0, 0.0, 100.0, 400.0)),
    );
    tick(&mut grove);

    // The pane's sixty is escaped; the sheet's hundred and fifty is not.
    assert_eq!(
        grove.tree.clip(options),
        Section::from_edges(0.0, 0.0, 200.0, 150.0)
    );
    // And it is not content of the pane either.
    assert_eq!(extent(&grove, pane).height, 60.0);
    // Nor does it turn up one region further out. It does not have to be excluded there a second
    // time: a region contributes its own box and never its content to whatever contains it, so
    // nothing inside the pane -- floating or not -- was ever going to reach the sheet.
    assert_eq!(extent(&grove, sheet).height, 150.0);
}

/// Three regions deep: a pane inside a list inside a sidebar, and a menu on something in the pane.
///
/// The three answers are genuinely different here, which is the whole reason the callsite states
/// which it wants rather than the engine picking one.
fn nested_regions(grove: &mut Grove, escape: Escape) -> (Leaf, Leaf, Leaf, Leaf) {
    let sidebar = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 150.0))
            .scrolls(Axes::Vertical),
    );
    let list = grove.branch(
        sidebar,
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 60.0))
            .scrolls(Axes::Vertical),
    );
    let pane = grove.branch(
        list,
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 30.0))
            .scrolls(Axes::Vertical),
    );
    let menu = grove.branch(
        pane,
        Panel::new()
            .floats(escape)
            .at(at(0.0, 0.0, 100.0, 400.0)),
    );
    (sidebar, list, pane, menu)
}

/// Out of the pane it is in, and held by the list -- which is still cutting it.
#[test]
fn escaping_the_region_leaves_one_and_is_held_by_the_next() {
    let mut grove = grove();
    let (_, _, _, menu) = nested_regions(&mut grove, Escape::Region);
    tick(&mut grove);
    assert_eq!(
        grove.tree.clip(menu),
        Section::from_edges(0.0, 0.0, 200.0, 60.0)
    );
}

/// Out of all three, held by nothing -- so the whole of it survives, and the clip is wider than the
/// outermost region rather than equal to it.
#[test]
fn escaping_to_the_surface_leaves_every_region() {
    let mut grove = grove();
    let (sidebar, _, _, menu) = nested_regions(&mut grove, Escape::Surface);
    tick(&mut grove);
    let drawn = section(&grove, menu);
    assert_eq!(drawn.intersect(grove.tree.clip(menu)), drawn);
    assert!(grove.tree.clip(menu).bottom() > section(&grove, sidebar).bottom());
}

/// Out of the pane and the list, and held by the sidebar. Neither of the other two says this, which
/// is why the option names an element instead of counting.
#[test]
fn escaping_within_a_named_element_leaves_everything_up_to_it() {
    let mut grove = grove();
    let sidebar = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 150.0))
            .scrolls(Axes::Vertical),
    );
    let list = grove.branch(
        sidebar,
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 60.0))
            .scrolls(Axes::Vertical),
    );
    let pane = grove.branch(
        list,
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 30.0))
            .scrolls(Axes::Vertical),
    );
    let menu = grove.branch(
        pane,
        Panel::new()
            .floats(Escape::Within(sidebar))
            .at(at(0.0, 0.0, 100.0, 400.0)),
    );
    tick(&mut grove);
    // Out of the pane and out of the list; the sidebar still holds it.
    assert_eq!(
        grove.tree.clip(menu),
        Section::from_edges(0.0, 0.0, 200.0, 150.0)
    );
}

/// Naming something that is not above it holds nothing, so it escapes the region it is in and no
/// further. An element leaves no more than it was told to, and a mistake is not permission.
#[test]
fn escaping_within_something_that_is_not_above_it_falls_back_to_its_own_region() {
    let mut grove = grove();
    let elsewhere = grove.plant(Stem::new().at(at(0.0, 0.0, 10.0, 10.0)));
    let sidebar = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 150.0))
            .scrolls(Axes::Vertical),
    );
    let list = grove.branch(
        sidebar,
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 60.0))
            .scrolls(Axes::Vertical),
    );
    let menu = grove.branch(
        list,
        Panel::new()
            .floats(Escape::Within(elsewhere))
            .at(at(0.0, 0.0, 100.0, 400.0)),
    );
    tick(&mut grove);
    // Out of the list, and no further: the sidebar holds it, exactly as `Escape::Region` would
    // have. A name that holds nothing is not permission to leave everything.
    assert_eq!(
        grove.tree.clip(menu),
        Section::from_edges(0.0, 0.0, 200.0, 150.0)
    );
}

/// A release does not hand the claim outward.
///
/// A release re-delivers the pointer's last position, so the move it carries has a delta of zero on
/// every axis. Reading that as "this region can consume no more" ended every gesture one region
/// further out than it was made in -- which nothing observed until a coast made the claim's final
/// resting place visible, because the region holding the claim is the region handed the speed.
#[test]
fn a_release_leaves_the_claim_with_the_region_that_had_it() {
    let mut grove = grove();
    let (outer, inner) = nested(&mut grove, Axes::Vertical.into());
    tick(&mut grove);

    advance(&mut grove, 16);
    press(&mut grove, 50.0, 50.0);
    // Well inside the inner region's own range, so it is still the one that can consume.
    drag(&mut grove, 50.0, 30.0);
    release(&mut grove, 50.0, 30.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, inner).y, 20.0);
    assert_eq!(offset(&grove, outer).y, 0.0);

    // One frame of the coast is enough: whichever region moves is the one that was holding it.
    advance(&mut grove, 16);
    tick(&mut grove);
    assert!(offset(&grove, inner).y > 20.0);
    assert_eq!(offset(&grove, outer).y, 0.0);
}

