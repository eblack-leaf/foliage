//! `Card` with a placeholder visual in `main`, a title in `header`, and a description in
//! `desc`. Run with `cargo run --example card -p foliage`.

use foliage::{
    Card, Color, EcsExtension, Elevation, Entity, Foliage, FontSize, GridExt,
    HorizontalAlignment, Location, Panel, Sprout, Text, Tree, VerticalAlignment,
};

fn centered_text(text: &str, color: Color, size: u32) -> impl Sprout {
    Text::new(text)
        .size(FontSize::new(size))
        .color(color)
        .with((HorizontalAlignment::Center, VerticalAlignment::Middle))
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((300, 300));

    foliage.world.leaf(
        Card::new()
            .main(|tree: &mut Tree, slot: Entity| {
                tree.branch(
                    slot,
                    Panel::new()
                        .color(Color::gray(700))
                        .at(Location::new().xs(
                            0.pct().as_left().with(100.pct().as_right()),
                            0.pct().as_top().with(100.pct().as_bottom()),
                        ))
                        .elevate(Elevation::up(1)),
                )
            })
            .header(|tree: &mut Tree, slot: Entity| {
                tree.branch(
                    slot,
                    centered_text("Card Title", Color::gray(200), 16).at(Location::new().xs(
                        0.pct().as_left().with(100.pct().as_right()),
                        0.pct().as_top().with(100.pct().as_bottom()),
                    )).elevate(Elevation::up(1)),
                )
            })
            .desc(|tree: &mut Tree, slot: Entity| {
                tree.branch(
                    slot,
                    centered_text("A short description of this card.", Color::gray(400), 12).at(
                        Location::new().xs(
                            0.pct().as_left().with(100.pct().as_right()),
                            0.pct().as_top().with(100.pct().as_bottom()),
                        ),
                    ).elevate(Elevation::up(1)),
                )
            })
            .colors(Color::gray(800))
            .at(Location::new().xs(
                10.pct().as_left().with(90.pct().as_right()),
                10.pct().as_top().with(90.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    foliage.photosynthesize();
}
