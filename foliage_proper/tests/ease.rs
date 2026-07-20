//! `Easement::percent_changed` (`anim/ease.rs`) is the curve every `tree.animate(...)` call
//! in the framework runs through. Pure, fully public, zero ECS involved -- proves the Tier B
//! pattern works for non-ECS logic too, not just composite reactivity.

use foliage_proper::{Ease, Easement};

#[test]
fn linear_easing_is_the_identity_function() {
    let mut e = Easement::from(Ease::Linear);
    assert_eq!(e.percent_changed(0.0), 0.0);
    assert_eq!(e.percent_changed(0.37), 0.37);
    assert_eq!(e.percent_changed(1.0), 1.0);
}

#[test]
fn every_bezier_ease_lands_exactly_on_its_endpoints() {
    // the cubic bezier formula's structure guarantees this regardless of control points --
    // worth pinning down explicitly, since every eased animation in the framework depends
    // on actually reaching 0% and 100%, not asymptotically approaching them.
    for ease in [Ease::DECELERATE, Ease::ACCELERATE, Ease::EMPHASIS, Ease::INWARD] {
        let mut e = Easement::from(ease.clone());
        assert!((e.percent_changed(0.0) - 0.0).abs() < 1e-6, "{:?} should start at 0", ease_name(&ease));
        let mut e = Easement::from(ease.clone());
        assert!((e.percent_changed(1.0) - 1.0).abs() < 1e-6, "{:?} should end at 1", ease_name(&ease));
    }
}

#[test]
fn decelerate_starts_faster_than_it_finishes() {
    // DECELERATE's whole point: more progress in the first half than the second.
    let mut e = Easement::from(Ease::DECELERATE);
    let quarter = e.percent_changed(0.25);
    let mut e = Easement::from(Ease::DECELERATE);
    let three_quarter = e.percent_changed(0.75);
    assert!(
        quarter > 0.25,
        "a decelerating ease should be ahead of linear progress early on, got {quarter}"
    );
    assert!(
        (1.0 - three_quarter) < (1.0 - quarter),
        "and closing in on 1.0 more slowly than it opened -- remaining distance should shrink"
    );
}

fn ease_name(ease: &Ease) -> &'static str {
    match ease {
        Ease::Linear => "Linear",
        Ease::Bezier(_) => "Bezier",
    }
}
