//! Animation, through the frame.
//!
//! The clock is moved by hand, so every value here is exact rather than approximate: a tween told to
//! advance half its duration reads its midpoint and not a number near it.

use crate::aspen::Ease;
use crate::coordinate::{Area, Section};
use crate::panel::PanelInstance;
use crate::tests::{Observer, advance, grove, opacity, resize, section, tick, tick_with};
use crate::{
    Color, Fill, Grove, Grow, Leaf, Location, Motion, Palette, Panel, Place, Sap, Scheme, Source,
    Stem, Timing, Vein, anchor, left, top,
};

/// A box of a stated width on one line, so a placement reads as one number.
fn across(from: f32, width: f32) -> Location {
    Location::new().xs(
        left(from.px()).width(width.px()),
        top(0.px()).height(10.px()),
    )
}

fn held(grove: &Grove, leaf: Leaf) -> PanelInstance {
    grove
        .elm
        .panels
        .holding(leaf)
        .expect("the backend is holding this panel")
}

fn close(left: f32, right: f32) {
    assert!(
        (left - right).abs() < 1e-4,
        "expected {left} and {right} to agree"
    );
}

// Both endpoints re-resolve, every frame, in the same context.

/// The window changing size mid-motion moves the target, and the motion lands on where the target
/// went rather than on where it was when the motion started.
#[test]
fn a_resize_mid_motion_lands_exactly_on_the_target() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new().at(across(0.0, 10.0)));
    tick(&mut grove);

    grove.animate(
        leaf,
        Motion::Location(
            Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(10.px())),
        ),
        Timing::ms(200),
    );
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).width(), 205.0);

    // Halfway through, and the surface doubles. Both ends re-resolve at the new size, so the blend
    // is between the same two placements read afresh.
    resize(&mut grove, Area::new(800.0, 300.0));
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).width(), 405.0);

    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).width(), 800.0);
}

/// The target is a chain of breakpoints, and crossing one mid-motion picks the other link. Nothing
/// about the motion is re-stated: it lands on whichever link is in force when it arrives.
#[test]
fn a_breakpoint_crossed_mid_motion_lands_exactly_on_the_target() {
    let mut grove = Grove::new(Area::new(400.0, 800.0));
    let leaf = grove.plant(Stem::new().at(across(0.0, 10.0)));
    tick(&mut grove);

    grove.animate(
        leaf,
        Motion::Location(
            Location::new()
                .xs(left(0.px()).width(100.px()), top(0.px()).height(10.px()))
                .md(left(0.px()).width(300.px()), top(0.px()).height(10.px())),
        ),
        Timing::ms(200),
    );
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).width(), 55.0);

    resize(&mut grove, Area::new(700.0, 800.0));
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).width(), 300.0);
}

/// The target reads an anchor, and the anchor moves mid-motion. The element follows it and lands on
/// where it went.
#[test]
fn an_anchor_moved_mid_motion_is_followed_and_landed_on() {
    let mut grove = grove();
    let target = grove.plant(Stem::new().at(across(20.0, 60.0)));
    let mover = grove.plant(Stem::new().anchored(target).at(across(0.0, 10.0)));
    tick(&mut grove);

    grove.animate(
        mover,
        Motion::Location(Location::new().xs(
            left(anchor().left()).width(anchor().width()),
            top(0.px()).height(10.px()),
        )),
        Timing::ms(200),
    );
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(
        section(&grove, mover),
        Section::from_edges(10.0, 0.0, 45.0, 10.0)
    );

    grove.at(target, across(100.0, 40.0));
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(
        section(&grove, mover),
        Section::from_edges(100.0, 0.0, 140.0, 10.0)
    );
}

