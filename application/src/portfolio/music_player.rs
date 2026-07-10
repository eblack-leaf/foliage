use crate::icons::IconHandles;
use foliage::Justify::Center;
use foliage::{
    stack, Animation, Button, Color, EcsExtension, Elevation, Entity, FontSize, Grid, GridExt,
    HorizontalAlignment, Icon, Image, ImageView, Keyring, LeafBuilder, Line, Location, OnClick,
    Opacity, Panel, Primary, Rounding, Secondary, Stack, Stem, Tertiary, Text, TextInput,
    TextValue, Tree, Trigger, VerticalAlignment,
};

pub(crate) fn build(tree: &mut Tree, app: Entity, keyring: &Keyring) {
        tree.entity(app).insert((
            Panel::default(),
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
        let menu = Button::new()
            .icon(IconHandles::Menu.value())
            .colors(Color::gray(200), Color::gray(800))
            .rounding(Rounding::Full)
            .at(Location::new().xs(
                100.pct().right().adjust(-16).with(48.px().width()),
                16.px().top().with(48.px().height()),
            ))
            .stem(app)
            .spawn(tree);
        tree.on_click(menu, move |trigger: Trigger<OnClick>| {
            // nothing so far
        });
        let search = Panel::new()
            .rounding(Rounding::Md)
            .outline(2)
            .color(Color::gray(400))
            .at(Location::new().xs(
                50.pct().center_x().with(60.pct().width()).max(400.0),
                16.px().top().with(44.px().height()),
            ))
            .stem(app)
            .spawn(tree);
        tree.write_to(search, Grid::new(1.col(), 1.row()));
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
        let artist_name = Text::new("ALPHA & THE VAN")
            .size(FontSize::new(24))
            .color(Color::gray(400))
            .at(Location::new().xs(
                1.col().left().with(1.col().right()),
                1.row().top().with(1.row().bottom()),
            ))
            .stem(song_info)
            .spawn(tree);
        tree.write_to(artist_name, (VerticalAlignment::Middle, HorizontalAlignment::Center));
        let song_name = Text::new("A Walk in the Moonlight")
            .size(FontSize::new(16))
            .color(Color::gray(400))
            .at(Location::new().xs(
                1.col().left().with(1.col().right()),
                2.row().top().with(2.row().bottom()),
            ))
            .stem(song_info)
            .spawn(tree);
        tree.write_to(song_name, (VerticalAlignment::Middle, HorizontalAlignment::Center));
        let controls = Panel::new()
            .color(Color::gray(900))
            .at(Location::new().xs(
                1.col().left().with(12.col().right()).max(400.0),
                14.row().top().with(60.px().height()),
            ))
            .stem(app)
            .spawn(tree);
        tree.write_to(controls, Grid::new(5.col().gap(8), 1.row().gap(8)));
        let play_pause = Button::new()
            .icon(IconHandles::Play.value())
            .colors(Color::gray(200), Color::green(500))
            .rounding(Rounding::Full)
            .at(Location::new().xs(
                3.col().center_x().with(48.px().width()),
                1.row().center_y().with(48.px().height()),
            ))
            .stem(controls)
            .spawn(tree);
        let shuffle = Button::new()
            .icon(IconHandles::Shuffle.value())
            .colors(Color::gray(200), Color::gray(900))
            .rounding(Rounding::Full)
            .at(Location::new().xs(
                1.col().center_x().with(48.px().width()),
                1.row().center_y().with(48.px().height()),
            ))
            .stem(controls)
            .spawn(tree);
        let left = Button::new()
            .icon(IconHandles::SkipLeft.value())
            .colors(Color::gray(200), Color::gray(900))
            .rounding(Rounding::Full)
            .at(Location::new().xs(
                2.col().center_x().with(48.px().width()),
                1.row().center_y().with(48.px().height()),
            ))
            .stem(controls)
            .spawn(tree);
        let right = Button::new()
            .icon(IconHandles::SkipRight.value())
            .colors(Color::gray(200), Color::gray(900))
            .rounding(Rounding::Full)
            .at(Location::new().xs(
                4.col().center_x().with(48.px().width()),
                1.row().center_y().with(48.px().height()),
            ))
            .stem(controls)
            .spawn(tree);
        let repeat = Button::new()
            .icon(IconHandles::Repeat.value())
            .colors(Color::gray(200), Color::gray(900))
            .rounding(Rounding::Full)
            .at(Location::new().xs(
                5.col().center_x().with(48.px().width()),
                1.row().center_y().with(48.px().height()),
            ))
            .stem(controls)
            .spawn(tree);
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
        let slider = Panel::new()
            .rounding(Rounding::Full)
            .color(Color::green(300))
            .at(Location::new().xs(
                stack().right().center_x().with(16.px().width()),
                50.pct().center_y().with(16.px().height()),
            ))
            .elevate(Elevation::up(3))
            .stem(duration)
            .spawn(tree);
        tree.write_to(slider, Stack::new(elapsed_line));
}
