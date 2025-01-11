use foliage::{auto, AutoHeight, Color, EcsExtension, Elevation, Foliage, FontSize, Grid, GridExt, InteractionListener, Location, Panel, Stem, Text, View};
use tracing_subscriber::filter::Targets;
fn main() {
    let mut foliage = Foliage::new(); // library-handle
    foliage.enable_tracing(Targets::new().with_target("foliage", tracing::Level::TRACE));
    foliage.desktop_size((1600, 900));
    foliage.url("foliage");
    let root = foliage.leaf((
        Grid::new(25.col().gap(8), 25.row().gap(8)),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())),
        InteractionListener::new().scroll(true),
        Elevation::new(100),
        Stem::none(),
    ));
    let root_backdrop = foliage.leaf((
        Panel::new(),
        Color::gray(800),
        Elevation::new(-1),
        Location::new().xs(1.col().to(25.col()), 1.row().to(2000.px())),
        View::context(root),
        Stem::some(root),
    ));
    let nested = foliage.leaf((
        Grid::new(1.col().gap(0), 1.row().gap(0)),
        Stem::some(root),
        Elevation::new(-1),
        Location::new().xs(1.col().to(10.col()), 1.row().to(4.row())),
        View::context(root),
        InteractionListener::new().scroll(true),
    ));
    let element = foliage.leaf((
        Text::new("etuhas u tehase tusae unsaentu e uthet usaentuhsaonet usanoet uhsanoteuhsan oteuheute sanetuhs anoethus etuhte sanutehsantoehsunaot untesunaotehu"),
        FontSize::new(24),
        Location::new().xs(1.col().to(1.col()), 1.row().to(auto())),
        AutoHeight(true),
        View::context(nested),
        Elevation::new(-1),
        Stem::some(nested),
    ));
    foliage.photosynthesize(); // run
}
