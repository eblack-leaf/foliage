#![allow(unused)]

use foliage::{load_asset, Foliage, GridExt};

mod home;
#[path = "assets/icons/gen/generated.rs"]
mod icons;
mod portfolio;
mod widgets;

fn main() {
    let mut foliage = Foliage::new();
    // foliage.enable_tracing(
    //     tracing_subscriber::filter::Targets::new()
    //         .with_target(
    //             "foliage_proper::composite::text_input",
    //             tracing::Level::TRACE,
    //         )
    //         .with_target("foliage_proper::grid::view", tracing::Level::TRACE)
    //         .with_target("foliage_proper::text", tracing::Level::TRACE)
    //         // "foliage_proper::text" above is a prefix match, so it also swept in
    //         // "foliage_proper::text::pipeline" -- that one fires every render frame
    //         // (unrelated to the resize/scroll bug), which is what was flooding the output.
    //         // A more specific target wins over the less specific one, so this carves it
    //         // back out without losing Text::update's own tracing.
    //         .with_target(
    //             "foliage_proper::text::pipeline",
    //             tracing_subscriber::filter::LevelFilter::OFF,
    //         )
    //         .with_default(tracing_subscriber::filter::LevelFilter::OFF),
    // );
    foliage.desktop_size((360, 800));
    foliage.url("foliage");
    load_asset!(foliage, "assets/music-player.png", "music-player");
    load_asset!(foliage, "assets/artist-blog.png", "artist-blog");
    load_asset!(foliage, "assets/album-cover.jpg", "album-cover");
    icons::register(&mut foliage);
    home::build(&mut foliage.world);
    foliage.photosynthesize(); // run
}
