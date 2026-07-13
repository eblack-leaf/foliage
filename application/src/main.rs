#![allow(unused)]

use foliage::{load_asset, Foliage, GridExt};

mod home;
#[path = "assets/icons/generated.rs"]
mod icons;
mod portfolio;
mod widgets;

fn main() {
    let mut foliage = Foliage::new();
    // foliage.enable_tracing(
    //     tracing_subscriber::filter::Targets::new().with_target("foliage", tracing::Level::TRACE),
    // );
    foliage.desktop_size((360, 800));
    foliage.url("foliage");
    let music_player = load_asset!(foliage, "assets/music-player.png");
    let artist_blog = load_asset!(foliage, "assets/artist-blog.png");
    let album_cover = load_asset!(foliage, "assets/album-cover.jpg");
    foliage.store(music_player, "music-player");
    foliage.store(artist_blog, "artist-blog");
    foliage.store(album_cover, "album-cover");
    icons::register(&mut foliage);
    home::build(&mut foliage.world);
    foliage.photosynthesize(); // run
}
