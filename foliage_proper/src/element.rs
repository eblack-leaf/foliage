use std::collections::{HashMap, HashSet};

use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::Res;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, Component, DetectChanges, ParamSet, Query, Resource};
use bevy_ecs::query::{Or, With};

use crate::anim::{Animate, Interpolations};
use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::elevation::{Elevation, RenderLayer};
use crate::coordinate::placement::Placement;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::LogicalContext;
use crate::differential::{Remove, Visibility};
use crate::grid::{Grid, GridPlacement, Layout, LayoutGrid};

#[derive(Bundle, Default)]
pub(crate) struct Element {
    stem: Stem,
    dependents: Dependents,
    placement: Placement<LogicalContext>,
    remove: Remove,
    visibility: Visibility,
    opacity: Opacity,
}
#[derive(Default, Component)]
pub(crate) struct Stem(pub(crate) Option<LeafHandle>);
impl Stem {
    pub(crate) fn new<TH: Into<LeafHandle>>(th: TH) -> Self {
        Self(Some(th.into()))
    }
}
#[derive(Clone, PartialEq, Component, Default)]
pub(crate) struct Dependents(pub(crate) HashSet<LeafHandle>);
impl Dependents {
    pub(crate) fn new<THS: AsRef<[LeafHandle]>>(ths: THS) -> Self {
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
                    Changed<Stem>,
                    Changed<Dependents>,
                    Changed<RenderLayer>,
                )>,
                With<GridPlacement>,
                With<RenderLayer>,
                With<Stem>,
                With<Dependents>,
            ),
        >,
        Query<(
            Entity,
            &GridPlacement,
            Option<&Grid>,
            &Stem,
            &Dependents,
            &Position<LogicalContext>,
            &Area<LogicalContext>,
            &Elevation,
            &RenderLayer,
        )>,
        Query<(
            Option<&mut Grid>,
            &mut Position<LogicalContext>,
            &mut Area<LogicalContext>,
            &mut RenderLayer,
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
        let stems = elements
            .p1()
            .iter()
            .map(|(entity, _, _, root, _, _, _, _, _)| {
                if root.0.is_none() {
                    return Some(entity);
                }
                None
            })
            .collect::<Vec<Option<Entity>>>();
        for s in stems {
            if let Some(r) = s {
                let elevation = *elements.p1().get(r).unwrap().7;
                let (stem_placement, stem_offset) =
                    layout_grid
                        .grid
                        .place(elements.p1().get(r).unwrap().1, elevation, *layout);
                let chain = recursive_placement_inner(
                    &elements.p1(),
                    stem_placement,
                    r,
                    stem_offset,
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
                    *elements.p2().get_mut(entity).unwrap().3 = placement.render_layer;
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
        &Stem,
        &Dependents,
        &Position<LogicalContext>,
        &Area<LogicalContext>,
        &Elevation,
        &RenderLayer,
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
        let dep_entity = id_table.lookup_leaf(dep.clone()).unwrap();
        let dep_grid_placement = query.get(dep_entity).unwrap().1;
        let (dep_placement, dep_offset) = grid.place(
            dep_grid_placement,
            *query.get(dep_entity).unwrap().7,
            layout,
        );
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
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct LeafHandle(pub String);
impl<S: AsRef<str>> From<S> for LeafHandle {
    fn from(value: S) -> Self {
        Self(value.as_ref().to_string())
    }
}
impl LeafHandle {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self(s.as_ref().to_string())
    }
    pub const DELIMITER: &'static str = ":";
    pub fn extend<S: AsRef<str>>(&self, e: S) -> Self {
        Self::new(self.0.clone() + Self::DELIMITER + e.as_ref())
    }
}
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct TwigHandle(pub String);
impl<S: AsRef<str>> From<S> for TwigHandle {
    fn from(value: S) -> Self {
        Self(value.as_ref().to_string())
    }
}
#[derive(Resource, Default)]
pub(crate) struct IdTable {
    pub(crate) leafs: HashMap<LeafHandle, Entity>,
    pub(crate) twigs: HashMap<TwigHandle, Entity>,
}
impl IdTable {
    pub fn add_target<TH: Into<LeafHandle>>(&mut self, th: TH, entity: Entity) -> Option<Entity> {
        self.leafs.insert(th.into(), entity)
    }
    pub fn add_twig<AH: Into<TwigHandle>>(&mut self, ah: AH, entity: Entity) -> Option<Entity> {
        self.twigs.insert(ah.into(), entity)
    }
    pub fn lookup_leaf<TH: Into<LeafHandle>>(&self, th: TH) -> Option<Entity> {
        self.leafs.get(&th.into()).copied()
    }
    pub fn lookup_twig<AH: Into<TwigHandle>>(&self, ah: AH) -> Option<Entity> {
        self.twigs.get(&ah.into()).copied()
    }
}

#[derive(Default)]
pub struct OnEnd {
    pub actions: HashSet<TwigHandle>,
}
impl OnEnd {
    pub fn new<AH: Into<TwigHandle>>(ah: AH) -> Self {
        Self {
            actions: {
                let mut a = HashSet::new();
                a.insert(ah.into());
                a
            },
        }
    }
    pub fn with<AH: Into<TwigHandle>>(mut self, ah: AH) -> Self {
        self.actions.insert(ah.into());
        self
    }
}
impl Animate for Opacity {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        Interpolations::new().with(start.value, end.value)
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(o) = interpolations.read(0) {
            self.value = o;
        }
    }
}
#[derive(Copy, Clone, Component)]
pub struct Opacity {
    value: f32,
}
impl Default for Opacity {
    fn default() -> Self {
        Self::new(1.0)
    }
}
impl Opacity {
    pub fn new(o: f32) -> Self {
        Self {
            value: o.clamp(0.0, 1.0),
        }
    }
}
pub(crate) fn opacity(
    mut opaque: ParamSet<(
        Query<Entity, Or<(Changed<Color>, Changed<Opacity>, Changed<Dependents>)>>,
        Query<(&Opacity, &Dependents)>,
        Query<&mut Color>,
    )>,
    roots: Query<&Stem>,
    id_table: Res<IdTable>,
) {
    let mut to_check = vec![];
    for entity in opaque.p0().iter() {
        to_check.push(entity);
    }
    for entity in to_check {
        let inherited = if let Ok(r) = roots.get(entity) {
            if let Some(rh) = r.0.as_ref() {
                let e = id_table.lookup_leaf(rh.clone()).unwrap();
                let inherited = *opaque.p1().get(e).unwrap().0;
                Some(inherited.value)
            } else {
                None
            }
        } else {
            None
        };
        let changed = recursive_opacity(&opaque.p1(), entity, &id_table, inherited);
        for (entity, o) in changed {
            if let Ok(mut color) = opaque.p2().get_mut(entity) {
                color.set_alpha(o);
            }
        }
    }
}
fn recursive_opacity(
    query: &Query<(&Opacity, &Dependents)>,
    current: Entity,
    id_table: &IdTable,
    inherited_opacity: Option<f32>,
) -> Vec<(Entity, f32)> {
    let mut changed = vec![];
    if let Ok((opacity, deps)) = query.get(current) {
        let blended = opacity.value * inherited_opacity.unwrap_or(1.0);
        changed.push((current, blended));
        for dep in deps.0.iter() {
            let e = id_table.lookup_leaf(dep.clone()).unwrap();
            changed.extend(recursive_opacity(query, e, id_table, Some(blended)));
        }
    }
    changed
}
