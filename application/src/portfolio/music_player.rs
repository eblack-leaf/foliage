use crate::icons::IconHandles;
use foliage::Justify::Center;
use foliage::{
    bevy_ecs, stack, Animation, Attachment, Button, Color, EcsExtension, Elevation, Event, Foliage,
    FontSize, Grid, GridExt, HorizontalAlignment, Icon, IconValue, Image, ImageView, Keyring, Line,
    Location, OnClick, Opacity, Outline, Panel, Primary, Res, Rounding, Secondary, Stack, Stem,
    Tertiary, Text, TextInput, TextValue, Tree, Trigger, VerticalAlignment,
};

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
                100.pct().right().adjust(-16).with(48.px().width()),
                16.px().top().with(48.px().height()),
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
            Color::gray(400),
            Location::new().xs(
                50.pct().center_x().with(60.pct().width()).max(400.0),
                16.px().top().with(44.px().height()),
            ),
            Grid::new(1.col(), 1.row()),
        ));
        let search_icon = tree.leaf((
            Icon::new(IconHandles::Search.value()),
            Elevation::up(1),
            Stem::some(search),
            Location::new().xs(
                8.px().left().with(24.px().width()),
                50.pct().center_y().with(24.px().height()),
            ),
            Color::gray(400),
        ));
        let search_text = tree.leaf((
            TextInput::new(),
            TextValue("Search Library".to_string()),
            Location::new().xs(
                48.px().left().with(100.pct().right().adjust(-16)),
                50.pct().center_y().adjust(4).with(90.pct().height()),
            ),
            Primary(Color::gray(600)),
            Secondary(Color::gray(900)),
            Tertiary(Color::green(300)),
            Elevation::up(1),
            Stem::some(search),
        ));
        let album_cover = tree.leaf((
            Image::new(2, keyring.get("album-cover")),
            ImageView::Aspect,
            Elevation::up(1),
            Location::new().xs(
                1.col()
                    .left()
                    .with(12.col().right())
                    .max(600.0)
                    .justify(Center),
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
            Text::new("ALPHA & THE VAN"),
            FontSize::new(24),
            Color::gray(400),
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
            Color::gray(400),
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
            Secondary(Color::green(500)),
            IconValue(IconHandles::Play.value()),
            Rounding::Full,
            Location::new().xs(
                3.col().center_x().with(48.px().width()),
                1.row().center_y().with(48.px().height()),
            ),
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
            ),
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
            ),
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
            ),
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
            ),
        ));
        let duration = tree.leaf((
            Stem::some(app),
            Elevation::up(1),
            Location::new().xs(
                3.col().left().with(10.col().right()).max(700.0),
                16.row().top().with(24.px().height()),
            ),
            Grid::default(),
        ));
        let back_line = tree.leaf((
            Line::new(4),
            Stem::some(duration),
            Location::new().xs(
                0.pct().x().with(50.pct().y()),
                100.pct().x().with(50.pct().y()),
            ),
            Color::gray(700),
            Elevation::up(1),
        ));
        let elapsed_line = tree.leaf((
            Line::new(4),
            Stem::some(duration),
            Location::new().xs(
                0.pct().x().with(50.pct().y()),
                35.pct().x().with(50.pct().y()),
            ),
            Color::green(300),
            Elevation::up(2),
        ));
        let slider = tree.leaf((
            Panel::new(),
            Rounding::Full,
            Stem::some(duration),
            Stack::new(elapsed_line),
            Location::new().xs(
                stack().right().center_x().with(16.px().width()),
                50.pct().center_y().with(16.px().height()),
            ),
            Elevation::up(3),
            Color::green(300),
        ));
    }
}
