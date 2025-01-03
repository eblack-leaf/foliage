use foliage::{
    auto, stack, Animation, AutoHeight, Color, Foliage, FontSize, Grid, GridExt,
    InteractionListener, Location, OnEnd, Opacity, Stack, Stem, Text, Trigger, View,
};
use tracing_subscriber::filter::Targets;

mod icon;
mod image;
fn main() {
    let mut foliage = Foliage::new(); // library-handle
    foliage.enable_tracing(Targets::new().with_target("foliage", tracing::Level::TRACE));
    foliage.desktop_size((400, 600)); // window-size
    foliage.url("foliage"); // web-path
    let root = foliage.leaf((
        Grid::new(12.col(), 24.px()),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())),
        InteractionListener::new().scroll(true),
        Stem::none(),
    ));
    let a = foliage.leaf((
        Text::new("asonetuhsanot aoentuh"),
        FontSize::new(20).sm(32),
        AutoHeight(true),
        Stem::some(root),
        View::context(root),
        Location::new().xs(1.col().to(4.col()), 1.row().to(auto())),
    ));
    let b = foliage.leaf((
        Text::new("xxxx xxx x xx x xxxx x "),
        FontSize::new(20),
        AutoHeight(true),
        Color::gray(500),
        Stem::some(root),
        Stack::new(a),
        View::context(root),
        Location::new().xs(1.col().to(4.col()), stack().to(auto())),
    ));
    let _c = foliage.leaf((
        Text::new("yyyy yyy y yy"),
        FontSize::new(20),
        AutoHeight(true),
        Color::gray(500),
        Stem::some(root),
        Stack::new(b),
        View::context(root),
        Location::new().xs(5.col().to(9.col()), stack().to(auto())),
    ));
    let seq = foliage.sequence();
    foliage.animate(
        seq,
        Animation::new(Opacity::new(0.0))
            .start(100)
            .finish(500)
            .targeting(a),
    );
    foliage.sequence_end(seq, |trigger: Trigger<OnEnd>| {
        println!("finished {:?}", trigger.entity());
    });
    foliage.photosynthesize(); // run
}