/// The endpoint a motion *left* re-resolves too, not only the one it is going to.
///
/// This is the half a `content()` endpoint will exercise once there is a measure to read: the two
/// endpoints go through the same resolver, in the same context, at the same position in dependency
/// order, so whatever moves one of them moves the other.
#[test]
fn the_endpoint_a_motion_left_re_resolves_too() {
    let mut grove = grove();
    let target = grove.plant(Stem::new().at(across(20.0, 20.0)));
    let mover = grove.plant(Stem::new().anchored(target).at(Location::new().xs(
        left(anchor().left()).width(10.px()),
        top(0.px()).height(10.px()),
    )));
    tick(&mut grove);
    assert_eq!(section(&grove, mover).left(), 20.0);

    grove.animate(
        mover,
        Motion::Location(across(200.0, 10.0)),
        Timing::ms(200),
    );
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(section(&grove, mover).left(), 110.0);

    // The same instant of the same motion, with only the placement it left having changed.
    grove.at(target, across(60.0, 20.0));
    tick(&mut grove);
    assert_eq!(section(&grove, mover).left(), 130.0);
}

/// An element anchored to a moving one sees its **interpolated** box, which is what it should
/// follow: the motion, not the destination.
#[test]
fn an_element_anchored_to_a_moving_one_tracks_the_blend() {
    let mut grove = grove();
    let mover = grove.plant(Stem::new().at(across(0.0, 10.0)));
    let follower = grove.plant(Stem::new().anchored(mover).at(Location::new().xs(
        left(anchor().right()).width(10.px()),
        top(0.px()).height(10.px()),
    )));
    tick(&mut grove);
    assert_eq!(section(&grove, follower).left(), 10.0);

    grove.animate(
        mover,
        Motion::Location(across(100.0, 10.0)),
        Timing::ms(200),
    );
    tick(&mut grove);
    assert_eq!(section(&grove, follower).left(), 10.0);

    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(section(&grove, follower).left(), 60.0);

    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(section(&grove, follower).left(), 110.0);
}

// Where the target lives.

/// The blend at the end already equals the plain reading of the declaration, so there is no settling
/// step and nothing to land a pixel off.
#[test]
fn the_frame_a_motion_lands_and_the_ones_after_are_identical() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new().at(across(0.0, 10.0)));
    tick(&mut grove);

    grove.animate(leaf, Motion::Location(across(100.0, 40.0)), Timing::ms(100));
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    let landed = section(&grove, leaf);
    assert_eq!(landed, Section::from_edges(100.0, 0.0, 140.0, 10.0));

    for _ in 0..3 {
        tick(&mut grove);
        assert_eq!(section(&grove, leaf), landed);
    }
}

/// F8. The drain runs before this phase, so a direct write cancels the motion on that property and
/// the element is simply at what was written. There is no stale state to reconcile.
#[test]
fn a_direct_write_mid_motion_drops_it_and_lands_on_what_was_written() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new().at(across(0.0, 10.0)));
    tick(&mut grove);

    grove.animate(leaf, Motion::Location(across(100.0, 10.0)), Timing::ms(200));
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).left(), 50.0);

    grove.at(leaf, across(30.0, 10.0));
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).left(), 30.0);
    assert!(grove.aspen.idle());

    // Nothing left to advance, so more time changes nothing.
    advance(&mut grove, 1000);
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).left(), 30.0);
}

/// A write to one property leaves a motion on another alone. F8 is per property, not per element.
#[test]
fn a_write_to_another_property_leaves_a_motion_alone() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new().at(across(0.0, 10.0)));
    tick(&mut grove);

    grove.animate(leaf, Motion::Location(across(100.0, 10.0)), Timing::ms(200));
    tick(&mut grove);
    grove.opacity(leaf, 0.5);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).left(), 50.0);
    assert_eq!(opacity(&grove, leaf), 0.5);
}

/// Retargeting starts from where the element **is**, not from where the first motion began --
/// otherwise it would jump back to the old start on the frame it was retargeted.
#[test]
fn retargeting_mid_motion_does_not_jump() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new().at(across(0.0, 10.0)));
    tick(&mut grove);

    grove.animate(leaf, Motion::Location(across(100.0, 10.0)), Timing::ms(200));
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).left(), 50.0);

    grove.animate(leaf, Motion::Location(across(300.0, 10.0)), Timing::ms(200));
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).left(), 50.0);

    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).left(), 175.0);

    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).left(), 300.0);
}

