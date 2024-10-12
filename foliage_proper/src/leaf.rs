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
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::{Component, OnRemove, Trigger};
use bevy_ecs::system::Query;

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
#[derive(Event, Copy, Clone)]
pub struct UpdateStem(pub Option<Entity>);
pub(crate) fn update_stem_trigger(
    trigger: Trigger<UpdateStem>,
    mut stems: Query<&mut Stem>,
    mut dependents: Query<&mut Dependents>,
    mut tree: Tree,
) {
    if let Ok(mut current) = stems.get_mut(trigger.entity()) {
        let old = current.0.take();
        current.0 = trigger.event().0;
        if let Some(c) = current.0 {
            if let Ok(mut deps) = dependents.get_mut(c) {
                deps.0.insert(trigger.entity());
            }
        }
        if let Some(o) = old {
            if let Ok(mut deps) = dependents.get_mut(o) {
                deps.0.remove(&trigger.entity());
            }
        }
    } else {
        tracing::trace!("adding stem: {:?}", trigger.event().0);
        tree.entity(trigger.entity())
            .insert(Stem(trigger.event().0));
        if let Some(s) = trigger.event().0 {
            if let Ok(mut deps) = dependents.get_mut(s) {
                deps.0.insert(trigger.entity());
            }
        }
    }
}
pub(crate) fn stem_remove(
    trigger: Trigger<OnRemove, Stem>,
    stems: Query<&Stem>,
    mut dependents: Query<&mut Dependents>,
) {
    if let Ok(s) = stems.get(trigger.entity()) {
        if let Some(s) = s.0 {
            if let Ok(mut deps) = dependents.get_mut(s) {
                deps.0.remove(&trigger.entity());
            }
        }
    }
}
#[derive(Default, Component, Debug, Clone)]
pub(crate) struct Stem(pub(crate) Option<Entity>);
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
#[derive(Event, Default, Copy, Clone)]
pub struct ResolveVisibility(pub bool);
pub(crate) fn resolve_visibility(
    trigger: Trigger<ResolveVisibility>,
    mut query: Query<(&mut Visibility, &Dependents)>,
    links: Query<&RenderLink>,
    mut remove_queue: ResMut<RenderRemoveQueue>,
    mut tree: Tree,
) {
    let entity = trigger.entity();
    let value = trigger.event().0;
    if let Ok((mut visibility, deps)) = query.get_mut(entity) {
        visibility.visible = value;
        if !value {
            if let Ok(link) = links.get(trigger.entity()) {
                remove_queue.queue.get_mut(link).unwrap().insert(entity);
            }
        }
        tree.trigger_targets(
            *trigger.event(),
            deps.0.iter().map(|e| *e).collect::<Vec<Entity>>(),
        );
    }
}
