//! The ramp behind the palette: what a scheme derives from a seed, and what an element carrying a
//! step resolves to.

use crate::color::Color;
use crate::tests::{Observer, grove, tick, tick_with};
use crate::{Boxed, Grove, Grow, Leaf, Location, Palette, Panel, Scheme, Source, Step, left, top};

/// Every role, at its base step, and a form or two of the hues.
const ROLES: [Palette; 11] = [
    Palette::Surface,
    Palette::Raised,
    Palette::Muted,
    Palette::Ink,
    Palette::Accent,
    Palette::Signal,
    Palette::Danger,
    Palette::Caution,
    Palette::Positive,
    Palette::Contrast,
    Palette::Signal.mark(),
];

/// Every hue, at its ground.
const HUES: [Palette; 5] = [
    Palette::Accent,
    Palette::Signal,
    Palette::Danger,
    Palette::Caution,
    Palette::Positive,
];

/// Every step, deepest into the ground first.
const STEPS: [Step; 5] = [
    Step::Farthest,
    Step::Far,
    Step::Base,
    Step::Near,
    Step::Nearest,
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
fn a_ramp_is_ordered_from_its_farthest_step_to_its_nearest() {
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
        Palette::Accent.at(Step::Nearest)
    );
    assert_eq!(
        Palette::Accent.advance().advance().advance(),
        Palette::Accent.at(Step::Nearest)
    );
    assert_eq!(
        Palette::Accent.recede().recede().recede(),
        Palette::Accent.at(Step::Farthest)
    );
}

/// A theme is seven decisions rather than thirty-five.
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
    for step in STEPS.into_iter().filter(|step| *step != Step::Near) {
        let tone = Palette::Accent.at(step);
        assert_eq!(before.color(tone), after.color(tone), "{step:?} moved");
    }
}

/// Seeding a hue's ground moves that hue -- its ground, and the mark and the `on` derived from it --
/// and nothing else.
#[test]
fn a_scheme_only_moves_the_roles_it_was_given() {
    let before = Scheme::new();
    let after = before.set(Palette::Accent, Color::rgb(0.42, 0.13, 0.77));
    let accent = [
        Palette::Accent,
        Palette::Accent.mark(),
        Palette::Accent.on(),
    ];
    for role in ROLES.into_iter().filter(|role| !accent.contains(role)) {
        for step in STEPS {
            let tone = role.at(step);
            assert_eq!(before.color(tone), after.color(tone), "{tone:?} moved");
        }
    }
    assert_eq!(before.moved(&after), accent.len() * STEPS.len());
}

/// What makes a hue usable as a fill and as a mark at once: the mark is the ground's hue at text
/// lightness, and what is on the ground is legible on it.
#[test]
fn a_hue_derives_a_mark_and_what_is_read_on_it() {
    for scheme in [Scheme::new(), Scheme::light()] {
        for hue in HUES {
            let ground = scheme.color(hue);
            let mark = scheme.color(hue.mark());
            let on = scheme.color(hue.on());
            let surface = scheme.color(Palette::Surface);
            assert!(
                (lightness(on) - lightness(ground)).abs() > 0.35,
                "{hue:?} has nothing legible on it"
            );
            assert!(
                (lightness(mark) - lightness(surface)).abs() > 0.35,
                "{hue:?}'s mark does not read on the surface"
            );
            let (_, a, b) = ground.oklab();
            let (_, ma, mb) = mark.oklab();
            if a.hypot(b) > 0.03 {
                let drift = (mb.atan2(ma) - b.atan2(a)).abs();
                assert!(drift < 0.1, "{hue:?}'s mark drifted {drift} radians");
            }
        }
    }
}

/// A form seeded outright is the app's, and a later seed of the ground does not derive over it.
#[test]
fn a_seeded_form_survives_its_ground_being_seeded() {
    let mark = Color::rgb(0.62, 0.86, 0.84);
    let scheme = Scheme::new()
        .set(Palette::Signal.mark(), mark)
        .set(Palette::Signal, Color::rgb(0.16, 0.40, 0.38));
    assert_eq!(scheme.color(Palette::Signal.mark()), mark);
    assert_eq!(scheme.color(Palette::Signal), Color::rgb(0.16, 0.40, 0.38));
}

#[test]
fn a_slot_answers_as_the_accent_until_it_is_seeded() {
    let scheme = Scheme::new();
    for step in STEPS {
        assert_eq!(
            scheme.color(Palette::hue(0).at(step)),
            scheme.color(Palette::Accent.at(step))
        );
        assert_eq!(
            scheme.color(Palette::hue(0).on().at(step)),
            scheme.color(Palette::Accent.on().at(step))
        );
    }
    let moss = Color::rgb(0.30, 0.50, 0.20);
    let seeded = scheme.set(Palette::hue(0), moss);
    assert_eq!(seeded.color(Palette::hue(0)), moss);
    assert_eq!(seeded.color(Palette::hue(1)), seeded.color(Palette::Accent));
    assert_ne!(
        seeded.color(Palette::hue(0).mark()),
        seeded.color(Palette::Accent.mark())
    );
}

#[test]
fn contrast_is_what_is_read_on_the_accent() {
    assert_eq!(Palette::Contrast, Palette::Accent.on());
    assert_eq!(Palette::Surface.on(), Palette::Ink);
    assert_eq!(
        Palette::Raised.at(Step::Near).mark(),
        Palette::Ink.at(Step::Near)
    );
    assert_eq!(Palette::Ink.on(), Palette::Surface);
    assert_eq!(Palette::Danger.on().ground(), Palette::Danger);
}

#[test]
fn a_spectrum_is_its_stops_or_the_accents_ramp() {
    let scheme = Scheme::new();
    let unstated = scheme.stops(0);
    assert_eq!(unstated.len(), STEPS.len());
    assert_eq!(unstated[2], scheme.color(Palette::Accent));
    let stops = [Color::rgb(0.9, 0.6, 0.2), Color::rgb(0.5, 0.1, 0.1)];
    let stated = scheme.spectrum(1, &stops);
    assert_eq!(stated.stops(1), stops.to_vec());
    assert_eq!(stated.stops(0), unstated);
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

/// A repaint is reported, for what an app computed from the scheme and has to compute again.
#[test]
fn a_repaint_is_reported_the_frame_after() {
    let mut grove = grove();
    let frame = |grove: &mut Grove| {
        let mut app = Observer::default();
        tick_with(grove, &mut app);
        app.last().clone()
    };
    frame(&mut grove);
    grove.repaint(Scheme::light());
    let heard = [frame(&mut grove), frame(&mut grove)];
    assert!(heard.iter().any(|pollen| pollen.repainted()));
    assert!(!frame(&mut grove).repainted());
}
