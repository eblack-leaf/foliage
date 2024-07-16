use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::placement::Placement;
use crate::coordinate::position::Position;
use crate::coordinate::LogicalContext;
use crate::differential::Remove;
use crate::grid::{Grid, GridPlacement, Layout, LayoutGrid};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::Res;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, Component, DetectChanges, ParamSet, Query, Resource};
use bevy_ecs::query::{Or, With};
use std::collections::{HashMap, HashSet};

#[derive(Bundle, Default)]
pub struct Element {
    root: Root,
    dependents: Dependents,
    placement: Placement<LogicalContext>,
    remove: Remove,
}
#[derive(Default, Component)]
pub struct Root(pub Option<TargetHandle>);
impl Root {
    pub fn new<TH: Into<TargetHandle>>(th: TH) -> Self {
        Self(Some(th.into()))
    }
}
#[derive(Clone, PartialEq, Component, Default)]
pub struct Dependents(pub HashSet<TargetHandle>);
impl Dependents {
    pub fn new<THS: AsRef<[TargetHandle]>>(ths: THS) -> Self {
        let mut set = HashSet::new();
        for d in ths.as_ref() {
            let th = d.clone().into();
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
        )>,
    )>,
    id_table: Res<IdTable>,
    layout: Res<Layout>,
    layout_grid: Res<LayoutGrid>,
) {
    if layout.is_changed() || layout_grid.is_changed() || !elements.p0().is_empty() {
        let roots = elements
            .p1()
            .iter()
            .map(
                |(entity, grid_placement, opt_grid, root, deps, pos, area, layer)| {
                    if root.0.is_none() {
                        return Some(entity);
                    }
                    None
                },
            )
            .collect::<Vec<Option<Entity>>>();
        for r in roots {
            if let Some(r) = r {
                let root_placement = Placement::default();
                let chain = recursive_placement_inner(&elements.p1(), root_placement, r, &id_table);
                for (entity, placement, new_grid_placement) in chain {
                    if new_grid_placement.is_some() {
                        if let Some(mut grid) = elements.p2().get_mut(entity).unwrap().0 {
                            *grid = grid.clone().placed_at(new_grid_placement.unwrap());
                        }
                    }
                    *elements.p2().get_mut(entity).unwrap().1 = placement.section.position;
                    *elements.p2().get_mut(entity).unwrap().2 = placement.section.area;
                    *elements.p2().get_mut(entity).unwrap().3 = placement.layer;
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
    root_placement: Placement<LogicalContext>,
    current_entity: Entity,
    id_table: &IdTable,
) -> Vec<(
    Entity,
    Placement<LogicalContext>,
    Option<Placement<LogicalContext>>,
)> {
    todo!()
}
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct TargetHandle(pub String);
impl<S: AsRef<str>> From<S> for TargetHandle {
    fn from(value: S) -> Self {
        Self(value.as_ref().to_string())
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
pub struct IdTable {
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
    pub fn lookup_target<TH: Into<TargetHandle>>(&self, th: TH) -> Entity {
        *self.targets.get(&th.into()).unwrap()
    }
    pub fn lookup_action<AH: Into<ActionHandle>>(&self, ah: AH) -> Entity {
        *self.actions.get(&ah.into()).unwrap()
    }
}
