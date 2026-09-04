//! The box stack and gesture claiming, through the frame.
//!
//! Every case here is written as a picture at a point: what is where, and what a press at one
//! coordinate reaches. That is deliberate -- the rule under test is that the answer follows from
//! geometry and two declarations, so a test that had to describe a walk would be testing something
//! else.

use crate::coordinate::{Area, Axes, Position, Section};
use crate::elm::Key;
use crate::tests::{
    Observer, advance, cancel, drag, grove, past_the_hold, press, release, section, tick, tick_with,
    wheel,
};
use crate::{
    Boxed,
    Elevation, Grove, Grow, Leaf, Location, Panel, Place, Pollen, Source, Stem, left, top,
};

/// A box at a stated place, so a point can be aimed at it.
fn at(x: f32, y: f32, width: f32, height: f32) -> Location {
    Location::new().xs(
        left(x.px()).width(width.px()),
        top(y.px()).height(height.px()),
    )
}

/// How far a region has been scrolled.
fn offset(grove: &Grove, leaf: Leaf) -> Position {
    grove.tree.offset(leaf)
}

/// What the last extraction told the backend to clip an element to.
fn clip(grove: &Grove, leaf: Leaf) -> Section {
    grove
        .elm
        .panels
        .written
        .iter()
        .find(|written| written.key == Key::from(leaf))
        .expect("the element was written this frame")
        .clip
}

/// One frame, with an app in it to read this frame's emissions.
fn frame(grove: &mut Grove) -> Pollen {
    let mut app = Observer::default();
    tick_with(grove, &mut app);
    app.last().clone()
}

/// A press and a release at one point, and what the frame they land in put out.
fn tap(grove: &mut Grove, x: f32, y: f32) -> Pollen {
    press(grove, x, y);
    release(grove, x, y);
    frame(grove)
}

// -- The stack -----------------------------------------------------------------------------------

/// The mark a composite states on its own decoration, and the whole reason it exists.
#[test]
fn a_pass_through_child_never_wins_a_tap() {
    let mut grove = grove();
    let button = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).interactive());
    let label = grove.branch(
        button,
        Panel::new()
            .at(at(10.0, 10.0, 80.0, 30.0))
            .interactive()
            .pass_through(),
    );
    tick(&mut grove);

    let pollen = tap(&mut grove, 50.0, 25.0);
    assert!(pollen.clicked(button));
    assert!(!pollen.clicked(label));
}

/// The same picture with the mark taken off. The child is drawn over its trunk, so it is the top of
/// the stack there and the press is its -- which is exactly what the mark exists to prevent.
#[test]
fn an_unmarked_child_wins_the_tap_instead() {
    let mut grove = grove();
    let button = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).interactive());
    let label = grove.branch(
        button,
        Panel::new().at(at(10.0, 10.0, 80.0, 30.0)).interactive(),
    );
    tick(&mut grove);

    let pollen = tap(&mut grove, 50.0, 25.0);
    assert!(pollen.clicked(label));
    assert!(!pollen.clicked(button));
}

/// The hit test reads the top of the stack and stops. It does not continue downward looking for
/// something willing to take the gesture, so an element that declared nothing still eats one.
#[test]
fn an_undeclared_element_at_the_top_eats_the_gesture() {
    let mut grove = grove();
    let button = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).interactive());
    // Planted later at the same elevation, so it is in front where the two overlap.
    grove.plant(Panel::new().at(at(0.0, 0.0, 200.0, 100.0)));
    tick(&mut grove);

    let pollen = tap(&mut grove, 50.0, 25.0);
    assert!(!pollen.clicked(button));
    assert!(!pollen.engaged(button));
}

