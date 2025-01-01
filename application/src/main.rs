use foliage::Justify::Right;
use foliage::{Color, EcsExtension, Event, Foliage, FontSize, Grid, GridExt, Location, Stem, Text};

mod icon;
mod image;
fn main() {
    let mut foliage = Foliage::new(); // library-handle
    foliage.desktop_size((400, 600)); // window-size
    foliage.url("foliage"); // web-path
    let root = foliage.leaf((
        Grid::new(12.col(), 6.row()),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())),
        Stem::none(),
    ));
    let leaf = foliage.leaf((
        Text::new("hello world!"),
        FontSize::new(14).sm(20).md(24).lg(32).xl(48),
        Stem::some(root),
        Color::gray(500),
        Location::new()
            .sm(2.col().to(9.col()), 1.row().to(1.row()))
            .md(3.col().to(5.col()), 2.row().span(1.row()))
            .xl(
                4.col().to(9.col()).max(100.px()).justify(Right),
                1.row().to(9.row()),
            ),
    )); // add single node
    let button = foliage.leaf((
        // Button::new(),
        // ForegroundColor::RED,
        // BackgroundColor::BLUE,
        // ButtonText::new("example"),
        // ButtonIcon::new(IconHandle::Git),
        Stem::some(leaf),
    ));
    foliage.photosynthesize(); // run
}
