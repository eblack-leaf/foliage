#![allow(unused)]
use foliage::Justify::Far;
use foliage::{
    stack, Animation, Color, EcsExtension, Elevation, Foliage, FontSize, GlyphColors, Grid,
    GridExt, HorizontalAlignment, Icon, InteractionListener, Line, Location, Logical, Opacity,
    Outline, Panel, Query, Rounding, Section, Stack, Stem, Text, Tree, Trigger, VerticalAlignment,
    Write,
};

fn main() {
    let mut foliage = Foliage::new();
    // foliage.enable_tracing(Targets::new().with_target("foliage", tracing::Level::TRACE));
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
        Location::new().xs(1.col().to(12.col()).max(600.px()), 4.row().to(8.row())),
        Stem::some(root),
        Elevation::up(1),
    ));
    let name = foliage.leaf((
        Text::new("foliage.rs"),
        FontSize::new(44),
        HorizontalAlignment::Center,
        Location::new().xs(2.col().to(11.col()), 1.row().to(3.row())),
        Elevation::up(1),
        GlyphColors::new().add(7..10, Color::green(400)),
        Stem::some(name_container),
        Opacity::new(0.0),
    ));
    let top_desc = foliage.leaf((
        Text::new("w: 0.0"),
        FontSize::new(14),
        Location::new().xs(5.col().to(8.col()), 4.row().to(4.row())),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
        Opacity::new(0.0),
    ));
    let top_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(4.col().y(5.row()), 4.col().y(5.row())),
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
        Location::new().xs(9.col().to(11.col()), 4.row().to(4.row())),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
        Opacity::new(0.0),
    ));
    let side_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(12.col().y(-2.row()), 12.col().y(-2.row())),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
        Opacity::new(0.0),
    ));
    foliage.world.commands().entity(side_line).observe(
        move |trigger: Trigger<Write<Section<Logical>>>,
              mut tree: Tree,
              sections: Query<&Section<Logical>>| {
            let h = sections.get(trigger.entity()).unwrap().height();
            tree.write_to(side_desc, Text::new(format!("h: {:.01}", h)));
        },
    );
    let pad_connector = foliage.leaf((
        Line::new(2),
        Location::new().xs(7.col().y(6.row()), 7.col().y(6.row())),
        Stem::some(name_container),
        Elevation::up(1),
        Color::gray(700),
    ));
    let pad_desc = foliage.leaf((
        Text::new("pad: 0.0"),
        FontSize::new(14),
        Location::new().xs(8.col().to(11.col()), 7.row().to(8.row())),
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
        Text::new("native + web ui"),
        FontSize::new(24),
        HorizontalAlignment::Center,
        Location::new().xs(1.col().to(12.col()), 9.row().to(12.row())),
        Elevation::up(1),
        GlyphColors::new()
            // .add(0..6, Color::green(700))
            .add(7..8, Color::orange(500))
            .add(13..15, Color::green(400)),
        Stem::some(name_container),
        Opacity::new(0.0),
        Color::gray(500),
    ));
    let github = foliage.leaf((
        Panel::new(),
        Rounding::Full,
        Location::new().xs(1.col().span(40.px()), 1.row().span(40.px())),
        Elevation::up(1),
        Stem::some(root),
        Color::gray(800),
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
        Color::gray(500),
    ));
    let options_container = foliage.leaf((
        Grid::new(5.col().gap(4), 3.row().gap(8)),
        Location::new().xs(
            1.col().to(12.col()).max(600.px()),
            10.row()
                .to(100.pct())
                .pad((0, 8))
                .min(175.px())
                .max(200.px())
                .justify(Far),
        ),
        Stem::some(root),
        Elevation::up(1),
    ));
    let option_one_color = Color::green(700);
    let option_one = foliage.leaf((
        Panel::new(),
        Rounding::Full,
        Location::new().xs(
            3.col().to(3.col()).max(40.px()).min(40.px()),
            1.row().to(1.row()).max(40.px()).min(40.px()),
        ),
        Elevation::up(1),
        Stem::some(options_container),
        Outline::new(2),
        option_one_color,
        Opacity::new(0.0),
    ));
    foliage.world.spawn(Icon::memory(
        1,
        include_bytes!("assets/icons/terminal.icon"),
    ));
    let option_one_icon = foliage.leaf((
        Icon::new(1),
        Location::new().xs(
            3.col().to(3.col()).max(24.px()).min(24.px()),
            1.row().to(1.row()).max(24.px()).min(24.px()),
        ),
        Elevation::up(2),
        Stem::some(options_container),
        option_one_color,
        Opacity::new(0.0),
    ));
    let option_one_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(1.col().y(1.row()), 1.col().y(1.row())),
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
        Location::new().xs(4.col().to(5.col()), 1.row().to(1.row())),
        Elevation::up(1),
        Stem::some(options_container),
        Opacity::new(0.0),
        Color::gray(500),
    ));
    let option_two_color = Color::green(500);
    let option_two = foliage.leaf((
        Panel::new(),
        Rounding::Full,
        Location::new().xs(
            3.col().to(3.col()).max(40.px()).min(40.px()),
            2.row().to(2.row()).max(40.px()).min(40.px()),
        ),
        Elevation::up(1),
        Stem::some(options_container),
        Outline::new(2),
        option_two_color,
        Opacity::new(0.0),
    ));
    foliage
        .world
        .spawn(Icon::memory(2, include_bytes!("assets/icons/layers.icon")));
    let option_two_icon = foliage.leaf((
        Icon::new(2),
        Location::new().xs(
            3.col().to(3.col()).max(24.px()).min(24.px()),
            2.row().to(2.row()).max(24.px()).min(24.px()),
        ),
        Elevation::up(2),
        Stem::some(options_container),
        option_two_color,
        Opacity::new(0.0),
    ));
    let option_two_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(5.col().y(2.row()), 5.col().y(2.row())),
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
        Location::new().xs(1.col().to(2.col()), 2.row().to(2.row())),
        Elevation::up(1),
        Stem::some(options_container),
        Color::gray(500),
        Opacity::new(0.0),
    ));
    let option_three_color = Color::green(300);
    let option_three = foliage.leaf((
        Panel::new(),
        Rounding::Full,
        Location::new().xs(
            3.col().to(3.col()).max(40.px()).min(40.px()),
            3.row().to(3.row()).max(40.px()).min(40.px()),
        ),
        Elevation::up(1),
        Stem::some(options_container),
        Outline::new(2),
        option_three_color,
        Opacity::new(0.0),
    ));
    foliage.world.spawn(Icon::memory(
        3,
        include_bytes!("assets/icons/book-open.icon"),
    ));
    let option_three_icon = foliage.leaf((
        Icon::new(3),
        Location::new().xs(
            3.col().to(3.col()).max(24.px()).min(24.px()),
            3.row().to(3.row()).max(24.px()).min(24.px()),
        ),
        Elevation::up(2),
        Stem::some(options_container),
        option_three_color,
        Opacity::new(0.0),
    ));
    let option_three_line = foliage.leaf((
        Line::new(2),
        Location::new().xs(1.col().y(3.row()), 1.col().y(3.row())),
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
        Location::new().xs(4.col().to(5.col()), 3.row().to(3.row())),
        Elevation::up(1),
        Stem::some(options_container),
        Color::gray(500),
        Opacity::new(0.0),
    ));
    let options_backdrop = foliage.leaf((
        Panel::new(),
        Rounding::Xs,
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(0.pct())),
        Color::gray(900),
        Elevation::up(0),
        Stem::some(options_container),
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
        .start(3000)
        .finish(3500)
        .targeting(option_one);
    foliage.animate(seq, anim);
    let anim = Animation::new(Opacity::new(1.0))
        .start(3250)
        .finish(3750)
        .targeting(option_one_icon);
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
        .start(4250)
        .finish(4750)
        .targeting(option_two_icon);
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
        .start(5500)
        .finish(6000)
        .targeting(option_three_icon);
    foliage.animate(seq, anim);
    let anim = Animation::new(Opacity::new(1.0))
        .start(5750)
        .finish(6250)
        .targeting(option_three_desc);
    foliage.animate(seq, anim);
    let anim = Animation::new(Location::new().xs(4.col().y(5.row()), 9.col().y(5.row())))
        .start(1000)
        .finish(3000)
        .targeting(top_line);
    foliage.animate(seq, anim);
    let anim = Animation::new(Location::new().xs(12.col().y(-2.row()), 12.col().y(5.row())))
        .start(1250)
        .finish(3500)
        .targeting(side_line);
    foliage.animate(seq, anim);
    let anim = Animation::new(Location::new().xs(7.col().y(6.row()), 7.col().y(8.row())))
        .start(1750)
        .finish(3000)
        .targeting(pad_connector);
    foliage.animate(seq, anim);
    let anim = Animation::new(Location::new().xs(
        stack().y(1.row()).pad((16, 0)),
        stack().y(1.row()).pad((64, 0)),
    ))
        .start(1750)
        .finish(2500)
        .targeting(github_line);
    foliage.animate(seq, anim);
    let anim = Animation::new(Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())))
        .start(2500)
        .finish(3500)
        .targeting(options_backdrop);
    foliage.animate(seq, anim);
    let anim = Animation::new(Location::new().xs(1.col().y(1.row()), 2.col().y(1.row())))
        .start(2500)
        .finish(3000)
        .targeting(option_one_line);
    foliage.animate(seq, anim);
    let anim = Animation::new(Location::new().xs(4.col().y(2.row()), 5.col().y(2.row())))
        .start(3500)
        .finish(4000)
        .targeting(option_two_line);
    foliage.animate(seq, anim);
    let anim = Animation::new(Location::new().xs(1.col().y(3.row()), 2.col().y(3.row())))
        .start(4750)
        .finish(5250)
        .targeting(option_three_line);
    foliage.animate(seq, anim);
    foliage.photosynthesize(); // run
}