/// There is no such thing as an obscured element, only an element that is below another *at this
/// point*.
#[test]
fn a_partly_covered_target_is_pressable_where_it_is_not_covered() {
    let mut grove = grove();
    let button = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).interactive());
    grove.plant(Panel::new().at(at(50.0, 0.0, 50.0, 50.0)));
    tick(&mut grove);

    assert!(tap(&mut grove, 25.0, 25.0).clicked(button));
    assert!(!tap(&mut grove, 75.0, 25.0).clicked(button));
}

/// The same covering panel, marked. It is still in the stack -- a drag over it still finds what
/// contains it -- but it is never the top of one.
#[test]
fn a_target_covered_by_a_pass_through_element_is_pressable_through_it() {
    let mut grove = grove();
    let button = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).interactive());
    grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).pass_through());
    tick(&mut grove);

    assert!(tap(&mut grove, 50.0, 25.0).clicked(button));
}

/// An element that says nothing about where it sits fills the surface, which is what a backdrop is.
#[test]
fn a_full_screen_target_blocks_everything_beneath_it() {
    let mut grove = grove();
    let button = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).interactive());
    let backdrop = grove.plant(Panel::new().interactive().elevate(Elevation::up(1)));
    tick(&mut grove);

    let pollen = tap(&mut grove, 50.0, 25.0);
    assert!(pollen.clicked(backdrop));
    assert!(!pollen.clicked(button));
}

/// Fully transparent is not there. Anything above zero is.
#[test]
fn a_fully_transparent_element_is_not_in_the_stack() {
    let mut grove = grove();
    let button = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).interactive());
    let veil = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).opacity(0.0));
    tick(&mut grove);
    assert!(tap(&mut grove, 50.0, 25.0).clicked(button));

    grove.opacity(veil, 0.01);
    tick(&mut grove);
    assert!(!tap(&mut grove, 50.0, 25.0).clicked(button));
}

/// Hiding takes an element out of the stack the same way, and for the same reason: it is not there.
#[test]
fn a_hidden_element_is_not_in_the_stack() {
    let mut grove = grove();
    let button = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).interactive());
    let veil = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).visible(false));
    tick(&mut grove);
    assert!(tap(&mut grove, 50.0, 25.0).clicked(button));

    grove.visible(veil, true);
    tick(&mut grove);
    assert!(!tap(&mut grove, 50.0, 25.0).clicked(button));
}

/// A round control does not take presses in the square corners it does not draw.
#[test]
fn a_round_hit_area_excludes_the_corners() {
    let mut grove = grove();
    let dot = grove.plant(
        Panel::new()
            .at(at(0.0, 0.0, 100.0, 100.0))
            .interactive()
            .round_hit_area(),
    );
    tick(&mut grove);

    assert!(tap(&mut grove, 50.0, 50.0).clicked(dot));
    assert!(!tap(&mut grove, 2.0, 2.0).clicked(dot));
}

// -- Gestures ------------------------------------------------------------------------------------

/// A press and a release with nothing in between is a tap, and it is announced once.
#[test]
fn a_press_and_release_without_movement_is_a_tap() {
    let mut grove = grove();
    let button = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).interactive());
    tick(&mut grove);

    press(&mut grove, 50.0, 25.0);
    let pressed = frame(&mut grove);
    assert!(pressed.engaged(button));
    assert!(!pressed.clicked(button));

    release(&mut grove, 50.0, 25.0);
    let released = frame(&mut grove);
    assert!(released.clicked(button));
    assert!(released.disengaged(button));
}

/// Taken away rather than finished. Nothing was issued early, so there is nothing to retract --
/// there is simply no tap.
#[test]
fn a_cancelled_gesture_is_not_a_tap() {
    let mut grove = grove();
    let button = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).interactive());
    tick(&mut grove);

    press(&mut grove, 50.0, 25.0);
    cancel(&mut grove);
    let pollen = frame(&mut grove);
    assert!(!pollen.clicked(button));
    assert!(pollen.disengaged(button));
}

