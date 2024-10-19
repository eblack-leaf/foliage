use std::collections::HashSet;

use crate::ash::ClippingContext;
use crate::coordinate::elevation::{Elevation, RenderLayer};
use crate::coordinate::placement::Placement;
use crate::coordinate::points::Points;
use crate::coordinate::section::{GpuSection, Section};
use crate::coordinate::LogicalContext;
use crate::differential::{RenderLink, RenderRemoveQueue};
use crate::grid::responsive::evaluate::EvaluateLocation;
use crate::grid::Grid;
use crate::interaction::ClickInteractionListener;
use crate::opacity::{EvaluateOpacity, Opacity};
use crate::tree::Tree;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::component::StorageType::{SparseSet, Table};
use bevy_ecs::component::{ComponentHooks, ComponentId, StorageType};
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::{Component, OnRemove, Trigger};
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;

#[derive(Bundle, Default, Clone)]
pub struct Leaf {
    stem: Stem,
    dependents: Dependents,
    placement: Placement<LogicalContext>,
    elevation: Elevation,
    remove: Remove,
    visibility: Visibility,
    opacity: Opacity,
    clipping_context: ClippingContext,
    grid: Grid,
    gs: GpuSection,
    points: Points<LogicalContext>,
}

impl Leaf {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn stem(mut self, s: Option<Entity>) -> Self {
        self.stem.0 = s;
        self
    }
    pub fn elevation<E: Into<Elevation>>(mut self, e: E) -> Self {
        self.elevation = e.into();
        self
    }
    pub fn opacity<O: Into<Opacity>>(mut self, o: O) -> Self {
        self.opacity = o.into();
        self
    }
    pub fn visibility<V: Into<Visibility>>(mut self, v: V) -> Self {
        self.visibility = v.into();
        self
    }
    pub fn grid(mut self, grid: Grid) -> Self {
        self.grid = grid;
        self
    }
    pub fn section<S: Into<Section<LogicalContext>>>(mut self, s: S) -> Self {
        self.placement.section = s.into();
        self
    }
    pub fn points<P: Into<Points<LogicalContext>>>(mut self, points: P) -> Self {
        self.points = points.into();
        self
    }
}

#[derive(Event, Copy, Clone)]
pub struct InteractionsEnabled(pub bool);
pub(crate) fn trigger_interactions_enable(
    trigger: Trigger<InteractionsEnabled>,
    mut query: Query<&mut ClickInteractionListener>,
) {
    if let Ok(mut listener) = query.get_mut(trigger.entity()) {
        if trigger.event().0 {
            listener.enable();
        } else {
            listener.disable();
        }
    }
}
#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct Stem(pub(crate) Option<Entity>);
impl Stem {
    pub(crate) fn on_insert(mut world: DeferredWorld, entity: Entity, _c: ComponentId) {
        let stem = world.get::<Stem>(entity).copied().unwrap();
        if let Some(s) = stem.0 {
            if let Some(mut deps) = world.get_mut::<Dependents>(s) {
                deps.0.insert(entity);
            }
        }
    }
    pub(crate) fn on_replace(mut world: DeferredWorld, entity: Entity, _c: ComponentId) {
        let stem = world.get::<Stem>(entity).copied().unwrap();
        if let Some(s) = stem.0 {
            if let Some(mut deps) = world.get_mut::<Dependents>(s) {
                deps.0.remove(&entity);
            }
        }
    }
}
impl Component for Stem {
    const STORAGE_TYPE: StorageType = Table;

    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        _hooks.on_insert(Stem::on_insert);
        _hooks.on_remove(Stem::on_replace);
    }
}
#[derive(Clone, PartialEq, Component, Default)]
pub(crate) struct Dependents(pub(crate) HashSet<Entity>);
#[derive(Copy, Clone)]
pub struct EvaluateElevation {
    recursive: bool,
}
impl EvaluateElevation {
    pub fn recursive() -> Self {
        Self { recursive: true }
    }
    pub fn no_deps() -> Self {
        Self { recursive: false }
    }
    pub(crate) fn on_insert(mut world: DeferredWorld, entity: Entity, _c: ComponentId) {
        let current = if let Some(stem) = world.get::<Stem>(entity) {
            if let Some(s) = stem.0 {
                world.get::<RenderLayer>(s).copied().unwrap_or_default()
            } else {
                RenderLayer::default()
            }
        } else {
            RenderLayer::default()
        };
        let resolved = RenderLayer::new(
            current.0
                + world
                    .get::<Elevation>(entity)
                    .copied()
                    .unwrap_or_default()
                    .0,
        );
        world.commands().entity(entity).insert(resolved);
        if let Some(ds) = world.get::<Dependents>(entity).cloned() {
            for d in ds.0 {
                world
                    .commands()
                    .entity(d)
                    .insert(EvaluateElevation::recursive());
            }
        }
    }
}
impl Component for EvaluateElevation {
    const STORAGE_TYPE: StorageType = SparseSet;
    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        _hooks.on_insert(EvaluateElevation::on_insert);
    }
}
#[derive(Event, Copy, Clone, Default)]
pub struct Remove {}

