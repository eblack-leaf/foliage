use crate::icons::IconHandles;
use crate::portfolio;
use foliage::{
    anchor, Anchor, Animation, Button, Children, Color, EcsExtension, Elevation, Entity,
    EntityEvent, FontSize, GlyphColors, Grid, GridExt, HorizontalAlignment, HrefLink, Keyring,
    Leaf, Photosynthesis, Seed, Sprout, Line, Location, Logical, OnClick, OnEnd, Opacity, Outline, Query, Res,
    Rounding, Section, Sequence, Text, TextValue, Tree, Trigger, VerticalAlignment, Write,
};

pub(crate) fn build<T: EcsExtension>(tree: &mut T) {
    let row_size = 40;
    let root = Leaf::sprout()
        .at(Location::new().xs(
            0.pct().as_left().with(100.pct().as_right()),
            0.pct().as_top().with(100.pct().as_bottom()),
        ))
        .elevate(Elevation::abs(0))
        .with(Grid::new(12.col().gap(8), row_size.px().gap(8)))
        .photosynthesize(tree);
    tree.name(root, "home");
    let name_container = Leaf::sprout()
        .at(Location::new().xs(
            1.col().as_left().with(12.col().as_right()).max(600.0),
            4.row().as_top().with(8.row().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .stem(root)
        .with(Grid::new(12.col().gap(4), 12.row().gap(4)))
        .photosynthesize(tree);
    let name = Text::new("foliage.rs")
        .size(FontSize::new(44))
        .at(Location::new().xs(
            2.col().as_left().with(11.col().as_right()),
            1.row().as_top().with(3.row().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .stem(name_container)
        .with((
            HorizontalAlignment::Center,
            GlyphColors::new().add(7..10, Color::green(400)),
            Opacity::new(0.0),
        ))
        .photosynthesize(tree);
    let top_desc = Text::new("w: 0.0")
        .size(FontSize::new(14))
        .color(Color::gray(700))
        .at(Location::new().xs(
            5.col().as_left().with(8.col().as_right()),
            4.row().as_top().with(4.row().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .stem(name_container)
        .with(Opacity::new(0.0))
        .photosynthesize(tree);
    let top_line = Line::new(2)
        .color(Color::gray(700))
        .at(Location::new().xs(4.col().as_x().with(5.row().as_y()), 4.col().as_x().with(5.row().as_y())))
        .elevate(Elevation::up(1))
        .stem(name_container)
        .photosynthesize(tree);
    tree.subscribe(
        top_line,
        move |trigger: Trigger<Write<Section<Logical>>>,
              mut tree: Tree,
              sections: Query<&Section<Logical>>| {
            let w = sections.get(trigger.event_target()).unwrap().width();
            tree.write_to(
                top_desc,
                Text {
                    value: format!("w: {:.01}", w),
                },
            );
        },
    );
    let side_desc = Text::new("h: 0.0")
        .size(FontSize::new(14))
        .color(Color::gray(700))
        .at(Location::new().xs(
            9.col().as_left().with(11.col().as_right()),
            4.row().as_top().with(4.row().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .stem(name_container)
        .with(Opacity::new(0.0))
        .photosynthesize(tree);
    tree.subscribe(
        top_line,
        move |trigger: Trigger<Write<Section<Logical>>>,
              mut tree: Tree,
              sections: Query<&Section<Logical>>| {
            let h = sections.get(trigger.event_target()).unwrap().width() * 0.5;
            tree.write_to(
                side_desc,
                Text {
                    value: format!("h: {:.01}", h),
                },
            );
        },
    );
    let pad_connector = Line::new(2)
        .color(Color::gray(700))
        .at(Location::new().xs(7.col().as_x().with(5.row().as_y()), 7.col().as_x().with(5.row().as_y())))
        .elevate(Elevation::up(1))
        .stem(name_container)
        .photosynthesize(tree);
    let pad_desc = Text::new("pad: 0.0")
        .size(FontSize::new(14))
        .color(Color::gray(700))
        .at(Location::new().xs(
            8.col().as_left().with(11.col().as_right()),
            7.row().as_top().with(8.row().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .stem(name_container)
        .with(Opacity::new(0.0))
        .photosynthesize(tree);
    tree.subscribe(
        pad_connector,
        move |trigger: Trigger<Write<Section<Logical>>>,
              mut tree: Tree,
              sections: Query<&Section<Logical>>| {
            let h = sections.get(trigger.event_target()).unwrap().height();
            tree.write_to(
                pad_desc,
                Text {
                    value: format!("pad: {:.01}", h),
                },
            );
        },
    );
    let desc = Text::new("native + web ui")
        .size(FontSize::new(24))
        .color(Color::gray(500))
        .at(Location::new().xs(
            1.col().as_left().with(12.col().as_right()),
            9.row().as_top().with(12.row().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .stem(name_container)
        .with((
            HorizontalAlignment::Center,
            GlyphColors::new()
                .add(7..8, Color::orange(700))
                .add(13..15, Color::green(400)),
            Opacity::new(0.0),
        ))
        .photosynthesize(tree);
    let github = Button::new()
        .icon(IconHandles::Github.value())
        .rounding(Rounding::Full)
        .colors(Color::gray(200), Color::gray(800))
        .at(Location::new().xs(
            1.col().as_left().with(48.px().as_width()),
            1.row().as_top().with(48.px().as_height()),
        ))
        .elevate(Elevation::up(1))
        .stem(root)
        .photosynthesize(tree);
    tree.graft(github)
        .write((FontSize::new(16), Opacity::new(0.0)))
        .on_click(|trigger: Trigger<OnClick>| {
            HrefLink::new("https://github.com/eblack-leaf/foliage").navigate()
        });
    let github_line = Line::new(2)
        .color(Color::gray(700))
        .at(Location::new().xs(
            anchor().right().as_x().adjust(16).with(1.row().as_y()),
            anchor().right().as_x().adjust(16).with(1.row().as_y()),
        ))
        .elevate(Elevation::up(1))
        .stem(root)
        .with(Anchor::new(github))
        .photosynthesize(tree);
    let github_desc = Text::new("on-click: github")
        .size(FontSize::new(14))
        .color(Color::gray(500))
        .at(Location::new().xs(
            anchor().right().as_left().adjust(16).with(10.col().as_right()),
            1.row().as_top().adjust(8).with(2.row().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .stem(root)
        .with((
            GlyphColors::new().add(10..16, Color::green(300)),
            Anchor::new(github_line),
            Opacity::new(0.0),
        ))
        .photosynthesize(tree);
    let options_container = Leaf::sprout()
        .at(Location::new().xs(
            1.col().as_left().with(12.col().as_right()).max(600.0),
            10.row().as_top().with(13.row().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .stem(root)
        .with(Grid::new(5.col().gap(4), 3.row().gap(8)))
        .photosynthesize(tree);
    // row, icon, color, desc text, desc highlight range, desc column range, line column
    let option_rows: Vec<(Entity, Entity, Entity)> = Children::new(options_container, tree).each(
        [
            (1, IconHandles::Terminal.value(), Color::green(700), "on-click: usage", 10..15, (4, 5), 1),
            (2, IconHandles::Layers.value(), Color::green(500), "on-click: impl", 10..14, (1, 2), 5),
            (3, IconHandles::BookOpen.value(), Color::green(300), "on-click: docs", 10..14, (4, 5), 1),
        ],
        |_, (row, icon, color, desc_text, highlight, (desc_left, desc_right), line_col), children| {
            let button = children.spawn(
                Button::new()
                    .rounding(Rounding::Full)
                    .icon(icon)
                    .colors(color, Color::gray(900))
                    .outline(2)
                    .at(Location::new().xs(
                        3.col().as_left().with(3.col().as_right()).max(48.0).min(48.0),
                        row.row().as_top().with(row.row().as_bottom()).max(48.0).min(48.0),
                    ))
                    .elevate(Elevation::up(1)),
            );
            children
                .tree()
                .graft(button)
                .write(Opacity::new(0.0))
                .on_click(move |trigger: Trigger<OnClick>| HrefLink::new("tbd").navigate());
            let line = Line::new(2)
                .color(color)
                .at(Location::new().xs(
                    line_col.col().as_x().with(row.row().as_y()),
                    line_col.col().as_x().with(row.row().as_y()),
                ))
                .elevate(Elevation::up(1))
                .stem(options_container)
                .photosynthesize(children.tree());
            let desc = children.spawn(
                Text::new(desc_text)
                    .size(FontSize::new(16))
                    .color(Color::gray(500))
                    .at(Location::new().xs(
                        desc_left.col().as_left().with(desc_right.col().as_right()),
                        row.row().as_top().with(row.row().as_bottom()),
                    ))
                    .elevate(Elevation::up(1))
                    .with((
                        HorizontalAlignment::Center,
                        VerticalAlignment::Middle,
                        GlyphColors::new().add(highlight, color),
                        Opacity::new(0.0),
                    )),
            );
            (button, line, desc)
        },
    );
    let (option_one, option_one_line, option_one_desc) = option_rows[0];
    let (option_two, option_two_line, option_two_desc) = option_rows[1];
    let (option_three, option_three_line, option_three_desc) = option_rows[2];
    let portfolio = Button::new()
        .icon(IconHandles::Code.value())
        .text("Portfolio")
        .rounding(Rounding::Sm)
        .colors(Color::orange(500), Color::gray(900))
        .outline(2)
        .at(Location::new().xs(
            3.col().as_left().with(10.col().as_right()).min(175.0).max(350.0),
            15.row().as_top().with(48.px().as_height()),
        ))
        .elevate(Elevation::up(1))
        .stem(root)
        .photosynthesize(tree);
    tree.graft(portfolio)
        .write((FontSize::new(20), Opacity::new(0.0)))
        .on_click(
            move |trigger: Trigger<OnClick>, mut tree: Tree, keyring: Res<Keyring>| {
                tree.disable(root);
                portfolio::build(&mut tree, root, &keyring);
            },
        );
    let spacing = Leaf::sprout()
        .at(Location::new().xs(
            0.pct().as_left().with(100.pct().as_right()),
            17.row().as_top().with(17.row().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .stem(root)
        .photosynthesize(tree);
    Sequence::new(tree)
        .animate(Animation::new(Opacity::new(1.0)).start(500).finish(1500).targeting(name))
        .animate(Animation::new(Opacity::new(1.0)).start(1000).finish(1500).targeting(github))
        .animate(Animation::new(Opacity::new(1.0)).start(1000).finish(1250).targeting(top_desc))
        .animate(Animation::new(Opacity::new(1.0)).start(1100).finish(1350).targeting(side_desc))
        .animate(Animation::new(Opacity::new(1.0)).start(1500).finish(1750).targeting(pad_desc))
        .animate(Animation::new(Opacity::new(1.0)).start(1750).finish(2750).targeting(desc))
        .animate(Animation::new(Opacity::new(1.0)).start(2500).finish(3000).targeting(github_desc))
        .animate(Animation::new(Opacity::new(1.0)).start(500).finish(1000).targeting(option_one))
        .animate(Animation::new(Opacity::new(1.0)).start(1000).finish(1500).targeting(option_one_desc))
        .animate(Animation::new(Opacity::new(1.0)).start(1500).finish(2000).targeting(option_two))
        .animate(Animation::new(Opacity::new(1.0)).start(2000).finish(2500).targeting(option_two_desc))
        .animate(Animation::new(Opacity::new(1.0)).start(2500).finish(3000).targeting(option_three))
        .animate(Animation::new(Opacity::new(1.0)).start(3000).finish(3500).targeting(option_three_desc))
        .animate(Animation::new(Opacity::new(1.0)).start(3000).finish(3500).targeting(portfolio))
        .animate(
            Animation::new(Location::new().xs(4.col().as_x().with(5.row().as_y()), 9.col().as_x().with(5.row().as_y())))
                .start(1000)
                .finish(3000)
                .targeting(top_line),
        )
        .animate(
            Animation::new(Location::new().xs(7.col().as_x().with(5.row().as_y()), 7.col().as_x().with(8.row().as_y())))
                .start(1750)
                .finish(3000)
                .targeting(pad_connector),
        )
        .animate(
            Animation::new(Location::new().xs(
                anchor().right().as_x().adjust(16).with(1.row().as_y()),
                anchor().right().as_x().adjust(64).with(1.row().as_y()),
            ))
            .start(1750)
            .finish(2500)
            .targeting(github_line),
        )
        .animate(
            Animation::new(Location::new().xs(1.col().as_x().with(1.row().as_y()), 2.col().as_x().with(1.row().as_y())))
                .start(500)
                .finish(1000)
                .targeting(option_one_line),
        )
        .animate(
            Animation::new(Location::new().xs(4.col().as_x().with(2.row().as_y()), 5.col().as_x().with(2.row().as_y())))
                .start(1500)
                .finish(2000)
                .targeting(option_two_line),
        )
        .animate(
            Animation::new(Location::new().xs(1.col().as_x().with(3.row().as_y()), 2.col().as_x().with(3.row().as_y())))
                .start(2500)
                .finish(3000)
                .targeting(option_three_line),
        )
        .end(move |trigger: Trigger<OnEnd>, mut tree: Tree| {
            tree.enable([option_one, option_two, option_three, portfolio]);
        });
    tree.disable([github, option_one, option_two, option_three, portfolio]);
    tree.timer(1500, move |trigger: Trigger<OnEnd>, mut tree: Tree| {
        tree.enable(github);
    });
}