/// The threshold decides the kind of the gesture, and it is per axis: the same travel that is a
/// drag downward is still a pending tap across.
#[test]
fn the_claim_threshold_is_per_axis() {
    let mut grove = grove();
    let slider = grove.plant(
        Panel::new()
            .at(at(0.0, 0.0, 200.0, 200.0))
            .interactive()
            .drags(Axes::Both),
    );
    tick(&mut grove);
    let claim = grove.claim;

    press(&mut grove, 100.0, 100.0);
    drag(&mut grove, 100.0 + claim.vertical, 100.0);
    let across = frame(&mut grove);
    assert!(!across.drag_started(slider));

    drag(&mut grove, 100.0, 100.0 + claim.vertical);
    let down = frame(&mut grove);
    assert!(down.drag_started(slider));
}

// -- Scrolling regions ---------------------------------------------------------------------------

/// A column of content taller than the box it is seen through.
fn column(grove: &mut Grove) -> (Leaf, Leaf) {
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    let content = grove.branch(region, Panel::new().at(at(0.0, 0.0, 200.0, 400.0)));
    (region, content)
}

/// Scroll ownership is structural. The decoration asked for nothing, and on touch a drag is the
/// only way to scroll at all.
#[test]
fn a_drag_on_decoration_scrolls_the_region_containing_it() {
    let mut grove = grove();
    let (region, content) = column(&mut grove);
    tick(&mut grove);

    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 20.0);
    tick(&mut grove);

    assert_eq!(offset(&grove, region).y, 30.0);
    assert_eq!(section(&grove, content).top(), -30.0);
}

/// The mobile-correct default: a target holds a gesture only until it becomes a drag, and then
/// yields -- so the list scrolls and the button it began on gets nothing.
#[test]
fn a_drag_on_a_button_scrolls_the_region_and_the_button_gets_no_tap() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    let button = grove.branch(
        region,
        Panel::new().at(at(0.0, 0.0, 200.0, 50.0)).interactive(),
    );
    tick(&mut grove);

    press(&mut grove, 50.0, 25.0);
    let pressed = frame(&mut grove);
    assert!(pressed.engaged(button));

    drag(&mut grove, 50.0, -15.0);
    release(&mut grove, 50.0, -15.0);
    let dragged = frame(&mut grove);

    assert_eq!(offset(&grove, region).y, 40.0);
    assert!(!dragged.clicked(button));
    // It let the gesture go the moment the gesture became a drag, which is where a pressed visual
    // is put back.
    assert!(dragged.disengaged(button));
    assert!(!dragged.drag_started(button));
}

/// The same button, pressed and released where it was. Nothing about the region changed, and the
/// tap is not a special case of it.
#[test]
fn a_press_and_release_on_a_button_in_a_region_is_a_tap() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    let button = grove.branch(
        region,
        Panel::new().at(at(0.0, 0.0, 200.0, 50.0)).interactive(),
    );
    tick(&mut grove);

    assert!(tap(&mut grove, 50.0, 25.0).clicked(button));
    assert_eq!(offset(&grove, region).y, 0.0);
}

/// A drag along the axis a target declared belongs to the target.
#[test]
fn a_drag_along_a_slider_moves_the_slider_and_not_the_column() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    let slider = grove.branch(
        region,
        Panel::new()
            .at(at(0.0, 0.0, 200.0, 50.0))
            .interactive()
            .drags(Axes::Horizontal),
    );
    tick(&mut grove);

    press(&mut grove, 50.0, 25.0);
    drag(&mut grove, 120.0, 25.0);
    let pollen = frame(&mut grove);

    assert!(pollen.drag_started(slider));
    let drag = pollen.dragged(slider).expect("the slider is being dragged");
    assert_eq!(drag.start, Position::new(50.0, 25.0));
    assert_eq!(drag.current, Position::new(120.0, 25.0));
    assert_eq!(offset(&grove, region).y, 0.0);
}