impl Remove {
    pub fn new() -> Self {
        Self {}
    }
}
pub(crate) fn triggered_remove(
    trigger: Trigger<Remove>,
    dependents: Query<&Dependents>,
    mut tree: Tree,
) {
    tree.entity(trigger.entity()).despawn();
    if let Ok(deps) = dependents.get(trigger.entity()) {
        tree.trigger_targets(
            Remove::new(),
            deps.0.iter().map(|e| *e).collect::<Vec<Entity>>(),
        );
    }
}
pub(crate) fn render_link_on_remove(
    trigger: Trigger<OnRemove, RenderLink>,
    mut links: Query<&RenderLink>,
    mut remove_queue: ResMut<RenderRemoveQueue>,
) {
    let links = links.get(trigger.entity()).unwrap();
    remove_queue
        .queue
        .get_mut(links)
        .unwrap()
        .insert(trigger.entity());
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Component)]
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
#[derive(Copy, Clone)]
pub struct EvaluateVisibility {
    first: bool,
    recursive: bool,
}
impl EvaluateVisibility {
    pub fn recursive() -> Self {
        Self {
            first: true,
            recursive: true,
        }
    }
    pub fn no_deps() -> Self {
        Self {
            first: true,
            recursive: false,
        }
    }
    pub(crate) fn on_insert(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        let value = world.get::<EvaluateVisibility>(entity).copied().unwrap();
        let stem = world.get::<Stem>(entity).copied().unwrap_or_default();
        let inherited = if let Some(s) = stem.0 {
            world.get::<Visibility>(s).copied().unwrap_or_default()
        } else {
            Visibility::default()
        };
        let current = world.get::<Visibility>(entity).copied().unwrap();
        let resolved = if value.first { current } else { inherited };
        world.commands().entity(entity).insert(resolved);
        if !resolved.visible {
            if let Some(link) = world.get::<RenderLink>(entity).copied() {
                world
                    .resource_mut::<RenderRemoveQueue>()
                    .queue
                    .get_mut(&link)
                    .unwrap()
                    .insert(entity);
            }
        }
        if !value.recursive {
            return;
        }
        if let Some(ds) = world.get::<Dependents>(entity).cloned() {
            for d in ds.0 {
                world.commands().entity(d).insert(EvaluateVisibility {
                    first: false,
                    recursive: true,
                });
            }
        }
    }
}
impl Component for EvaluateVisibility {
    const STORAGE_TYPE: StorageType = Table;
    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(EvaluateVisibility::on_insert);
    }
}
#[derive(Copy, Clone)]
pub struct EvaluateCore {
    full: bool,
}
impl EvaluateCore {
    pub fn recursive() -> Self {
        Self { full: true }
    }
    pub fn no_deps() -> Self {
        Self { full: false }
    }
    pub(crate) fn on_insert(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        let config = world.get::<EvaluateCore>(entity).copied().unwrap();
        world
            .commands()
            .entity(entity)
            .insert(EvaluateLocation {
                skip_deps: !config.full,
            })
            .insert(EvaluateElevation::recursive())
            .insert(EvaluateOpacity::recursive());
    }
}
impl Component for EvaluateCore {
    const STORAGE_TYPE: StorageType = SparseSet;
    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(EvaluateCore::on_insert);
    }
}
