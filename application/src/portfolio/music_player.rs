use crate::icons::IconHandles;
use foliage::Justify::{Center, Far, Near};
use foliage::{bevy_ecs, Animation, Attachment, Button, Color, EcsExtension, Elevation, Event, Foliage, FontSize, Grid, GridExt, HorizontalAlignment, IconValue, Image, ImageView, Keyring, Location, OnClick, Opacity, Outline, Panel, Primary, Res, Rounding, Secondary, Stem, Text, Tree, Trigger, VerticalAlignment};

#[derive(Event)]
pub(crate) struct MusicPlayer {}
impl Attachment for MusicPlayer {
    fn attach(foliage: &mut Foliage) {
        foliage.define(MusicPlayer::init);
    }
}
impl MusicPlayer {
    pub(crate) fn init(trigger: Trigger<Self>, mut tree: Tree, keyring: Res<Keyring>) {
        let app = trigger.entity();
        tree.entity(app).insert((
            Panel::new(),
            Elevation::up(1),
            Grid::new(12.col().gap(8), 40.px().gap(8)),
            Color::gray(900),
        ));
        let seq = tree.sequence();
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .start(1000)
                .finish(1500)
                .during(seq)
                .targeting(app),
        );
        let menu = tree.leaf((
            Button::new(),
            IconValue(IconHandles::Menu.value()),
            Elevation::up(1),
            Location::new().xs(
                100.pct().right().adjust(-16).with(40.px().width()),
                16.px().top().with(40.px().height()),
            ),
            Stem::some(app),
            Primary(Color::gray(200)),
            Secondary(Color::gray(800)),
            Rounding::Full,
        ));
        tree.on_click(menu, move |trigger: Trigger<OnClick>| {
            // nothing so far
        });
        let search = tree.leaf((
            Panel::new(),
            Rounding::Md,
            Elevation::up(1),
            Stem::some(app),
            Outline::new(2),
            Color::gray(300),
            Location::new().xs(
                3.col()
                    .left()
                    .with(100.pct().right().adjust(-72))
                    .max(400.0)
                    .justify(Far),
                16.px().top().with(40.px().height()),
            ),
        ));
        let album_cover = tree.leaf((
            Image::new(2, keyring.get("album-cover")),
            ImageView::Aspect,
            Elevation::up(1),
            Location::new().xs(
                1.col().left().with(12.col().right()).max(600.0).justify(Center),
                3.row().top().with(10.row().bottom()),
            ),
            Stem::some(app),
        ));
        let song_info = tree.leaf((
            Location::new().xs(
                1.col().left().with(12.col().right()).max(600.0),
                11.row().top().with(13.row().bottom()),
            ),
            Elevation::up(1),
            Grid::new(1.col().gap(12), 2.row().gap(8)),
            Stem::some(app),
        ));
        let artist_name = tree.leaf((
            Stem::some(song_info),
            Elevation::up(1),
            Location::new().xs(
                1.col().left().with(1.col().right()),
                1.row().top().with(1.row().bottom()),
            ),
            Text::new("Shiners in the City"),
            FontSize::new(24),
            VerticalAlignment::Middle,
            HorizontalAlignment::Center,
        ));
        let song_name = tree.leaf((
            Stem::some(song_info),
            Elevation::up(1),
            Location::new().xs(
                1.col().left().with(1.col().right()),
                2.row().top().with(2.row().bottom()),
            ),
            Text::new("A Walk in the Moonlight"),
            FontSize::new(16),
            VerticalAlignment::Middle,
            HorizontalAlignment::Center,
        ));
        let controls = tree.leaf((
            Stem::some(app),
            Elevation::up(1),
            Location::new().xs(
                1.col().left().with(12.col().right()).max(400.0),
                14.row().top().with(60.px().height()),
            ),
            Panel::new(),
            Color::gray(900),
            Grid::new(5.col().gap(8), 1.row().gap(8)),
        ));
        let play_pause = tree.leaf((
            Stem::some(controls),
            Button::new(),
            Elevation::up(1),
            Primary(Color::gray(200)),
            Secondary(Color::green(700)),
            IconValue(IconHandles::Play.value()),
            Rounding::Full,
            Location::new().xs(
                3.col().center_x().with(48.px().width()),
                1.row().center_y().with(48.px().height()),
            )
        ));
        let shuffle = tree.leaf((
            Stem::some(controls),
            Button::new(),
            Elevation::up(1),
            Primary(Color::gray(200)),
            Secondary(Color::gray(900)),
            IconValue(IconHandles::Shuffle.value()),
            Rounding::Full,
            Location::new().xs(
                1.col().center_x().with(48.px().width()),
                1.row().center_y().with(48.px().height()),
            )
        ));
        let left = tree.leaf((
            Stem::some(controls),
            Button::new(),
            Elevation::up(1),
            Primary(Color::gray(200)),
            Secondary(Color::gray(900)),
            IconValue(IconHandles::SkipLeft.value()),
            Rounding::Full,
            Location::new().xs(
                2.col().center_x().with(48.px().width()),
                1.row().center_y().with(48.px().height()),
            )
        ));
        let right = tree.leaf((
            Stem::some(controls),
            Button::new(),
            Elevation::up(1),
            Primary(Color::gray(200)),
            Secondary(Color::gray(900)),
            IconValue(IconHandles::SkipRight.value()),
            Rounding::Full,
            Location::new().xs(
                4.col().center_x().with(48.px().width()),
                1.row().center_y().with(48.px().height()),
            )
        ));
        let repeat = tree.leaf((
            Stem::some(controls),
            Button::new(),
            Elevation::up(1),
            Primary(Color::gray(200)),
            Secondary(Color::gray(900)),
            IconValue(IconHandles::Repeat.value()),
            Rounding::Full,
            Location::new().xs(
                5.col().center_x().with(48.px().width()),
                1.row().center_y().with(48.px().height()),
            )
        ));
    }
}