/// The same slider, dragged along the axis it did not declare. Nothing at spawn time knew which of
/// these the gesture would be, and nothing had to.
#[test]
fn a_drag_down_the_same_slider_moves_the_column_and_not_the_slider() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    let slider = grove.branch(
        region,
        Panel::new()
            .at(at(0.0, 0.0, 200.0, 50.0))
            .interactive()
            .drags(Axes::Horizontal),
    );
    tick(&mut grove);

    press(&mut grove, 50.0, 25.0);
    // Along the axis the column scrolls. Toward the near edge, because a column sitting at its
    // start has nowhere to go the other way.
    drag(&mut grove, 50.0, -35.0);
    let pollen = frame(&mut grove);

    assert!(!pollen.drag_started(slider));
    assert_eq!(pollen.dragged(slider), None);
    assert!(pollen.disengaged(slider));
    assert_eq!(offset(&grove, region).y, 60.0);
}

/// A wheel notch is not a gesture: it moves what is under it and it is over. It finds its region
/// the same way a drag does.
#[test]
fn a_wheel_notch_scrolls_the_region_under_it() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    tick(&mut grove);

    wheel(&mut grove, (50.0, 50.0), (0.0, -40.0));
    tick(&mut grove);

    assert_eq!(offset(&grove, region).y, 40.0);
}

/// A region cannot be moved along an axis it did not declare, and the gesture does not become
/// something else because of it.
#[test]
fn an_undeclared_axis_does_not_scroll() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(region, Panel::new().at(at(0.0, 0.0, 600.0, 400.0)));
    tick(&mut grove);

    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, -50.0, 50.0);
    tick(&mut grove);

    assert_eq!(offset(&grove, region).x, 0.0);
}

/// Two regions, one inside the other. The inner one takes the drag until it cannot, and then hands
/// it outward without the gesture ending.
#[test]
fn a_region_at_its_extent_hands_a_continuing_drag_outward() {
    let mut grove = grove();
    let outer = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 200.0))
            .scrolls(Axes::Vertical),
    );
    // Each region's own content is grown with it, so what is nearest the viewer at a point is what
    // is innermost there.
    grove.branch(outer, Panel::new().at(at(0.0, 0.0, 200.0, 600.0)));
    let inner = grove.branch(
        outer,
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(inner, Panel::new().at(at(0.0, 0.0, 200.0, 150.0)));
    tick(&mut grove);

    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 20.0);
    tick(&mut grove);
    // The inner region's own range is 150 less the 100 it is seen through.
    assert_eq!(offset(&grove, inner).y, 30.0);
    assert_eq!(offset(&grove, outer).y, 0.0);

    drag(&mut grove, 50.0, -60.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, inner).y, 50.0);
    assert_eq!(offset(&grove, outer).y, 0.0);

    // Mid-gesture, with no second press: the inner one can no longer consume, so the claim passed
    // outward.
    drag(&mut grove, 50.0, -100.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, inner).y, 50.0);
    assert_eq!(offset(&grove, outer).y, 40.0);
}

/// Three deep. Each one is handed the gesture only once the one inside it is done with it, and the
/// claim never travels back inward.
#[test]
fn nested_regions_hand_off_outward_in_order() {
    let mut grove = grove();
    let outer = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 200.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(outer, Panel::new().at(at(0.0, 0.0, 200.0, 260.0)));
    let middle = grove.branch(
        outer,
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(middle, Panel::new().at(at(0.0, 0.0, 200.0, 130.0)));
    let inner = grove.branch(
        middle,
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 60.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(inner, Panel::new().at(at(0.0, 0.0, 200.0, 80.0)));
    tick(&mut grove);

    // Ranges: 20 innermost, 30 in the middle, 60 outermost.
    press(&mut grove, 50.0, 30.0);
    drag(&mut grove, 50.0, 0.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, inner).y, 20.0);
    assert_eq!(offset(&grove, middle).y, 0.0);
    assert_eq!(offset(&grove, outer).y, 0.0);

    // The innermost can take no more, so the next one out has it -- and only the next one, because
    // the claim passes one region at a time rather than spilling through the whole chain.
    drag(&mut grove, 50.0, -100.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, inner).y, 20.0);
    assert_eq!(offset(&grove, middle).y, 30.0);
    assert_eq!(offset(&grove, outer).y, 0.0);

    drag(&mut grove, 50.0, -200.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, inner).y, 20.0);
    assert_eq!(offset(&grove, middle).y, 30.0);
    assert_eq!(offset(&grove, outer).y, 60.0);
}

