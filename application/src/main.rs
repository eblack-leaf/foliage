use foliage::{load_asset, Animation, Color, EcsExtension, Elevation, Foliage, Grid, GridExt, Image, ImageView, InteractionListener, Location, OnEnd, Outline, Panel, Rounding, Stem, Tree, Trigger};
mod icon;
mod image;
fn main() {
    let mut foliage = Foliage::new(); // library-handle
    // foliage.enable_tracing(Targets::new().with_target("foliage", tracing::Level::TRACE));
    foliage.desktop_size((1024, 750)); // window-size
    foliage.url("foliage"); // web-path
    let root = foliage.leaf((
        Grid::new(12.col(), 12.row()),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())),
        InteractionListener::new().scroll(true),
        Stem::none(),
    ));
    let key = load_asset!(foliage, "assets/test.jpg");
    foliage.world.spawn(Image::memory(0, (333, 500)));
    let img = foliage.leaf((
        Image::new(0, key),
        Location::new().xs(1.col().to(4.col()), 1.row().to(6.row())),
        ImageView::None,
        Elevation::new(2),
        Stem::some(root),
    ));
    // let a = foliage.leaf((
    //     Text::new("Lorem ipsum dolor sit amet, consectetur adipiscing"),
    //     FontSize::new(32),
    //     AutoHeight(true),
    //     Stem::some(root),
    //     View::context(root),
    //     Location::new().xs(1.col().to(7.col()), 1.row().to(auto())),
    //     Grid::new(1.col().gap(0), 1.row().gap(0)),
    // ));
    let back = foliage.leaf((
        Panel::new(),
        Outline::new(0),
        Rounding::Xl,
        Stem::some(root),
        Elevation::new(4),
        Color::gray(500),
        Location::new().xs(0.pct().to(500.px()).pad(-10), 0.pct().to(500.px()).pad(-10)),
    ));
    // let b = foliage.leaf((
    //     Text::new("bbbbbbbbbb"),
    //     FontSize::new(20),
    //     AutoHeight(true),
    //     Color::gray(200),
    //     Elevation::new(6),
    //     Stem::some(root),
    //     Stack::new(a),
    //     View::context(root),
    //     Location::new().xs(2.col().to(7.col()), stack().to(auto())),
    // ));
    // let _c = foliage.leaf((
    //     Text::new("ccccccccc"),
    //     FontSize::new(16),
    //     AutoHeight(true),
    //     Color::gray(50),
    //     Stem::some(root),
    //     Stack::new(b),
    //     Elevation::new(2),
    //     View::context(root),
    //     Location::new().xs(2.col().to(7.col()), stack().to(auto())),
    // ));
    let seq = foliage.sequence();
    // foliage.animate(
    //     seq,
    //     Animation::new(Location::new().xs(300.px().to(10.col()), 4.row().to(auto())))
    //         .start(100)
    //         .finish(1000)
    //         .targeting(a),
    // );
    foliage.animate(
        seq,
        Animation::new(Outline::new(310))
            .start(100)
            .finish(2000)
            .targeting(back),
    );
    foliage.sequence_end(seq, move |trigger: Trigger<OnEnd>, mut tree: Tree| {
        tree.entity(img).insert(ImageView::Aspect);
        println!("finished {:?}", trigger.entity());
    });
    foliage.photosynthesize(); // run
}
