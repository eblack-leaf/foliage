//! Section navigation. Peers, not a sequence -- there is no prev/next here on purpose.

use foliage::{
    EcsExtension, Elevation, Entity, FontSize, GridExt, HorizontalAlignment, Icon, IconId,
    InteractionListener, Location, OnClick, PageIndex, Panel, Rounding, Sprout, Text, Tree,
    Trigger, VerticalAlignment,
};

use crate::icons::IconHandles;
use crate::site::shell::rail_surface;
use crate::site::{role, space, type_scale};

/// One entry per route, in the order `entry.rs` registers them.
pub(crate) const SECTIONS: [&str; 5] = ["overview", "layout", "motion", "composites", "text"];

const ENTRY_H: i32 = 40;
const ENTRY_GAP: i32 = space::XS;
const FIRST_ENTRY_TOP: i32 = 96;

/// The brand mark, which is also the way back to the hero -- the convention everywhere, and
/// it fills the space the first entry already left above itself. Coloured like the hero's
/// wordmark so it reads as the same thing, smaller.
const BRAND: &str = "foliage.rs";
const BRAND_EXTENSION_AT: usize = 7;
/// A chevron above the brand, pointing back the way the hero's own chevron pointed forward.
/// The pair reads as one gesture: down to enter, up to leave.
const BRAND_CHEVRON: i32 = 16;
const BRAND_CHEVRON_TOP: i32 = 16;
const BRAND_TOP: i32 = BRAND_CHEVRON_TOP + BRAND_CHEVRON + space::XS;
const BRAND_H: i32 = 26;
/// Separates the brand from the section list -- they do different things (one leaves the
/// site's body, the others move within it), and a hairline says so without a label.
const DIVIDER_TOP: i32 = BRAND_TOP + BRAND_H + space::MD;

/// Builds the rail and wires each entry to its route.
///
/// `active` is `None` on the hero route, which has no rail at all -- so nothing is built.
/// Section indices here are 0-based; the router's are one higher, since route 0 is the hero.
pub(crate) fn build(tree: &mut Tree, parent: Entity, router: Entity, active: Option<usize>) {
    let Some(active) = active else {
        return;
    };
    let surface = rail_surface(tree, parent);

    let chevron = tree.branch(
        surface,
        Icon::new(IconId::from(IconHandles::ChevronUp))
            .color(role::accent())
            .at(Location::new().xs(
                50.pct().as_center_x().with(BRAND_CHEVRON.px().as_width()),
                BRAND_CHEVRON_TOP
                    .px()
                    .as_top()
                    .with(BRAND_CHEVRON.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with(InteractionListener::new()),
    );
    tree.on_click(chevron, move |_: Trigger<OnClick>, mut tree: Tree| {
        tree.write_to(router, PageIndex(0));
    });

    let brand = tree.branch(
        surface,
        Text::new(BRAND)
            .size(FontSize::new(type_scale::TITLE))
            .color(role::on_surface())
            .glyph_colors(|i| {
                if i >= BRAND_EXTENSION_AT {
                    role::accent()
                } else {
                    role::on_surface()
                }
            })
            .at(Location::new().xs(
                space::MD
                    .px()
                    .as_left()
                    .with(100.pct().as_right().adjust(-space::SM)),
                BRAND_TOP.px().as_top().with(BRAND_H.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Left,
                VerticalAlignment::Middle,
                InteractionListener::new(),
            )),
    );
    tree.on_click(brand, move |_: Trigger<OnClick>, mut tree: Tree| {
        tree.write_to(router, PageIndex(0));
    });

    tree.branch(
        surface,
        Panel::new()
            .color(role::outline())
            .rounding(Rounding::None)
            .at(Location::new().xs(
                space::MD
                    .px()
                    .as_left()
                    .with(100.pct().as_right().adjust(-space::MD)),
                DIVIDER_TOP.px().as_top().with(1.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );

    for (index, name) in SECTIONS.iter().enumerate() {
        let top = FIRST_ENTRY_TOP + index as i32 * (ENTRY_H + ENTRY_GAP);
        let is_active = index == active;
        // the active entry gets a filled pill behind it -- M3's rail indicator. Inactive
        // entries are label-only, so the accent stays scarce enough to mean something.
        if is_active {
            tree.branch(
                surface,
                Panel::new()
                    .color(role::accent())
                    // `Full` on a 40px-tall pill is a 20px radius -- a stadium. `Sm` (0.3 of
                    // the short side) reads as a rounded rectangle instead.
                    .rounding(Rounding::Sm)
                    .at(Location::new().xs(
                        space::SM
                            .px()
                            .as_left()
                            .with(100.pct().as_right().adjust(-space::SM)),
                        top.px().as_top().with(ENTRY_H.px().as_height()),
                    ))
                    .elevate(Elevation::up(1)),
            );
        }
        let label = tree.branch(
            surface,
            Text::new(*name)
                .size(FontSize::new(type_scale::TITLE))
                .color(if is_active {
                    role::on_accent()
                } else {
                    role::on_surface_variant()
                })
                .at(Location::new().xs(
                    space::MD
                        .px()
                        .as_left()
                        .with(100.pct().as_right().adjust(-space::SM)),
                    top.px().as_top().with(ENTRY_H.px().as_height()),
                ))
                .elevate(Elevation::up(2))
                .with((HorizontalAlignment::Left, VerticalAlignment::Middle)),
        );
        tree.on_click(label, move |_: Trigger<OnClick>, mut tree: Tree| {
            tree.write_to(router, PageIndex(index + 1));
        });
    }
}
