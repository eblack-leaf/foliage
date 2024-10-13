use foliage::anim::Animation;
use foliage::bevy_ecs::entity::Entity;
use foliage::bevy_ecs::prelude::{Component, IntoSystemSetConfigs};
use foliage::bevy_ecs::query::{Added, Changed, Or};
use foliage::bevy_ecs::schedule::IntoSystemConfigs;
use foliage::bevy_ecs::system::Query;
use foliage::color::{Grey, Monochromatic, Orange};
use foliage::coordinate::section::Section;
use foliage::coordinate::{Coordinates, LogicalContext};
use foliage::elm::{Elm, ExternalStage};
use foliage::grid::aspect::stem;
use foliage::grid::unit::TokenUnit;
use foliage::panel::{Panel, Rounding};
use foliage::shape::line::Line;
use foliage::tree::{EcsExtension, Tree};
use foliage::{bevy_ecs, schedule_stage, Branch, Root};
use std::collections::HashMap;

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
pub(crate) const REGION_AREA: Coordinates = Coordinates::new(20.0, 20.0);
pub(crate) fn configure_leaf_part(
    mut parts: Query<
        (Entity, &mut LeafPartComponent, &Section<LogicalContext>),
        Or<(Changed<Section<LogicalContext>>, Added<LeafPartComponent>)>,
    >,
    mut tree: Tree,
) {
    for (entity, mut part, section) in parts.iter_mut() {
        // divide section into whole 11x11
        let section = Section::new(section.position, (section.area - (8, 8).into()).max((0, 0)));
        let num_regions = (section.area / REGION_AREA.into()).floored().coordinates;
        let mut changed = true;
        for x in 0..num_regions.horizontal() as i32 {
            let x_identifier = LineIdentifier::X(x);
            if !part.lines_present.contains_key(&x_identifier) {
                // spawn
                let e = tree.add_leaf();
                tree.stem(e, Some(entity));
                tree.elevation(e, 1);
                tree.entity(e)
                    .insert(Line::new(BRANCH_GRID_WEIGHT, Grey::plus_three()));
                part.lines_present.insert(x_identifier, e);
            }
            tree.location(
                part.lines_present.get(&x_identifier).copied().unwrap(),
                GridLocation::new()
                    .point_ax(stem().left() + (x * REGION_AREA.horizontal() as i32).px())
                    .point_ay(stem().top() + 0.px())
                    .point_bx(stem().left() + (x * REGION_AREA.horizontal() as i32).px())
                    .point_by(
                        stem().top()
                            + (REGION_AREA.horizontal() as i32 * num_regions.vertical() as i32)
                                .px(),
                    ),
            );
            for y in 0..num_regions.vertical() as i32 {
                let y_identifier = LineIdentifier::Y(y);
                if !part.lines_present.contains_key(&y_identifier) {
                    // spawn
                    let e = tree.add_leaf();
                    tree.stem(e, Some(entity));
                    tree.elevation(e, 1);
                    tree.entity(e)
                        .insert(Line::new(BRANCH_GRID_WEIGHT, Grey::plus_three()));
                    part.lines_present.insert(y_identifier, e);
                }
                tree.location(
                    part.lines_present.get(&y_identifier).copied().unwrap(),
                    GridLocation::new()
                        .point_ax(stem().left() + 0.px())
                        .point_ay(stem().top() + (y * REGION_AREA.horizontal() as i32).px())
                        .point_bx(
                            stem().left()
                                + (REGION_AREA.horizontal() as i32
                                    * num_regions.horizontal() as i32)
                                    .px(),
                        )
                        .point_by(stem().top() + (y * REGION_AREA.horizontal() as i32).px()),
                );
                let box_identifier = BoxIdentifier { x, y };
                if !part.boxes_present.contains_key(&box_identifier) {
                    // spawn
                    let e = tree.add_leaf();
                    tree.stem(e, Some(entity));
                    tree.elevation(e, 1);
                    tree.entity(e)
                        .insert(Panel::new(Rounding::all(0.0), Orange::minus_one()));
                    part.boxes_present.insert(box_identifier, e);
                }
                tree.location(
                    part.boxes_present.get(&box_identifier).copied().unwrap(),
                    GridLocation::new()
                        .center_x(
                            stem().left()
                                + (REGION_AREA.horizontal() as i32 * x
                                    + (REGION_AREA.horizontal() / 2f32) as i32)
                                    .px(),
                        )
                        .center_y(
                            stem().top()
                                + (REGION_AREA.horizontal() as i32 * y
                                    + (REGION_AREA.horizontal() / 2f32) as i32)
                                    .px(),
                        )
                        .width(24.px())
                        .height(24.px()),
                );
            }
        }
        if changed {
            tree.flush(entity);
        }
        // if no/less sub-entities => spawn how many need + draw-sequence + box-fade-in
        // if more sub-entities => remove excess => queue_remove
    }
}
pub(crate) fn configure_leaf_part_colors(mut parts: Query<&mut LeafPartComponent>, mut tree: Tree) {
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
    pub(crate) lines_present: HashMap<LineIdentifier, Entity>,
    pub(crate) boxes_present: HashMap<BoxIdentifier, Entity>,
}
#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
pub(crate) struct BoxIdentifier {
    pub(crate) x: i32,
    pub(crate) y: i32,
}
#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
pub(crate) enum LineIdentifier {
    X(i32),
    Y(i32),
}
impl LeafPartComponent {
    pub(crate) fn new() -> Self {
        Self {
            lines_present: Default::default(),
            boxes_present: Default::default(),
        }
    }
}
pub(crate) struct LeafPartModel {}
impl Branch for LeafPartModel {
    type Handle = Entity;
    fn grow(self, tree: &mut Tree) -> Self::Handle {
        let root = tree.add_leaf();
        tree.stem(root, twig.stem);
        tree.location(root, twig.location);
        tree.elevation(root, -1);
        tree.entity(root).insert(LeafPartComponent::new());
        tree.flush(root);
        // start part-stem-line sequence
        // spawn leaf-part-components on root with correct values
        root
    }
}
impl Branch for LeafModelArgs {
    type Handle = LeafModel;
    fn grow(self, tree: &mut Tree) -> Self::Handle {
        let this = tree.add_leaf();
        tree.stem(this, twig.stem);
        tree.location(this, twig.location);
        tree.elevation(this, twig.elevation);
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
        let stem_line = tree.add_leaf();
        tree.stem(stem_line, Some(this));
        tree.elevation(stem_line, -1);
        tree.location(
            stem_line,
            GridLocation::new()
                .point_ax(stem().center_x())
                .point_ay(5.percent().from(stem()))
                .point_bx(stem().center_x())
                .point_by(5.percent().from(stem())),
        );
        tree.entity(stem_line)
            .insert(Line::new(STEM_LINE_WEIGHT, Grey::plus_three()));
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
        tree.flush(this);
        LeafModel {
            this,
            parts: [one, two, three],
            stem_line,
        }
    }
}
