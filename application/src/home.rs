use foliage::branch::{Branch, Tree};
use foliage::color::Grey;
use foliage::color::{Blue, Monochromatic, Orange};
use foliage::grid::{GridLocation, TokenUnit};
use foliage::leaf::Leaf;
use foliage::panel::{Panel, Rounding};
use foliage::shape::line::{Line, LineWeight};

#[derive(Clone)]
pub(crate) struct Home {}
impl Branch for Home {
    fn grow(self, mut tree: Tree) {
        let mut last = None;
        for x in (0..1600).step_by(50) {
            let y = ((x as f32 / 225.0).sin() * 160.0) as i32 + 220;
            if last.is_none() {
                last = Some((x, y));
                continue;
            }
            tree.add_leaf(
                Leaf::new(|l| {
                    l.give(Line::new(LineWeight::new(4), Blue::base()));
                })
                .named(format!("line-{}", x))
                .located(
                    GridLocation::new()
                        .point_ax(last.unwrap().0.px())
                        .point_ay(last.unwrap().1.px())
                        .point_bx(x.px())
                        .point_by(y.px()),
                )
                .elevation(6),
            );
            tree.add_leaf(
                Leaf::new(|l| {
                    l.give(Panel::new(Rounding::all(1.0), Blue::base()));
                })
                .named(format!("panel-{}", x))
                .located(
                    GridLocation::new()
                        .center_x(x.px())
                        .center_y(y.px())
                        .width(3.px())
                        .height(3.px()),
                )
                .elevation(5),
            );
            last.replace((x, y));
        }
        last = None;
        for x in (0..1600).step_by(50) {
            let y = ((x as f32 / 225.0).cos() * 160.0) as i32 + 420;
            if last.is_none() {
                last = Some((x, y));
                continue;
            }
            tree.add_leaf(
                Leaf::new(|l| {
                    l.give(Line::new(LineWeight::new(4), Orange::base()));
                })
                .named(format!("line-cos-{}", x))
                .located(
                    GridLocation::new()
                        .point_ax(last.unwrap().0.px())
                        .point_ay(last.unwrap().1.px())
                        .point_bx(x.px())
                        .point_by(y.px()),
                )
                .elevation(4),
            );
            tree.add_leaf(
                Leaf::new(|l| {
                    l.give(Panel::new(Rounding::all(1.0), Orange::base()));
                })
                .named(format!("panel-cos-{}", x))
                .located(
                    GridLocation::new()
                        .center_x(x.px())
                        .center_y(y.px())
                        .width(3.px())
                        .height(3.px()),
                )
                .elevation(3),
            );
            last.replace((x, y));
        }
        last = None;
        for x in (0..1600).step_by(10) {
            let y = ((x as f32 / 225.0).tan() * 20.0) as i32 + 620;
            if last.is_none() {
                last = Some((x, y));
                continue;
            }
            tree.add_leaf(
                Leaf::new(|l| {
                    l.give(Line::new(LineWeight::new(4), Grey::base()));
                })
                .named(format!("line-tan-{}", x))
                .located(
                    GridLocation::new()
                        .point_ax(last.unwrap().0.px())
                        .point_ay(last.unwrap().1.px())
                        .point_bx(x.px())
                        .point_by(y.px()),
                )
                .elevation(2),
            );
            tree.add_leaf(
                Leaf::new(|l| {
                    l.give(Panel::new(Rounding::all(1.0), Grey::base()));
                })
                .named(format!("panel-tan-{}", x))
                .located(
                    GridLocation::new()
                        .center_x(x.px())
                        .center_y(y.px())
                        .width(3.px())
                        .height(3.px()),
                )
                .elevation(1),
            );
            last.replace((x, y));
        }
    }
}
