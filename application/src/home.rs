use foliage::branch::{Branch, Tree};
use foliage::color::Color;
use foliage::grid::{GridLocation, TokenUnit};
use foliage::leaf::Leaf;
use foliage::panel::{Panel, Rounding};

#[derive(Clone)]
pub(crate) struct Home {}
impl Branch for Home {
    fn grow(self, mut tree: Tree) {
        for x in (0..300).step_by(10) {
            for y in (0..100).step_by(10) {
                tree.add_leaf(
                    Leaf::new(|l| {
                        l.give(Panel::new(Rounding::all(0.0), Color::WHITE));
                    })
                    .named(format!("leaf {}+{}", x, y))
                    .located(
                        GridLocation::new()
                            .left(x.px())
                            .width(5.px())
                            .top(y.px())
                            .height(5.px()),
                    )
                    .elevation(5),
                );
            }
        }
    }
}
