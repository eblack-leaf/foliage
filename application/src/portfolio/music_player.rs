use crate::icons::IconHandles;
use foliage::Justify::Center;
use foliage::{
    anchor, Anchor, Animation, Button, Children, Color, EcsExtension, Elevation, Entity, FontSize,
    Grid, GridExt, HorizontalAlignment, Icon, Image, ImageView, Keyring, Leaf, Line, Location,
    OnClick, Opacity, Panel, Photosynthesis, Rounding, Seed, Sequence, Sprout, Text, TextInput,
    Tree, Trigger, VerticalAlignment,
};

pub(crate) fn build(tree: &mut Tree, app: Entity, keyring: &Keyring) {
    Sequence::new(tree).animate(
        Animation::new(Opacity::new(1.0))
            .start(1000)
            .finish(1500)
            .targeting(app),
    );
    let menu = Button::new()
        .icon(IconHandles::Menu.value())
        .colors(Color::gray(200), Color::gray(800))
        .rounding(Rounding::Full)
        .at(Location::new().xs(
            100.pct().as_right().adjust(-16).with(48.px().as_width()),
            16.px().as_top().with(48.px().as_height()),
        ))
        .elevate(Elevation::up(1))
        .stem(app)
        .photosynthesize(tree);
    tree.on_click(menu, move |trigger: Trigger<OnClick>| {
        // nothing so far
    });
    let search = Panel::new()
        .rounding(Rounding::Md)
        .outline(2)
        .color(Color::gray(400))
        .at(Location::new().xs(
            50.pct().as_center_x().with(60.pct().as_width()).max(400.0),
            16.px().as_top().with(44.px().as_height()),
        ))
        .elevate(Elevation::up(1))
        .stem(app)
        .with(Grid::new(1.col(), 1.row()))
        .photosynthesize(tree);
    let search_icon = Icon::new(IconHandles::Search.value())
        .color(Color::gray(400))
        .at(Location::new().xs(
            8.px().as_left().with(24.px().as_width()),
            50.pct().as_center_y().with(24.px().as_height()),
        ))
        .elevate(Elevation::up(1))
        .stem(search)
        .photosynthesize(tree);
    let search_text = TextInput::new()
        .text("Search Library")
        .primary(Color::gray(600))
        .secondary(Color::gray(900))
        .tertiary(Color::green(300))
        .at(Location::new().xs(
            48.px().as_left().with(100.pct().as_right().adjust(-16)),
            50.pct().as_center_y().adjust(4).with(90.pct().as_height()),
        ))
        .elevate(Elevation::up(1))
        .stem(search)
        .photosynthesize(tree);
    let album_cover = Image::new(2, keyring.get("album-cover"))
        .view(ImageView::Aspect)
        .at(Location::new().xs(
            1.col()
                .as_left()
                .with(12.col().as_right())
                .max(600.0)
                .justify(Center),
            3.row().as_top().with(10.row().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .stem(app)
        .photosynthesize(tree);
    let song_info = Leaf::sprout()
        .at(Location::new().xs(
            1.col().as_left().with(12.col().as_right()).max(600.0),
            11.row().as_top().with(13.row().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .stem(app)
        .with(Grid::new(1.col().gap(12), 2.row().gap(8)))
        .photosynthesize(tree);
    let artist_name = Text::new("ALPHA & THE VAN")
        .size(FontSize::new(24))
        .color(Color::gray(400))
        .at(Location::new().xs(
            1.col().as_left().with(1.col().as_right()),
            1.row().as_top().with(1.row().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .stem(song_info)
        .with((VerticalAlignment::Middle, HorizontalAlignment::Center))
        .photosynthesize(tree);
    let song_name = Text::new("A Walk in the Moonlight")
        .size(FontSize::new(16))
        .color(Color::gray(400))
        .at(Location::new().xs(
            1.col().as_left().with(1.col().as_right()),
            2.row().as_top().with(2.row().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .stem(song_info)
        .with((VerticalAlignment::Middle, HorizontalAlignment::Center))
        .photosynthesize(tree);
    let controls = Panel::new()
        .color(Color::gray(900))
        .at(Location::new().xs(
            1.col().as_left().with(12.col().as_right()).max(400.0),
            14.row().as_top().with(60.px().as_height()),
        ))
        .elevate(Elevation::up(1))
        .stem(app)
        .with(Grid::new(5.col().gap(8), 1.row().gap(8)))
        .photosynthesize(tree);
    Children::new(controls, tree).each(
        [
            (1, IconHandles::Shuffle.value(), Color::gray(900)),
            (2, IconHandles::SkipLeft.value(), Color::gray(900)),
            (3, IconHandles::Play.value(), Color::green(500)),
            (4, IconHandles::SkipRight.value(), Color::gray(900)),
            (5, IconHandles::Repeat.value(), Color::gray(900)),
        ],
        |_, (col, icon, secondary), children| {
            children.spawn(
                Button::new()
                    .icon(icon)
                    .colors(Color::gray(200), secondary)
                    .rounding(Rounding::Full)
                    .at(Location::new().xs(
                        col.col().as_center_x().with(48.px().as_width()),
                        1.row().as_center_y().with(48.px().as_height()),
                    ))
                    .elevate(Elevation::up(1)),
            )
        },
    );
    let duration = Leaf::sprout()
        .at(Location::new().xs(
            3.col().as_left().with(10.col().as_right()).max(700.0),
            16.row().as_top().with(24.px().as_height()),
        ))
        .elevate(Elevation::up(1))
        .stem(app)
        .with(Grid::default())
        .photosynthesize(tree);
    let back_line = Line::new(4)
        .color(Color::gray(700))
        .at(Location::new().xs(
            0.pct().as_x().with(50.pct().as_y()),
            100.pct().as_x().with(50.pct().as_y()),
        ))
        .elevate(Elevation::up(1))
        .stem(duration)
        .photosynthesize(tree);
    let elapsed_line = Line::new(4)
        .color(Color::green(300))
        .at(Location::new().xs(
            0.pct().as_x().with(50.pct().as_y()),
            35.pct().as_x().with(50.pct().as_y()),
        ))
        .elevate(Elevation::up(2))
        .stem(duration)
        .photosynthesize(tree);
    let slider = Panel::new()
        .rounding(Rounding::Full)
        .color(Color::green(300))
        .at(Location::new().xs(
            anchor().right().as_center_x().with(16.px().as_width()),
            50.pct().as_center_y().with(16.px().as_height()),
        ))
        .elevate(Elevation::up(3))
        .stem(duration)
        .with(Anchor::new(elapsed_line))
        .photosynthesize(tree);
}