/// The snapshot a retarget starts from does not re-resolve, and the target still does -- so a
/// retargeted motion is exactly as responsive about where it is going.
#[test]
fn a_retargeted_motion_still_lands_on_a_target_that_moved() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new().at(across(0.0, 10.0)));
    tick(&mut grove);

    grove.animate(leaf, Motion::Location(across(100.0, 10.0)), Timing::ms(200));
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);

    grove.animate(
        leaf,
        Motion::Location(
            Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(10.px())),
        ),
        Timing::ms(200),
    );
    tick(&mut grove);
    resize(&mut grove, Area::new(800.0, 300.0));
    advance(&mut grove, 200);
    tick(&mut grove);
    assert_eq!(
        section(&grove, leaf),
        Section::from_edges(0.0, 0.0, 800.0, 10.0)
    );
}

// Timing.

/// The harness's own obligation: advancing is exact.
#[test]
fn a_motion_advanced_half_its_duration_reads_its_midpoint() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    tick(&mut grove);

    grove.animate(leaf, Motion::Opacity(0.0), Timing::ms(200));
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(opacity(&grove, leaf), 0.5);
}

/// A tween created at the drain begins at this frame's instant, and the frame's delta is how long
/// the interval *ending* at that instant took. Charging it would move the element on the frame it
/// was told to start, away from where it currently is.
#[test]
fn a_motion_takes_no_time_on_the_frame_it_starts() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    tick(&mut grove);

    grove.animate(leaf, Motion::Opacity(0.0), Timing::ms(200));
    advance(&mut grove, 50);
    tick(&mut grove);
    assert_eq!(opacity(&grove, leaf), 1.0);

    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(opacity(&grove, leaf), 0.5);
}

/// A delay holds the element where it was. The motion is running from the frame it was asked for,
/// so it is cancellable throughout -- it is not queued somewhere waiting to begin.
#[test]
fn a_delay_holds_the_element_where_it_was() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    tick(&mut grove);

    grove.animate(leaf, Motion::Opacity(0.0), Timing::ms(100).after(100));
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(opacity(&grove, leaf), 1.0);

    advance(&mut grove, 50);
    tick(&mut grove);
    assert_eq!(opacity(&grove, leaf), 0.5);

    advance(&mut grove, 50);
    tick(&mut grove);
    assert_eq!(opacity(&grove, leaf), 0.0);
}

/// Idle frames are not a source of drift. What a tween has done is a function of the clock, and the
/// clock did not move.
#[test]
fn idle_frames_move_nothing() {
    let mut batched = grove();
    let one = batched.plant(Stem::new().at(across(0.0, 10.0)));
    batched.animate(one, Motion::Location(across(100.0, 10.0)), Timing::ms(200));
    tick(&mut batched);
    advance(&mut batched, 100);
    tick(&mut batched);

    let mut interleaved = grove();
    let two = interleaved.plant(Stem::new().at(across(0.0, 10.0)));
    interleaved.animate(two, Motion::Location(across(100.0, 10.0)), Timing::ms(200));
    tick(&mut interleaved);
    tick(&mut interleaved);
    tick(&mut interleaved);
    advance(&mut interleaved, 100);
    tick(&mut interleaved);
    tick(&mut interleaved);

    assert_eq!(section(&batched, one), section(&interleaved, two));
}

#[test]
fn two_identical_scripts_animate_identically() {
    let script = |grove: &mut Grove| {
        let leaf = grove.plant(Stem::new().at(across(0.0, 10.0)));
        grove.animate(
            leaf,
            Motion::Location(across(137.0, 41.0)),
            Timing::ms(250).ease(Ease::Emphasis),
        );
        tick(grove);
        advance(grove, 90);
        tick(grove);
        leaf
    };
    let mut one = grove();
    let first = script(&mut one);
    let mut two = grove();
    let second = script(&mut two);
    assert_eq!(section(&one, first), section(&two, second));
}

