use foliage::{
    auto, stack, AutoHeight, Color, EcsExtension, Event, Foliage, FontSize, Grid, GridExt,
    Location, Stack, Stem, Text,
};

mod icon;
mod image;
fn main() {
    let mut foliage = Foliage::new(); // library-handle
    foliage.desktop_size((400, 600)); // window-size
    foliage.url("foliage"); // web-path
    let root = foliage.leaf((
        Grid::new(12.col(), 24.px()),
        Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())),
        Stem::none(),
    ));
    let a = foliage.leaf((
        Text::new("asonetuhsanot aoentuh aes tanoes uaneotusaoent uetuh asnetuh usanoetua"),
        FontSize::new(20),
        AutoHeight(true),
        Stem::some(root),
        Location::new().xs(1.col().to(4.col()), 1.row().to(auto())),
    ));
    let b = foliage.leaf((
        Text::new("xxxx xxx x xx x xxxx x x x xx xxxx x xx  xxx x xx xxxx xx x x x x xxx x x"),
        FontSize::new(20),
        AutoHeight(true),
        Color::gray(500),
        Stem::some(root),
        Stack::new(a),
        Location::new().xs(1.col().to(4.col()), stack().to(auto())),
    ));
    let c = foliage.leaf((
        Text::new("yyyy yyy y yy y yyyy y y y yy yyyy y yy  yyy y yy yyyy yy y y y y yyy y y"),
        FontSize::new(20),
        AutoHeight(true),
        Color::gray(500),
        Stem::some(root),
        Stack::new(a),
        Location::new().xs(5.col().to(9.col()), stack().to(auto())),
    ));
    foliage.photosynthesize(); // run
}
