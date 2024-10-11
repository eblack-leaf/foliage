use crate::coordinate::area::Area;
use crate::coordinate::points::Points;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::LogicalContext;
use crate::ginkgo::viewport::ViewportHandle;
use crate::grid::animation::GridLocationAnimationHook;
use crate::grid::location::GridLocation;
use crate::grid::Grid;
use crate::layout::{Layout, LayoutGrid};
use crate::leaf::{Dependents, Stem};
use crate::tree::Tree;
use bevy_ecs::change_detection::{DetectChanges, Res, ResMut};
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::{ParamSet, Query, Trigger};
use bevy_ecs::query::With;
use ordermap::OrderSet;
use std::collections::HashMap;

pub(crate) struct ReferentialOrderDeterminant {
    chain: OrderSet<Entity>,
}
#[derive(Default, Event, Copy, Clone)]
pub struct ResolveGridLocation {}
pub(crate) fn triggered_resolve_grid_locations(
    trigger: Trigger<ResolveGridLocation>,
    mut locations: Query<&mut GridLocation>,
    dependents: Query<&Dependents>,
    grids: Query<&Grid>,
    stems: Query<&Stem>,
    layout: Res<Layout>,
    layout_grid: Res<LayoutGrid>,
    viewport: Res<ViewportHandle>,
    mut sections: Query<&mut Section<LogicalContext>>,
    mut points: Query<&mut Points<LogicalContext>>,
    mut tree: Tree,
) {
    // get stem / screen
    // location of trigger.ent -> resolve(stem, screen, layout)
    // apply to section / points / hook_update
    // trigger-target ResolveGridLocation{} => deps.0.iter().copied().collect()
}
pub(crate) fn resolve_grid_locations(
    mut check_read_and_update: ParamSet<(
        Query<Entity, (With<ResolveGridLocation>, With<GridLocation>)>,
        Query<(
            Entity,
            &Stem,
            &Dependents,
            &GridLocation,
            &Grid,
            &Position<LogicalContext>,
            &Area<LogicalContext>,
            &Points<LogicalContext>,
        )>,
        Query<(
            &mut Position<LogicalContext>,
            &mut Area<LogicalContext>,
            &mut Points<LogicalContext>,
            &mut GridLocation,
        )>,
    )>,
    viewport_handle: ResMut<ViewportHandle>,
    layout_grid: Res<LayoutGrid>,
    layout: Res<Layout>,
    mut tree: Tree,
) {
    if check_read_and_update.p0().is_empty() && !layout_grid.is_changed() {
        return;
    }
    let mut check = vec![];
    for e in check_read_and_update.p0().iter() {
        check.push(e);
        tree.entity(e).remove::<ResolveGridLocation>();
    }
    let mut ref_context = ReferentialContext::new(viewport_handle.section(), layout_grid.grid);
    let read = check_read_and_update.p1();
    if layout_grid.is_changed() {
        for (e, _, _, _, _, _, _, _) in read.iter() {
            check.push(e);
        }
    }
    for e in check {
        ref_context.resolve_leaf(e, &read, *layout);
    }
    let updates = ref_context.updates();
    drop(ref_context);
    drop(read);
    for (e, resolved) in updates {
        *check_read_and_update.p2().get_mut(e).unwrap().0 = resolved.section.position;
        *check_read_and_update.p2().get_mut(e).unwrap().1 = resolved.section.area;
        if let Some(p) = resolved.points {
            *check_read_and_update.p2().get_mut(e).unwrap().2 = p;
        }
        if let Some(hook) = resolved.hook_update {
            match &mut check_read_and_update
                .p2()
                .get_mut(e)
                .unwrap()
                .3
                .animation_hook
            {
                GridLocationAnimationHook::SectionDriven(s) => {
                    s.diff = hook[0].unwrap();
                }
                GridLocationAnimationHook::PointDriven(p) => {
                    if let Some(h) = hook[0] {
                        p.point_a.diff = h;
                    }
                    if let Some(h) = hook[1] {
                        p.point_b.diff = h;
                    }
                    if let Some(h) = hook[2] {
                        p.point_c.diff = h;
                    }
                    if let Some(h) = hook[3] {
                        p.point_d.diff = h;
                    }
                }
            }
        }
    }
}

pub(crate) struct ReferentialContext {
    solved: HashMap<Entity, ReferentialData>,
    screen: ReferentialData,
    order_chains: Vec<ReferentialOrderDeterminant>,
}

impl ReferentialContext {
    pub(crate) fn new(screen_section: Section<LogicalContext>, layout_grid: Grid) -> Self {
        Self {
            solved: HashMap::default(),
            order_chains: vec![],
            screen: ReferentialData::new(
                ResolvedLocation::new().section(screen_section),
                layout_grid,
            ),
        }
    }
    pub(crate) fn resolve_leaf(
        &mut self,
        entity: Entity,
        read: &Query<(
            Entity,
            &Stem,
            &Dependents,
            &GridLocation,
            &Grid,
            &Position<LogicalContext>,
            &Area<LogicalContext>,
            &Points<LogicalContext>,
        )>,
        layout: Layout,
    ) {
        if read.get(entity).is_err() {
            return;
        }
        for d in self.order_chains.iter() {
            if d.chain.contains(&entity) {
                return;
            }
        }
        let chain = self.recursive_chain(entity, read, layout);
        self.order_chains
            .push(ReferentialOrderDeterminant { chain });
    }
    fn recursive_chain(
        &mut self,
        entity: Entity,
        read: &Query<(
            Entity,
            &Stem,
            &Dependents,
            &GridLocation,
            &Grid,
            &Position<LogicalContext>,
            &Area<LogicalContext>,
            &Points<LogicalContext>,
        )>,
        layout: Layout,
    ) -> OrderSet<Entity> {
        let mut set = OrderSet::new();
        set.insert(entity);
        let current = read.get(entity).unwrap();
        let stem = {
            current.1 .0.clone().and_then(|s| {
                if let Some(solve) = self.solved.get(&s) {
                    Some(*solve)
                } else {
                    let stem = read.get(s).unwrap();
                    let mut resolved =
                        ResolvedLocation::new().section(Section::new(*stem.5, *stem.6));
                    resolved.points.replace(stem.7.clone());
                    Some(ReferentialData::new(resolved, *stem.4))
                }
            })
        };
        if let Some(res) = current.3.resolve(stem, self.screen, layout) {
            self.solved
                .insert(current.0, ReferentialData::new(res, *current.4));
        }
        for dep in current.2 .0.iter() {
            if read.get(*dep).is_err() {
                continue;
            }
            let dep_set = self.recursive_chain(*dep, read, layout);
            set.extend(dep_set);
        }
        set
    }
    pub(crate) fn updates(&mut self) -> Vec<(Entity, ResolvedLocation)> {
        self.solved.drain().map(|(k, v)| (k, v.resolved)).collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvedLocation {
    pub(crate) section: Section<LogicalContext>,
    pub(crate) points: Option<Points<LogicalContext>>,
    pub(crate) hook_update: Option<[Option<Section<LogicalContext>>; 4]>,
}

impl ResolvedLocation {
    pub(crate) fn new() -> Self {
        Self {
            section: Section::default(),
            points: None,
            hook_update: None,
        }
    }
    pub(crate) fn section(mut self, section: Section<LogicalContext>) -> Self {
        self.section = section;
        self
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ReferentialData {
    pub(crate) resolved: ResolvedLocation,
    pub(crate) grid: Grid,
}

impl ReferentialData {
    pub(crate) fn new(resolved: ResolvedLocation, grid: Grid) -> Self {
        Self { resolved, grid }
    }
}