// Easing.

/// Every shape starts where the motion started and ends exactly on its target, so a landing is
/// never a shape's rounding error.
#[test]
fn every_shape_is_exact_at_both_ends() {
    for ease in [
        Ease::Linear,
        Ease::Decelerate,
        Ease::Accelerate,
        Ease::Emphasis,
        Ease::Curve {
            x1: 0.9,
            y1: 0.1,
            x2: 0.1,
            y2: 0.9,
        },
    ] {
        assert_eq!(ease.at(0.0), 0.0, "{ease:?} at the start");
        assert_eq!(ease.at(1.0), 1.0, "{ease:?} at the end");
    }
}

#[test]
fn a_shape_is_what_makes_progress_differ_from_elapsed() {
    assert_eq!(Ease::Linear.at(0.5), 0.5);
    assert!(Ease::Decelerate.at(0.5) > 0.5, "arriving is front-loaded");
    assert!(Ease::Accelerate.at(0.5) < 0.5, "leaving is back-loaded");
}

/// The curve is read as a function of the fraction, which means recovering the parameter whose `x`
/// is that fraction. Control points on the diagonal make `y` equal `x`, so the answer is the
/// fraction itself -- and it only is if the parameter was actually solved for.
#[test]
fn a_curve_is_read_as_a_function_of_the_elapsed_fraction() {
    let diagonal = Ease::Curve {
        x1: 0.25,
        y1: 0.25,
        x2: 0.9,
        y2: 0.9,
    };
    for step in 1..10 {
        let fraction = step as f32 / 10.0;
        close(diagonal.at(fraction), fraction);
    }
}

/// A shape is stated beside a duration and says nothing about it: the landing is at the same instant
/// whatever the shape, and only the way there differs.
#[test]
fn a_shape_does_not_change_when_a_motion_ends() {
    for ease in [Ease::Linear, Ease::Decelerate, Ease::Emphasis] {
        let mut grove = grove();
        let leaf = grove.plant(Stem::new().at(across(0.0, 10.0)));
        tick(&mut grove);
        grove.animate(
            leaf,
            Motion::Location(across(100.0, 10.0)),
            Timing::ms(200).ease(ease),
        );
        tick(&mut grove);
        advance(&mut grove, 199);
        tick(&mut grove);
        assert!(!grove.aspen.idle(), "{ease:?} ended early");
        advance(&mut grove, 1);
        tick(&mut grove);
        assert!(grove.aspen.idle(), "{ease:?} ran long");
        assert_eq!(section(&grove, leaf).left(), 100.0);
    }
}

// Opacity: a blend of the same type as its declaration, written back over it.

/// The blend is written where the declaration was, so reading the element back reads where the
/// motion has reached -- which is what is on screen.
#[test]
fn an_opacity_motion_is_read_back_where_it_has_reached() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    tick(&mut grove);
    assert_eq!(opacity(&grove, leaf), 1.0);

    grove.animate(leaf, Motion::Opacity(0.2), Timing::ms(100));
    tick(&mut grove);
    advance(&mut grove, 25);
    tick(&mut grove);
    assert_eq!(opacity(&grove, leaf), 0.8);

    advance(&mut grove, 75);
    tick(&mut grove);
    assert_eq!(opacity(&grove, leaf), 0.2);
}

/// Opacity is a product over the whole ancestry, and animating one element's carries everything
/// grown under it -- which is what makes fading a page one write.
#[test]
fn an_animated_opacity_multiplies_through_what_is_grown_under_it() {
    let mut grove = grove();
    let page = grove.plant(Stem::new());
    let panel = grove.branch(page, Panel::new().color(Palette::Accent));
    tick(&mut grove);
    assert_eq!(held(&grove, panel).color.alpha, 1.0);

    grove.animate(page, Motion::Opacity(0.0), Timing::ms(100));
    tick(&mut grove);
    advance(&mut grove, 50);
    tick(&mut grove);
    assert_eq!(held(&grove, panel).color.alpha, 0.5);
}

