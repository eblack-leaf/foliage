//! `AspectRatio::constrain`/`fit`/`config` (`grid/aspect_ratio.rs`) are pure, fully public,
//! zero-ECS functions -- no reason to guess at them through the composite system when they
//! can be called directly.

use foliage_proper::{AspectRatio, Layout, Logical, Position, Section};

fn section(w: f32, h: f32) -> Section<Logical> {
    Section::new(Position::logical((0.0, 0.0)), (w, h))
}

#[test]
fn constrain_shrinks_width_until_the_ratio_fits_within_the_original_height() {
    // a 200x50 box constrained to a 2:1 (width:height) ratio -- width=200 would need
    // height=100 to hold that ratio, which overflows the original 50-tall box, so it
    // shrinks width down to exactly match: 100x50.
    let ratio = AspectRatio::new().xs(2.0);
    let constrained = ratio.constrain(section(200.0, 50.0), Layout::Xs).unwrap();
    assert_eq!(constrained.width(), 100.0);
    assert_eq!(constrained.height(), 50.0);
}

#[test]
fn constrain_leaves_a_box_that_already_fits_the_ratio_untouched() {
    let ratio = AspectRatio::new().xs(2.0);
    let constrained = ratio.constrain(section(200.0, 100.0), Layout::Xs).unwrap();
    assert_eq!(constrained.width(), 200.0);
    assert_eq!(constrained.height(), 100.0);
}

#[test]
fn fit_grows_width_until_the_ratio_reaches_the_original_height_and_recenters() {
    // a 100x100 box fit to a 2:1 ratio -- width=100 only gives height=50, short of the
    // original 100-tall box, so width grows to 200 (height=100), then gets recentered
    // horizontally around the original box's own center (shifted left by half the growth).
    let ratio = AspectRatio::new().xs(2.0);
    let fitted = ratio.fit(section(100.0, 100.0), Layout::Xs).unwrap();
    assert_eq!(fitted.width(), 200.0);
    assert_eq!(fitted.height(), 100.0);
    assert_eq!(fitted.left(), -50.0, "recentered: grew by 100, so shifted left by half that");
}

#[test]
fn config_falls_back_to_the_nearest_smaller_configured_breakpoint() {
    // only `xs` and `lg` are configured -- `Md` should fall back to `xs` (the nearest
    // smaller breakpoint that's actually set), not `None` and not `lg` (a *larger*
    // breakpoint should never apply to a smaller request).
    let ratio = AspectRatio::new().xs(1.0).lg(3.0);
    assert_eq!(ratio.config(Layout::Xs), Some(1.0));
    assert_eq!(ratio.config(Layout::Sm), Some(1.0), "falls back to xs");
    assert_eq!(ratio.config(Layout::Md), Some(1.0), "falls back to xs, not lg");
    assert_eq!(ratio.config(Layout::Lg), Some(3.0), "lg is set directly");
    assert_eq!(ratio.config(Layout::Xl), Some(3.0), "falls back to lg");
}

#[test]
fn config_is_none_when_nothing_at_or_below_the_requested_breakpoint_is_set() {
    let ratio = AspectRatio::new().lg(3.0);
    assert_eq!(ratio.config(Layout::Xs), None);
    assert_eq!(ratio.config(Layout::Md), None);
    assert_eq!(ratio.config(Layout::Lg), Some(3.0));
}
