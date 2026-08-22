use crate::{Component, ResolvedVisibility, Resource};
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, ParamSet, Query};
use bevy_ecs::query::With;
use bevy_ecs::world::World;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

#[derive(Component, Clone)]
pub(crate) struct Differential<
    R: Clone + Send + Sync + 'static,
    RP: Clone + Send + Sync + 'static + PartialEq,
> {
    pub(crate) cache: Option<RP>,
    _phantom: PhantomData<R>,
}

impl<R: Clone + Send + Sync + 'static, RP: Clone + Send + Sync + 'static + PartialEq>
    Differential<R, RP>
{
    fn new() -> Self {
        Self {
            cache: None,
            _phantom: Default::default(),
        }
    }
    pub(crate) fn different(&mut self, packet: RP) -> bool {
        let mut different = false;
        if let Some(cached) = self.cache.as_ref() {
            if cached != &packet {
                different = true;
            }
        } else {
            different = true;
        }
        self.cache.replace(packet);
        different
    }
}

impl<R: Clone + Send + Sync + 'static, RP: Clone + Send + Sync + 'static + PartialEq> Default
    for Differential<R, RP>
{
    fn default() -> Self {
        Self::new()
    }
}
pub(crate) fn cached_differential<
    R: Clone + Send + Sync + 'static,
    RP: Clone + Send + Sync + 'static + Component + PartialEq,
>(
    mut values: ParamSet<(
        Query<(Entity, &RP), (Changed<RP>, With<Differential<R, RP>>)>,
        Query<&RP>,
    )>,
    mut caches: Query<&mut Differential<R, RP>>,
    mut visibility: ParamSet<(
        Query<&ResolvedVisibility>,
        Query<Entity, (Changed<ResolvedVisibility>, With<Differential<R, RP>>)>,
    )>,
    mut queue: ResMut<RenderQueue<R, RP>>,
) {
    // if visibility changed && is-visible => send current value && continue
    // (skip-on-missing throughout: an entity mid-teardown can have its attribute or cache
    // already removed in the same frame its visibility changes — nothing to restore then)
    let changed = visibility.p1().iter().collect::<Vec<_>>();
    for c in changed {
        let Ok(visible) = visibility.p0().get(c).map(|v| v.visible()) else {
            continue;
        };
        if visible {
            let Ok(v) = values.p1().get(c).map(|v| v.clone()) else {
                continue;
            };
            let Ok(mut cache) = caches.get_mut(c) else {
                continue;
            };
            cache.cache.replace(v.clone());
            tracing::trace!(
                entity = ?c,
                rp = std::any::type_name::<RP>(),
                "differential: visibility-restore queue"
            );
            queue.queue.insert(c, v);
        }
    }
    // if is-visible && != cached => send new + set cache
    for (e, v) in values.p0().iter() {
        let Ok(visible) = visibility.p0().get(e).map(|v| v.visible()) else {
            continue;
        };
        if visible {
            let Ok(mut cache) = caches.get_mut(e) else {
                continue;
            };
            if cache.different(v.clone()) {
                tracing::trace!(
                    entity = ?e,
                    rp = std::any::type_name::<RP>(),
                    "differential: changed queue"
                );
                queue.queue.insert(e, v.clone());
            }
        }
    }
}

#[derive(Resource)]
pub(crate) struct RenderQueue<R: Clone + Send + Sync + 'static, RP: Clone + Send + Sync + 'static> {
    pub(crate) queue: HashMap<Entity, RP>,
    _phantom: PhantomData<R>,
}

impl<R: Clone + Send + Sync + 'static, RP: Clone + Send + Sync + 'static> RenderQueue<R, RP> {
    pub(crate) fn new() -> Self {
        Self {
            queue: HashMap::new(),
            _phantom: Default::default(),
        }
    }
}

/// What `R`'s pipeline must tear down this frame, kept in two lanes by who asked.
///
/// `hidden` is retractable: visibility can flip more than once in a frame, and only where
/// it lands matters to the renderer. `despawned` is not -- the entity is gone, and no later
/// event in the same frame can make dropping it wrong. Held apart because the queue is
/// keyed by entity alone, so a single set would let a re-show cancel a despawn and strand
/// the entity's group on the GPU with nothing left to remove it.
#[derive(Resource)]
pub(crate) struct RenderRemoveQueue<R: Clone + Send + Sync + 'static> {
    pub(crate) hidden: HashSet<Entity>,
    pub(crate) despawned: HashSet<Entity>,
    _phantom: PhantomData<R>,
}

impl<R: Clone + Send + Sync + 'static> RenderRemoveQueue<R> {
    pub(crate) fn new() -> Self {
        Self {
            hidden: HashSet::new(),
            despawned: HashSet::new(),
            _phantom: Default::default(),
        }
    }
    /// Everything to be torn down, whichever lane asked for it.
    fn all(&self) -> HashSet<Entity> {
        self.hidden.union(&self.despawned).copied().collect()
    }
}
pub(crate) struct RenderQueueHandle<'a> {
    pub(crate) world: &'a mut World,
}
impl<'a> RenderQueueHandle<'a> {
    pub(crate) fn new(world: &'a mut World) -> Self {
        Self { world }
    }
    /// Read without draining, so [`attribute`](Self::attribute) can still see what was
    /// removed. Cleared once per frame by [`clear_removes`](Self::clear_removes).
    pub(crate) fn removes<R: Clone + Send + Sync + 'static>(&mut self) -> Vec<Entity> {
        self.world
            .get_resource::<RenderRemoveQueue<R>>()
            .unwrap()
            .all()
            .into_iter()
            .collect()
    }
    /// Ends the frame for `R`'s removals. Called once, after every pipeline has prepared.
    pub(crate) fn clear_removes<R: Clone + Send + Sync + 'static>(&mut self) {
        let mut queue = self
            .world
            .get_resource_mut::<RenderRemoveQueue<R>>()
            .unwrap();
        queue.hidden.clear();
        queue.despawned.clear();
    }
    pub(crate) fn remove_attr<
        R: Clone + Send + Sync + 'static,
        RP: Clone + Send + Sync + 'static,
    >(
        &mut self,
        entity: Entity,
    ) {
        self.world
            .get_resource_mut::<RenderQueue<R, RP>>()
            .unwrap()
            .queue
            .remove(&entity);
    }
    /// Drains `RP` updates, skipping entities `R` has removed this frame.
    ///
    /// An update and a removal can land in the same frame: the differential gates on
    /// visibility, but an entity visible earlier in the frame has already queued its
    /// attributes, and going invisible then pushes it into the remove queue (see
    /// `Visibility::push_remove_packet`). Pipelines drain removals first and tear the
    /// entity's group down, so those attributes would be read against a group that is gone
    /// -- an unwrap on `entity_to_group` away from a panic.
    ///
    /// Filtered here rather than guarded in each pipeline: every renderer resolves its group
    /// the same way, so handling it per-pipeline is one bug fixed six times and missed the
    /// seventh.
    pub(crate) fn attribute<R: Clone + Send + Sync + 'static, RP: Clone + Send + Sync + 'static>(
        &mut self,
    ) -> Vec<(Entity, RP)> {
        let removed = self
            .world
            .get_resource::<RenderRemoveQueue<R>>()
            .map(|q| q.all())
            .unwrap_or_default();
        self.world
            .get_resource_mut::<RenderQueue<R, RP>>()
            .unwrap()
            .queue
            .drain()
            .filter(|(entity, _)| !removed.contains(entity))
            .collect()
    }
}