/// Retargeting a value written back over its own declaration needs no snapshot: what the element
/// declares already *is* where it currently is.
#[test]
fn retargeting_an_opacity_motion_starts_from_where_it_reached() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    tick(&mut grove);

    grove.animate(leaf, Motion::Opacity(0.0), Timing::ms(100));
    tick(&mut grove);
    advance(&mut grove, 50);
    tick(&mut grove);
    assert_eq!(opacity(&grove, leaf), 0.5);

    grove.animate(leaf, Motion::Opacity(1.0), Timing::ms(100));
    tick(&mut grove);
    assert_eq!(opacity(&grove, leaf), 0.5);

    advance(&mut grove, 50);
    tick(&mut grove);
    assert_eq!(opacity(&grove, leaf), 0.75);
}

// Color: a blend of a different type from its declaration, taken where roles become colors.

#[test]
fn a_palette_motion_blends_the_two_roles_it_names() {
    let mut grove = grove();
    let leaf = grove.plant(Panel::new().color(Palette::Muted));
    tick(&mut grove);
    let scheme = grove.scheme();
    assert_eq!(held(&grove, leaf).color, scheme.color(Palette::Muted));

    grove.animate(leaf, Motion::Palette(Palette::Accent), Timing::ms(200));
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(
        held(&grove, leaf).color,
        scheme
            .color(Palette::Muted)
            .blend(scheme.color(Palette::Accent), 0.5)
    );

    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).color, scheme.color(Palette::Accent));
}

/// What a role resolves to is the scheme's answer, taken every frame for *both* ends. So a repaint
/// mid-motion moves the motion, exactly as a resize moves a placement's.
#[test]
fn a_repaint_mid_palette_motion_moves_both_ends() {
    let mut grove = grove();
    let leaf = grove.plant(Panel::new().color(Palette::Muted));
    tick(&mut grove);

    grove.animate(leaf, Motion::Palette(Palette::Accent), Timing::ms(200));
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);

    let repainted = Scheme::new()
        .set(Palette::Muted, Color::rgb(0.0, 0.0, 0.0))
        .set(Palette::Accent, Color::rgb(1.0, 1.0, 1.0));
    grove.repaint(repainted);
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).color, Color::rgb(0.5, 0.5, 0.5));

    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).color, Color::rgb(1.0, 1.0, 1.0));
}

/// A blend of two roles is a color and not a role, so a retarget snapshots the color. It is a
/// starting pixel and nothing else, which is all it has to be.
#[test]
fn retargeting_a_fill_motion_starts_from_the_blend() {
    let mut grove = grove();
    let leaf = grove.plant(Panel::new().color(Palette::Muted));
    tick(&mut grove);
    let scheme = grove.scheme();
    let midway = scheme
        .color(Palette::Muted)
        .blend(scheme.color(Palette::Accent), 0.5);

    grove.animate(leaf, Motion::Palette(Palette::Accent), Timing::ms(200));
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).color, midway);

    grove.animate(leaf, Motion::Palette(Palette::Ink), Timing::ms(200));
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).color, midway);

    advance(&mut grove, 200);
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).color, scheme.color(Palette::Ink));
}

/// A fill stated outright needs no scheme to mean anything, and lands on exactly the color named.
#[test]
fn a_color_motion_lands_on_the_color_it_names() {
    let mut grove = grove();
    let leaf = grove.plant(Panel::new().color(Color::rgb(0.0, 0.0, 0.0)));
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).color, Color::rgb(0.0, 0.0, 0.0));

    grove.animate(
        leaf,
        Motion::Color(Color::rgb(1.0, 1.0, 1.0)),
        Timing::ms(200),
    );
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).color, Color::rgb(0.5, 0.5, 0.5));

    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).color, Color::rgb(1.0, 1.0, 1.0));
}

