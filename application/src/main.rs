use foliage::Justify::Center;
use foliage::{AutoHeight, Color, EcsExtension, Event, Foliage, FontSize, GlyphColors, Grid, GridExt, HorizontalAlignment, Location, Opacity, Stem, Text, VerticalAlignment};

mod icon;
mod image;
fn main() {
    let mut foliage = Foliage::new(); // library-handle
    foliage.desktop_size((400, 600)); // window-size
    foliage.url("foliage"); // web-path
    let root = foliage.leaf((
        Grid::new(12.col(), 10.pct()),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())),
        Stem::none(),
    ));
    let leaf = foliage.leaf((
        Text::new("hello world!"),
        AutoHeight(false),
        HorizontalAlignment::Center,
        VerticalAlignment::Middle,
        Opacity::new(1.0),
        GlyphColors::new().add(0..5, Color::gray(50)),
        FontSize::new(14).sm(20).md(24).lg(32).xl(48),
        Stem::some(root),
        Color::gray(500),
        Location::new()
            // .xs(2.col().to(9.col()), 1.row().to(1.row()))
            .sm(2.col().to(9.col()), 1.row().to(1.row()))
            .md(3.col().to(5.col()), 2.row().span(1.row()))
            .xl(
                4.col().to(9.col()).max(300.px()).justify(Center),
                1.row().to(9.row()),
            ),
    )); // add single node
    foliage.photosynthesize(); // run
}
