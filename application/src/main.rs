#![allow(unused)]
use foliage::{stack, Animation, Color, EcsExtension, Elevation, Foliage, FontSize, GlyphColors, Grid, GridExt, Icon, InteractionListener, Line, Location, Logical, Opacity, Outline, Panel, Query, Rounding, Section, Stack, Stem, Text, Tree, Trigger, Write};
use tracing_subscriber::filter::Targets;
fn main() {
    let mut foliage = Foliage::new();
    foliage.enable_tracing(Targets::new().with_target("foliage", tracing::Level::TRACE));
    foliage.desktop_size((360, 800));
    foliage.url("foliage");
    let row_size = 40;
    let root = foliage.leaf((
        Grid::new(12.col().gap(8), row_size.px().gap(8)),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())),
        InteractionListener::new().scroll(true),
        Elevation::abs(0),
        Stem::none(),
    ));
    let name_container = foliage.leaf((
        Grid::new(12.col().gap(4), 12.row().gap(4)),
        Location::new().xs(1.col().to(12.col()), 4.row().to(8.row())),
        Stem::some(root),
        Elevation::up(1),
    ));
    let name = foliage.leaf((
        Text::new("foliage.rs"),
        FontSize::new(44),
        Location::new().xs(2.col().to(11.col()), 3.row().to(6.row())),
        Elevation::up(1),
        GlyphColors::new().add(7..10, Color::orange(600)),
        Stem::some(name_container),
        Opacity::new(0.0),
    ));
    let top_desc = foliage.leaf((
        Text::new("w: 0.0"),
        FontSize::new(14),
        Location::new().xs(2.col().to(5.col()), 1.row().to(2.row())),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
        Opacity::new(0.0),
    ));
    let top_line = foliage.leaf((
        Line::new(2),
        Location::new().xs((-1).col().y(2.row()), (-1).col().y(2.row())),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
    ));
    foliage.world.commands().entity(top_line).observe(
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
        Location::new().xs(6.col().to(9.col()), 1.row().to(2.row())),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
        Opacity::new(0.0),
    ));
    let side_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(1.col().y(-2.row()), 1.col().y(-2.row())),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
    ));
    foliage.world.commands().entity(side_line).observe(
        move |trigger: Trigger<Write<Section<Logical>>>,
              mut tree: Tree,
              sections: Query<&Section<Logical>>| {
            let h = sections.get(trigger.entity()).unwrap().height();
            tree.write_to(side_desc, Text::new(format!("h: {:.01}", h)));
        },
    );
    let pad_top = foliage.leaf((
        Line::new(2),
        Location::new().xs(4.col().y(6.row()), 4.col().y(6.row())),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
    ));
    let pad_connector = foliage.leaf((
        Line::new(2),
        Location::new().xs(5.col().y(6.row()), 5.col().y(6.row())),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
    ));
    let pad_bot = foliage.leaf((
        Line::new(2),
        Location::new().xs(5.col().y(8.row()), 5.col().y(8.row())),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
    ));
    let pad_desc = foliage.leaf((
        Text::new("pad: 0.0"),
        FontSize::new(14),
        Location::new().xs(6.col().to(9.col()), 7.row().to(8.row())),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
        Opacity::new(0.0),
    ));
    foliage.world.commands().entity(pad_connector).observe(
        move |trigger: Trigger<Write<Section<Logical>>>,
              mut tree: Tree,
              sections: Query<&Section<Logical>>| {
            let h = sections.get(trigger.entity()).unwrap().height();
            tree.write_to(pad_desc, Text::new(format!("pad: {:.01}", h)));
        },
    );
    let desc = foliage.leaf((
        Text::new("cross-platform ui"),
        FontSize::new(24),
        Location::new().xs(3.col().to(12.col()), 9.row().to(12.row())),
        Elevation::up(1),
        GlyphColors::new().add(15..17, Color::green(600)),
        Stem::some(name_container),
        Opacity::new(0.0),
        Color::gray(400),
    ));
    let github = foliage.leaf((
        Panel::new(),
        Rounding::Full,
        Location::new().xs(1.col().span(40.px()), 1.row().span(40.px())),
        Elevation::up(1),
        Stem::some(root),
        Color::gray(900),
    ));
    foliage
        .world
        .spawn(Icon::memory(0, include_bytes!("assets/icons/github.icon")));
    let github_icon = foliage.leaf((
        Icon::new(0),
        Location::new().xs(
            1.col().span(row_size.px()).max(24.px()).min(24.px()),
            1.row().span(row_size.px()).max(24.px()).min(24.px()),
        ),
        Elevation::up(2),
        Stem::some(root),
        Color::gray(400),
    ));
    let github_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(
            stack().y(1.row()).pad((16, 0)),
            stack().y(1.row()).pad((16, 0)),
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
            stack().to(10.col()).pad((16, 0)),
            1.row().to(2.row()).pad((8, 0)),
        ),
        Elevation::up(1),
        GlyphColors::new().add(10..16, Color::green(300)),
        Stem::some(root),
        Stack::new(github_line),
        Opacity::new(0.0),
        Color::gray(700),
    ));
    let grid_container = foliage.leaf((
        Grid::new(12.col().gap(4), 14.row().gap(4)),
        Location::new().xs(4.col().to(12.col()), 11.row().to(16.row())),
        Stem::some(root),
        Elevation::up(1),
    ));
    let grid_desc = foliage.leaf((
        Text::new("grid: 3.col() 3.row()"),
        FontSize::new(14),
        Location::new().xs(1.col().to(12.col()), 1.row().to(2.row())),
        Elevation::up(1),
        GlyphColors::new().add(8..11, Color::green(600)).add(16..19, Color::green(600)),
        Stem::some(grid_container),
        Opacity::new(1.0),
        Color::gray(700),
    ));
    let first_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(5.col().y(3.row()), 5.col().y(12.row())),
        Stem::some(grid_container),
        Elevation::up(1),
        Color::gray(700),
    ));
    let first = foliage.leaf((
        Panel::new(),
        Rounding::Sm,
        Location::new().xs(2.col().to(4.col()), 3.row().to(5.row())),
        Elevation::up(1),
        Stem::some(grid_container),
        // Opacity::new(0.0),
        Color::gray(800),
    ));
    let second_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(3.col().y(6.row()), 6.col().y(6.row())),
        Stem::some(grid_container),
        Elevation::up(1),
        Color::gray(700),
    ));
    let second = foliage.leaf((
        Panel::new(),
        Rounding::Sm,
        Outline::new(2),
        Location::new().xs(2.col().to(8.col()), 7.row().to(9.row())),
        Elevation::up(1),
        Stem::some(grid_container),
        // Opacity::new(0.0),
        Color::orange(800),
    ));
    let second_desc = foliage.leaf((
        Text::new("IMPL"),
        FontSize::new(24),
        Location::new().xs(4.col().to(7.col()), 7.row().to(9.row()).pad((10, 0))),
        Elevation::up(2),
        Stem::some(grid_container),
        Opacity::new(1.0),
        Color::gray(400),
    ));
    let third_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(1.col().y(10.row()), 10.col().y(10.row())),
        Stem::some(grid_container),
        Elevation::up(1),
        Color::gray(700),
    ));
    let third = foliage.leaf((
        Panel::new(),
        Rounding::Sm,
        Location::new().xs(6.col().to(11.col()), 11.row().to(13.row())),
        Elevation::up(1),
        Stem::some(grid_container),
        // Opacity::new(0.0),
        Color::gray(800),
    ));
    let fourth_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(9.col().y(9.row()), 9.col().y(14.row())),
        Stem::some(grid_container),
        Elevation::up(1),
        Color::gray(700),
    ));
    let seq = foliage.sequence();
    let anim = Animation::new(Opacity::new(1.0))
        .start(500)
        .finish(1500)
        .targeting(name);
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
        .start(3500)
        .finish(4000)
        .targeting(grid_desc);
    foliage.animate(seq, anim);
    let anim = Animation::new(Location::new().xs((-1).col().y(2.row()), 7.col().y(2.row())))
        .start(1000)
        .finish(3000)
        .targeting(top_line);
    foliage.animate(seq, anim);
    let anim = Animation::new(Location::new().xs(1.col().y(-2.row()), 1.col().y(5.row())))
        .start(1250)
        .finish(3500)
        .targeting(side_line);
    foliage.animate(seq, anim);
    let anim = Animation::new(Location::new().xs(4.col().y(6.row()), 6.col().y(6.row())))
        .start(1500)
        .finish(3000)
        .targeting(pad_top);
    foliage.animate(seq, anim);
    let anim = Animation::new(Location::new().xs(5.col().y(6.row()), 5.col().y(8.row())))
        .start(1750)
        .finish(3000)
        .targeting(pad_connector);
    foliage.animate(seq, anim);
    let anim = Animation::new(Location::new().xs(
        5.col().y(8.row()).pad((-12, 0)),
        9.col().y(8.row()),
    ))
        .start(2000)
        .finish(2500)
        .targeting(pad_bot);
    foliage.animate(seq, anim);
    let anim = Animation::new(Location::new().xs(
        stack().y(1.row()).pad((16, 0)),
        stack().y(1.row()).pad((64, 0)),
    ))
        .start(1750)
        .finish(2500)
        .targeting(github_line);
    foliage.animate(seq, anim);
    foliage.photosynthesize(); // run
}
