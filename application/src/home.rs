use foliage::branch::{Branch, Tree};
use foliage::color::Color;
use foliage::coordinate::Coordinates;
use foliage::grid::{GridCoordinate, GridPlacement};
use foliage::shape::{EdgePoints, Shape, ShapeDescriptor};

#[derive(Clone)]
pub(crate) struct Home {}
impl Branch for Home {
    fn grow(self, mut tree: Tree) {
        tree.add_leaf(
            "shape-test",
            GridPlacement::new(0.px().to(0.px()), 0.px().to(0.px())),
            1,
            None,
            |e| {
                e.give_attr(Shape::new(
                    ShapeDescriptor::new(
                        EdgePoints::new(Coordinates::new(10.0, 10.0), Coordinates::new(10.0, 10.0)),
                        EdgePoints::new(
                            Coordinates::new(10.0, 10.0),
                            Coordinates::new(720.0, 340.0),
                        ),
                        EdgePoints::new(
                            Coordinates::new(720.0, 340.0),
                            Coordinates::new(550.0, 550.0),
                        ),
                        EdgePoints::new(
                            Coordinates::new(10.0, 10.0),
                            Coordinates::new(550.0, 550.0),
                        ),
                    ),
                    Color::WHITE,
                ))
            },
        );
    }
}
