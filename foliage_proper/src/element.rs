use std::collections::{HashMap, HashSet};

use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::Res;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, Component, DetectChanges, ParamSet, Query, Resource};
use bevy_ecs::query::{Or, With};

use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::placement::Placement;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::LogicalContext;
use crate::differential::{Remove, Visibility};
use crate::grid::{Grid, GridPlacement, Layout, LayoutGrid};

#[derive(Bundle, Default)]
pub(crate) struct Element {
    root: Root,
    dependents: Dependents,
    placement: Placement<LogicalContext>,
    remove: Remove,
    visibility: Visibility,
}
#[derive(Default, Component)]
pub(crate) struct Root(pub(crate) Option<TargetHandle>);
impl Root {
    pub(crate) fn new<TH: Into<TargetHandle>>(th: TH) -> Self {
        Self(Some(th.into()))
    }
}
#[derive(Clone, PartialEq, Component, Default)]
pub(crate) struct Dependents(pub(crate) HashSet<TargetHandle>);
impl Dependents {
    pub(crate) fn new<THS: AsRef<[TargetHandle]>>(ths: THS) -> Self {
        let mut set = HashSet::new();
        for d in ths.as_ref() {
            let th = d.clone();
            set.insert(th);
        }
        Self(set)
    }
}
pub(crate) fn recursive_placement(
    mut elements: ParamSet<(
        Query<
            (),
            (
                Or<(
                    Changed<Grid>,
                    Changed<GridPlacement>,
                    Changed<Root>,
                    Changed<Dependents>,
                )>,
                With<GridPlacement>,
                With<Root>,
                With<Dependents>,
            ),
        >,
        Query<(
            Entity,
            &GridPlacement,
            Option<&Grid>,
            &Root,
            &Dependents,
            &Position<LogicalContext>,
            &Area<LogicalContext>,
            &Layer,
        )>,
        Query<(
            Option<&mut Grid>,
            &mut Position<LogicalContext>,
            &mut Area<LogicalContext>,
            &mut Layer,
            &mut GridPlacement,
        )>,
    )>,
    id_table: Res<IdTable>,
    layout: Res<Layout>,
    layout_grid: Res<LayoutGrid>,
) {
    let x = layout.is_changed();
    let y = layout_grid.is_changed();
    let z = !elements.p0().is_empty();
    if x || y || z {
        let roots = elements
            .p1()
            .iter()
            .map(|(entity, _, _, root, _, _, _, _)| {
                if root.0.is_none() {
                    return Some(entity);
                }
                None
            })
            .collect::<Vec<Option<Entity>>>();
        for r in roots {
            if let Some(r) = r {
                let (root_placement, root_offset) = layout_grid
                    .grid
                    .place(elements.p1().get(r).unwrap().1, *layout);
                let chain = recursive_placement_inner(
                    &elements.p1(),
                    root_placement,
                    r,
                    root_offset,
                    &id_table,
                    *layout,
                );
                for (entity, placement, new_grid_placement, offset) in chain {
                    if new_grid_placement.is_some() {
                        if let Some(mut grid) = elements.p2().get_mut(entity).unwrap().0 {
                            grid.size_to(new_grid_placement.unwrap());
                        }
                    }
                    *elements.p2().get_mut(entity).unwrap().1 = placement.section.position;
                    *elements.p2().get_mut(entity).unwrap().2 = placement.section.area;
                    *elements.p2().get_mut(entity).unwrap().3 = placement.layer;
                    elements
                        .p2()
                        .get_mut(entity)
                        .unwrap()
                        .4
                        .update_queued_offset(offset);
                }
            }
        }
    }
}
fn recursive_placement_inner(
    query: &Query<(
        Entity,
        &GridPlacement,
        Option<&Grid>,
        &Root,
        &Dependents,
        &Position<LogicalContext>,
        &Area<LogicalContext>,
        &Layer,
    )>,
    current_placement: Placement<LogicalContext>,
    current_entity: Entity,
    current_offset: Option<Section<LogicalContext>>,
    id_table: &IdTable,
    layout: Layout,
) -> Vec<(
    Entity,
    Placement<LogicalContext>,
    Option<Placement<LogicalContext>>,
    Option<Section<LogicalContext>>,
)> {
    let mut placed = vec![];
    if query.get(current_entity).unwrap().4 .0.is_empty() {
        placed.push((current_entity, current_placement, None, current_offset));
        return placed;
    }
    let grid = (*query.get(current_entity).unwrap().2.unwrap()).sized(current_placement);
    placed.push((
        current_entity,
        current_placement,
        Some(current_placement),
        current_offset,
    ));
    for dep in query.get(current_entity).unwrap().4 .0.iter() {
        let dep_entity = id_table.lookup_target(dep.clone()).unwrap();
        let dep_grid_placement = query.get(dep_entity).unwrap().1;
        let (dep_placement, dep_offset) = grid.place(dep_grid_placement, layout);
        if query.get(dep_entity).unwrap().4 .0.is_empty() {
            placed.push((dep_entity, dep_placement, None, dep_offset));
        } else {
            let recursion = recursive_placement_inner(
                query,
                dep_placement,
                dep_entity,
                dep_offset,
                id_table,
                layout,
            );
            placed.extend(recursion);
        }
    }
    placed
}
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct TargetHandle(pub String);
impl<S: AsRef<str>> From<S> for TargetHandle {
    fn from(value: S) -> Self {
        Self(value.as_ref().to_string())
    }
}
impl TargetHandle {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self(s.as_ref().to_string())
    }
    pub const DELIMITER: &'static str = ":";
    pub fn extend<S: AsRef<str>>(&self, e: S) -> Self {
        Self::new(self.0.clone() + Self::DELIMITER + e.as_ref())
    }
}
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ActionHandle(pub String);
impl<S: AsRef<str>> From<S> for ActionHandle {
    fn from(value: S) -> Self {
        Self(value.as_ref().to_string())
    }
}
#[derive(Resource, Default)]
pub(crate) struct IdTable {
    pub(crate) targets: HashMap<TargetHandle, Entity>,
    pub(crate) actions: HashMap<ActionHandle, Entity>,
}
impl IdTable {
    pub fn add_target<TH: Into<TargetHandle>>(&mut self, th: TH, entity: Entity) {
        self.targets.insert(th.into(), entity);
    }
    pub fn add_action<AH: Into<ActionHandle>>(&mut self, ah: AH, entity: Entity) {
        self.actions.insert(ah.into(), entity);
    }
    pub fn lookup_target<TH: Into<TargetHandle>>(&self, th: TH) -> Option<Entity> {
        self.targets.get(&th.into()).copied()
    }
    pub fn lookup_action<AH: Into<ActionHandle>>(&self, ah: AH) -> Option<Entity> {
        self.actions.get(&ah.into()).copied()
    }
}

#[derive(Default)]
pub struct OnEnd {
    pub actions: HashSet<ActionHandle>,
}
impl OnEnd {
    pub fn new<AH: Into<ActionHandle>>(ah: AH) -> Self {
        Self {
            actions: {
                let mut a = HashSet::new();
                a.insert(ah.into());
                a
            },
        }
    }
    pub fn with<AH: Into<ActionHandle>>(mut self, ah: AH) -> Self {
        self.actions.insert(ah.into());
        self
    }
}
