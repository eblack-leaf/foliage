#![allow(unused)]

use foliage::{Foliage, GridExt, bundled_asset};

mod home;
#[path = "assets/icons/gen/generated.rs"]
mod icons;
mod portfolio;
mod widgets;

/// This app's own hosting convention -- `foliage_proper` makes no assumption about it, so it
/// lives here, the one place that actually knows where these assets are served from.
#[cfg(target_family = "wasm")]
fn asset_url(path: &str) -> String {
    format!("{}/foliage/{path}", Foliage::window_origin())
}

fn main() {
    let mut foliage = Foliage::new();
    // foliage.enable_tracing(
    //     tracing_subscriber::filter::Targets::new()
    //         .with_target("foliage_proper::grid::view", tracing::Level::TRACE)
    //         .with_default(tracing_subscriber::filter::LevelFilter::OFF),
    // );
    foliage.desktop_size((360, 800));

    let music_player = bundled_asset!(foliage, "assets/music-player.png", asset_url);
    foliage.store(music_player, "music-player");
    let artist_blog = bundled_asset!(foliage, "assets/artist-blog.png", asset_url);
    foliage.store(artist_blog, "artist-blog");
    let album_cover = bundled_asset!(foliage, "assets/album-cover.jpg", asset_url);
    foliage.store(album_cover, "album-cover");

    icons::register(&mut foliage);
    home::build(&mut foliage.world);
    // perf probe for Polyline's teardown-and-rebuild-per-write pattern -- see
    // portfolio::composites::drive_polyline_draw. `foliage.user` is the app-code schedule,
    // run every tick alongside (but separately timed from) the engine's own `main`/`diff`.
    foliage
        .user
        .add_systems(portfolio::composites::drive_polyline_draw);
    foliage.photosynthesize(); // run
}
