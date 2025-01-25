#![allow(unused)]

use crate::docs::Docs;
use crate::home::Home;
use crate::icons::IconHandles;
use crate::implementation::Implementation;
use crate::portfolio::Portfolio;
use crate::usage::Usage;
use foliage::{load_asset, Foliage, Icon, Image};

mod docs;
mod home;
mod icons;
mod implementation;
mod portfolio;
mod usage;

fn main() {
    let mut foliage = Foliage::new();
    foliage.enable_tracing(
        tracing_subscriber::filter::Targets::new().with_target("foliage", tracing::Level::TRACE),
    );
    foliage.desktop_size((360, 800));
    foliage.url("foliage");
    foliage.attach::<Home>();
    foliage.attach::<Usage>();
    foliage.attach::<Portfolio>();
    foliage.attach::<Implementation>();
    foliage.attach::<Docs>();
    let music_player = load_asset!(foliage, "assets/music-player.png");
    foliage.world.spawn(Image::memory(0, (689, 591)));
    let artist_blog = load_asset!(foliage, "assets/artist-blog.png");
    foliage.world.spawn(Image::memory(1, (1298, 785)));
    foliage.world.spawn(Icon::memory(
        IconHandles::Box.value(),
        include_bytes!("assets/icons/box.icon"),
    ));
    foliage.world.spawn(Icon::memory(
        IconHandles::Code.value(),
        include_bytes!("assets/icons/code.icon"),
    ));
    foliage.world.spawn(Icon::memory(
        IconHandles::BookOpen.value(),
        include_bytes!("assets/icons/book-open.icon"),
    ));
    foliage.world.spawn(Icon::memory(
        IconHandles::Layers.value(),
        include_bytes!("assets/icons/layers.icon"),
    ));
    foliage.world.spawn(Icon::memory(
        IconHandles::Terminal.value(),
        include_bytes!("assets/icons/terminal.icon"),
    ));
    foliage.world.spawn(Icon::memory(
        IconHandles::Github.value(),
        include_bytes!("assets/icons/github.icon"),
    ));
    foliage.store(music_player, "music-player");
    foliage.store(artist_blog, "artist-blog");
    foliage.send(Home {});
    foliage.photosynthesize(); // run
}
