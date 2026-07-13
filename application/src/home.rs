use crate::icons::IconHandles;
use crate::portfolio;
use crate::widgets::icon_button;
use foliage::{
    anchor, Anchor, Animation, Color, EcsExtension, Elevation, Entity, FontSize, GlyphColors, Grid,
    GridExt, HorizontalAlignment, HrefLink, IconId, Keyring, Leaf, Line, Location, Logical,
    OnClick, OnEnd, Opacity, Query, Res, Rounding, Section, Sequence, Sprout, Text, TextValue,
    Tree, Trigger, VerticalAlignment, Write,
};

struct HeaderEntities {
    name: Entity,
    top_desc: Entity,
    top_line: Entity,
    side_desc: Entity,
    pad_connector: Entity,
    pad_desc: Entity,
    desc: Entity,
}

/// Title, three live layout-measurement readouts (width/half-width/pad-height), and the
/// tagline -- everything anchored under one `name_container`.
fn header<T: EcsExtension>(tree: &mut T, root: Entity) -> HeaderEntities {
    let name_container = tree.branch(
        root,
        Leaf::sprout()
            .at(Location::new().xs(
                1.col().as_left().with(12.col().as_right()).max(600.0),
                4.row().as_top().with(8.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(12.col().gap(4), 12.row().gap(4))),
    );
    let name = tree.branch(
        name_container,
        Text::new("foliage.rs")
            .size(FontSize::new(44))
            .at(Location::new().xs(
                2.col().as_left().with(11.col().as_right()),
                1.row().as_top().with(3.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with((
                HorizontalAlignment::Center,
                GlyphColors::new().add(7..10, Color::green(400)),
                Opacity::new(0.0),
            )),
    );
    let top_line = tree.branch(
        name_container,
        Line::new(2)
            .color(Color::gray(700))
            .at(Location::new().xs(
                4.col().as_x().with(5.row().as_y()),
                4.col().as_x().with(5.row().as_y()),
            ))
            .elevate(Elevation::up(1)),
    );
    let top_desc = measurement_readout(
        tree,
        name_container,
        top_line,
        "w",
        |s| s.width(),
        Location::new().xs(
            5.col().as_left().with(8.col().as_right()),
            4.row().as_top().with(4.row().as_bottom()),
        ),
    );
    let side_desc = measurement_readout(
        tree,
        name_container,
        top_line,
        "h",
        |s| s.width() * 0.5,
        Location::new().xs(
            9.col().as_left().with(11.col().as_right()),
            4.row().as_top().with(4.row().as_bottom()),
        ),
    );
    let pad_connector = tree.branch(
        name_container,
        Line::new(2)
            .color(Color::gray(700))
            .at(Location::new().xs(
                7.col().as_x().with(5.row().as_y()),
                7.col().as_x().with(5.row().as_y()),
            ))
            .elevate(Elevation::up(1)),
    );
    let pad_desc = measurement_readout(
        tree,
        name_container,
        pad_connector,
        "pad",
        |s| s.height(),
        Location::new().xs(
            8.col().as_left().with(11.col().as_right()),
            7.row().as_top().with(8.row().as_bottom()),
        ),
    );
    let desc = tree.branch(
        name_container,
        Text::new("native + web ui")
            .size(FontSize::new(24))
            .color(Color::gray(500))
            .at(Location::new().xs(
                1.col().as_left().with(12.col().as_right()),
                9.row().as_top().with(12.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with((
                HorizontalAlignment::Center,
                GlyphColors::new()
                    .add(7..8, Color::orange(700))
                    .add(13..15, Color::green(400)),
                Opacity::new(0.0),
            )),
    );
    HeaderEntities {
        name,
        top_desc,
        top_line,
        side_desc,
        pad_connector,
        pad_desc,
        desc,
    }
}

/// A faded-in `Text` that tracks `watch`'s `Section<Logical>` live, formatted as
/// `"<label>: <value>"` -- the shape all three of `header`'s readouts share, differing only in
/// which entity/dimension they read and their label.
fn measurement_readout<T: EcsExtension>(
    tree: &mut T,
    parent: Entity,
    watch: Entity,
    label: &'static str,
    read: impl Fn(&Section<Logical>) -> f32 + Send + Sync + 'static,
    at: Location,
) -> Entity {
    let desc = tree.branch(
        parent,
        Text::new(format!("{label}: 0.0"))
            .size(FontSize::new(14))
            .color(Color::gray(700))
            .at(at)
            .elevate(Elevation::up(1))
            .with(Opacity::new(0.0)),
    );
    tree.subscribe(
        watch,
        move |trigger: Trigger<Write<Section<Logical>>>,
              mut tree: Tree,
              sections: Query<&Section<Logical>>| {
            let value = read(sections.get(trigger.event_target()).unwrap());
            tree.write_to(desc, TextValue(format!("{label}: {value:.01}")));
        },
    );
    desc
}

struct GithubEntities {
    button: Entity,
    line: Entity,
    desc: Entity,
}

/// The github icon button + its connecting line + highlighted description -- kept separate
/// from `options` below despite the superficial resemblance: different font size, no
/// `HorizontalAlignment`/`VerticalAlignment`, and the description is anchored to the *line*
/// rather than laid out on the grid, so a shared helper would need about as many parameters as
/// just writing both out plainly.
fn github_link<T: EcsExtension>(tree: &mut T, root: Entity) -> GithubEntities {
    let button = tree.branch(
        root,
        icon_button(IconHandles::Github, Color::gray(200), Color::gray(800))
            .at(Location::new().xs(
                1.col().as_left().with(48.px().as_width()),
                1.row().as_top().with(48.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    tree.graft(button)
        .write((FontSize::new(16), Opacity::new(0.0)))
        .on_click(|_: Trigger<OnClick>| {
            HrefLink::new("https://github.com/eblack-leaf/foliage").navigate()
        });
    let line = tree.branch(
        root,
        Line::new(2)
            .color(Color::gray(700))
            .at(Location::new().xs(
                anchor().right().as_x().adjust(16).with(1.row().as_y()),
                anchor().right().as_x().adjust(16).with(1.row().as_y()),
            ))
            .elevate(Elevation::up(1))
            .with(Anchor::new(button)),
    );
    let desc = tree.branch(
        root,
        Text::new("on-click: github")
            .size(FontSize::new(14))
            .color(Color::gray(500))
            .at(Location::new().xs(
                anchor()
                    .right()
                    .as_left()
                    .adjust(16)
                    .with(10.col().as_right()),
                1.row().as_top().adjust(8).with(2.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with((
                GlyphColors::new().add(10..16, Color::green(300)),
                Anchor::new(line),
                Opacity::new(0.0),
            )),
    );
    GithubEntities { button, line, desc }
}

struct OptionRow {
    button: Entity,
    line: Entity,
    desc: Entity,
}

/// The three "on-click: usage/impl/docs" rows -- already a single `.map()` body rather than
/// copy-pasted, so this just gives that body a name and pulls its button through the
/// `icon_button` preset.
fn options<T: EcsExtension>(tree: &mut T, root: Entity) -> Vec<OptionRow> {
    let options_container = tree.branch(
        root,
        Leaf::sprout()
            .at(Location::new().xs(
                1.col().as_left().with(12.col().as_right()).max(600.0),
                10.row().as_top().with(13.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(5.col().gap(4), 3.row().gap(8))),
    );
    // row, icon, color, desc text, desc highlight range, desc column range, line column
    [
        (
            1,
            IconHandles::Terminal.into(),
            Color::green(700),
            "on-click: usage",
            10..15,
            (4, 5),
            1,
        ),
        (
            2,
            IconHandles::Layers.into(),
            Color::green(500),
            "on-click: impl",
            10..14,
            (1, 2),
            5,
        ),
        (
            3,
            IconHandles::BookOpen.into(),
            Color::green(300),
            "on-click: docs",
            10..14,
            (4, 5),
            1,
        ),
    ]
    .into_iter()
    .map(
        |(row, icon, color, desc_text, highlight, (desc_left, desc_right), line_col): (
            i32,
            IconId,
            Color,
            &str,
            std::ops::Range<usize>,
            (i32, i32),
            i32,
        )| {
            let button = tree.branch(
                options_container,
                icon_button(icon, color, Color::gray(900))
                    .outline(2)
                    .at(Location::new().xs(
                        3.col()
                            .as_left()
                            .with(3.col().as_right())
                            .max(48.0)
                            .min(48.0),
                        row.row()
                            .as_top()
                            .with(row.row().as_bottom())
                            .max(48.0)
                            .min(48.0),
                    ))
                    .elevate(Elevation::up(1)),
            );
            tree.graft(button)
                .write(Opacity::new(0.0))
                .on_click(move |_: Trigger<OnClick>| HrefLink::new("tbd").navigate());
            let line = tree.branch(
                options_container,
                Line::new(2)
                    .color(color)
                    .at(Location::new().xs(
                        line_col.col().as_x().with(row.row().as_y()),
                        line_col.col().as_x().with(row.row().as_y()),
                    ))
                    .elevate(Elevation::up(1)),
            );
            let desc = tree.branch(
                options_container,
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
            OptionRow { button, line, desc }
        },
    )
    .collect()
}

pub(crate) fn build<T: EcsExtension>(tree: &mut T) {
    let row_size = 40;
    let root = tree.leaf(
        Leaf::sprout()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::abs(0))
            .with(Grid::new(12.col().gap(8), row_size.px().gap(8))),
    );
    tree.name(root, "home");

    let header = header(tree, root);
    let github = github_link(tree, root);
    let rows = options(tree, root);

    let portfolio = tree.branch(
        root,
        foliage::Button::new()
            .icon(IconHandles::Code.into())
            .text("Portfolio")
            .rounding(Rounding::Sm)
            .colors(Color::orange(500), Color::gray(900))
            .outline(2)
            .at(Location::new().xs(
                3.col()
                    .as_left()
                    .with(10.col().as_right())
                    .min(175.0)
                    .max(350.0),
                15.row().as_top().with(48.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    tree.graft(portfolio)
        .write((FontSize::new(20), Opacity::new(0.0)))
        .on_click(
            move |_: Trigger<OnClick>, mut tree: Tree, keyring: Res<Keyring>| {
                tree.disable(root);
                portfolio::build(&mut tree, root, &keyring);
            },
        );

    let _spacing = tree.branch(
        root,
        Leaf::sprout()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                17.row().as_top().with(17.row().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    let [option_one, option_two, option_three] = [rows[0].button, rows[1].button, rows[2].button];
    let [option_one_desc, option_two_desc, option_three_desc] =
        [rows[0].desc, rows[1].desc, rows[2].desc];
    let [option_one_line, option_two_line, option_three_line] =
        [rows[0].line, rows[1].line, rows[2].line];
    Sequence::new(tree)
        .animate(
            Animation::new(Opacity::new(1.0))
                .start(500)
                .finish(1500)
                .targeting(header.name),
        )
        .animate(
            Animation::new(Opacity::new(1.0))
                .start(1000)
                .finish(1500)
                .targeting(github.button),
        )
        .animate(
            Animation::new(Opacity::new(1.0))
                .start(1000)
                .finish(1250)
                .targeting(header.top_desc),
        )
        .animate(
            Animation::new(Opacity::new(1.0))
                .start(1100)
                .finish(1350)
                .targeting(header.side_desc),
        )
        .animate(
            Animation::new(Opacity::new(1.0))
                .start(1500)
                .finish(1750)
                .targeting(header.pad_desc),
        )
        .animate(
            Animation::new(Opacity::new(1.0))
                .start(1750)
                .finish(2750)
                .targeting(header.desc),
        )
        .animate(
            Animation::new(Opacity::new(1.0))
                .start(2500)
                .finish(3000)
                .targeting(github.desc),
        )
        .animate(
            Animation::new(Opacity::new(1.0))
                .start(500)
                .finish(1000)
                .targeting(option_one),
        )
        .animate(
            Animation::new(Opacity::new(1.0))
                .start(1000)
                .finish(1500)
                .targeting(option_one_desc),
        )
        .animate(
            Animation::new(Opacity::new(1.0))
                .start(1500)
                .finish(2000)
                .targeting(option_two),
        )
        .animate(
            Animation::new(Opacity::new(1.0))
                .start(2000)
                .finish(2500)
                .targeting(option_two_desc),
        )
        .animate(
            Animation::new(Opacity::new(1.0))
                .start(2500)
                .finish(3000)
                .targeting(option_three),
        )
        .animate(
            Animation::new(Opacity::new(1.0))
                .start(3000)
                .finish(3500)
                .targeting(option_three_desc),
        )
        .animate(
            Animation::new(Opacity::new(1.0))
                .start(3000)
                .finish(3500)
                .targeting(portfolio),
        )
        .animate(
            Animation::new(Location::new().xs(
                4.col().as_x().with(5.row().as_y()),
                9.col().as_x().with(5.row().as_y()),
            ))
            .start(1000)
            .finish(3000)
            .targeting(header.top_line),
        )
        .animate(
            Animation::new(Location::new().xs(
                7.col().as_x().with(5.row().as_y()),
                7.col().as_x().with(8.row().as_y()),
            ))
            .start(1750)
            .finish(3000)
            .targeting(header.pad_connector),
        )
        .animate(
            Animation::new(Location::new().xs(
                anchor().right().as_x().adjust(16).with(1.row().as_y()),
                anchor().right().as_x().adjust(64).with(1.row().as_y()),
            ))
            .start(1750)
            .finish(2500)
            .targeting(github.line),
        )
        .animate(
            Animation::new(Location::new().xs(
                1.col().as_x().with(1.row().as_y()),
                2.col().as_x().with(1.row().as_y()),
            ))
            .start(500)
            .finish(1000)
            .targeting(option_one_line),
        )
        .animate(
            Animation::new(Location::new().xs(
                4.col().as_x().with(2.row().as_y()),
                5.col().as_x().with(2.row().as_y()),
            ))
            .start(1500)
            .finish(2000)
            .targeting(option_two_line),
        )
        .animate(
            Animation::new(Location::new().xs(
                1.col().as_x().with(3.row().as_y()),
                2.col().as_x().with(3.row().as_y()),
            ))
            .start(2500)
            .finish(3000)
            .targeting(option_three_line),
        )
        .end(move |_: Trigger<OnEnd>, mut tree: Tree| {
            tree.enable([option_one, option_two, option_three, portfolio]);
        });
    tree.disable([
        github.button,
        option_one,
        option_two,
        option_three,
        portfolio,
    ]);
    // aesthetic pacing, not a GPU-readiness wait -- the render pipeline queues anything not yet
    // ready on its own, this just avoids github becoming interactive the instant its own fade
    // finishes while everything else is still visibly settling in.
    tree.timer(1500, move |_: Trigger<OnEnd>, mut tree: Tree| {
        tree.enable(github.button);
    });
}
