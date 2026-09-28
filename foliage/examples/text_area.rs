use foliage::{
    Area, Boxed, Foliage, Grove, Grow, Location, Palette, Panel, Pollen, Root, Source, TextArea,
    left, top,
};

fn main() {
    let mut foliage = Foliage::new();
    foliage.title("foliage -- text-area");
    foliage.app_id("foliage");
    foliage.desktop_size(Area::new(500f32, 500f32));
    foliage.root::<Example>();
    foliage.photosynthesize();
}
struct Example;
impl Root for Example {
    fn take_root(grove: &mut Grove) -> Self {
        grove.plant(Panel::new().color(Palette::Raised).at(Location::new().xs(
            left(8.px()).right(100.pct() - 8.px()),
            top(16.px()).bottom(100.pct() - 16.px()),
        )));
        grove.plant(TextArea::new().at(Location::new().xs(
            left(16.px()).right(100.pct() - 16.px()),
            top(24.px()).bottom(100.pct() - 24.px()),
        )));
        Self
    }

    fn frame(&mut self, _grove: &mut Grove, _pollen: Pollen) {}
}
