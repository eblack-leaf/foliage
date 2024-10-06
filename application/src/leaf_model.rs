use foliage::anim::Animation;
use foliage::bevy_ecs::entity::Entity;
use foliage::bevy_ecs::prelude::{Component, IntoSystemSetConfigs};
use foliage::bevy_ecs::query::{Changed, Or};
use foliage::bevy_ecs::system::Query;
use foliage::color::{Grey, Monochromatic};
use foliage::coordinate::area::Area;
use foliage::coordinate::position::Position;
use foliage::coordinate::{Coordinates, LogicalContext};
use foliage::elm::{Elm, ExternalStage};
use foliage::grid::aspect::stem;
use foliage::grid::location::GridLocation;
use foliage::grid::unit::TokenUnit;
use foliage::leaf::Leaf;
use foliage::opacity::Opacity;
use foliage::shape::line::{Line, LineJoin};
use foliage::tree::{EcsExtension, Tree};
use foliage::twig::{Branch, Twig};
use foliage::{bevy_ecs, schedule_stage, Root};

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
        elm.scheduler
            .main
            .add_systems(configure_leaf_part.in_set(LeafModelStage::First));
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
        )>,
    >,
    mut tree: Tree,
) {
    for (part, pos, area) in parts.iter_mut() {
        // divide section into whole 8x8
        // if no/less sub-entities => spawn how many need
        // if more sub-entities => remove excess
        //
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
pub(crate) struct LeafPartComponent {}
impl LeafPartComponent {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
pub(crate) struct LeafPartModel {}
impl Branch for LeafPartModel {
    type Handle = Entity;
    fn grow(twig: Twig<Self>, tree: &mut Tree) -> Self::Handle {
        let root = tree.spawn_empty().id();
        // start stem line sequence
        // on-end => spawn leaf-parts with correct values
        root
    }
}
impl Branch for LeafModelArgs {
    type Handle = LeafModel;
    fn grow(twig: Twig<Self>, tree: &mut Tree) -> Self::Handle {
        let this = tree
            .spawn(Leaf::new().elevation(twig.elevation).stem_from(twig.stem))
            .insert(twig.location)
            .id();
        let one = tree.branch(
            Twig::new(LeafPartModel {})
                .elevation(-1)
                .stem_from(this)
                .located(
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
                .located(
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
                .located(
                    GridLocation::new()
                        .left(50.percent().width().from(stem()))
                        .top(75.percent().height().from(stem()))
                        .right(90.percent().width().from(stem()))
                        .bottom(95.percent().height().from(stem())),
                ),
        );
        let stem_line = tree
            .spawn(Leaf::new().elevation(-1).stem_from(Some(this)))
            .insert(Line::new(STEM_LINE_WEIGHT, Grey::plus_three()))
            .insert(
                GridLocation::new()
                    .point_ax(stem().center_x())
                    .point_ay(5.percent().from(stem()))
                    .point_bx(stem().center_x())
                    .point_by(5.percent().from(stem())),
            )
            .id();
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
