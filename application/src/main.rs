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
    //         .with_target("foliage_proper::grid::view", tracing::Level::TRACE)
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
