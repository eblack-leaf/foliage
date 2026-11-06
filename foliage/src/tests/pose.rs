//! Pose: an element turned, scaled or slid where it is drawn, and everything under it with it.

use core::f32::consts::FRAC_PI_2;

use crate::coordinate::{Position, Section};
use crate::pose::Affine;
use crate::tests::{Observer, advance, grove, press, release, section, tick, tick_with};
use crate::{
    Boxed, Grove, Grow, Leaf, Location, Motion, Panel, Place, Pollen, Source, Stem, Timing, left,
    top,
};

fn at(x: f32, y: f32, width: f32, height: f32) -> Location {
    Location::new().xs(
        left(x.px()).width(width.px()),
        top(y.px()).height(height.px()),
    )
}

/// A press and a release at one point, and what the frame they land in put out.
fn tap(grove: &mut Grove, x: f32, y: f32) -> Pollen {
    press(grove, x, y);
    release(grove, x, y);
    let mut app = Observer::default();
    tick_with(grove, &mut app);
    app.last().clone()
}

/// How the backend was last told to draw a panel.
fn drawn_through(grove: &Grove, leaf: Leaf) -> Affine {
    grove
        .elm
        .panels
        .holding(leaf)
        .expect("the backend is holding this panel")
        .pose
}

/// A bar turned a quarter about its middle stands upright, and is pressed where it is drawn:
/// above and below its middle, and not at its old ends.
#[test]
fn a_turned_bar_is_pressed_where_it_is_drawn() {
    let mut grove = grove();
    // 100 across and 20 down, its middle at (100, 100).
    let bar = grove.plant(
        Panel::new()
            .interactive()
            .turn(FRAC_PI_2)
            .at(at(50.0, 90.0, 100.0, 20.0)),
    );
    tick(&mut grove);
    assert!(
        tap(&mut grove, 100.0, 140.0).clicked(bar),
        "below the middle, where it now reaches"
    );
    crate::tests::past_the_taps(&mut grove);
    assert!(
        !tap(&mut grove, 140.0, 100.0).clicked(bar),
        "at its old end, where it no longer is"
    );
}

/// A tap's place within what it landed on is in the element's own upright space, so a control
/// that turned still knows which end was pressed.
#[test]
fn a_tap_within_a_turned_element_is_in_its_own_space() {
    let mut grove = grove();
    let bar = grove.plant(
        Panel::new()
            .interactive()
            .turn(FRAC_PI_2)
            .at(at(50.0, 90.0, 100.0, 20.0)),
    );
    tick(&mut grove);
    // Turned clockwise, its left end now points up: 30 above the middle is 20 in from that end.
    let pollen = tap(&mut grove, 100.0, 70.0);
    let within = pollen.clicked_within(bar).expect("tapped");
    assert!(
        (within.x - 20.0).abs() < 0.01 && (within.y - 10.0).abs() < 0.01,
        "{within:?}"
    );
}

/// A trunk's scale carries everything under it, about the trunk's own middle -- and an element's
/// own turn is about its own middle, inside that.
#[test]
fn a_scaled_trunk_scales_what_is_under_it() {
    let mut grove = grove();
    let card = grove.plant(Stem::new().scale(2.0).at(at(0.0, 0.0, 100.0, 100.0)));
    let face = grove.branch(card, Panel::new().at(at(60.0, 40.0, 20.0, 20.0)));
    tick(&mut grove);
    let through = drawn_through(&grove, face);
    // The face's corner, 10 right and 10 up of the card's middle, lands twice as far out.
    let corner = through.apply(Position::new(60.0, 40.0));
    assert!(
        (corner.x - 70.0).abs() < 1e-3 && (corner.y - 30.0).abs() < 1e-3,
        "{corner:?}"
    );
    // What it reads as its box is still where the layout put it.
    assert_eq!(
        section(&grove, face),
        Section::from_edges(60.0, 40.0, 80.0, 60.0)
    );
}

/// A slide is a shift in the element's own size: one of its widths across, whatever that came to.
#[test]
fn a_slide_is_measured_in_the_element_itself() {
    let mut grove = grove();
    let sheet = grove.plant(Panel::new().at(at(10.0, 10.0, 120.0, 40.0)));
    let child = grove.branch(sheet, Panel::new().at(at(10.0, 10.0, 10.0, 10.0)));
    tick(&mut grove);
    grove.slide(sheet, Position::new(1.0, -0.5));
    tick(&mut grove);
    assert_eq!(
        section(&grove, sheet),
        Section::from_edges(130.0, -10.0, 250.0, 30.0)
    );
    // Carried with it by the sheet's width, as a shift is -- not by its own.
    assert_eq!(section(&grove, child).left(), 140.0);
    // And it adds to a shift.
    grove.shift(sheet, Position::new(5.0, 0.0));
    tick(&mut grove);
    assert_eq!(section(&grove, sheet).left(), 135.0);
}

