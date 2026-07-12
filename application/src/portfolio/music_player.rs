use crate::icons::IconHandles;
use crate::widgets::{Scrubbed, Scrubber};
use foliage::bevy_ecs;
use foliage::Component;
use foliage::Justify::Center;
use foliage::{
    Animation, Button, ButtonStyle, Children, Color, EcsExtension, Elevation, Entity, FontSize,
    Grid, GridExt, HorizontalAlignment, Icon, Image, ImageView, Keyring, Leaf, Location, OnClick,
    Opacity, Outline, Panel, Query, Rounding, Sequence, Sprout, Text, TextInput, Tree, Trigger,
    VerticalAlignment,
};

/// End-user data riding on the play Button's root entity -- widget entities carry the
/// caller's components alongside the widget's own.
#[derive(Component, Copy, Clone)]
struct Playing(bool);

pub(crate) fn build(tree: &mut Tree, app: Entity, keyring: &Keyring) {
    Sequence::new(tree).animate(
        Animation::new(Opacity::new(1.0))
            .start(1000)
            .finish(1500)
            .targeting(app),
    );
    let mut children = Children::new(app, tree);
    let menu = children.spawn(
        Button::new()
            .icon(IconHandles::Menu.value())
            .colors(Color::gray(200), Color::gray(800))
            .rounding(Rounding::Full)
            .at(Location::new().xs(
                100.pct().as_right().adjust(-16).with(48.px().as_width()),
                16.px().as_top().with(48.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    children
        .tree()
        .on_click(menu, move |trigger: Trigger<OnClick>| {
            // nothing so far
        });
    let search = children.spawn(
        Panel::new()
            .rounding(Rounding::Md)
            .outline(2)
            .color(Color::gray(400))
            .at(Location::new().xs(
                50.pct().as_center_x().with(60.pct().as_width()).max(400.0),
                16.px().as_top().with(44.px().as_height()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(1.col(), 1.row())),
    );
    let mut search_children = Children::new(search, children.tree());
    search_children.spawn(
        Icon::new(IconHandles::Search.value())
            .color(Color::gray(400))
            .at(Location::new().xs(
                8.px().as_left().with(24.px().as_width()),
                50.pct().as_center_y().with(24.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    search_children.spawn(
        TextInput::new()
            .text("Search Library")
            .foreground(Color::gray(600))
            .background(Color::gray(900))
            .accent(Color::green(300))
            .at(Location::new().xs(
                48.px().as_left().with(100.pct().as_right().adjust(-16)),
                50.pct().as_center_y().adjust(4).with(90.pct().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    children.spawn(
        Image::new(2, keyring.get("album-cover"))
            .view(ImageView::Aspect)
            .at(Location::new().xs(
                1.col()
                    .as_left()
                    .with(12.col().as_right())
                    .max(600.0)
                    .justify(Center),
                3.row().as_top().with(10.row().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
    let song_info = children.spawn(
        Leaf::sprout()
            .at(Location::new().xs(
                1.col().as_left().with(12.col().as_right()).max(600.0),
                11.row().as_top().with(13.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(1.col().gap(12), 2.row().gap(8))),
    );
    let mut song_info_children = Children::new(song_info, children.tree());
    song_info_children.spawn(
        Text::new("ALPHA & THE VAN")
            .size(FontSize::new(24))
            .color(Color::gray(400))
            .at(Location::new().xs(
                1.col().as_left().with(1.col().as_right()),
                1.row().as_top().with(1.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with((VerticalAlignment::Middle, HorizontalAlignment::Center)),
    );
    song_info_children.spawn(
        Text::new("A Walk in the Moonlight")
            .size(FontSize::new(16))
            .color(Color::gray(400))
            .at(Location::new().xs(
                1.col().as_left().with(1.col().as_right()),
                2.row().as_top().with(2.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with((VerticalAlignment::Middle, HorizontalAlignment::Center)),
    );
    let controls = children.spawn(
        Panel::new()
            .color(Color::gray(900))
            .at(Location::new().xs(
                1.col().as_left().with(12.col().as_right()).max(400.0),
                14.row().as_top().with(60.px().as_height()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(5.col().gap(8), 1.row().gap(8))),
    );
    let control_buttons = Children::new(controls, children.tree()).each(
        [
            (1, IconHandles::Shuffle.value(), Color::gray(900)),
            (2, IconHandles::SkipLeft.value(), Color::gray(900)),
            (3, IconHandles::Play.value(), Color::green(500)),
            (4, IconHandles::SkipRight.value(), Color::gray(900)),
            (5, IconHandles::Repeat.value(), Color::gray(900)),
        ],
        |_, (col, icon, secondary), row_children| {
            row_children.spawn(
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
    // end-user state on a widget entity + poking its public style: clicking play flips
    // Playing and restyles the button through the same ButtonStyle door any config write
    // uses -- no special toggle API on Button.
    let play = control_buttons[2];
    children.tree().write_to(play, Playing(false));
    children.tree().on_click(
        play,
        move |trigger: Trigger<OnClick>, playing: Query<&Playing>, mut tree: Tree| {
            let e = trigger.event_target();
            let now_playing = !playing.get(e).unwrap().0;
            tree.write_to(
                e,
                (
                    Playing(now_playing),
                    ButtonStyle {
                        foreground: Color::gray(if now_playing { 900 } else { 200 }),
                        background: Color::green(if now_playing { 300 } else { 500 }),
                        outline: Outline::default(),
                        rounding: Rounding::Full,
                    },
                ),
            );
        },
    );
    // the whole duration cluster (track line + elapsed line + anchored knob + drag math)
    // is now one widget spawn -- Progress in, Scrubbed out. Programmatic writes share the
    // drag's door: tree.write_to(scrubber, Progress(0.0)) on track change.
    let scrubber = children.spawn(
        Scrubber::new()
            .progress(0.35)
            .at(Location::new().xs(
                3.col().as_left().with(10.col().as_right()).max(700.0),
                16.row().as_top().with(24.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    children
        .tree()
        .subscribe(scrubber, move |trigger: Trigger<Scrubbed>| {
            let _progress = trigger.event().progress; // a real player would seek here
        });
}