/// Both ways of naming a fill move the same property, so the second replaces the first rather than
/// running beside it. A fill can only be going to one place.
#[test]
fn a_role_and_a_color_are_one_property() {
    let mut grove = grove();
    let leaf = grove.plant(Panel::new().color(Color::rgb(0.0, 0.0, 0.0)));
    tick(&mut grove);

    grove.animate(
        leaf,
        Motion::Color(Color::rgb(1.0, 1.0, 1.0)),
        Timing::ms(200),
    );
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);
    let midway = Color::rgb(0.5, 0.5, 0.5);
    assert_eq!(held(&grove, leaf).color, midway);

    grove.animate(leaf, Motion::Palette(Palette::Accent), Timing::ms(200));
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).color, midway, "no jump at the retarget");

    advance(&mut grove, 200);
    tick(&mut grove);
    let accent = grove.scheme().color(Palette::Accent);
    assert_eq!(held(&grove, leaf).color, accent);
    assert!(grove.aspen.idle(), "one property, so one motion");
}

/// A motion may cross between the two, and each end keeps its own answer to a repaint: the role it
/// left follows the scheme, the color it is going to does not.
#[test]
fn a_motion_may_cross_between_a_role_and_a_color() {
    let mut grove = grove();
    let leaf = grove.plant(Panel::new().color(Palette::Muted));
    tick(&mut grove);

    grove.animate(
        leaf,
        Motion::Color(Color::rgb(1.0, 1.0, 1.0)),
        Timing::ms(200),
    );
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);

    grove.repaint(Scheme::new().set(Palette::Muted, Color::rgb(0.0, 0.0, 0.0)));
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).color, Color::rgb(0.5, 0.5, 0.5));

    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(held(&grove, leaf).color, Color::rgb(1.0, 1.0, 1.0));
}

/// The target is written to the element the moment the motion starts, and a fill is the one property
/// where that is readable: the element declares where it is going, in the words it was named in.
#[test]
fn a_fill_declares_its_target_from_the_frame_the_motion_starts() {
    let mut grove = grove();
    let leaf = grove.plant(Panel::new().color(Palette::Muted));
    tick(&mut grove);
    assert_eq!(
        grove.tap(leaf, Vein::Color),
        Some(Sap::Color(Fill::Role(Palette::Muted)))
    );

    let named = Color::rgb(1.0, 0.0, 0.0);
    grove.animate(leaf, Motion::Color(named), Timing::ms(200));
    tick(&mut grove);
    assert_eq!(
        grove.tap(leaf, Vein::Color),
        Some(Sap::Color(Fill::Literal(named)))
    );
}

/// Dropped like any other op naming something it does not apply to.
#[test]
fn animating_the_fill_of_something_that_draws_nothing_is_dropped() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    tick(&mut grove);

    grove.animate(leaf, Motion::Palette(Palette::Accent), Timing::ms(200));
    tick(&mut grove);
    assert!(grove.aspen.idle());
}

#[test]
fn animating_a_withered_leaf_is_dropped() {
    let mut grove = grove();
    let leaf = grove.plant(Stem::new());
    tick(&mut grove);
    grove.prune(leaf);
    grove.animate(leaf, Motion::Opacity(0.0), Timing::ms(200));
    tick(&mut grove);
    assert!(grove.aspen.idle());
}

#[test]
fn a_motion_goes_with_the_element_it_was_moving() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    let branch = grove.branch(trunk, Stem::new().at(across(0.0, 10.0)));
    tick(&mut grove);

    grove.animate(
        branch,
        Motion::Location(across(100.0, 10.0)),
        Timing::ms(200),
    );
    tick(&mut grove);
    assert!(!grove.aspen.idle());

    grove.prune(trunk);
    tick(&mut grove);
    assert!(grove.aspen.idle());
}

// What is reported.

