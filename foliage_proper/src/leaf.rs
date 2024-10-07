use std::collections::{HashMap, HashSet};

use crate::ash::ClippingContext;
use crate::coordinate::elevation::{Elevation, RenderLayer};
use crate::coordinate::placement::Placement;
use crate::coordinate::points::Points;
use crate::coordinate::LogicalContext;
use crate::differential::{RenderLink, RenderRemoveQueue};
use crate::grid::resolve::ResolveGridLocation;
use crate::grid::Grid;
use crate::interaction::ClickInteractionListener;
use crate::opacity::{Opacity, ResolveOpacity};
use crate::tree::Tree;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{Event, EventWriter};
use bevy_ecs::prelude::Component;
use bevy_ecs::query::{Changed, Or, With};
use bevy_ecs::system::{Commands, ParamSet, Query};

#[derive(Bundle, Default)]
pub(crate) struct Leaf {
    stem: Stem,
    rs: ResolveStem,
    dependents: Dependents,
    placement: Placement<LogicalContext>,
    elevation: Elevation,
    re: ResolveElevation,
    remove: Remove,
    visibility: Visibility,
    rv: ResolveVisibility,
    opacity: Opacity,
    ro: ResolveOpacity,
    clipping_context: ClippingContext,
    grid: Grid,
    rg: ResolveGridLocation,
    points: Points<LogicalContext>,
}
#[derive(Component)]
pub struct InteractionsEnabled(pub bool);
pub(crate) fn interaction_enable(
    mut query: Query<(Entity, &InteractionsEnabled, &mut ClickInteractionListener)>,
    mut tree: Tree,
) {
    for (entity, enabled, mut listener) in query.iter_mut() {
        if enabled.0 {
            listener.enable();
        } else {
            listener.disable();
        }
        tree.entity(entity).remove::<InteractionsEnabled>();
    }
}
#[derive(Component, Copy, Clone)]
pub struct ChangeStem(pub Option<Entity>);
pub(crate) fn change_stem(
    mut query: Query<(Entity, &ChangeStem, &mut Stem)>,
    mut dependents: Query<&mut Dependents>,
    mut visibility: Query<&mut Visibility>,
    mut tree: Tree,
) {
    for (entity, change, mut stem) in query.iter_mut() {
        if let Some(old) = stem.0.take() {
            if let Ok(mut deps) = dependents.get_mut(old) {
                deps.0.remove(&entity);
            }
        }
        stem.0 = change.0;
        tree.entity(entity).remove::<ChangeStem>();
        tree.entity(entity)
            .insert(ResolveGridLocation {})
            .insert(ResolveElevation {})
            .insert(ResolveOpacity {});
        if let Some(s) = stem.0 {
            if let Ok(stem_vis) = visibility.get(s).copied() {
                if let Ok(mut v) = visibility.get_mut(entity) {
                    v.visible = stem_vis.visible;
                }
            }
        }
    }
}
#[derive(Component, Copy, Clone, Default)]
pub struct ResolveStem {}
pub(crate) fn update_stem_deps(
    mut query: Query<(Entity, &mut Stem), With<ResolveStem>>,
    mut dependents: Query<&mut Dependents>,
    mut tree: Tree,
) {
    for (entity, mut stem) in query.iter_mut() {
        if let Some(stem_entity) = stem.0 {
            if let Ok(mut deps) = dependents.get_mut(stem_entity) {
                deps.0.insert(entity);
                tree.entity(stem_entity)
                    .insert(ResolveOpacity {})
                    .insert(ResolveElevation {});
            }
        }
        tree.entity(entity).remove::<ResolveStem>();
    }
}

#[derive(Default, Component, Debug)]
pub(crate) struct Stem(pub(crate) Option<Entity>);
#[derive(Clone, PartialEq, Component, Default)]
pub(crate) struct Dependents(pub(crate) HashSet<Entity>);
#[derive(Copy, Clone, Component)]
pub struct Trigger(pub Entity);
impl Trigger {
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }
}
#[derive(Component)]
pub struct TriggeredEvent<E: Event + Clone + Send + Sync + 'static>(pub E);
impl<E: Event + Clone + Send + Sync + 'static> TriggeredEvent<E> {
    pub fn new(event: E) -> Self {
        Self(event)
    }
}
#[derive(Component, Default)]
pub struct TriggerEventSignal(pub(crate) bool);
pub(crate) fn apply_triggered<E: Event + Clone + Send + Sync + 'static>(
    signaled: Query<(&TriggeredEvent<E>, &TriggerEventSignal)>,
    mut writer: EventWriter<E>,
) {
    for (te, ts) in signaled.iter() {
        if ts.0 {
            writer.send(te.0.clone());
        }
    }
}
pub(crate) fn clear_trigger_signal(
    mut signals: Query<&mut TriggerEventSignal, Changed<TriggerEventSignal>>,
) {
    for mut trigger in signals.iter_mut() {
        trigger.0 = false;
    }
}
#[derive(Component, Copy, Clone, Default)]
pub struct ResolveElevation {}
pub(crate) fn dependent_elevation(
    mut check_and_update: ParamSet<(
        Query<Entity, With<ResolveElevation>>,
        Query<&mut RenderLayer>,
    )>,
    read: Query<(Entity, &Elevation, &Stem, &Dependents)>,
    mut tree: Tree,
) {
    if check_and_update.p0().is_empty() {
        return;
    }
    for entity in check_and_update.p0().iter() {
        tree.entity(entity).remove::<ResolveElevation>();
    }
    let mut updates = HashMap::new();
    for (e, elevation, stem, dependents) in read.iter() {
        if stem.0.is_none() {
            let layer = RenderLayer::new(elevation.0);
            updates.insert(e, layer);
            for dep in dependents.0.iter() {
                recursive_elevation(*dep, layer, &mut updates, &read);
            }
        }
    }
    for (e, l) in updates {
        *check_and_update.p1().get_mut(e).unwrap() = l;
    }
}
fn recursive_elevation(
    current: Entity,
    current_layer: RenderLayer,
    updates: &mut HashMap<Entity, RenderLayer>,
    query: &Query<(Entity, &Elevation, &Stem, &Dependents)>,
) {
    let data = query.get(current).unwrap();
    let layer = RenderLayer::new(current_layer.0 + data.1 .0);
    updates.insert(current, layer);
    for dep in data.3 .0.iter() {
        recursive_elevation(*dep, layer, updates, query);
    }
}