/// The claim is not handed back inward. Once the outer region has it, reversing scrolls the outer
/// one -- which is what keeps a drag from stuttering between two regions as a hand wanders.
#[test]
fn a_reversed_drag_stays_with_the_region_that_took_it() {
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
    grove.branch(inner, Panel::new().at(at(0.0, 0.0, 200.0, 150.0)));
    tick(&mut grove);

    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, -60.0);
    drag(&mut grove, 50.0, -160.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, inner).y, 50.0);
    assert_eq!(offset(&grove, outer).y, 100.0);

    drag(&mut grove, 50.0, -120.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, inner).y, 50.0);
    assert_eq!(offset(&grove, outer).y, 60.0);
}

/// Extent comes from where the children landed, and is clamped at the near side and at the
/// region's own box: content behind the origin creates no range to scroll back into, and an empty
/// region has none at all.
#[test]
fn an_empty_region_has_no_range() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    grove.branch(region, Panel::new().at(at(0.0, -400.0, 200.0, 100.0)));
    tick(&mut grove);

    assert_eq!(grove.tree.extent(region), Area::new(200.0, 100.0));
    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 0.0);
    tick(&mut grove);
    assert_eq!(offset(&grove, region).y, 0.0);
}

/// Hiding a child removes it from the extent, and its subtree with it. Scrolled-out content is not
/// hidden and still counts, which is what makes scrolling back to it work.
#[test]
fn a_hidden_child_contributes_nothing_to_extent() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    let holder = grove.branch(region, Stem::new().at(at(0.0, 0.0, 200.0, 50.0)));
    grove.branch(holder, Panel::new().at(at(0.0, 0.0, 200.0, 400.0)));
    tick(&mut grove);
    assert_eq!(grove.tree.extent(region).height, 400.0);

    grove.visible(holder, false);
    tick(&mut grove);
    assert_eq!(grove.tree.extent(region).height, 100.0);
}

/// An element scrolled out of its region is absent from the batch and unchanged in every other
/// respect, so nothing has to be undone to scroll back to it.
#[test]
fn an_element_scrolled_out_of_its_region_is_not_in_the_stack() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    let far = grove.branch(
        region,
        Panel::new().at(at(0.0, 200.0, 200.0, 100.0)).interactive(),
    );
    tick(&mut grove);
    assert_eq!(grove.elm.panels.len(), 0);

    wheel(&mut grove, (50.0, 50.0), (0.0, -200.0));
    tick(&mut grove);
    assert_eq!(grove.elm.panels.len(), 1);
    assert!(tap(&mut grove, 50.0, 50.0).clicked(far));
}

// -- A press that was held -----------------------------------------------------------------------

/// The lifecycle's other threshold is a distance, and a distance cannot tell a gesture that is
/// sitting still from one that has not moved yet. This is the one that is a duration.
#[test]
fn a_press_held_past_the_duration_is_reported_as_held() {
    let mut grove = grove();
    let button = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).interactive());
    tick(&mut grove);

    press(&mut grove, 50.0, 25.0);
    let pressed = frame(&mut grove);
    assert!(pressed.engaged(button));
    assert!(!pressed.held(button));

    past_the_hold(&mut grove);
    let held = frame(&mut grove);
    assert!(held.held(button));
    // Where the press landed, which is where a menu opens and where a selection begins.
    assert_eq!(held.held_at(button), Some(Position::new(50.0, 25.0)));
    // Once. The gesture has left resolving, so however much longer the press goes on there is
    // nothing further for it to become.
    past_the_hold(&mut grove);
    assert!(!frame(&mut grove).held(button));
}

