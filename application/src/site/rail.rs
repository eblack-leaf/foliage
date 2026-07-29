//! Section navigation. Peers, not a sequence -- there is no prev/next here on purpose.

use foliage::{
    Color, EcsExtension, Elevation, Entity, FontSize, GridExt, HorizontalAlignment, Location,
    OnClick, PageIndex, Panel, Rounding, Sprout, Text, Tree, Trigger, VerticalAlignment,
};

use crate::site::shell::rail_surface;
use crate::site::{ACCENT, role, space, type_scale};

/// One entry per route, in the order `entry.rs` registers them.
pub(crate) const SECTIONS: [&str; 5] = ["overview", "layout", "motion", "composites", "text"];

const ENTRY_H: i32 = 40;
const ENTRY_GAP: i32 = space::XS;
const FIRST_ENTRY_TOP: i32 = 96;

/// Builds the rail and wires each entry to its route.
pub(crate) fn build(tree: &mut Tree, parent: Entity, router: Entity, active: usize) {
    let surface = rail_surface(tree, parent);
    for (index, name) in SECTIONS.iter().enumerate() {
        let top = FIRST_ENTRY_TOP + index as i32 * (ENTRY_H + ENTRY_GAP);
        let is_active = index == active;
        // the active entry gets a filled pill behind it -- M3's rail indicator. Inactive
        // entries are label-only, so the accent stays scarce enough to mean something.
        if is_active {
            tree.branch(
                surface,
                Panel::new()
                    .color(Color::green(ACCENT))
                    .rounding(Rounding::Full)
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
                    Color::gray(950)
                } else {
                    Color::slate(role::ON_SURFACE_VARIANT)
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
            tree.write_to(router, PageIndex(index));
        });
    }
}
