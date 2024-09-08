use foliage::branch::{Branch, Tree};
use foliage::color::Color;
use foliage::grid::{GridLocation, TokenUnit};
use foliage::leaf::Leaf;
use foliage::panel::{Panel, Rounding};
use foliage::shape::line::{Line, LineWeight};

#[derive(Clone)]
pub(crate) struct Home {}
impl Branch for Home {
    fn grow(self, mut tree: Tree) {
        tree.add_leaf(
            Leaf::new(|l| {
                l.give(Line::new(LineWeight::new(150), Color::WHITE));
            })
                .named("line-test")
                .located(
                    GridLocation::new()
                        .point_ax(100.px())
                        .point_ay(100.px())
                        .point_bx(500.px())
                        .point_by(500.px()),
                )
                .elevation(5),
        );
    }
}