/// The drag out of a hold is claimed by whoever took the hold. The button declares no drags at all
/// -- a drag on it without the hold scrolls the column and it gets nothing -- and it still takes
/// this one, down the very axis the column scrolls.
#[test]
fn a_drag_out_of_a_hold_is_claimed_by_whoever_took_it() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    let button = grove.branch(
        region,
        Panel::new().at(at(0.0, 0.0, 200.0, 50.0)).interactive(),
    );
    tick(&mut grove);

    press(&mut grove, 50.0, 25.0);
    frame(&mut grove);
    past_the_hold(&mut grove);
    assert!(frame(&mut grove).held(button));

    drag(&mut grove, 50.0, -35.0);
    let dragged = frame(&mut grove);
    assert!(dragged.drag_started(button));
    let held = dragged.dragged(button).expect("the hold took the drag");
    assert_eq!(held.start, Position::new(50.0, 25.0));
    assert_eq!(held.current, Position::new(50.0, -35.0));
    assert_eq!(offset(&grove, region).y, 0.0);
}

/// A press reported as held has already stopped being a gesture that could end as a tap, so nothing
/// is issued when it ends -- and, a tap being the only thing that moves focus, nothing is focused
/// either.
#[test]
fn a_press_held_past_the_duration_and_released_is_not_a_tap() {
    let mut grove = grove();
    let button = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).interactive());
    tick(&mut grove);

    press(&mut grove, 50.0, 25.0);
    frame(&mut grove);
    past_the_hold(&mut grove);
    assert!(frame(&mut grove).held(button));

    release(&mut grove, 50.0, 25.0);
    let lifted = frame(&mut grove);
    assert!(!lifted.clicked(button));
    assert!(lifted.disengaged(button));
    assert_eq!(grove.focused(), None);
}

/// A press that is not moving is the one gesture that changes with nothing arriving to change it,
/// so the frames that would notice it are owed (F9). Idling under a finger is how the duration
/// would pass unremarked, and nothing else in the engine would be any the wiser.
#[test]
fn a_press_that_could_still_be_held_owes_a_frame() {
    let mut grove = grove();
    let button = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).interactive());
    tick(&mut grove);

    press(&mut grove, 50.0, 25.0);
    frame(&mut grove);
    assert!(grove.incoming.awaiting_hold(), "the press is on its way");

    past_the_hold(&mut grove);
    assert!(frame(&mut grove).held(button));
    assert!(
        !grove.incoming.awaiting_hold(),
        "it arrived, and there is nothing left to wait for"
    );
}

/// A press on nothing that receives has nobody to be a hold's fact about, so it stays what it was.
/// Holding still on a backdrop and lifting is the slow version of tapping one, and it dismisses.
#[test]
fn a_press_on_nothing_that_receives_is_never_held() {
    let mut grove = grove();
    let button = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).interactive());
    let backdrop = grove.plant(Panel::new().elevate(Elevation::up(1)));
    grove.focus(button);
    tick(&mut grove);
    assert_eq!(grove.focused(), Some(button));

    press(&mut grove, 300.0, 200.0);
    frame(&mut grove);
    assert!(!grove.incoming.awaiting_hold());
    past_the_hold(&mut grove);
    let waited = frame(&mut grove);
    assert!(!waited.held(backdrop));

    release(&mut grove, 300.0, 200.0);
    frame(&mut grove);
    assert_eq!(grove.focused(), None);
}

// -- Disabled ------------------------------------------------------------------------------------

