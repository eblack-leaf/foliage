use foliage::branch::{Branch, Tree};
use foliage::color::Color;
use foliage::grid::{screen, GridLocation, TokenUnit};
use foliage::leaf::Leaf;
use foliage::panel::{Panel, Rounding};

#[derive(Clone)]
pub(crate) struct Home {}
impl Branch for Home {
    fn grow(self, mut tree: Tree) {
        tree.add_leaf(
            Leaf::new(|l| {
                l.give(Panel::new(Rounding::all(0.2), Color::WHITE));
            })
            .named("tester")
            .located(
                GridLocation::new()
                    .left(screen().left() + 25.percent_width(screen()))
                    .right(75.percent_width(screen()))
                    .top(10.percent_height(screen()))
                    .center_y(10.percent_height(screen()) + 16.px()),
            )
            .elevation(5),
        );
    }
}
