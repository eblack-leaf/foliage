//! `Tabs` -- header is a real `SegmentedControl` composed in, content slots swap
//! `Visibility` on switch rather than respawning. Adapted from
//! `application/src/portfolio/composites.rs`, with the per-tab `Icon` dropped (no
//! registered icon bytes here). Run with `cargo run --example tabs -p foliage`.

use foliage::{
    Color, EcsExtension, Elevation, Entity, Foliage, FontSize, GridExt, HorizontalAlignment,
    Location, Panel, Rounding, Sprout, Tabs, TabsPages, Text, Tree,
};

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((320, 220));

    foliage.world.leaf(
        Tabs::new()
            .pages(TabsPages::new(
                vec!["Tab 1".into(), "Tab 2".into(), "Tab 3".into()],
                |tree: &mut Tree, slot: Entity, i| {
                    let backing = [Color::teal(700), Color::indigo(700), Color::gray(500)];
                    tree.branch(
                        slot,
                        Panel::new()
                            .color(backing[i % backing.len()])
                            .at(Location::new().xs(
                                0.pct().as_left().with(100.pct().as_right()),
                                0.pct().as_top().with(100.pct().as_bottom()),
                            ))
                            .elevate(Elevation::up(1)),
                    );
                    tree.branch(
                        slot,
                        Text::new(format!("content for tab {}", i + 1))
                            .size(FontSize::new(16))
                            .color(Color::gray(200))
                            .at(Location::new().xs(
                                8.px().as_left().with(100.pct().as_right().adjust(-8)),
                                12.px().as_top().with(28.px().as_height()),
                            ))
                            .elevate(Elevation::up(2))
                            .with(HorizontalAlignment::Center),
                    );
                },
            ))
            .colors(Color::green(500), Color::gray(600), Color::gray(900))
            .rounding(Rounding::Sm)
            .at(Location::new().xs(
                8.px().as_left().with(100.pct().as_right().adjust(-8)),
                20.px().as_top().with(180.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    foliage.photosynthesize();
}
