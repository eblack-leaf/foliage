//! The opener: a full-bleed first screen, then a hint to keep going.
//!
//! Nothing types itself in and nothing spins. Shapes resolve in place -- rough polygons
//! becoming their real form -- which reads as arrival rather than as travel.

use foliage::{
    Anchor, Animation, Color, Ease, EcsExtension, Elevation, Entity, FontSize, Grid, GridExt,
    HorizontalAlignment, Icon, IconId, InteractionListener, InteractionPropagation,
    InteractionShape, Leaf, Location, OnClick, Opacity, PageIndex, Query, Sprout, Text,
    TextContentHeight, Tree, Trigger, VerticalAlignment, With, anchor,
};

use crate::entry::AppRouter;
use crate::icons::IconHandles;
use crate::site::{POLY_BUTTON_ROW_H, PolyButton, fade_in, poly_button, role, space, type_scale};

const WORDMARK: &str = "foliage.rs";
/// Where ".rs" starts, so the extension carries the accent while the name stays plain.
const EXTENSION_AT: usize = 7;
/// Sized to fit ten monospace characters inside the viewport. JetBrains Mono advances about
/// 0.6em, so ten characters run ~6x the font size -- 40px overflows a 360px phone, which is
/// what clipped the first attempt.
/// The project's namesake, so it takes as much room as each viewport can give it. Ten
/// monospace characters run about 6x the font size, which is what caps each step.
const WORDMARK_XS: u32 = 40;
const WORDMARK_MD: u32 = 76;
/// Landscape is `md`-wide, so it would otherwise take `WORDMARK_MD` in a ~250px-tall
/// viewport. Still clips on something both very short *and* narrow -- accepted, since the
/// name earns its size everywhere else.
const WORDMARK_SHORT: u32 = 30;

const TAGLINE: &str = "cross-platform UI in Rust";

const BUTTONS_TOP_PCT: f32 = 62.0;
/// The buttons live in their own row so `max` has something to constrain. Positioned as
/// thirds of the *hero*, they drifted to the far corners of a wide monitor -- a max on the
/// hero could never fix that, because the hero is supposed to be full width.
const ROW_MAX: f32 = 420.0;

/// The opening beat, in ms. Ordered rather than staggered by a single step: the name lands
/// first and alone, then what it is, and only then do the destinations resolve.
const AT_WORDMARK: u64 = 0;
const AT_TAGLINE: u64 = 320;
const AT_BUTTONS: u64 = 700;
/// Between one button and the next. Wide enough to watch each shape become itself, which is
/// the whole reason they are three different shapes.
const BUTTON_STEP: u64 = 300;
const AT_HINT: u64 = AT_BUTTONS + BUTTON_STEP * 3 + 260;
/// Short viewports get the hero side by side instead of stacked: wordmark and tagline on
/// the left, destinations on the right. Stacked, the wordmark, buttons and the fold hint
/// all land on top of each other -- there simply is not 100% of a 250px-tall viewport to
/// share out. Landscape has the opposite budget, so the layout turns ninety degrees.
const SHORT_TEXT_RIGHT_PCT: f32 = 52.0;
const SHORT_BUTTONS_LEFT_PCT: f32 = 52.0;
const SHORT_BUTTONS_TOP_PCT: f32 = 46.0;

const HINT_TEXT: &str = "more";
const CHEVRON: i32 = 22;
/// A breath, not a bounce.
const BOB_PX: i32 = 8;
const BOB_MS: u64 = 900;

/// Warm set rather than one cool outlier -- amber through rose reads as a family the
/// sand/orange scheme belongs to.
fn destinations() -> [PolyButton; 3] {
    [
        PolyButton {
            label: "docs",
            icon: IconHandles::Code,
            href: "https://eblack-leaf.github.io/foliage/api/foliage/index.html",
            sides: 7.0,
            face: Color::amber(400),
        },
        PolyButton {
            label: "book",
            icon: IconHandles::BookOpen,
            href: "https://eblack-leaf.github.io/foliage/book/",
            sides: 6.0,
            face: role::accent(),
        },
        PolyButton {
            label: "github",
            icon: IconHandles::Github,
            href: "https://github.com/eblack-leaf/foliage",
            sides: 5.0,
            face: Color::rose(400),
        },
    ]
}