/// The report is an arrival, not a request to settle: the element declared the target from the
/// moment the motion started, so this is the hook for whatever happens next.
#[test]
fn a_motion_reports_where_it_landed() {
    let mut grove = grove();
    let mut app = Observer::default();
    let leaf = grove.plant(Stem::new());
    tick_with(&mut grove, &mut app);

    grove.animate(leaf, Motion::Opacity(0.0), Timing::ms(100));
    tick_with(&mut grove, &mut app);
    advance(&mut grove, 100);
    // The frame it lands in: the report is made at step 5, after this frame's `Pollen` was sealed.
    tick_with(&mut grove, &mut app);
    assert!(!app.last().landed(leaf));

    tick_with(&mut grove, &mut app);
    assert!(app.last().landed(leaf));

    tick_with(&mut grove, &mut app);
    assert!(!app.last().landed(leaf));
}

/// A channel is the engine's clock and easing handed to a value it has no concept of. It writes
/// nothing; the value is only ever reported.
#[test]
fn a_channel_reports_a_value_every_frame_it_runs() {
    let mut grove = grove();
    let mut app = Observer::default();
    let channel = grove.tween(0.0, 100.0, Timing::ms(200));

    tick_with(&mut grove, &mut app);
    assert_eq!(app.last().tween(channel), None);

    advance(&mut grove, 100);
    tick_with(&mut grove, &mut app);
    assert_eq!(app.last().tween(channel), Some(0.0));

    advance(&mut grove, 100);
    tick_with(&mut grove, &mut app);
    assert_eq!(app.last().tween(channel), Some(50.0));

    tick_with(&mut grove, &mut app);
    assert_eq!(app.last().tween(channel), Some(100.0));
    assert!(app.last().finished(channel));

    tick_with(&mut grove, &mut app);
    assert_eq!(app.last().tween(channel), None);
    assert!(!app.last().finished(channel));
}

/// A timer set for zero fires no earlier than the next frame: queued at step 3, applied at 4,
/// advanced at 5, and reported at step 3 of the frame after. Honest rather than special-cased.
#[test]
fn a_timer_of_zero_fires_no_earlier_than_the_next_frame() {
    let mut grove = grove();
    let mut app = Observer::default();
    let timer = grove.timer(Timing::ms(0));

    tick_with(&mut grove, &mut app);
    assert!(!app.last().finished(timer));

    tick_with(&mut grove, &mut app);
    assert!(app.last().finished(timer));
    assert!(grove.aspen.idle());
}

/// A channel has no declaration for a direct write to cancel it through, so stopping one is a verb.
#[test]
fn a_stopped_channel_reports_nothing_more() {
    let mut grove = grove();
    let mut app = Observer::default();
    let channel = grove.tween(0.0, 1.0, Timing::ms(1000));
    tick_with(&mut grove, &mut app);
    assert!(!grove.aspen.idle());

    grove.stop(channel);
    tick_with(&mut grove, &mut app);
    assert!(grove.aspen.idle());

    for _ in 0..3 {
        advance(&mut grove, 1000);
        tick_with(&mut grove, &mut app);
        assert!(!app.last().finished(channel));
    }
}

/// Two motions on one element are two properties, and neither is a second writer of the other's.
#[test]
fn two_properties_of_one_element_move_independently() {
    let mut grove = grove();
    let leaf = grove.plant(Panel::new().color(Palette::Muted).at(across(0.0, 10.0)));
    tick(&mut grove);

    grove.animate(leaf, Motion::Location(across(100.0, 10.0)), Timing::ms(200));
    grove.animate(leaf, Motion::Opacity(0.0), Timing::ms(100));
    tick(&mut grove);
    advance(&mut grove, 100);
    tick(&mut grove);

    assert_eq!(section(&grove, leaf).left(), 50.0);
    assert_eq!(opacity(&grove, leaf), 0.0);
    assert!(!grove.aspen.idle(), "the placement is still moving");

    advance(&mut grove, 100);
    tick(&mut grove);
    assert_eq!(section(&grove, leaf).left(), 100.0);
    assert!(grove.aspen.idle());
}
