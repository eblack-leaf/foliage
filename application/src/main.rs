use foliage::{
    load_asset, Animation, Color, Elevation, Foliage, Grid, GridExt, Icon, Image, ImageView,
    InteractionListener, Justify, Location, OnEnd, Outline, Panel, Rounding, Stem, Tree, Trigger,
};
use tracing_subscriber::filter::Targets;
mod icon;
mod image;
fn main() {
    let mut foliage = Foliage::new(); // library-handle
    foliage.enable_tracing(Targets::new().with_target("foliage", tracing::Level::TRACE));
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
        Location::new().sm(1.col().to(4.col()), 1.row().to(6.row())),
        ImageView::Stretch,
        Elevation::new(2),
        Stem::some(root),
    ));
    foliage
        .world
        .spawn(Icon::memory(0, include_bytes!("assets/icons/grid.icon")));
    let icon = foliage.leaf((
        Icon::new(0),
        Color::gray(200),
        Location::new().xs(
            1.col()
                .to(3.col())
                .min(24.px())
                .max(24.px())
                .justify(Justify::Far),
            4.row().to(9.row()).min(24.px()).max(24.px()),
        ),
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
        Location::new().sm(0.pct().to(500.px()).pad(-10), 0.pct().to(500.px()).pad(-10)),
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
    foliage.animate(
        seq,
        Animation::new(Location::new().sm(300.px().to(10.col()), 4.row().to(12.row())))
            .start(100)
            .finish(1000)
            .targeting(img),
    );
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