/// The hero is its own route, and the only one without the rail.
///
/// It was the top of the overview page, which put two scroll contexts back to back and left
/// the rail sitting beside a landing screen it has nothing to do with. As a route it owns
/// the whole viewport, and `more` navigates rather than scrolls -- which is what people try
/// to click anyway.
pub fn build(tree: &mut Tree, slot: Entity) {
    let seq = tree.sequence();
    let hero = tree.branch(
        slot,
        Leaf::sprout()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with((
                Grid::new(1.col().gap(0), 1.row().gap(0)),
                InteractionPropagation::pass_through(),
            )),
    );

    let wordmark = tree.branch(
        hero,
        Text::new(WORDMARK)
            .size(
                FontSize::new(WORDMARK_XS)
                    .md(WORDMARK_MD)
                    .short(WORDMARK_SHORT),
            )
            .color(role::on_surface())
            .glyph_colors(|i| {
                if i >= EXTENSION_AT {
                    role::accent()
                } else {
                    role::on_surface()
                }
            })
            .at(Location::new()
                .xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    30.pct().as_center_y().with(1.letters().as_height()),
                )
                .short(
                    0.pct()
                        .as_left()
                        .with(SHORT_TEXT_RIGHT_PCT.pct().as_right()),
                    38.pct().as_center_y().with(1.letters().as_height()),
                ))
            .elevate(Elevation::up(3))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                Opacity::new(0.0),
            )),
    );
    fade_in(tree, wordmark, seq, AT_WORDMARK);

    let tagline = tree.branch(
        hero,
        Text::new(TAGLINE)
            .size(FontSize::new(type_scale::BODY))
            .color(role::on_surface_variant())
            .at(Location::new()
                .xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    anchor()
                        .bottom()
                        .as_top()
                        .adjust(space::MD)
                        .with(20.px().as_height()),
                )
                .short(
                    0.pct()
                        .as_left()
                        .with(SHORT_TEXT_RIGHT_PCT.pct().as_right()),
                    anchor()
                        .bottom()
                        .as_top()
                        .adjust(space::SM)
                        .with(20.px().as_height()),
                ))
            .elevate(Elevation::up(3))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Top,
                // grows to whatever it wraps to instead of being scissored -- the fixed
                // height cut the second line off in landscape
                TextContentHeight(true),
                Anchor::new(wordmark),
                Opacity::new(0.0),
            )),
    );
    fade_in(tree, tagline, seq, AT_TAGLINE);

    // capped and centred, so three buttons stay a group instead of drifting to the corners
    let row = tree.branch(
        hero,
        Leaf::sprout()
            .at(Location::new()
                .xs(
                    0.pct().as_left().with(100.pct().as_right()).max(ROW_MAX),
                    BUTTONS_TOP_PCT
                        .pct()
                        .as_center_y()
                        .with(POLY_BUTTON_ROW_H.px().as_height()),
                )
                .short(
                    SHORT_BUTTONS_LEFT_PCT
                        .pct()
                        .as_left()
                        .with(100.pct().as_right())
                        .max(ROW_MAX),
                    SHORT_BUTTONS_TOP_PCT
                        .pct()
                        .as_center_y()
                        .with(POLY_BUTTON_ROW_H.px().as_height()),
                ))
            .elevate(Elevation::up(1))
            .with((
                Grid::new(1.col().gap(0), 1.row().gap(0)),
                InteractionPropagation::pass_through(),
            )),
    );
    let specs = destinations();
    let third = 100.0 / specs.len() as f32;
    for (i, spec) in specs.iter().enumerate() {
        let center = third * i as f32 + third / 2.0;
        poly_button(
            tree,
            row,
            spec,
            center,
            seq,
            AT_BUTTONS + BUTTON_STEP * i as u64,
        );
    }
    hint(tree, hero, seq);
}

