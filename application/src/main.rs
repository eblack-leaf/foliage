#![allow(unused)]

use foliage::{
    Color, Elevation, Foliage, GridExt, LineConstraint, Location, Primary, Secondary, Stem,
    Tertiary, TextInput,
};

mod home;
mod icons;

mod portfolio;
fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((600, 400));
    foliage.leaf((
        TextInput::new(),
        Location::new().xs(
            10.px().left().with(400.px().right()),
            10.px().top().with(40.px().height()),
        ),
        Primary(Color::gray(200)),
        Secondary(Color::gray(800)),
        Tertiary(Color::gray(500)),
        Elevation::abs(0),
        Stem::none(),
    ));
    foliage.leaf((
        TextInput::new(),
        LineConstraint::Multiple,
        Location::new().xs(
            10.px().left().with(400.px().right()),
            200.px().top().with(100.px().height()),
        ),
        Primary(Color::gray(200)),
        Secondary(Color::gray(800)),
        Tertiary(Color::gray(500)),
        Elevation::abs(0),
        Stem::none(),
    ));
    foliage.photosynthesize();
}

// fn main() {
//     let mut foliage = Foliage::new();
//     foliage.enable_tracing(
//         tracing_subscriber::filter::Targets::new().with_target("foliage", tracing::Level::TRACE),
//     );
//     foliage.desktop_size((360, 800));
//     foliage.url("foliage");
//     foliage.attach::<Home>();
//     foliage.attach::<Portfolio>();
//     foliage.attach::<MusicPlayer>();
//     let music_player = load_asset!(foliage, "assets/music-player.png");
//     foliage.world.spawn(Image::memory(0, (569, 419)));
//     let artist_blog = load_asset!(foliage, "assets/artist-blog.png");
//     foliage.world.spawn(Image::memory(1, (1298, 785)));
//     let album_cover = load_asset!(foliage, "assets/album-cover.jpg");
//     foliage.world.spawn(Image::memory(2, (1800, 1800)));
//     foliage.store(music_player, "music-player");
//     foliage.store(artist_blog, "artist-blog");
//     foliage.store(album_cover, "album-cover");
//     foliage.world.spawn(Icon::memory(
//         IconHandles::Box.value(),
//         include_bytes!("assets/icons/box.icon"),
//     ));
//     foliage.world.spawn(Icon::memory(
//         IconHandles::Code.value(),
//         include_bytes!("assets/icons/code.icon"),
//     ));
//     foliage.world.spawn(Icon::memory(
//         IconHandles::BookOpen.value(),
//         include_bytes!("assets/icons/book-open.icon"),
//     ));
//     foliage.world.spawn(Icon::memory(
//         IconHandles::Layers.value(),
//         include_bytes!("assets/icons/layers.icon"),
//     ));
//     foliage.world.spawn(Icon::memory(
//         IconHandles::Terminal.value(),
//         include_bytes!("assets/icons/terminal.icon"),
//     ));
//     foliage.world.spawn(Icon::memory(
//         IconHandles::Github.value(),
//         include_bytes!("assets/icons/github.icon"),
//     ));
//     foliage.world.spawn(Icon::memory(
//         IconHandles::ArrowUp.value(),
//         include_bytes!("assets/icons/arrow-up.icon"),
//     ));
//     foliage.world.spawn(Icon::memory(
//         IconHandles::X.value(),
//         include_bytes!("assets/icons/x.icon"),
//     ));
//     foliage.world.spawn(Icon::memory(
//         IconHandles::Menu.value(),
//         include_bytes!("assets/icons/menu.icon"),
//     ));
//     foliage.world.spawn(Icon::memory(
//         IconHandles::Play.value(),
//         include_bytes!("assets/icons/play-circle.icon"),
//     ));
//     foliage.world.spawn(Icon::memory(
//         IconHandles::Shuffle.value(),
//         include_bytes!("assets/icons/shuffle.icon"),
//     ));
//     foliage.world.spawn(Icon::memory(
//         IconHandles::Repeat.value(),
//         include_bytes!("assets/icons/repeat.icon"),
//     ));
//     foliage.world.spawn(Icon::memory(
//         IconHandles::SkipLeft.value(),
//         include_bytes!("assets/icons/chevrons-left.icon"),
//     ));
//     foliage.world.spawn(Icon::memory(
//         IconHandles::SkipRight.value(),
//         include_bytes!("assets/icons/chevrons-right.icon"),
//     ));
//     foliage.world.spawn(Icon::memory(
//         IconHandles::Search.value(),
//         include_bytes!("assets/icons/search.icon"),
//     ));
//     foliage.send(Home {});
//     foliage.photosynthesize(); // run
// }