/// Each part of a pose moves over time and lands on what it was told, written back as it goes.
#[test]
fn a_pose_moves_and_lands() {
    let mut grove = grove();
    let card = grove.plant(Panel::new().at(at(0.0, 0.0, 100.0, 100.0)));
    tick(&mut grove);
    grove.animate(card, Motion::Scale(0.5), Timing::ms(100));
    grove.animate(card, Motion::Turn(1.0), Timing::ms(100));
    grove.animate(
        card,
        Motion::Slide(Position::new(1.0, 0.0)),
        Timing::ms(100),
    );
    tick(&mut grove);
    advance(&mut grove, 50);
    tick(&mut grove);
    let midway = grove.tree.pose(card);
    assert!((midway.scale - 0.75).abs() < 1e-4, "{midway:?}");
    assert!((midway.turn - 0.5).abs() < 1e-4);
    assert!((midway.slide.x - 0.5).abs() < 1e-4);
    advance(&mut grove, 100);
    tick(&mut grove);
    let landed = grove.tree.pose(card);
    assert_eq!((landed.scale, landed.turn, landed.slide.x), (0.5, 1.0, 1.0));

    // A direct write cancels what is moving it.
    grove.animate(card, Motion::Scale(2.0), Timing::ms(100));
    tick(&mut grove);
    grove.scale(card, 1.0);
    advance(&mut grove, 50);
    tick(&mut grove);
    assert_eq!(grove.tree.pose(card).scale, 1.0);
}

/// Scaled to nothing, there is nothing to press.
#[test]
fn an_element_scaled_to_nothing_takes_no_press() {
    let mut grove = grove();
    let button = grove.plant(
        Panel::new()
            .interactive()
            .scale(0.0)
            .at(at(0.0, 0.0, 50.0, 50.0)),
    );
    tick(&mut grove);
    assert!(!tap(&mut grove, 25.0, 25.0).clicked(button));
}

/// What is culled is decided by where it lands once posed: a box off the window whose scale
/// reaches back onto it is drawn.
#[test]
fn what_a_pose_brings_on_screen_is_drawn() {
    let mut grove = grove();
    // The window is 400 across. This sits just past it, and twice its size reaches back in.
    let edge = grove.plant(Panel::new().scale(3.0).at(at(410.0, 10.0, 20.0, 20.0)));
    tick(&mut grove);
    assert!(grove.elm.panels.holding(edge).is_some());
}

/// A list on a card scaled to half clips its rows where the card is drawn, not where the layout
/// put it: rows scrolled past the list's edge stay cut off while the card is small, and a press
/// on the part of the upright box the scaled list no longer covers reaches none of them.
#[test]
fn a_scaled_list_clips_where_it_is_drawn() {
    let mut grove = grove();
    // 100 square, its middle at (50, 50): scaled to half, it is drawn over 25..75 both ways.
    let card = grove.plant(Stem::new().scale(0.5).at(at(0.0, 0.0, 100.0, 100.0)));
    let list = grove.branch(
        card,
        Stem::new()
            .scrolls(crate::Axes::Vertical)
            .at(at(0.0, 0.0, 100.0, 100.0)),
    );
    let row = grove.branch(list, Panel::new().interactive().at(at(0.0, 0.0, 100.0, 300.0)));
    tick(&mut grove);
    assert_eq!(
        grove.elements.clip(row),
        Section::from_edges(25.0, 25.0, 75.0, 75.0)
    );
    // Inside the upright box, outside the drawn one: the row reaches here as laid out (drawn
    // through the scale it lands at 25 + 90 / 2), but is cut off.
    assert!(!tap(&mut grove, 50.0, 90.0).clicked(row));
    crate::tests::past_the_taps(&mut grove);
    assert!(tap(&mut grove, 50.0, 50.0).clicked(row));

    // Full size again, the clip is the list's own box.
    grove.animate(card, Motion::Scale(1.0), Timing::ms(0));
    tick(&mut grove);
    tick(&mut grove);
    assert_eq!(
        grove.elements.clip(row),
        Section::from_edges(0.0, 0.0, 100.0, 100.0)
    );
}
