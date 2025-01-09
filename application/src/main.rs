use foliage::{Animation, Color, Elevation, Foliage, Grid, GridExt, InteractionListener, Location, OnEnd, Outline, Panel, Rounding, Stem, Text, Tree, Trigger};
mod icon;
mod image;
fn main() {
    let mut foliage = Foliage::new(); // library-handle
    // foliage.enable_tracing(Targets::new().with_target("foliage", tracing::Level::TRACE));
    foliage.desktop_size((1600, 900)); // window-size
    foliage.url("foliage"); // web-path
    let root = foliage.leaf((
        Grid::new(50.col().gap(2), 50.row().gap(2)),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())),
        InteractionListener::new().scroll(true),
        Stem::none(),
    ));
    let mut testers = vec![];
    let amount = 2500;
    let align = 50;
    for i in 0..amount {
        let elev = i % 99;
        println!("elev {}", elev);
        let e = foliage.leaf((
            Panel::new(),
            Outline::new(2),
            Rounding::Md,
            Stem::some(root),
            Elevation::new(elev),
            Color::gray(500),
            Location::new().xs(
                (0 + i).px().to((100 + i).px()),
                (0 + i).px().to((100 + i).px()),
            ),
        ));
        let elev = i % 27;
        let o = foliage.leaf((
            Text::new("testing..."),
            Location::new().xs(
                (0 + i).px().to((100 + i).px()),
                (0 + i).px().to((100 + i).px()),
            ),
            Stem::some(root),
            Elevation::new(elev),
        ));
        testers.push((o, e));
    }
    let seq = foliage.sequence();
    let mut locations = vec![];
    for x in 0..align {
        for y in 0..align {
            locations.push((x + 1, y + 1));
        }
    }
    for (i, (o, e)) in testers.iter().enumerate() {
        let loc = locations[i];
        let animation = Animation::new(
            Location::new().xs(loc.0.col().to(loc.0.col()), loc.1.row().to(loc.1.row())),
        )
            .start(1000)
            .finish(3000);
        foliage.animate(seq, animation.clone().targeting(*e));
        foliage.animate(seq, animation.targeting(*o));
    }
    foliage.sequence_end(seq, move |trigger: Trigger<OnEnd>, mut tree: Tree| {
        println!("finished {:?}", trigger.entity());
    });
    foliage.photosynthesize(); // run
}
