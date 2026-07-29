//! The opener: a full-bleed first screen, then a hint to keep going.
//!
//! Nothing types itself in and nothing spins. Shapes resolve in place -- rough polygons
//! becoming their real form -- which reads as arrival rather than as travel.

use foliage::{
    Anchor, Animation, Color, Ease, EcsExtension, Elevation, Entity, FontSize, Grid, GridExt,
    HorizontalAlignment, HrefLink, Icon, IconId, InteractionListener, InteractionPropagation,
    InteractionShape, Leaf, Location, OnClick, Opacity, Polygon, Sprout, Text, Tree, Trigger,
    VerticalAlignment, anchor,
};

use crate::icons::IconHandles;
use crate::site::{ACCENT, fade_in, morph_in, motion, role, space, type_scale};

const WORDMARK: &str = "foliage.rs";
/// Where ".rs" starts, so the extension carries the accent while the name stays plain.
const EXTENSION_AT: usize = 7;
/// Sized to fit ten monospace characters inside the viewport. JetBrains Mono advances about
/// 0.6em, so ten characters run ~6x the font size -- 40px overflows a 360px phone, which is
/// what clipped the first attempt.
const WORDMARK_XS: u32 = 34;
const WORDMARK_MD: u32 = 56;

const TAGLINE: &str = "cross-platform UI in Rust";

/// One destination: a shadowed polygon with an icon in it, and a label underneath.
const BUTTON: i32 = 56;
const SHADOW_OFF: i32 = 7;
const ICON_SCALE: f32 = 0.44;
const BUTTON_LABEL_H: i32 = 24;
const BUTTONS_TOP_PCT: f32 = 62.0;

const HINT_TEXT: &str = "more";
const CHEVRON: i32 = 22;
/// A breath, not a bounce.
const BOB_PX: i32 = 8;
const BOB_MS: u64 = 900;

struct Destination {
    label: &'static str,
    icon: IconHandles,
    href: &'static str,
    sides: f32,
}

const DESTINATIONS: [Destination; 3] = [
    Destination {
        label: "docs",
        icon: IconHandles::Code,
        href: "https://eblack-leaf.github.io/foliage/api/foliage/index.html",
        sides: 7.0,
    },
    Destination {
        label: "book",
        icon: IconHandles::BookOpen,
        href: "https://eblack-leaf.github.io/foliage/book/",
        sides: 6.0,
    },
    Destination {
        label: "github",
        icon: IconHandles::Github,
        href: "https://github.com/eblack-leaf/foliage",
        sides: 5.0,
    },
];

/// Builds the hero as the first child of the scroll container, sized to the whole first
/// screen so the hint sits exactly at the fold.
pub(crate) fn build(tree: &mut Tree, container: Entity) -> Entity {
    let seq = tree.sequence();
    let hero = tree.branch(
        container,
        Leaf::sprout()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.px().as_top().with(100.pct().as_height()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
    );
    // The hero bleeds full width, but its *content* has to clear the rail -- positioning in
    // thirds of the whole viewport put the first button underneath it.
    let hero = tree.branch(
        hero,
        Leaf::sprout()
            .at(Location::new()
                .xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    0.pct().as_top().with(100.pct().as_bottom()),
                )
                .md(
                    crate::site::shell::RAIL_W
                        .px()
                        .as_left()
                        .with(100.pct().as_right()),
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
            .size(FontSize::new(WORDMARK_XS).md(WORDMARK_MD))
            .color(Color::slate(role::ON_SURFACE))
            .glyph_colors(|i| {
                if i >= EXTENSION_AT {
                    Color::green(ACCENT)
                } else {
                    Color::slate(role::ON_SURFACE)
                }
            })
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                30.pct().as_center_y().with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(3))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                Opacity::new(0.0),
            )),
    );
    fade_in(tree, wordmark, seq, 0);

    let tagline = tree.branch(
        hero,
        Text::new(TAGLINE)
            .size(FontSize::new(type_scale::BODY))
            .color(Color::slate(role::ON_SURFACE_VARIANT))
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                anchor()
                    .bottom()
                    .as_top()
                    .adjust(space::MD)
                    .with(20.px().as_height()),
            ))
            .elevate(Elevation::up(3))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                Anchor::new(wordmark),
                Opacity::new(0.0),
            )),
    );
    fade_in(tree, tagline, seq, motion::STAGGER);

    for (i, dest) in DESTINATIONS.iter().enumerate() {
        destination(tree, hero, dest, i, seq);
    }
    hint(tree, hero, seq);
    hero
}

