use std::collections::HashSet;
use foliage::anim::Animation;
use foliage::bevy_ecs::entity::Entity;
use foliage::bevy_ecs::prelude::{Component, IntoSystemSetConfigs};
use foliage::bevy_ecs::query::{Added, Changed, Or};
use foliage::bevy_ecs::schedule::IntoSystemConfigs;
use foliage::bevy_ecs::system::Query;
use foliage::color::{Grey, Monochromatic};
use foliage::coordinate::area::Area;
use foliage::coordinate::position::Position;
use foliage::coordinate::LogicalContext;
use foliage::elm::{Elm, ExternalStage};
use foliage::grid::aspect::stem;
use foliage::grid::location::GridLocation;
use foliage::grid::unit::TokenUnit;
use foliage::shape::line::Line;
use foliage::tree::{EcsExtension, Tree};
use foliage::twig::{Branch, Twig};
use foliage::{bevy_ecs, schedule_stage, Root};
use foliage::coordinate::section::Section;

pub(crate) struct LeafModel {
    pub(crate) this: Entity,
    pub(crate) parts: [Entity; 3],
    pub(crate) stem_line: Entity,
}
#[schedule_stage]
pub(crate) enum LeafModelStage {
    First,
}
impl Root for LeafModel {
    fn attach(elm: &mut Elm) {
        elm.scheduler
            .main
            .configure_sets(LeafModelStage::First.in_set(ExternalStage::Configure));
        elm.scheduler.main.add_systems(
            (configure_leaf_part, configure_leaf_part_colors)
                .chain()
                .in_set(LeafModelStage::First),
        );
    }
}
pub(crate) fn configure_leaf_part(
    mut parts: Query<
        (
            &mut LeafPartComponent,
            &Position<LogicalContext>,
            &Area<LogicalContext>,
        ),
        Or<(
            Changed<Position<LogicalContext>>,
            Changed<Area<LogicalContext>>,
            Added<LeafPartComponent>,
        )>,
    >,
    mut tree: Tree,
) {
    for (part, pos, area) in parts.iter_mut() {
        // divide section into whole 11x11
        let section = Section::new(*pos, (*area - (8, 8).into()).max((0, 0)));
        let num_regions = (section.area / (11, 11).into()).floored().coordinates;
        for x in 0..num_regions.horizontal() as i32 {
            let x_identifier = LineIdentifier::X(x);
            if !part.lines_present.contains(&x_identifier) {
                // spawn
            }
            for y in 0..num_regions.vertical() as i32 {
                let y_identifier = LineIdentifier::Y(y);
                if !part.lines_present.contains(&y_identifier) {
                    // spawn
                }
                let box_identifier = BoxIdentifier {
                    x,
                    y,
                };
                if !part.boxes_present.contains(&box_identifier) {
                    // spawn
                }
            }
        }
        // if no/less sub-entities => spawn how many need + draw-sequence + box-fade-in
        // if more sub-entities => remove excess => queue_remove
    }
}
pub(crate) fn configure_leaf_part_colors(
    mut parts: Query<(&mut LeafPartComponent)>,
    mut tree: Tree,
) {
    // update color sequences based on currently-sequenced
    // or random-ish selection of new sequences
}
pub(crate) const STEM_LINE_WEIGHT: i32 = 10;
pub(crate) const BRANCH_LINE_WEIGHT: i32 = 5;
pub(crate) const BRANCH_GRID_WEIGHT: i32 = 1;
pub(crate) struct LeafModelArgs {}
impl LeafModel {
    pub(crate) fn args() -> LeafModelArgs {
        LeafModelArgs {}
    }
}
#[derive(Component)]
pub(crate) struct LeafPartComponent {
    pub(crate) lines: Vec<Entity>,
    pub(crate) boxes: Vec<Entity>,
    pub(crate) lines_present: HashSet<LineIdentifier>,
    pub(crate) boxes_present: HashSet<BoxIdentifier>
}
#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
pub(crate) struct BoxIdentifier {
    pub(crate) x: i32,
    pub(crate) y: i32,
}
#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
pub(crate) enum LineIdentifier {
    X(i32),
    Y(i32)
}
impl LeafPartComponent {
    pub(crate) fn new() -> Self {
        Self {
            lines: vec![],
            boxes: vec![],
            lines_present: Default::default(),
            boxes_present: Default::default(),
        }
    }
}
pub(crate) struct LeafPartModel {}
impl Branch for LeafPartModel {
    type Handle = Entity;
    fn grow(twig: Twig<Self>, tree: &mut Tree) -> Self::Handle {
        let root = tree.spawn_empty().id();
        // start part-stem-line sequence
        // spawn leaf-part-components on root with correct values
        root
    }
}
impl Branch for LeafModelArgs {
    type Handle = LeafModel;
    fn grow(twig: Twig<Self>, tree: &mut Tree) -> Self::Handle {
        let this = tree.add_leaf(|l| {
            l.elevation(twig.elevation);
            l.stem_from(twig.stem);
            l.location(twig.location);
        });
        let one = tree.branch(
            Twig::new(LeafPartModel {})
                .elevation(-1)
                .stem_from(this)
                .location(
                    GridLocation::new()
                        .left(50.percent().width().from(stem()))
                        .top(5.percent().height().from(stem()))
                        .right(95.percent().width().from(stem()))
                        .bottom(30.percent().height().from(stem())),
                ),
        );
        let two = tree.branch(
            Twig::new(LeafPartModel {})
                .elevation(-1)
                .stem_from(this)
                .location(
                    GridLocation::new()
                        .left(5.percent().width().from(stem()))
                        .top(35.percent().height().from(stem()))
                        .right(50.percent().width().from(stem()))
                        .bottom(65.percent().height().from(stem())),
                ),
        );
        let three = tree.branch(
            Twig::new(LeafPartModel {})
                .elevation(-1)
                .stem_from(this)
                .location(
                    GridLocation::new()
                        .left(50.percent().width().from(stem()))
                        .top(75.percent().height().from(stem()))
                        .right(90.percent().width().from(stem()))
                        .bottom(95.percent().height().from(stem())),
                ),
        );
        let stem_line = tree.add_leaf(|l| {
            l.elevation(-1);
            l.stem_from(Some(this));
            l.location(
                GridLocation::new()
                    .point_ax(stem().center_x())
                    .point_ay(5.percent().from(stem()))
                    .point_bx(stem().center_x())
                    .point_by(5.percent().from(stem())),
            );
            l.give(Line::new(STEM_LINE_WEIGHT, Grey::plus_three()));
        });
        tree.start_sequence(|seq| {
            seq.animate(
                Animation::new(
                    GridLocation::new()
                        .point_ax(stem().center_x())
                        .point_ay(5.percent().from(stem()))
                        .point_bx(stem().center_x())
                        .point_by(95.percent().from(stem())),
                )
                .targeting(stem_line)
                .start(100)
                .end(1000),
            );
        });
        LeafModel {
            this,
            parts: [one, two, three],
            stem_line,
        }
    }
}
