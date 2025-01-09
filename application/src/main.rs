use foliage::{
    Animation, Color, Elevation, Foliage, FontSize, Grid, GridExt, Icon, InteractionListener,
    Location, OnEnd, Outline, Panel, Rounding, Stem, Text, Trigger, View,
};
mod icon;
mod image;
fn main() {
    let mut foliage = Foliage::new(); // library-handle
                                      // foliage.enable_tracing(Targets::new().with_target("foliage", tracing::Level::TRACE));
    foliage.desktop_size((1600, 900)); // window-size
    foliage.url("foliage"); // web-path
    let root = foliage.leaf((
        Grid::new(25.col().gap(2), 25.row().gap(2)),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())),
        InteractionListener::new().scroll(true),
        Stem::none(),
    ));
    let mut testers = vec![];
    let amount = 2500;
    let align = 50;
    foliage.leaf(Icon::memory(0, include_bytes!("assets/icons/at-sign.icon")));
    for i in 0..amount {
        let elev = i % 9;
        let mut color = Color::gray(500);
        color.set_red(i as f32 / amount as f32);
        let e = foliage.leaf((
            Panel::new(),
            Outline::new(2),
            Rounding::Md,
            Stem::some(root),
            View::context(root),
            Elevation::new(elev),
            color,
            Location::new().xs(
                (0 + i).px().to((100 + i).px()),
                (0 + i).px().to((100 + i).px()),
            ),
        ));
        let elev = i % 2;
        let o = foliage.leaf((
            Text::new("testing..."),
            Location::new().xs(
                (0 + i).px().to((100 + i).px()),
                (0 + i).px().to((100 + i).px()),
            ),
            FontSize::new(12),
            Stem::some(root),
            View::context(root),
            color,
            Elevation::new(elev),
        ));
        let ic = foliage.leaf((
            Icon::new(0),
            Location::new().xs(
                (0 + i).px().to((100 + i).px()),
                (0 + i).px().to((100 + i).px()),
            ),
            Stem::some(root),
            color,
            Elevation::new(i % 6),
        ));
        testers.push((o, e, ic));
    }
    let seq = foliage.sequence();
    let mut locations = vec![];
    for x in 0..align {
        for y in 0..align {
            locations.push((x + 1, y + 1));
        }
    }
    for (i, (o, e, ic)) in testers.iter().enumerate() {
        let loc = locations[i];
        let animation = Animation::new(
            Location::new().xs(loc.0.col().to(loc.0.col()), loc.1.row().to(loc.1.row())),
        )
        .start(1000)
        .finish(3000);
        foliage.animate(seq, animation.clone().targeting(*e));
        foliage.animate(seq, animation.clone().targeting(*o));
        foliage.animate(seq, animation.targeting(*ic));
    }
    foliage.sequence_end(seq, move |trigger: Trigger<OnEnd>| {
        println!("finished {:?}", trigger.entity());
    });
    foliage.photosynthesize(); // run
}