#[derive(Component, Copy, Clone, Default)]
pub struct Remove {
    should_remove: bool,
}

impl Remove {
    pub fn should_keep(&self) -> bool {
        !self.should_remove
    }
    pub fn should_remove(&self) -> bool {
        self.should_remove
    }
    pub fn queue_remove() -> Self {
        Self {
            should_remove: true,
        }
    }
    pub fn keep() -> Self {
        Self {
            should_remove: false,
        }
    }
}

pub(crate) fn recursive_removal(
    mut query: ParamSet<(
        Query<(Entity, &Remove), Changed<Remove>>,
        Query<&mut Remove>,
    )>,
    dependents: Query<&Dependents>,
) {
    if query.p0().is_empty() {
        return;
    }
    let mut set = HashSet::new();
    for (entity, remove) in query.p0().iter() {
        if remove.should_remove {
            let d = recursive_removal_inner(entity, &dependents);
            set.extend(d);
        }
    }
    for e in set.drain() {
        if let Ok(mut remove) = query.p1().get_mut(e) {
            remove.should_remove = true;
        }
    }
}

fn recursive_removal_inner(entity: Entity, query: &Query<&Dependents>) -> HashSet<Entity> {
    let mut set = HashSet::new();
    if let Ok(deps) = query.get(entity) {
        for d in deps.0.iter() {
            set.insert(*d);
            set.extend(recursive_removal_inner(*d, query));
        }
    }
    set
}

pub(crate) fn remove(
    removals: Query<
        (Entity, &Remove, Option<&RenderLink>, &Visibility),
        Or<(Changed<Remove>, Changed<Visibility>)>,
    >,
    mut cmd: Commands,
    mut remove_queue: ResMut<RenderRemoveQueue>,
) {
    for (entity, remove, opt_link, visibility) in removals.iter() {
        if remove.should_remove() || !visibility.visible() {
            if let Some(link) = opt_link {
                remove_queue.queue.get_mut(link).unwrap().insert(entity);
            }
            if remove.should_remove() {
                cmd.entity(entity).despawn();
            }
        }
    }
}

#[derive(Component, Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct Visibility {
    visible: bool,
}

impl Visibility {
    pub fn new(v: bool) -> Self {
        Self { visible: v }
    }
    pub fn visible(&self) -> bool {
        self.visible
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Self::new(true)
    }
}
#[derive(Component, Default, Copy, Clone)]
pub struct ResolveVisibility {}
pub(crate) fn recursive_visibility(
    mut query: ParamSet<(
        Query<(Entity, &Visibility), With<ResolveVisibility>>,
        Query<&mut Visibility>,
    )>,
    dependents: Query<&Dependents>,
    mut tree: Tree,
) {
    if query.p0().is_empty() {
        return;
    }
    let mut to_check = HashSet::new();
    for (entity, visibility) in query.p0().iter() {
        to_check.insert((entity, *visibility));
        tree.entity(entity).remove::<ResolveVisibility>();
    }
    let mut updated = HashSet::new();
    for (e, v) in to_check.drain() {
        let d = recursive_visibility_inner(e, v, &dependents);
        updated.extend(d);
    }
    for (e, v) in updated.drain() {
        if let Ok(mut visibility) = query.p1().get_mut(e) {
            visibility.visible = v.visible;
        }
    }
}

fn recursive_visibility_inner(
    entity: Entity,
    v: Visibility,
    query: &Query<&Dependents>,
) -> HashSet<(Entity, Visibility)> {
    let mut set = HashSet::new();
    if let Ok(deps) = query.get(entity) {
        for d in deps.0.iter() {
            set.insert((*d, v));
            set.extend(recursive_visibility_inner(*d, v, &query));
        }
    }
    set
}
pub trait HasRenderLink {
    fn has_link() -> bool {
        false
    }
}
