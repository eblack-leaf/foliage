use crate::icons::IconHandles;
use foliage::{bevy_ecs, Animation, Attachment, Button, Color, EcsExtension, Elevation, Event, Foliage, FontSize, Grid, GridExt, IconValue, Image, ImageView, Keyring, Location, OnClick, Opacity, Outline, Panel, Primary, Res, Rounding, Secondary, Stem, Text, Tree, Trigger, VerticalAlignment};

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
                .targeting(app)
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
            Location::new().xs(
                3.col().left().with(100.pct().right().adjust(-72)),
                16.px().top().with(40.px().height()),
            ),
        ));
        let album_cover = tree.leaf((
            Image::new(2, keyring.get("album-cover")),
            ImageView::Crop,
            Elevation::up(1),
            Location::new().xs(
                3.col().left().with(10.col().right()),
                3.row().top().with(10.row().bottom()),
            ),
            Stem::some(app),
        ));
        let song_info = tree.leaf((
            Location::new().xs(
                1.col().left().with(12.col().right()),
                11.row().top().with(14.row().bottom()),
            ),
            Elevation::up(1),
            Grid::new(1.col(), 2.row()),
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
        ));
    }
}
