//! `Popover` -- tap-triggered (no hover concept), opens to the right of its trigger. The
//! trigger content is a plain circular `Panel` (no `Icon`, matching this example's no-icon
//! convention) with `.pass_through()` so the click actually reaches the popover's own root
//! instead of being grabbed by the panel itself -- the exact requirement the module's own
//! doc comment calls out. Run with `cargo run --example popover -p foliage`.

use foliage::{
    Color, EcsExtension, Elevation, Entity, Foliage, FontSize, GridExt, HorizontalAlignment,
    InteractionPropagation, Location, Panel, Popover, PopoverPlacement, Rounding, Sprout, Text,
    Tree, VerticalAlignment,
};

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((260, 120));

    foliage.world.leaf(
        Popover::new()
            .trigger(|tree: &mut Tree, slot: Entity| {
                tree.branch(
                    slot,
                    Panel::new()
                        .rounding(Rounding::Full)
                        .color(Color::gray(700))
                        .at(Location::new().xs(
                            0.px().as_left().with(44.px().as_width()),
                            50.pct().as_center_y().with(44.px().as_height()),
                        ))
                        .elevate(Elevation::up(1))
                        .with(InteractionPropagation::pass_through()),
                )
            })
            .content(|tree: &mut Tree, slot: Entity| {
                tree.branch(
                    slot,
                    Text::new("popover content")
                        .size(FontSize::new(14))
                        .color(Color::gray(200))
                        .at(Location::new().xs(
                            8.px().as_left().with(100.pct().as_right()),
                            0.pct().as_top().with(100.pct().as_bottom()),
                        ))
                        .elevate(Elevation::up(1))
                        .with((
                            HorizontalAlignment::Center,
                            VerticalAlignment::Middle,
                            InteractionPropagation::pass_through(),
                        )),
                )
            })
            .placement(PopoverPlacement::Right)
            .extent(160.px())
            .colors(Color::gray(600))
            .at(Location::new().xs(
                20.px().as_left().with(44.px().as_width()),
                20.px().as_top().with(64.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    foliage.photosynthesize();
}
