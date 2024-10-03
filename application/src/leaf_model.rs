use foliage::anim::Animation;
use foliage::bevy_ecs::entity::Entity;
use foliage::color::{Grey, Monochromatic};
use foliage::coordinate::Coordinates;
use foliage::grid::aspect::stem;
use foliage::grid::location::GridLocation;
use foliage::grid::unit::TokenUnit;
use foliage::leaf::Leaf;
use foliage::opacity::Opacity;
use foliage::shape::line::{Line, LineJoin};
use foliage::tree::{EcsExtension, Tree};
use foliage::twig::{Branch, Twig};

pub(crate) struct LeafModel {
    pub(crate) lines: Vec<Entity>,
    pub(crate) joins: Vec<Entity>,
}
pub(crate) struct LeafModelArgs {}
impl LeafModel {
    pub(crate) fn args() -> LeafModelArgs {
        LeafModelArgs {}
    }
}
impl Branch for LeafModelArgs {
    type Handle = LeafModel;
    fn grow(twig: Twig<Self>, tree: &mut Tree) -> Self::Handle {
        let this = tree
            .spawn(Leaf::new().elevation(twig.elevation).stem_from(twig.stem))
            .insert(twig.location)
            .id();
        let mut joins = Vec::new();
        let mut lines = Vec::new();
        for point in MODEL_POINTS {
            joins.push(
                tree.spawn(Leaf::new().elevation(-1).stem_from(Some(this)))
                    .insert(LineJoin::new(Grey::base()))
                    .insert(Opacity::new(1.0))
                    .insert(
                        GridLocation::new()
                            .center_x(point.horizontal().percent().width().from(stem()))
                            .center_y(point.vertical().percent().height().from(stem()))
                            .width(MODEL_LINE_WEIGHT.px())
                            .height(MODEL_LINE_WEIGHT.px()),
                    )
                    .id(),
            );
        }
        tree.start_sequence(|seq| {
            for join in joins.iter() {
                let anim = Animation::new(Opacity::new(1.0))
                    .start(0)
                    .end(350)
                    .targeting(*join);
                seq.animate(anim);
            }
        });
        let mut endings = Vec::new();
        for line in LINE_INDICES {
            let a = MODEL_POINTS[line.0];
            let b = MODEL_POINTS[line.1];
            let end_location = GridLocation::new()
                .point_ax(a.horizontal().percent().width().from(stem()))
                .point_ay(a.vertical().percent().height().from(stem()))
                .point_bx(b.horizontal().percent().width().from(stem()))
                .point_by(b.vertical().percent().height().from(stem()));
            endings.push(end_location);
            lines.push(
                tree.spawn(
                    Leaf::new()
                        .elevation(-1)
                        .stem_from(Some(this))
                        .opacity(Opacity::new(1.0)),
                )
                .insert(Line::new(MODEL_LINE_WEIGHT, Grey::base()))
                .insert(
                    GridLocation::new()
                        .point_ax(a.horizontal().percent().width().from(stem()))
                        .point_ay(a.vertical().percent().height().from(stem()))
                        .point_bx(a.horizontal().percent().width().from(stem()))
                        .point_by(a.vertical().percent().height().from(stem())),
                )
                .id(),
            );
        }
        tree.start_sequence(|seq| {
            let mut delta = 400;
            let mut now = 650;
            for (i, ending) in endings.drain(..).enumerate() {
                let anim = Animation::new(ending)
                    .start(now)
                    .end(now + delta)
                    .targeting(*lines.get(i).unwrap());
                seq.animate(anim);
                now += delta / 2;
            }
        });
        LeafModel { lines, joins }
    }
}
pub(crate) const MODEL_LINE_WEIGHT: i32 = 5;
pub(crate) const MODEL_POINTS: [Coordinates; 20] = [
    Coordinates::new(50.0, 10.0),
    Coordinates::new(50.0, 90.0),
    Coordinates::new(50.0, 55.0),
    Coordinates::new(85.0, 55.0),
    Coordinates::new(65.0, 55.0),
    Coordinates::new(75.0, 40.0),
    Coordinates::new(0.0, 0.0),
    Coordinates::new(0.0, 0.0),
    Coordinates::new(0.0, 0.0),
    Coordinates::new(0.0, 0.0),
    Coordinates::new(0.0, 0.0),
    Coordinates::new(0.0, 0.0),
    Coordinates::new(0.0, 0.0),
    Coordinates::new(0.0, 0.0),
    Coordinates::new(0.0, 0.0),
    Coordinates::new(0.0, 0.0),
    Coordinates::new(0.0, 0.0),
    Coordinates::new(0.0, 0.0),
    Coordinates::new(0.0, 0.0),
    Coordinates::new(0.0, 0.0),
];
pub(crate) const LINE_INDICES: [(usize, usize); 20] = [
    (0, 1),
    (2, 3),
    (4, 5),
    (0, 1),
    (0, 1),
    (0, 1),
    (0, 1),
    (0, 1),
    (0, 1),
    (0, 1),
    (0, 1),
    (0, 1),
    (0, 1),
    (0, 1),
    (0, 1),
    (0, 1),
    (0, 1),
    (0, 1),
    (0, 1),
    (0, 1),
];
