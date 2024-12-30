mod layout;
mod location;
mod aspect_ratio;

use crate::{CoordinateUnit, Coordinates};
pub use aspect_ratio::AspectRatio;
pub use layout::Layout;
pub use location::Location;
#[test]
fn behavior() {
    use crate::FontSize;
    let grid = Grid::new(12.col().gap(4), 8.px().gap(4))
        .md(12.col().gap(4), 8.px().gap(4))
        .lg(12.col().gap(8), 16.px().gap(8))
        .xl(12.col().gap(12), 24.px().gap(12)); // canon
    let root = Location::new().sm(0.pct().to(100.pct()), 0.pct().to(100.pct()));
    // let view = View::context(root); // scrolling
    let location = Location::new().sm(50.px().y(100.px()), 50.px().y(150.px())); // points
    let location = Location::new().sm(1.col().to(12.col()), 1.row().to(19.row()));
    let location = Location::new()
        .sm(
            2.col().to(11.col()).max(400.px()).justify(Center).pad(4),
            4.row().to(10.row()).pad((4, 8)), // debug-assert max only on width
        )
        .md(3.col().to(10.col()).max(500.px()).justify(Center), 4.to(10));
    let aspect_ratio = AspectRatio::new().sm(16 / 9);
    let font_size = FontSize::default().md(20).lg(24).xl(32);
    let location = Location::new()
        .sm(
            3.col().to(10.col()).max(300.px()).justify(Left),
            6.row().to(9.row()),
        )
        .md(
            4.col().to(9.col()).max(400.px()).justify(Left),
            6.row().to(9.row()),
        );
    let location = Location::new().sm(1.col().to(1.col()), 2.to(auto()));
    let location = Location::new().sm(1.col().to(1.col()), stack().to(auto())); // stack uses stem().bottom() as this.top()
    let location = Location::new().sm(1.col().to(1.col()), stack().to(25.row())); // explicit back to grid (acceptable content-range) or keep stacking
    let location = Location::new().sm(1.col().to(1.col()), stack().span(5.row()));
    // span cause unknown end (to)
    // span(5.row()) => 5 row lengths from current px() in stack (not necessarily aligned)
    // but spacing relative is guaranteed 5 rows for content
}
pub struct Grid {
    pub sm: GridConfiguration,
    pub md: Option<GridConfiguration>,
    pub lg: Option<GridConfiguration>,
    pub xl: Option<GridConfiguration>,
}
pub struct GridConfiguration {
    pub columns: GridAxisDescriptor,
}
pub struct GridAxisDescriptor {
    pub unit: GridAxisUnit,
    pub gap: Coordinates,
}
pub enum GridAxisUnit {
    Infinite(ScalarUnit),
    Explicit(AlignedUnit),
}
pub enum ScalarUnit {
    Px(CoordinateUnit),
    Pct(f32),
}
pub enum AlignedUnit {
    Columns(u32),
    Rows(u32),
}
