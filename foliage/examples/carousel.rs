//! `Carousel` with an embedded `Pagination` dot strip. Page content adapted from
//! `application/src/portfolio/composites.rs` (plain colored backing + a page-number label,
//! no artwork/icons). Run with `cargo run --example carousel -p foliage`.
//!
//! Two real mistakes this file had, both worth keeping the fix visible for:
//! - Page content (`Panel`/`Text`) had no `InteractionPropagation::pass_through()`, so it
//!   competed for clicks with the embedded `Pagination`'s dots -- both happened to resolve
//!   to the identical absolute elevation (same total `up(..)` depth, different order), a
//!   real tie broken by ECS query iteration order rather than anything deliberate. Without
//!   `pass_through()`, decorative content can silently steal a click meant for something
//!   else at the same elevation.
//! - The root itself used `Elevation::up(1)`. A stem-less root (spawned at world root, no
//!   parent) has nothing to resolve `up`/`down` against -- `coordinate/elevation.rs`
//!   documents the fallback (treats the missing parent as `abs(0)`) and its own comment
//!   states every existing stem-less root in this codebase uses `abs()`, never `up`/`down`.
//!   `Elevation::abs(0)` is the correct, conventional choice here.

use foliage::{
    Carousel, CarouselPages, Color, EcsExtension, Elevation, Entity, Foliage, FontSize, GridExt,
    HorizontalAlignment, Location, PaginationMode, Panel, Sprout, Text, Tree, VerticalAlignment,
};

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((320, 290));

    foliage.world.leaf(
        Carousel::new()
            .pages(CarouselPages::new(3, |tree: &mut Tree, slot: Entity, i| {
                let backing = [Color::blue(700), Color::orange(700), Color::gray(500)];
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
                    Text::new(format!("page {}", i + 1))
                        .size(FontSize::new(16))
                        .color(Color::gray(200))
                        .at(Location::new().xs(
                            0.pct().as_left().with(100.pct().as_right()),
                            0.pct().as_top().with(100.pct().as_bottom()),
                        ))
                        .elevate(Elevation::up(2))
                        .with((HorizontalAlignment::Center, VerticalAlignment::Middle)),
                );
            }))
            .pagination(PaginationMode::Dots)
            .colors(Color::green(300), Color::gray(600))
            .at(Location::new().xs(
                8.px().as_left().with(304.px().as_width()),
                20.px().as_top().with(180.px().as_bottom()),
            ))
            .elevate(Elevation::abs(0)),
    );

    foliage.photosynthesize();
}
