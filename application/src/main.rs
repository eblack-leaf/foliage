#![allow(unused)]
use foliage::{
    Animation, Color, EcsExtension, Elevation, Foliage, FontSize, GlyphColors, Grid, GridExt, Icon,
    InteractionListener, Line, Location, Logical, Opacity, Panel, Query, Rounding, Section, Stem,
    Text, Tree, Trigger, Write,
};
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
            1.col().span(40.px()).max(24.px()).min(24.px()),
            1.row().span(40.px()).max(24.px()).min(24.px()),
        ),
        Elevation::up(2),
        Stem::some(root),
        Color::gray(400),
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
    foliage.photosynthesize(); // run
}
