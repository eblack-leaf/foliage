use foliage::{
    auto, stack, Animation, AutoHeight, Color, Elevation, Foliage, FontSize, Grid, GridExt,
    InteractionListener, Location, OnEnd, Outline, Panel, Rounding, Stack, Stem, Text, Trigger,
    View,
};
mod icon;
mod image;
fn main() {
    let mut foliage = Foliage::new(); // library-handle
    // foliage.enable_tracing(Targets::new().with_target("foliage", tracing::Level::TRACE));
    foliage.desktop_size((1024, 750)); // window-size
    foliage.url("foliage"); // web-path
    let root = foliage.leaf((
        Grid::new(12.col(), 24.px()),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())),
        InteractionListener::new().scroll(true),
        Stem::none(),
    ));
    let a = foliage.leaf((
        Text::new("Lorem ipsum dolor sit amet, consectetur adipiscing"),
        FontSize::new(32),
        AutoHeight(true),
        Stem::some(root),
        View::context(root),
        Location::new().xs(1.col().to(7.col()), 1.row().to(auto())),
        Grid::new(1.col().gap(0), 1.row().gap(0)),
    ));
    let back = foliage.leaf((
        Panel::new(),
        Outline::new(4),
        Rounding::Xl,
        Stem::some(a),
        Elevation::new(1),
        Color::gray(500),
        Location::new().xs(
            0.pct().to(1000.px()).pad(-10),
            0.pct().to(100.px()).pad(-10),
        ),
    ));
    let b = foliage.leaf((
        Text::new("Lorem ipsum dolor sit amet, consectetur adipiscing"),
        FontSize::new(20),
        AutoHeight(true),
        Color::gray(200),
        Stem::some(root),
        Stack::new(a),
        View::context(root),
        Location::new().xs(2.col().to(7.col()), stack().to(auto())),
    ));
    let _c = foliage.leaf((
        Text::new("Lorem ipsum dolor sit amet, consectetur adipiscing"),
        FontSize::new(16),
        AutoHeight(true),
        Color::gray(500),
        Stem::some(root),
        Stack::new(b),
        View::context(root),
        Location::new().xs(2.col().to(7.col()), stack().to(auto())),
    ));
    let seq = foliage.sequence();
    foliage.animate(
        seq,
        Animation::new(Location::new().xs(3.col().to(10.col()), 4.row().to(auto())))
            .start(500)
            .finish(1500)
            .targeting(a),
    );
    foliage.sequence_end(seq, |trigger: Trigger<OnEnd>| {
        println!("finished {:?}", trigger.entity());
    });
    foliage.photosynthesize(); // run
}
