//! The ramp behind the palette: what a scheme derives from a seed, and what an element carrying a
//! step resolves to.

use crate::color::Color;
use crate::tests::{grove, tick};
use crate::{
    Boxed, Grove, Grow, Leaf, Location, Palette, Panel, Place, Scheme, Source, Step, left, top,
};

/// Every role, at its base step, in the order a scheme holds them.
const ROLES: [Palette; 6] = [
    Palette::Surface,
    Palette::Raised,
    Palette::Muted,
    Palette::Accent,
    Palette::Ink,
    Palette::Contrast,
];

/// Every step, deepest into the ground first.
const STEPS: [Step; 5] = [
    Step::RecedeMore,
    Step::Recede,
    Step::Base,
    Step::Advance,
    Step::AdvanceMore,
];

fn lightness(color: Color) -> f32 {
    color.oklab().0
}

/// Which way a reading's ground lies, as the sign an advancing step moves lightness in.
fn advancing(scheme: &Scheme) -> f32 {
    let ramp = ROLES[2];
    (lightness(scheme.color(ramp.advance())) - lightness(scheme.color(ramp))).signum()
}

fn painted(grove: &Grove, leaf: Leaf) -> Color {
    grove
        .elm
        .panels
        .holding(leaf)
        .expect("the backend is holding this panel")
        .color
}

#[test]
fn a_role_resolves_to_the_color_it_was_seeded_with() {
    let stated = Color::rgb(0.42, 0.13, 0.77);
    let scheme = Scheme::new().set(Palette::Accent, stated);
    assert_eq!(scheme.color(Palette::Accent), stated);
}

/// The whole of what a reading decides. A state written once as `advance` stands out further against
/// either ground, which is what makes a light and a dark scheme the same app.
#[test]
fn an_advancing_step_stands_out_from_the_ground_in_either_reading() {
    let dark = Scheme::new();
    let light = Scheme::light();
    for role in ROLES {
        assert!(
            lightness(dark.color(role.advance())) > lightness(dark.color(role)),
            "{role:?} does not advance against a dark ground"
        );
        assert!(
            lightness(light.color(role.advance())) < lightness(light.color(role)),
            "{role:?} does not advance against a light ground"
        );
    }
}

/// True of every seed, including one with no room on one side: compressing the short half is what
/// keeps a ramp from folding back on itself.
#[test]
fn a_ramp_is_ordered_from_its_deepest_step_to_its_furthest_out() {
    for scheme in [Scheme::new(), Scheme::light()] {
        let advancing = advancing(&scheme);
        for role in ROLES {
            let mut steps = STEPS
                .into_iter()
                .map(|step| advancing * lightness(scheme.color(role.at(step))));
            let mut previous = steps.next().expect("a ramp has a first step");
            for next in steps {
                assert!(next >= previous, "{role:?} falls back across its ramp");
                previous = next;
            }
        }
    }
}

/// What a seed away from the ends of the range buys: five steps that are five colors.
#[test]
fn a_ramp_with_room_on_both_sides_has_five_distinct_steps() {
    for scheme in [Scheme::new(), Scheme::light()] {
        for role in [Palette::Muted, Palette::Accent] {
            let mut seen: Vec<Color> = Vec::new();
            for step in STEPS {
                let color = scheme.color(role.at(step));
                assert!(
                    !seen.contains(&color),
                    "{role:?} repeats a color at {step:?}"
                );
                seen.push(color);
            }
        }
    }
}

/// The limit of a ramp, stated rather than hidden. A light scheme's raised surface is seeded at
/// white, which is as far from a light ground as anything gets -- so it has an advancing half and no
/// receding one, and asking for a step it has no room for answers the seed.
#[test]
fn a_seed_at_the_edge_of_the_range_has_no_ramp_on_that_side() {
    let scheme = Scheme::light();
    let seed = scheme.color(Palette::Raised);
    assert_eq!(lightness(seed), 1.0);
    assert_eq!(scheme.color(Palette::Raised.recede()), seed);
    assert_ne!(scheme.color(Palette::Raised.advance()), seed);
}

/// What separates a ramp from an interpolation toward black or white, which desaturates and drifts.
#[test]
fn a_ramp_holds_the_hue_it_was_seeded_with() {
    let seed = Color::rgb(0.38, 0.71, 0.51);
    let scheme = Scheme::new().set(Palette::Accent, seed);
    let (_, a, b) = seed.oklab();
    let hue = b.atan2(a);
    for step in STEPS {
        let (_, a, b) = scheme.color(Palette::Accent.at(step)).oklab();
        let drift = (b.atan2(a) - hue).abs();
        assert!(
            drift < 0.02,
            "{step:?} drifted {drift} radians from the seed"
        );
    }
}

#[test]
fn stepping_saturates_at_the_ends_of_a_ramp() {
    assert_eq!(
        Palette::Accent.advance().advance(),
        Palette::Accent.at(Step::AdvanceMore)
    );
    assert_eq!(
        Palette::Accent.advance().advance().advance(),
        Palette::Accent.at(Step::AdvanceMore)
    );
    assert_eq!(
        Palette::Accent.recede().recede().recede(),
        Palette::Accent.at(Step::RecedeMore)
    );
}

/// A theme is six decisions rather than thirty.
#[test]
fn seeding_a_role_derives_the_rest_of_its_ramp() {
    let before = Scheme::new();
    let after = before.set(Palette::Accent, Color::rgb(0.42, 0.13, 0.77));
    for step in STEPS {
        let tone = Palette::Accent.at(step);
        assert_ne!(
            before.color(tone),
            after.color(tone),
            "{step:?} did not move"
        );
    }
}

/// The way out when a derived step is not the one wanted: it replaces that step and nothing else.
#[test]
fn setting_a_step_that_is_not_the_base_leaves_the_ramp_it_sits_in() {
    let stated = Color::rgb(0.42, 0.13, 0.77);
    let before = Scheme::new();
    let after = before.set(Palette::Accent.advance(), stated);
    assert_eq!(after.color(Palette::Accent.advance()), stated);
    for step in STEPS.into_iter().filter(|step| *step != Step::Advance) {
        let tone = Palette::Accent.at(step);
        assert_eq!(before.color(tone), after.color(tone), "{step:?} moved");
    }
}

#[test]
fn a_scheme_only_moves_the_roles_it_was_given() {
    let before = Scheme::new();
    let after = before.set(Palette::Accent, Color::rgb(0.42, 0.13, 0.77));
    for role in ROLES.into_iter().filter(|role| *role != Palette::Accent) {
        for step in STEPS {
            let tone = role.at(step);
            assert_eq!(before.color(tone), after.color(tone), "{role:?} moved");
        }
    }
    assert_eq!(before.moved(&after), STEPS.len());
}

/// A step is not a literal: it is part of the scheme, and a repaint moves it like any other tone.
#[test]
fn an_element_filled_with_a_step_follows_a_repaint() {
    let mut grove = grove();
    let square = Location::new().xs(
        left(0.0f32.px()).width(48.0f32.px()),
        top(0.0f32.px()).height(48.0f32.px()),
    );
    let leaf = grove.plant(Panel::new().color(Palette::Accent.advance()).at(square));
    tick(&mut grove);
    assert_eq!(
        painted(&grove, leaf),
        Scheme::new().color(Palette::Accent.advance())
    );

    let repainted = Scheme::new().set(Palette::Accent, Color::rgb(0.42, 0.13, 0.77));
    grove.repaint(repainted);
    tick(&mut grove);
    assert_eq!(
        painted(&grove, leaf),
        repainted.color(Palette::Accent.advance())
    );
}