/// A destination: shadow polygon behind, front polygon on top, icon centred in it, label
/// beneath. The offset shadow is what gives it depth without a blur.

/// The chevron's placement, `drift` px lower than its resting spot.
///
/// Built in one place because the bob animates a whole `Location`: a target carrying only
/// `xs` would drop the `short` variant, snapping the chevron back to the stacked position
/// the moment the loop first ran on a landscape viewport.
fn chevron_at(drift: i32) -> Location {
    let bottom = |gap: i32| {
        100.pct()
            .as_bottom()
            .adjust(-(CHEVRON + gap) + drift)
            .with(CHEVRON.px().as_height())
    };
    Location::new()
        .xs(
            50.pct().as_center_x().with(CHEVRON.px().as_width()),
            bottom(space::MD),
        )
        .short(
            (SHORT_TEXT_RIGHT_PCT / 2.0)
                .pct()
                .as_center_x()
                .with(CHEVRON.px().as_width()),
            bottom(space::SM),
        )
}

/// "more", with a chevron breathing under it at the fold.
fn hint(tree: &mut Tree, hero: Entity, seq: Entity) {
    let label = tree.branch(
        hero,
        Text::new(HINT_TEXT)
            .size(FontSize::new(type_scale::TITLE))
            .color(role::on_surface_variant())
            .at(Location::new()
                .xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    100.pct()
                        .as_bottom()
                        .adjust(-(CHEVRON + space::XL))
                        .with(24.px().as_height()),
                )
                // stays under the text column, clear of the buttons on the right
                .short(
                    0.pct()
                        .as_left()
                        .with(SHORT_TEXT_RIGHT_PCT.pct().as_right()),
                    100.pct()
                        .as_bottom()
                        .adjust(-(CHEVRON + space::LG))
                        .with(24.px().as_height()),
                ))
            .elevate(Elevation::up(3))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                InteractionListener::new(),
                Opacity::new(0.0),
            )),
    );
    fade_in(tree, label, seq, AT_HINT);
    // the word is the affordance as much as the chevron is -- people aim at whichever is
    // nearer their thumb
    tree.on_click(
        label,
        move |_: Trigger<OnClick>, routers: Query<Entity, With<AppRouter>>, mut tree: Tree| {
            if let Ok(router) = routers.single() {
                tree.write_to(router, PageIndex(1));
            }
        },
    );

    let chevron = tree.branch(
        hero,
        Icon::new(IconId::from(IconHandles::ChevronDown))
            .color(role::accent())
            .at(chevron_at(0))
            .elevate(Elevation::up(3))
            .with((
                InteractionListener::new(),
                InteractionShape::Circle,
                Opacity::new(0.0),
            )),
    );
    fade_in(tree, chevron, seq, AT_HINT + 120);
    // people reach for the affordance rather than scrolling past it. The route fn only gets
    // its slot, so the router is found by its marker -- which is what `AppRouter` is for.
    tree.on_click(
        chevron,
        move |_: Trigger<OnClick>, routers: Query<Entity, With<AppRouter>>, mut tree: Tree| {
            if let Ok(router) = routers.single() {
                tree.write_to(router, PageIndex(1));
            }
        },
    );

    // Its own sequence, forever, backtracking so it returns rather than snapping -- a plain
    // loop would jerk back to the top every cycle. Tied to the chevron's own lifetime, so
    // leaving the route stops it.
    let bob = tree.sequence();
    tree.animate(
        Animation::new(chevron_at(BOB_PX))
            .targeting(chevron)
            .during(bob)
            .start(AT_HINT + 200)
            .finish(AT_HINT + 200 + BOB_MS)
            .eased(Ease::DECELERATE)
            .forever()
            .backtrack(),
    );
}
