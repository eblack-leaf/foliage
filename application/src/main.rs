use foliage::{
    EcsExtension, Foliage, Grid, GridExt, InteractionListener, Location, Stem,
};
fn main() {
    let mut foliage = Foliage::new(); // library-handle
    // foliage.enable_tracing(Targets::new().with_target("foliage", tracing::Level::TRACE));
    foliage.desktop_size((1600, 900));
    foliage.url("foliage");
    let root = foliage.leaf((
        Grid::new(25.col().gap(2), 25.row().gap(2)),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())),
        InteractionListener::new().scroll(true),
        Stem::none(),
    ));
    foliage.photosynthesize(); // run
}
