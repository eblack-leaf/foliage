#![allow(unused)]
use foliage::{
    auto, stack, Animation, AutoHeight, Color, EcsExtension, Elevation, Foliage, FontSize, Grid,
    GridExt, InteractionListener, Location, OnClick, OnEnd, Panel, Stack, Stem, Text, Tree,
    Trigger,
};
use tracing_subscriber::filter::Targets;
fn main() {
    let mut foliage = Foliage::new(); // library-handle
    foliage.enable_tracing(Targets::new().with_target("foliage", tracing::Level::TRACE));
    foliage.desktop_size((1860, 800));
    foliage.url("foliage");
    let root = foliage.leaf((
        Grid::new(4.col().gap(18), 25.row().gap(8)),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())),
        InteractionListener::new().scroll(true),
        Elevation::new(100),
        Stem::none(),
        // Visibility::new(false),
    ));
    // foliage.write_to(root, Visibility::new(false));
    let root_backdrop = foliage.leaf((
        Panel::new(),
        Color::gray(800),
        Elevation::new(-1),
        Location::new().xs(0.pct().to(110.pct()), 1.row().span(2000.px())),
        Stem::some(root),
    ));
    let n1 = foliage.leaf((
        Grid::new(1.col().gap(0), 1.row().gap(0)),
        Stem::some(root),
        Elevation::new(-1),
        Location::new().xs(1.col().to(2.col()), 7.row().to(12.row())),
        InteractionListener::new().scroll(true),
    ));
    let e1 = foliage.leaf((
        Text::new("abc def ghi jkl mnopqr stuvwx yzABC DEF GHIKJLM NOPQRSTU VWXY abc def ghi jkl mnopqr stuvwx yzABC DEF GHIKJLM NOPQRSTU VWXY abc def ghi jkl mnopqr stuvwx yzABC DEF GHIKJLM NOPQRSTU VWXY"),
        FontSize::new(24),
        Location::new().xs(0.pct().to(120.pct()), 1.row().span(auto())),
        Grid::default(),
        AutoHeight(true),
        Elevation::new(-2),
        Stem::some(n1),
    ));
    let dt1 = foliage.leaf((
        Panel::new(),
        Color::gray(250),
        Elevation::new(-2),
        Location::new().xs(0.pct().to(100.pct()), stack().span(100.px())),
        Stack::new(e1),
        Stem::some(n1),
        InteractionListener::new(),
    ));
    let sn1 = foliage.leaf((
        Panel::new(),
        Color::gray(350),
        Location::new().xs(0.pct().to(100.pct()), stack().span(200.px())),
        Elevation::new(-3),
        Stem::some(n1),
        Stack::new(dt1),
        Grid::default(),
        InteractionListener::new().scroll(true),
    ));
    let snt1 = foliage.leaf((
        Text::new(" osaeta oeu u uu u u u u u  u u u u  u u  uu  uu u u u  u uu u u  u u u u u  uuuuuuuuu u uuuuu uuuuuuuu uuuuu uuuuuu uuu u u u u u uu u u uuu u uuu u u u uuuuuuuu uuuuuuuuu"),
        FontSize::new(24),
        Elevation::new(-2),
        Stem::some(sn1),
        AutoHeight(true),
        Location::new().xs(0.pct().to(100.pct()), 10.px().span(auto())),
    ));
    foliage.world.commands().entity(dt1).observe(move |trigger: Trigger<OnClick>, mut tree: Tree| {
        let seq = tree.sequence();
        tree.animate(seq, Animation::new(Location::new().xs(1.col().to(4.col()), 7.row().to(12.row()))).start(0).finish(10000).targeting(n1));
        tree.sequence_end(seq, move |trigger: Trigger<OnEnd>, mut tree: Tree| {
            tree.remove(n1);
            println!("done");
        });
        println!("done-it --------------------------------------------------------------------------------------------------------------------------------------------------------------------------");
    });
    let nb1 = foliage.leaf((
        Panel::new(),
        Color::gray(500),
        Elevation::new(1),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(1000.px())),
        Stem::some(e1),
    ));
    println!(
        "r: {:?} rb: {:?} n: {:?} e: {:?} dt: {:?} sn: {:?} snt: {:?} nb: {:?}",
        root, root_backdrop, n1, e1, dt1, sn1, snt1, nb1
    );

    let nested = foliage.leaf((
        Grid::new(1.col().gap(0), 1.row().gap(0)),
        Stem::some(root),
        Elevation::new(-1),
        Location::new().xs(1.col().to(2.col()), 1.row().to(6.row())),
        InteractionListener::new().scroll(true),
    ));
    let element = foliage.leaf((
        Text::new("abc def ghi jkl mnopqr stuvwx yzABC DEF GHIKJLM NOPQRSTU VWXY abc def ghi jkl mnopqr stuvwx yzABC DEF GHIKJLM NOPQRSTU VWXY abc def ghi jkl mnopqr stuvwx yzABC DEF GHIKJLM NOPQRSTU VWXY"),
        FontSize::new(24),
        Location::new().xs(0.pct().to(100.pct()), 1.row().span(auto())),
        Grid::default(),
        AutoHeight(true),
        Elevation::new(-2),
        Stem::some(nested),
    ));
    let drag_test = foliage.leaf((
        Panel::new(),
        Color::gray(250),
        Elevation::new(-2),
        Location::new().xs(0.pct().to(100.pct()), stack().span(100.px())),
        Stack::new(element),
        Stem::some(nested),
        InteractionListener::new(),
    ));
    let supr_nest = foliage.leaf((
        Panel::new(),
        Color::gray(350),
        Location::new().xs(0.pct().to(100.pct()), stack().span(200.px())),
        Elevation::new(-3),
        Stem::some(nested),
        Stack::new(drag_test),
        Grid::default(),
        InteractionListener::new().scroll(true),
    ));
    let supr_nest_text = foliage.leaf((
        Text::new(" osaeta oeu u uu u u u u u  u u u u  u u  uu  uu u u u  u uu u u  u u u u u  uuuuuuuuu u uuuuu uuuuuuuu uuuuu uuuuuu uuu u u u u u uu u u uuu u uuu u u u uuuuuuuu uuuuuuuuu"),
        FontSize::new(24),
        Elevation::new(-2),
        Stem::some(supr_nest),
        AutoHeight(true),
        Location::new().xs(0.pct().to(100.pct()), 10.px().span(auto())),
    ));
    foliage.world.commands().entity(drag_test).observe(move |trigger: Trigger<OnClick>, mut tree: Tree| {
        let seq = tree.sequence();
        tree.animate(seq, Animation::new(Location::new().xs(1.col().to(4.col()), 1.row().to(6.row()))).start(0).finish(10000).targeting(nested));
        tree.sequence_end(seq, move |trigger: Trigger<OnEnd>, mut tree: Tree| {
            tree.remove(nested);
            println!("did");
        });
        println!("did-it --------------------------------------------------------------------------------------------------------------------------------------------------------------------------");
    });
    let nested_backdrop = foliage.leaf((
        Panel::new(),
        Color::gray(500),
        Elevation::new(1),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(1000.px())),
        Stem::some(element),
    ));
    println!(
        "r: {:?} rb: {:?} n: {:?} e: {:?} dt: {:?} sn: {:?} snt: {:?} nb: {:?}",
        root, root_backdrop, nested, element, drag_test, supr_nest, supr_nest_text, nested_backdrop
    );
    foliage.photosynthesize(); // run
}
