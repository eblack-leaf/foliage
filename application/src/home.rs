use crate::icons::IconHandles;
use crate::portfolio::Portfolio;
use foliage::{
    bevy_ecs, stack, Animation, Attachment, Button, Color, EcsExtension, Elevation, Event, Foliage,
    FontSize, GlyphColors, Grid, GridExt, HorizontalAlignment, HrefLink, IconValue, Line, Location,
    Logical, OnClick, OnEnd, Opacity, Outline, Primary, Query, Rounding, Secondary, Section, Stack,
    Stem, Text, TextValue, Tree, Trigger, VerticalAlignment, Write,
};

impl Attachment for Home {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Home::init);
    }
}
#[derive(Event)]
pub(crate) struct Home {}
impl Home {
    pub(crate) fn init(trigger: Trigger<Self>, mut tree: Tree) {
        let row_size = 40;
        let root = tree.leaf((
            Grid::new(12.col().gap(8), row_size.px().gap(8)),
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ),
            Elevation::abs(0),
            Stem::none(),
        ));
        tree.name(root, "home");
        let name_container = tree.leaf((
            Grid::new(12.col().gap(4), 12.row().gap(4)),
            Location::new().xs(
                1.col().left().with(12.col().right()).max(600.0),
                4.row().top().with(8.row().bottom()),
            ),
            Stem::some(root),
            Elevation::up(1),
        ));
        let name = tree.leaf((
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
        let top_desc = tree.leaf((
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
        let top_line = tree.leaf((
            Line::new(2),
            Location::new().xs(4.col().x().with(5.row().y()), 4.col().x().with(5.row().y())),
            Stem::some(name_container),
            Elevation::up(1),
            Color::gray(700),
        ));
        tree.subscribe(
            top_line,
            move |trigger: Trigger<Write<Section<Logical>>>,
                  mut tree: Tree,
                  sections: Query<&Section<Logical>>| {
                let w = sections.get(trigger.entity()).unwrap().width();
                tree.write_to(top_desc, Text::new(format!("w: {:.01}", w)));
            },
        );
        let side_desc = tree.leaf((
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
        tree.subscribe(
            top_line,
            move |trigger: Trigger<Write<Section<Logical>>>,
                  mut tree: Tree,
                  sections: Query<&Section<Logical>>| {
                let h = sections.get(trigger.entity()).unwrap().width() * 0.5;
                tree.write_to(side_desc, Text::new(format!("h: {:.01}", h)));
            },
        );
        let pad_connector = tree.leaf((
            Line::new(2),
            Location::new().xs(7.col().x().with(5.row().y()), 7.col().x().with(5.row().y())),
            Stem::some(name_container),
            Elevation::up(1),
            Color::gray(700),
        ));
        let pad_desc = tree.leaf((
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
        tree.subscribe(
            pad_connector,
            move |trigger: Trigger<Write<Section<Logical>>>,
                  mut tree: Tree,
                  sections: Query<&Section<Logical>>| {
                let h = sections.get(trigger.entity()).unwrap().height();
                tree.write_to(pad_desc, Text::new(format!("pad: {:.01}", h)));
            },
        );
        let desc = tree.leaf((
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
        let github = tree.leaf((
            Button::new(),
            IconValue(IconHandles::Github.value()),
            Rounding::Full,
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
        tree.on_click(github, |trigger: Trigger<OnClick>| {
            HrefLink::new("https://github.com/eblack-leaf/foliage").navigate()
        });
        let github_line = tree.leaf((
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
        let github_desc = tree.leaf((
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
        let options_container = tree.leaf((
            Grid::new(5.col().gap(4), 3.row().gap(8)),
            Location::new().xs(
                1.col().left().with(12.col().right()).max(600.0),
                10.row().top().with(13.row().bottom()),
            ),
            Stem::some(root),
            Elevation::up(1),
        ));
        let option_one_color = Color::green(700);
        let option_one = tree.leaf((
            Button::new(),
            Rounding::Full,
            IconValue(IconHandles::Terminal.value()),
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
        let option_one_line = tree.leaf((
            Line::new(2),
            Location::new().xs(1.col().x().with(1.row().y()), 1.col().x().with(1.row().y())),
            Stem::some(options_container),
            Elevation::up(1),
            option_one_color,
        ));
        let option_one_desc = tree.leaf((
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
        let option_two = tree.leaf((
            Button::new(),
            Rounding::Full,
            IconValue(IconHandles::Layers.value()),
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
        let option_two_line = tree.leaf((
            Line::new(2),
            Location::new().xs(5.col().x().with(2.row().y()), 5.col().x().with(2.row().y())),
            Stem::some(options_container),
            Elevation::up(1),
            option_two_color,
        ));
        let option_two_desc = tree.leaf((
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
        let option_three = tree.leaf((
            Button::new(),
            Rounding::Full,
            IconValue(IconHandles::BookOpen.value()),
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
        let option_three_line = tree.leaf((
            Line::new(2),
            Location::new().xs(1.col().x().with(3.row().y()), 1.col().x().with(3.row().y())),
            Stem::some(options_container),
            Elevation::up(1),
            option_three_color,
        ));
        let option_three_desc = tree.leaf((
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
        let portfolio = tree.leaf((
            Button::new(),
            IconValue(IconHandles::Code.value()),
            TextValue("Portfolio".to_string()),
            Rounding::Sm,
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
        let spacing = tree.leaf((
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                17.row().top().with(17.row().bottom()),
            ),
            Stem::some(root),
        ));
        tree.on_click(
            option_one,
            move |trigger: Trigger<OnClick>, mut tree: Tree| HrefLink::new("tbd").navigate(),
        );
        tree.on_click(
            option_two,
            move |trigger: Trigger<OnClick>, mut tree: Tree| HrefLink::new("tbd").navigate(),
        );
        tree.on_click(
            option_three,
            move |trigger: Trigger<OnClick>, mut tree: Tree| HrefLink::new("tbd").navigate(),
        );
        tree.on_click(
            portfolio,
            move |trigger: Trigger<OnClick>, mut tree: Tree| {
                tree.disable(root);
                tree.send(Portfolio {});
            },
        );
        let seq = tree.sequence();
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .start(500)
                .finish(1500)
                .during(seq)
                .targeting(name),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .start(1000)
                .finish(1500)
                .during(seq)
                .targeting(github),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .start(1000)
                .finish(1250)
                .during(seq)
                .targeting(top_desc),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .start(1100)
                .finish(1350)
                .during(seq)
                .targeting(side_desc),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .start(1500)
                .finish(1750)
                .during(seq)
                .targeting(pad_desc),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .start(1750)
                .finish(2750)
                .during(seq)
                .targeting(desc),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .start(2500)
                .finish(3000)
                .during(seq)
                .targeting(github_desc),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .start(500)
                .finish(1000)
                .during(seq)
                .targeting(option_one),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .start(1000)
                .finish(1500)
                .during(seq)
                .targeting(option_one_desc),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .start(1500)
                .finish(2000)
                .during(seq)
                .targeting(option_two),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .start(2000)
                .finish(2500)
                .during(seq)
                .targeting(option_two_desc),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .start(2500)
                .finish(3000)
                .during(seq)
                .targeting(option_three),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .start(3000)
                .finish(3500)
                .during(seq)
                .targeting(option_three_desc),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .start(3000)
                .finish(3500)
                .during(seq)
                .targeting(portfolio),
        );
        tree.animate(
            Animation::new(
                Location::new().xs(4.col().x().with(5.row().y()), 9.col().x().with(5.row().y())),
            )
            .start(1000)
            .finish(3000)
            .during(seq)
            .targeting(top_line),
        );
        tree.animate(
            Animation::new(
                Location::new().xs(7.col().x().with(5.row().y()), 7.col().x().with(8.row().y())),
            )
            .start(1750)
            .finish(3000)
            .during(seq)
            .targeting(pad_connector),
        );
        tree.animate(
            Animation::new(Location::new().xs(
                stack().right().x().adjust(16).with(1.row().y()),
                stack().right().x().adjust(64).with(1.row().y()),
            ))
            .start(1750)
            .finish(2500)
            .during(seq)
            .targeting(github_line),
        );
        tree.animate(
            Animation::new(
                Location::new().xs(1.col().x().with(1.row().y()), 2.col().x().with(1.row().y())),
            )
            .start(500)
            .finish(1000)
            .during(seq)
            .targeting(option_one_line),
        );
        tree.animate(
            Animation::new(
                Location::new().xs(4.col().x().with(2.row().y()), 5.col().x().with(2.row().y())),
            )
            .start(1500)
            .finish(2000)
            .during(seq)
            .targeting(option_two_line),
        );
        tree.animate(
            Animation::new(
                Location::new().xs(1.col().x().with(3.row().y()), 2.col().x().with(3.row().y())),
            )
            .start(2500)
            .finish(3000)
            .during(seq)
            .targeting(option_three_line),
        );
        tree.disable([github, option_one, option_two, option_three, portfolio]);
        tree.timer(1500, move |trigger: Trigger<OnEnd>, mut tree: Tree| {
            tree.enable(github);
        });
        tree.sequence_end(seq, move |trigger: Trigger<OnEnd>, mut tree: Tree| {
            tree.enable([option_one, option_two, option_three, portfolio]);
        });
    }
}
