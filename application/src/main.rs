use foliage::{
    auto, stack, AutoHeight, Color, EcsExtension, Elevation, Foliage, FontSize, Grid, GridExt,
    InteractionListener, Location, OnClick, Panel, Stack, Stem, Text, Trigger, View,
};
use tracing_subscriber::filter::Targets;
fn main() {
    let mut foliage = Foliage::new(); // library-handle
    foliage.enable_tracing(Targets::new().with_target("foliage", tracing::Level::TRACE));
    foliage.desktop_size((360, 800));
    foliage.url("foliage");
    let root = foliage.leaf((
        Grid::new(25.col().gap(18), 25.row().gap(8)),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())),
        InteractionListener::new().scroll(true),
        Elevation::new(100),
        Stem::none(),
    ));
    let root_backdrop = foliage.leaf((
        Panel::new(),
        Color::gray(800),
        Elevation::new(-1),
        Location::new().xs(0.pct().to(100.pct()), 1.row().span(2000.px())),
        View::context(root),
        Stem::some(root),
    ));
    let nested = foliage.leaf((
        Grid::new(1.col().gap(0), 1.row().gap(0)),
        Stem::some(root),
        Elevation::new(-1),
        Location::new().xs(1.col().to(20.col()), 1.row().to(6.row())),
        View::context(root),
        InteractionListener::new().scroll(true),
    ));
    let element = foliage.leaf((
        Text::new("etuhas u tehase tusae unsaentu e uthet usaentuhsaonet usanoet uhsanoteuhsan oteuheute sanetuhs anoethus etuhte sanutehsantoehsunaot untesunaotehu"),
        FontSize::new(24),
        Location::new().xs(0.pct().to(100.pct()), 1.row().span(auto())),
        Grid::default(),
        AutoHeight(true),
        View::context(nested),
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
        View::context(nested),
        InteractionListener::new(),
    ));
    let supr_nest = foliage.leaf((
        Panel::new(),
        Color::gray(350),
        Location::new().xs(0.pct().to(100.pct()), stack().span(200.px())),
        Elevation::new(-3),
        Stem::some(nested),
        View::context(nested),
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
        View::context(nested),
        Location::new().xs(0.pct().to(100.pct()), 10.px().span(auto())),
    ));
    foliage.world.commands().entity(drag_test).observe(|trigger: Trigger<OnClick>| {
        println!("done-it --------------------------------------------------------------------------------------------------------------------------------------------------------------------------");
    });
    let nested_backdrop = foliage.leaf((
        Panel::new(),
        Color::gray(500),
        Elevation::new(1),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(1000.px())),
        View::context(nested),
        Stem::some(element),
    ));
    println!("r: {:?} rb: {:?} n: {:?} e: {:?} dt: {:?} sn: {:?} snt: {:?} nb: {:?}", root, root_backdrop, nested, element, drag_test, supr_nest, supr_nest_text, nested_backdrop);
    foliage.photosynthesize(); // run
}