/// Disabled is present and inert. It blocks, which is the whole difference between a disabled
/// control and decoration -- and what makes disabling a page enough on its own.
#[test]
fn a_disabled_element_swallows_the_gesture() {
    let mut grove = grove();
    let button = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 50.0)).interactive());
    let sheet = grove.plant(Panel::new().at(at(0.0, 0.0, 200.0, 100.0)).interactive());
    grove.disable(sheet);
    tick(&mut grove);

    let pollen = tap(&mut grove, 50.0, 25.0);
    assert!(!pollen.clicked(sheet));
    assert!(!pollen.clicked(button));
    assert!(!pollen.engaged(button));
}

/// Swallowing covers every kind of input uniformly. A disabled region does not scroll, and the
/// gesture does not pass outward to one that would.
#[test]
fn a_disabled_region_neither_scrolls_nor_hands_outward() {
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
    grove.branch(inner, Panel::new().at(at(0.0, 0.0, 200.0, 400.0)));
    grove.disable(inner);
    tick(&mut grove);

    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, -50.0);
    tick(&mut grove);

    assert_eq!(offset(&grove, inner).y, 0.0);
    assert_eq!(offset(&grove, outer).y, 0.0);
}

/// What extraction hands the backend to scissor with. An element inside a region is clipped to it;
/// one with no scrolling ancestor is clipped to the surface, which scissors to the whole of it.
#[test]
fn an_element_in_a_region_is_extracted_with_that_region_s_clip() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    let inside = grove.branch(region, Panel::new().at(at(0.0, 0.0, 200.0, 60.0)));
    let outside = grove.plant(Panel::new().at(at(0.0, 0.0, 40.0, 40.0)));
    tick(&mut grove);

    assert_eq!(
        clip(&grove, inside),
        Section::from_edges(0.0, 0.0, 200.0, 100.0)
    );
    assert_eq!(
        clip(&grove, outside),
        Section::new(Position::default(), grove.viewport())
    );
}

/// A clip that changed is a change like any other: the element is written again, so the backend
/// never goes on scissoring to a region that has moved.
#[test]
fn a_moved_region_rewrites_what_its_children_are_clipped_to() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    let inside = grove.branch(region, Panel::new().at(at(0.0, 0.0, 200.0, 60.0)));
    tick(&mut grove);

    grove.at(region, at(0.0, 40.0, 200.0, 100.0));
    tick(&mut grove);
    assert_eq!(
        clip(&grove, inside),
        Section::from_edges(0.0, 40.0, 200.0, 140.0)
    );
}

/// A press that catches a running coast is spent on catching it. Stopping a moving list is a thing
/// the gesture did, and the element it happened to stop over hears nothing of it -- a reader
/// reaching for a list that is still going did not mean to choose what was under their hand when it
/// halted.
#[test]
fn a_press_that_catches_a_coast_is_not_a_tap() {
    let mut grove = grove();
    let region = grove.plant(
        Stem::new()
            .at(at(0.0, 0.0, 200.0, 100.0))
            .scrolls(Axes::Vertical),
    );
    let button = grove.branch(
        region,
        Panel::new().at(at(0.0, 0.0, 200.0, 400.0)).interactive(),
    );
    tick(&mut grove);

    advance(&mut grove, 16);
    press(&mut grove, 50.0, 50.0);
    drag(&mut grove, 50.0, 20.0);
    release(&mut grove, 50.0, 20.0);
    frame(&mut grove);
    assert!(!grove.coasting.idle());

    advance(&mut grove, 16);
    press(&mut grove, 50.0, 50.0);
    let caught = frame(&mut grove);
    advance(&mut grove, 16);
    release(&mut grove, 50.0, 50.0);
    let lifted = frame(&mut grove);
    assert!(grove.coasting.idle());
    // Nothing at all: no pressed visual to put back, and so no tap to take back either.
    assert!(!caught.engaged(button));
    assert!(!lifted.clicked(button));

    // Settled, the same press is an ordinary tap. What the catch is about is what the press did,
    // not where it landed.
    assert!(tap(&mut grove, 50.0, 50.0).clicked(button));
}
