//! `Dropdown` + `Pagination` (Dots and Numbered variants). Configs adapted from
//! `application/src/portfolio/composites.rs`, with the icon-bearing knobs (`.chevron(..)`,
//! `.step_icons(..)`) dropped -- both are genuinely optional (the library ships no icons of
//! its own), and this example has no registered icon bytes to point them at. Run with
//! `cargo run --example dropdown_and_pagination -p foliage`.

use foliage::{
    Color, Dropdown, EcsExtension, Elevation, Foliage, GridExt, Location, Pagination,
    PaginationMode, Sprout,
};

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((280, 300));

    foliage.world.leaf(
        Dropdown::new()
            .options(["Option 1", "Option 2", "Option 3"])
            .colors(Color::gray(200), Color::gray(900), Color::green(600))
            .at(Location::new().xs(
                8.px().as_left().with(220.px().as_width()),
                20.px().as_top().with(56.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    foliage.world.leaf(
        Pagination::new(5)
            .mode(PaginationMode::Dots)
            .colors(Color::green(300), Color::gray(600))
            .at(Location::new().xs(
                8.px().as_left().with(120.px().as_width()),
                100.px().as_top().with(116.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    foliage.world.leaf(
        Pagination::new(20)
            .mode(PaginationMode::Numbered)
            .colors(Color::green(300), Color::gray(600))
            .at(Location::new().xs(
                8.px().as_left().with(240.px().as_width()),
                160.px().as_top().with(196.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    foliage.photosynthesize();
}