/// A destination: shadow polygon behind, front polygon on top, icon centred in it, label
/// beneath. The offset shadow is what gives it depth without a blur.
fn destination(tree: &mut Tree, hero: Entity, dest: &Destination, index: usize, seq: Entity) {
    let third = 100.0 / DESTINATIONS.len() as f32;
    let center = third * index as f32 + third / 2.0;
    let start = motion::STAGGER * (2 + index as u64);
    // One hue each: three identical green shapes read as a set of the same thing, where the
    // point is that they go to three different places.
    let face = match index {
        0 => Color::green(ACCENT),
        1 => Color::cyan(ACCENT),
        _ => Color::orange(ACCENT),
    };

    let shadow = tree.branch(
        hero,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(-0.16)
            .color(Color::slate(role::SURFACE))
            .at(Location::new().xs(
                center
                    .pct()
                    .as_center_x()
                    .adjust(-SHADOW_OFF)
                    .with(BUTTON.px().as_width()),
                BUTTONS_TOP_PCT
                    .pct()
                    .as_center_y()
                    .adjust(SHADOW_OFF)
                    .with(BUTTON.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with(Opacity::new(0.0)),
    );
    morph_in(tree, shadow, seq, dest.sides, 0.15, start);

    let button = tree.branch(
        hero,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(-0.16)
            .color(face)
            .at(Location::new().xs(
                center.pct().as_center_x().with(BUTTON.px().as_width()),
                BUTTONS_TOP_PCT
                    .pct()
                    .as_center_y()
                    .with(BUTTON.px().as_height()),
            ))
            .elevate(Elevation::up(3))
            .with((
                InteractionListener::new(),
                InteractionShape::Circle,
                Opacity::new(0.0),
            )),
    );
    morph_in(tree, button, seq, dest.sides, 0.15, start);
    let href = dest.href;
    tree.on_click(button, move |_: Trigger<OnClick>, _: Tree| {
        HrefLink::new(href).navigate();
    });

    let icon = tree.branch(
        hero,
        Icon::new(IconId::from(dest.icon))
            .color(Color::gray(950))
            .at(Location::new().xs(
                anchor()
                    .center_x()
                    .as_center_x()
                    .with((anchor().width() * ICON_SCALE).as_width()),
                anchor()
                    .center_y()
                    .as_center_y()
                    .with((anchor().height() * ICON_SCALE).as_height()),
            ))
            .elevate(Elevation::up(4))
            .with((
                Anchor::new(button),
                // the icon draws above the button, so without this it wins the hit-test and
                // swallows the click meant for the shape under it
                InteractionPropagation::pass_through(),
                Opacity::new(0.0),
            )),
    );
    fade_in(tree, icon, seq, start);

    let label = tree.branch(
        hero,
        Text::new(dest.label)
            .size(FontSize::new(type_scale::TITLE))
            .color(Color::slate(role::ON_SURFACE_VARIANT))
            .at(Location::new().xs(
                center.pct().as_center_x().with(90.px().as_width()),
                anchor()
                    .bottom()
                    .as_top()
                    .adjust(space::SM)
                    .with(BUTTON_LABEL_H.px().as_height()),
            ))
            .elevate(Elevation::up(3))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                Anchor::new(button),
                Opacity::new(0.0),
            )),
    );
    fade_in(tree, label, seq, start);
}

/// "more", with a chevron breathing under it at the fold.
fn hint(tree: &mut Tree, hero: Entity, seq: Entity) {
    let label = tree.branch(
        hero,
        Text::new(HINT_TEXT)
            .size(FontSize::new(type_scale::TITLE))
            .color(Color::slate(role::ON_SURFACE_VARIANT))
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                100.pct()
                    .as_bottom()
                    .adjust(-(CHEVRON + space::XL))
                    .with(18.px().as_height()),
            ))
            .elevate(Elevation::up(3))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                Opacity::new(0.0),
            )),
    );
    fade_in(tree, label, seq, motion::STAGGER * 5);

    let chevron = tree.branch(
        hero,
        Icon::new(IconId::from(IconHandles::ChevronDown))
            .color(Color::green(ACCENT))
            .at(Location::new().xs(
                50.pct().as_center_x().with(CHEVRON.px().as_width()),
                100.pct()
                    .as_bottom()
                    .adjust(-(CHEVRON + space::MD))
                    .with(CHEVRON.px().as_height()),
            ))
            .elevate(Elevation::up(3))
            .with(Opacity::new(0.0)),
    );
    fade_in(tree, chevron, seq, motion::STAGGER * 6);

    // Its own sequence, forever, backtracking so it returns rather than snapping -- a plain
    // loop would jerk back to the top every cycle. Tied to the chevron's own lifetime, so
    // leaving the route stops it.
    let bob = tree.sequence();
    tree.animate(
        Animation::new(Location::new().xs(
            50.pct().as_center_x().with(CHEVRON.px().as_width()),
            100.pct()
                .as_bottom()
                .adjust(-(CHEVRON + space::MD) + BOB_PX)
                .with(CHEVRON.px().as_height()),
        ))
        .targeting(chevron)
        .during(bob)
        .start(motion::STAGGER * 7)
        .finish(motion::STAGGER * 7 + BOB_MS)
        .eased(Ease::DECELERATE)
        .forever()
        .backtrack(),
    );
}
