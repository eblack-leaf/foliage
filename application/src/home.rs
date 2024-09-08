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
                l.give(Line::new(LineWeight::new(60), Color::WHITE));
            })
                .named("line-test")
                .located(
                    GridLocation::new()
                        .point_ax(100.px())
                        .point_ay(10.px())
                        .point_bx(100.px())
                        .point_by(100.px()),
                )
                .elevation(5),
        );
        tree.add_leaf(
            Leaf::new(|l| {
                l.give(Line::new(LineWeight::new(60), Color::WHITE));
            })
                .named("line-test-2")
                .located(
                    GridLocation::new()
                        .point_ax(100.px())
                        .point_ay(100.px())
                        .point_bx(500.px())
                        .point_by(50.px()),
                )
                .elevation(5),
        );
        tree.add_leaf(
            Leaf::new(|l| {
                l.give(Panel::new(Rounding::all(1.0), Color::WHITE));
            })
                .named("line-test-3")
                .located(
                    GridLocation::new()
                        .center_x(100.px())
                        .center_y(100.px())
                        .width(58.px())
                        .height(58.px()),
                )
                .elevation(4),
        );
        tree.add_leaf(
            Leaf::new(|l| {
                l.give(Line::new(LineWeight::new(60), Color::WHITE));
            })
                .named("line-test-4")
                .located(
                    GridLocation::new()
                        .point_ax(500.px())
                        .point_ay(50.px())
                        .point_bx(500.px())
                        .point_by(350.px()),
                )
                .elevation(5),
        );
        tree.add_leaf(
            Leaf::new(|l| {
                l.give(Panel::new(Rounding::all(1.0), Color::WHITE));
            })
                .named("line-test-5")
                .located(
                    GridLocation::new()
                        .center_x(500.px())
                        .center_y(50.px())
                        .width(58.px())
                        .height(58.px()),
                )
                .elevation(4),
        );
    }
}
