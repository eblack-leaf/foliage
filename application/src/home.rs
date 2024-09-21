use foliage::branch::{Branch, Tree};
use foliage::color::{Grey, Monochromatic};
use foliage::grid::{screen, ContextUnit, GridLocation, LocationConfiguration, TokenUnit};
use foliage::layout::Layout;
use foliage::leaf::Leaf;
use foliage::shape::line::{Line, LineWeight};

#[derive(Clone)]
pub(crate) struct Home {}
impl Branch for Home {
    fn grow(self, mut tree: Tree) {
        tree.add_leaf(
            Leaf::new(|l| {}).named("screen-0").elevation(0).located(
                GridLocation::new()
                    .width(screen().width())
                    .height(screen().height())
                    .left(screen().left())
                    .top(screen().top()),
            ),
        );
        tree.add_leaf(
            Leaf::new(|l| {
                l.give(Line::new(LineWeight::new(8), Grey::plus_two()));
                l.stem_from("screen-0");
            })
            .named("stem-line")
            .located(
                GridLocation::new()
                    .point_ax(50.percent().width().from("screen-0"))
                    .point_ay("screen-0".bottom() - 16.px())
                    .point_bx(50.percent().width().from("screen-0"))
                    .point_by("screen-0".top() + 16.px())
                    .except_at(
                        Layout::LANDSCAPE_MOBILE | Layout::LANDSCAPE_EXT,
                        LocationConfiguration::new()
                            .point_ax("screen-0".left() + 16.px())
                            .point_ay(50.percent().height().from("screen-0"))
                            .point_bx("screen-0".right() - 16.px())
                            .point_by(50.percent().height().from("screen-0")),
                    ),
            )
            .elevation(2),
        );
        tree.add_leaf(
            Leaf::new(|l| {
                l.give(Line::new(LineWeight::new(12), Grey::plus_two()));
                l.stem_from("screen-0");
            })
            .named("branch-a")
            .located(
                GridLocation::new()
                    .point_ax(50.percent().width().from("screen-0"))
                    .point_ay("screen-0".bottom() - 25.percent().height().from("screen-0"))
                    .point_bx(50.percent().width().from("screen-0") + 120.px())
                    .point_by("screen-0".bottom() - 25.percent().height().from("screen-0") - 120.px()),
            )
            .elevation(2),
        );
        tree.add_leaf(
            Leaf::new(|l| {
                l.give(Line::new(LineWeight::new(12), Grey::plus_two()));
                l.stem_from("screen-0");
            })
            .named("branch-b")
            .located(
                GridLocation::new()
                    .point_ax(50.percent().width().from("screen-0"))
                    .point_ay("screen-0".top() + 25.percent().height().from("screen-0"))
                    .point_bx(50.percent().width().from("screen-0") - 80.px())
                    .point_by("screen-0".top() + 25.percent().height().from("screen-0") - 80.px()),
            )
            .elevation(2),
        );
    }
}
