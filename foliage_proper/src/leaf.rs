use std::collections::HashSet;

use crate::ash::ClippingContext;
use crate::coordinate::elevation::{Elevation, RenderLayer};
use crate::coordinate::placement::Placement;
use crate::coordinate::points::Points;
use crate::coordinate::LogicalContext;
use crate::differential::{RenderLink, RenderRemoveQueue};
use crate::grid::Grid;
use crate::interaction::ClickInteractionListener;
use crate::opacity::Opacity;
use crate::tree::Tree;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::component::StorageType::Table;
use bevy_ecs::component::{ComponentHooks, ComponentId, StorageType};
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::{Component, OnRemove, Trigger};
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;

#[derive(Bundle, Default, Clone)]
pub(crate) struct Leaf {
    stem: Stem,
    dependents: Dependents,
    placement: Placement<LogicalContext>,
    elevation: Elevation,
    remove: Remove,
    visibility: Visibility,
    opacity: Opacity,
    clipping_context: ClippingContext,
    grid: Grid,
    points: Points<LogicalContext>,
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
#[derive(Event, Copy, Clone, Default)]
pub struct ResolveElevation {}
pub(crate) fn resolve_elevation(
    trigger: Trigger<ResolveElevation>,
    mut layers: Query<&mut RenderLayer>,
    elevations: Query<&Elevation>,
    dependents: Query<(&Stem, &Dependents)>,
    mut tree: Tree,
) {
    if let Ok((s, d)) = dependents.get(trigger.entity()) {
        let current = if let Some(se) = s.0 {
            layers.get(se).copied().unwrap_or_default()
        } else {
            RenderLayer::default()
        };
        let resolved = RenderLayer::new(
            current.0
                + elevations
                    .get(trigger.entity())
                    .copied()
                    .unwrap_or_default()
                    .0,
        );
        if let Ok(mut layer) = layers.get_mut(trigger.entity()) {
            *layer = resolved;
        };
        tree.trigger_targets(
            ResolveElevation {},
            d.0.iter().copied().collect::<Vec<Entity>>(),
        );
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
#[derive(Event, Default, Copy, Clone)]
pub struct ResolveVisibility();
pub(crate) fn resolve_visibility(
    trigger: Trigger<ResolveVisibility>,
    stems: Query<&Stem>,
    mut query: Query<&mut Visibility>,
    dependents: Query<&Dependents>,
    links: Query<&RenderLink>,
    mut remove_queue: ResMut<RenderRemoveQueue>,
    mut tree: Tree,
) {
    let entity = trigger.entity();
    let stem = stems.get(entity).copied().unwrap_or_default();
    let value = if let Some(s) = stem.0 {
        query.get(s).copied().unwrap()
    } else {
        Visibility::default()
    };
    if let Ok(mut visibility) = query.get_mut(entity) {
        visibility.visible = value.visible;
        if !value.visible {
            if let Ok(link) = links.get(trigger.entity()) {
                remove_queue.queue.get_mut(link).unwrap().insert(entity);
            }
        }
        if let Ok(deps) = dependents.get(trigger.entity()) {
            if !deps.0.is_empty() {
                tree.trigger_targets(
                    ResolveVisibility {},
                    deps.0.iter().map(|e| *e).collect::<Vec<Entity>>(),
                );
            }
        };
    }
}
