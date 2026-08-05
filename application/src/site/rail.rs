//! Section navigation. Peers, not a sequence -- there is no prev/next here on purpose.

use foliage::{
    Bare, Canopy, Elevation, FontSize, GridExt, Grows, HorizontalAlignment, Icon, IconId, Leaf,
    Location, Panel, Rounding, Sprout, Text, VerticalAlignment,
};

use crate::icons::IconHandles;
use crate::site::shell::rail_surface;
use crate::site::{role, space, type_scale};

/// One entry per route, in the order the router registers them.
/// Order matters twice over: it is the reading order of the site, and the index into the
/// router's own route list (offset by one, since route 0 is the hero).
///
/// `leaf` sits second because it is the model the overview only gestures at -- everything
/// after it is a thing you do *to* a leaf, and none of it lands without that first.
///
/// `primitives` and `interaction` split what the framework actually ships: the types with a
/// renderer, and the ways an app hears about input. Assembly on top of the two is the app's job,
/// which is what this site is.
pub(crate) use crate::site::copy::rail::SECTIONS;

const ENTRY_H: i32 = 40;
const ENTRY_GAP: i32 = space::XS;
const FIRST_ENTRY_TOP: i32 = 96;

/// The brand mark, which is also the way back to the hero -- the convention everywhere, and
/// it fills the space the first entry already left above itself. Coloured like the hero's
/// wordmark so it reads as the same thing, smaller.
use crate::site::copy::rail::{BRAND, BRAND_EXTENSION_AT};

/// A chevron above the brand, pointing back the way the hero's own chevron pointed forward.
/// The pair reads as one gesture: down to enter, up to leave.
const BRAND_CHEVRON: i32 = 16;
const BRAND_CHEVRON_TOP: i32 = 16;
const BRAND_TOP: i32 = BRAND_CHEVRON_TOP + BRAND_CHEVRON + space::XS;
const BRAND_H: i32 = 26;
/// Separates the brand from the section list -- they do different things (one leaves the
/// site's body, the others move within it), and a hairline says so without a label.
const DIVIDER_TOP: i32 = BRAND_TOP + BRAND_H + space::MD;

/// What the rail currently is, so a click on one of its targets can be turned back into a
/// route and the whole thing torn down when the route changes.
///
/// Empty on the hero, which has no rail at all.
#[derive(Default)]
pub(crate) struct Rail {
    /// The one thing to prune to take the rail down -- everything else hangs off it.
    pub(crate) surface: Option<Leaf>,
    /// The brand block: the way back to the hero.
    back: Option<Leaf>,
    /// One per entry, in [`SECTIONS`] order.
    entries: Vec<Leaf>,
}

impl Rail {
    /// Which route `leaf` goes to, if it is one of the rail's own targets.
    pub(crate) fn route_for(&self, leaf: Leaf) -> Option<usize> {
        if self.back == Some(leaf) {
            return Some(0);
        }
        // sections are one-based in the router's list, since route 0 is the hero
        self.entries
            .iter()
            .position(|entry| *entry == leaf)
            .map(|index| index + 1)
    }
}

/// Builds the rail and records which route each entry reaches.
///
/// `active` is `None` on the hero route, which has no rail at all -- so nothing is built.
/// Section indices here are 0-based; the router's are one higher, since route 0 is the hero.
pub(crate) fn build(canopy: &mut Canopy, parent: Leaf, active: Option<usize>) -> Rail {
    let Some(active) = active else {
        return Rail::default();
    };
    let surface = rail_surface(canopy, parent);

    // One target covering the chevron and the wordmark together, rather than a listener on
    // each. They are one control -- "back to the hero" -- and as two they had two dead strips
    // between and around them where the obvious click did nothing. Unpainted: the rail's own
    // surface is the background here, so a panel would only be a shape to keep in sync.
    let back = canopy.branch(
        surface,
        Bare::new()
            .at(Location::new().xs(
                space::SM
                    .px()
                    .as_left()
                    .with(100.pct().as_right().adjust(-space::SM)),
                space::SM
                    .px()
                    .as_top()
                    .with((DIVIDER_TOP - space::SM).px().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .interactive(),
    );

    // both pass through, or each would win the hit-test on the pixels it covers and split the
    // one control back into three
    canopy.branch(
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
            .pass_through(),
    );

    canopy.branch(
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
            .align(HorizontalAlignment::Left, VerticalAlignment::Middle)
            .pass_through(),
    );

    canopy.branch(
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

    let mut entries = Vec::with_capacity(SECTIONS.len());
    for (index, name) in SECTIONS.iter().enumerate() {
        let top = FIRST_ENTRY_TOP + index as i32 * (ENTRY_H + ENTRY_GAP);
        let is_active = index == active;
        // the active entry gets a filled pill behind it -- M3's rail indicator. Inactive
        // entries are label-only, so the accent stays scarce enough to mean something.
        if is_active {
            canopy.branch(
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
        entries.push(canopy.branch(
            surface,
            Text::new(*name)
                .size(FontSize::new(type_scale::TITLE))
                // Not the variant tone: these sit on `surface()`, which is a good deal lighter
                // than the page, and the variant reads washed out against it. What separates an
                // inactive entry from the active one is the pill behind it, not a dimmer label.
                .color(if is_active {
                    role::on_accent()
                } else {
                    role::on_surface()
                })
                .at(Location::new().xs(
                    space::MD
                        .px()
                        .as_left()
                        .with(100.pct().as_right().adjust(-space::SM)),
                    top.px().as_top().with(ENTRY_H.px().as_height()),
                ))
                .elevate(Elevation::up(2))
                .align(HorizontalAlignment::Left, VerticalAlignment::Middle)
                // the entry *is* the target -- without this it is drawn and never grabbed, so
                // every section but the one the rail already shows was unreachable
                .interactive(),
        ));
    }

    Rail {
        surface: Some(surface),
        back: Some(back),
        entries,
    }
}
