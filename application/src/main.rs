#![allow(unused)]
use foliage::{
    stack, Animation, Button, ButtonShape, Color, EcsExtension, Elevation, Foliage, FontSize,
    GlyphColors, Grid, GridExt, HorizontalAlignment, HrefLink, Icon, IconValue,
    InteractionListener, Line, Location, Logical, OnClick, OnEnd, Opacity, Outline, Primary, Query,
    Secondary, Section, Stack, Stem, Text, TextValue, TimeDelta, Timer, Tree, Trigger,
    VerticalAlignment, Write,
};

fn main() {
    let mut foliage = Foliage::new();
    foliage.enable_tracing(
        tracing_subscriber::filter::Targets::new().with_target("foliage", tracing::Level::TRACE),
    );
    foliage.desktop_size((360, 800));
    foliage.url("foliage");
    let row_size = 40;
    let root = foliage.leaf((
        Grid::new(12.col().gap(8), row_size.px().gap(8)),
        Location::new().xs(
            0.pct().left().with(100.pct().right()),
            0.pct().top().with(100.pct().bottom()),
        ),
        InteractionListener::new().scroll(true),
        Elevation::abs(0),
        Stem::none(),
    ));
    let name_container = foliage.leaf((
        Grid::new(12.col().gap(4), 12.row().gap(4)),
        Location::new().xs(
            1.col().left().with(12.col().right()).max(600.0),
            4.row().top().with(8.row().bottom()),
        ),
        Stem::some(root),
        Elevation::up(1),
    ));
    let name = foliage.leaf((
        Text::new("foliage.rs"),
        FontSize::new(44),
        HorizontalAlignment::Center,
        Location::new().xs(
            2.col().left().with(11.col().right()),
            1.row().top().with(3.row().bottom()),
        ),
        Elevation::up(1),
        GlyphColors::new().add(7..10, Color::green(400)),
        Stem::some(name_container),
        Opacity::new(0.0),
    ));
    let top_desc = foliage.leaf((
        Text::new("w: 0.0"),
        FontSize::new(14),
        Location::new().xs(
            5.col().left().with(8.col().right()),
            4.row().top().with(4.row().bottom()),
        ),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
        Opacity::new(0.0),
    ));
    let top_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(4.col().x().with(5.row().y()), 4.col().x().with(5.row().y())),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
    ));
    foliage.subscribe(
        top_line,
        move |trigger: Trigger<Write<Section<Logical>>>,
              mut tree: Tree,
              sections: Query<&Section<Logical>>| {
            let w = sections.get(trigger.entity()).unwrap().width();
            tree.write_to(top_desc, Text::new(format!("w: {:.01}", w)));
        },
    );
    let side_desc = foliage.leaf((
        Text::new("h: 0.0"),
        FontSize::new(14),
        Location::new().xs(
            9.col().left().with(11.col().right()),
            4.row().top().with(4.row().bottom()),
        ),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
        Opacity::new(0.0),
    ));
    foliage.subscribe(
        top_line,
        move |trigger: Trigger<Write<Section<Logical>>>,
              mut tree: Tree,
              sections: Query<&Section<Logical>>| {
            let h = sections.get(trigger.entity()).unwrap().width() * 0.5;
            tree.write_to(side_desc, Text::new(format!("h: {:.01}", h)));
        },
    );
    let pad_connector = foliage.leaf((
        Line::new(2),
        Location::new().xs(7.col().x().with(5.row().y()), 7.col().x().with(5.row().y())),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
    ));
    let pad_desc = foliage.leaf((
        Text::new("pad: 0.0"),
        FontSize::new(14),
        Location::new().xs(
            8.col().left().with(11.col().right()),
            7.row().top().with(8.row().bottom()),
        ),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
        Opacity::new(0.0),
    ));
    foliage.subscribe(
        pad_connector,
        move |trigger: Trigger<Write<Section<Logical>>>,
              mut tree: Tree,
              sections: Query<&Section<Logical>>| {
            let h = sections.get(trigger.entity()).unwrap().height();
            tree.write_to(pad_desc, Text::new(format!("pad: {:.01}", h)));
        },
    );
    let desc = foliage.leaf((
        Text::new("native + web ui"),
        FontSize::new(24),
        HorizontalAlignment::Center,
        Location::new().xs(
            1.col().left().with(12.col().right()),
            9.row().top().with(12.row().bottom()),
        ),
        Elevation::up(1),
        GlyphColors::new()
            .add(7..8, Color::orange(700))
            .add(13..15, Color::green(400)),
        Stem::some(name_container),
        Opacity::new(0.0),
        Color::gray(500),
    ));
    foliage
        .world
        .spawn(Icon::memory(0, include_bytes!("assets/icons/github.icon")));
    let github = foliage.leaf((
        Button::new(),
        IconValue(0),
        ButtonShape::Circle,
        Primary(Color::gray(200)),
        Secondary(Color::gray(800)),
        FontSize::new(16),
        Outline::default(),
        Location::new().xs(
            1.col().left().with(48.px().width()),
            1.row().top().with(48.px().height()),
        ),
        Elevation::up(1),
        Stem::some(root),
        Opacity::new(0.0),
    ));
    foliage.on_click(github, |trigger: Trigger<OnClick>| {
        HrefLink::new("https://github.com/eblack-leaf/foliage").navigate()
    });
    let github_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(
            stack().right().x().adjust(16).with(1.row().y()),
            stack().right().x().adjust(16).with(1.row().y()),
        ),
        Stem::some(root),
        Stack::new(github),
        Elevation::up(1),
        Color::gray(700),
    ));
    let github_desc = foliage.leaf((
        Text::new("on-click: github"),
        FontSize::new(14),
        Location::new().xs(
            stack().right().left().adjust(16).with(10.col().right()),
            1.row().top().adjust(8).with(2.row().bottom()),
        ),
        Elevation::up(1),
        GlyphColors::new().add(10..16, Color::green(300)),
        Stem::some(root),
        Stack::new(github_line),
        Opacity::new(0.0),
        Color::gray(500),
    ));
    let options_container = foliage.leaf((
        Grid::new(5.col().gap(4), 3.row().gap(8)),
        Location::new().xs(
            1.col().left().with(12.col().right()).max(600.0),
            10.row().top().with(13.row().bottom()),
        ),
        Stem::some(root),
        Elevation::up(1),
    ));
    let option_one_color = Color::green(700);
    foliage.world.spawn(Icon::memory(
        1,
        include_bytes!("assets/icons/terminal.icon"),
    ));
    let option_one = foliage.leaf((
        Button::new(),
        ButtonShape::Circle,
        IconValue(1),
        Primary(option_one_color),
        Secondary(Color::gray(900)),
        Location::new().xs(
            3.col().left().with(3.col().right()).max(48.0).min(48.0),
            1.row().top().with(1.row().bottom()).max(48.0).min(48.0),
        ),
        Elevation::up(1),
        Stem::some(options_container),
        Outline::new(2),
        Opacity::new(0.0),
    ));
    foliage.on_click(option_one, |trigger: Trigger<OnClick>| {
        // TODO
    });
    let option_one_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(1.col().x().with(1.row().y()), 1.col().x().with(1.row().y())),
        Stem::some(options_container),
        Elevation::up(1),
        option_one_color,
    ));
    let option_one_desc = foliage.leaf((
        Text::new("on-click: usage"),
        HorizontalAlignment::Center,
        VerticalAlignment::Middle,
        FontSize::new(16),
        GlyphColors::new().add(10..15, option_one_color),
        Location::new().xs(
            4.col().left().with(5.col().right()),
            1.row().top().with(1.row().bottom()),
        ),
        Elevation::up(1),
        Stem::some(options_container),
        Opacity::new(0.0),
        Color::gray(500),
    ));
    let option_two_color = Color::green(500);
    foliage
        .world
        .spawn(Icon::memory(2, include_bytes!("assets/icons/layers.icon")));
    let option_two = foliage.leaf((
        Button::new(),
        ButtonShape::Circle,
        IconValue(2),
        Primary(option_two_color),
        Secondary(Color::gray(900)),
        Location::new().xs(
            3.col().left().with(3.col().right()).max(48.0).min(48.0),
            2.row().top().with(2.row().bottom()).max(48.0).min(48.0),
        ),
        Elevation::up(1),
        Stem::some(options_container),
        Outline::new(2),
        Opacity::new(0.0),
    ));
    foliage.on_click(option_two, |trigger: Trigger<OnClick>| {
        // TODO
    });
    let option_two_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(5.col().x().with(2.row().y()), 5.col().x().with(2.row().y())),
        Stem::some(options_container),
        Elevation::up(1),
        option_two_color,
    ));
    let option_two_desc = foliage.leaf((
        Text::new("on-click: impl"),
        HorizontalAlignment::Center,
        VerticalAlignment::Middle,
        FontSize::new(16),
        GlyphColors::new().add(10..14, option_two_color),
        Location::new().xs(
            1.col().left().with(2.col().right()),
            2.row().top().with(2.row().bottom()),
        ),
        Elevation::up(1),
        Stem::some(options_container),
        Color::gray(500),
        Opacity::new(0.0),
    ));
    let option_three_color = Color::green(300);
    foliage.world.spawn(Icon::memory(
        3,
        include_bytes!("assets/icons/book-open.icon"),
    ));
    let option_three = foliage.leaf((
        Button::new(),
        ButtonShape::Circle,
        IconValue(3),
        Primary(option_three_color),
        Secondary(Color::gray(900)),
        Location::new().xs(
            3.col().left().with(3.col().right()).max(48.0).min(48.0),
            3.row().top().with(3.row().bottom()).max(48.0).min(48.0),
        ),
        Elevation::up(1),
        Stem::some(options_container),
        Outline::new(2),
        Opacity::new(0.0),
    ));
    foliage.on_click(option_three, |trigger: Trigger<OnClick>| {
        // TODO
    });
    let option_three_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(1.col().x().with(3.row().y()), 1.col().x().with(3.row().y())),
        Stem::some(options_container),
        Elevation::up(1),
        option_three_color,
    ));
    let option_three_desc = foliage.leaf((
        Text::new("on-click: docs"),
        HorizontalAlignment::Center,
        VerticalAlignment::Middle,
        FontSize::new(16),
        GlyphColors::new().add(10..14, option_three_color),
        Location::new().xs(
            4.col().left().with(5.col().right()),
            3.row().top().with(3.row().bottom()),
        ),
        Elevation::up(1),
        Stem::some(options_container),
        Color::gray(500),
        Opacity::new(0.0),
    ));
    let portfolio = foliage.leaf((
        Button::new(),
        IconValue(3),
        TextValue("Portfolio".to_string()),
        FontSize::new(20),
        Primary(Color::orange(500)),
        Secondary(Color::gray(900)),
        Location::new().xs(
            3.col().left().with(10.col().right()).min(175.0).max(350.0),
            15.row().top().with(48.px().height()),
        ),
        Opacity::new(0.0),
        Elevation::up(1),
        Stem::some(root),
        Outline::new(2),
    ));
    let seq = foliage.sequence();
    let anim = Animation::new(Opacity::new(1.0))
        .start(500)
        .finish(1500)
        .targeting(name);
    foliage.animate(seq, anim);
    let anim = Animation::new(Opacity::new(1.0))
        .start(1000)
        .finish(1500)
        .targeting(github);
    foliage.animate(seq, anim);
    let anim = Animation::new(Opacity::new(1.0))
        .start(1000)
        .finish(1250)
        .targeting(top_desc);
    foliage.animate(seq, anim);
    let anim = Animation::new(Opacity::new(1.0))
        .start(1100)
        .finish(1350)
        .targeting(side_desc);
    foliage.animate(seq, anim);
    let anim = Animation::new(Opacity::new(1.0))
        .start(1500)
        .finish(1750)
        .targeting(pad_desc);
    foliage.animate(seq, anim);
    let anim = Animation::new(Opacity::new(1.0))
        .start(1750)
        .finish(2750)
        .targeting(desc);
    foliage.animate(seq, anim);
    let anim = Animation::new(Opacity::new(1.0))
        .start(2500)
        .finish(3000)
        .targeting(github_desc);
    foliage.animate(seq, anim);
    let anim = Animation::new(Opacity::new(1.0))
        .start(3000)
        .finish(3500)
        .targeting(option_one);
    foliage.animate(seq, anim);
    let anim = Animation::new(Opacity::new(1.0))
        .start(3500)
        .finish(4000)
        .targeting(option_one_desc);
    foliage.animate(seq, anim);
    let anim = Animation::new(Opacity::new(1.0))
        .start(4000)
        .finish(4500)
        .targeting(option_two);
    foliage.animate(seq, anim);
    let anim = Animation::new(Opacity::new(1.0))
        .start(4500)
        .finish(5000)
        .targeting(option_two_desc);
    foliage.animate(seq, anim);
    let anim = Animation::new(Opacity::new(1.0))
        .start(5250)
        .finish(5750)
        .targeting(option_three);
    foliage.animate(seq, anim);
    let anim = Animation::new(Opacity::new(1.0))
        .start(5750)
        .finish(6250)
        .targeting(option_three_desc);
    foliage.animate(seq, anim);
    let anim = Animation::new(Opacity::new(1.0))
        .start(6000)
        .finish(6500)
        .targeting(portfolio);
    foliage.animate(seq, anim);
    let anim = Animation::new(
        Location::new().xs(4.col().x().with(5.row().y()), 9.col().x().with(5.row().y())),
    )
        .start(1000)
        .finish(3000)
        .targeting(top_line);
    foliage.animate(seq, anim);
    let anim = Animation::new(
        Location::new().xs(7.col().x().with(5.row().y()), 7.col().x().with(8.row().y())),
    )
        .start(1750)
        .finish(3000)
        .targeting(pad_connector);
    foliage.animate(seq, anim);
    let anim = Animation::new(Location::new().xs(
        stack().right().x().adjust(16).with(1.row().y()),
        stack().right().x().adjust(64).with(1.row().y()),
    ))
        .start(1750)
        .finish(2500)
        .targeting(github_line);
    foliage.animate(seq, anim);
    let anim = Animation::new(
        Location::new().xs(1.col().x().with(1.row().y()), 2.col().x().with(1.row().y())),
    )
        .start(2500)
        .finish(3000)
        .targeting(option_one_line);
    foliage.animate(seq, anim);
    let anim = Animation::new(
        Location::new().xs(4.col().x().with(2.row().y()), 5.col().x().with(2.row().y())),
    )
        .start(3500)
        .finish(4000)
        .targeting(option_two_line);
    foliage.animate(seq, anim);
    let anim = Animation::new(
        Location::new().xs(1.col().x().with(3.row().y()), 2.col().x().with(3.row().y())),
    )
        .start(4750)
        .finish(5250)
        .targeting(option_three_line);
    foliage.animate(seq, anim);
    foliage.disable([github, option_one, option_two, option_three, portfolio]);
    foliage
        .world
        .spawn(Timer::new(TimeDelta::from_millis(1500)))
        .observe(move |trigger: Trigger<OnEnd>, mut tree: Tree| {
            tree.enable(github);
        });
    foliage.sequence_end(seq, move |trigger: Trigger<OnEnd>, mut tree: Tree| {
        tree.enable([option_one, option_two, option_three, portfolio]);
    });
    foliage.photosynthesize(); // run
}
