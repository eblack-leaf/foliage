use crate::text::ResolvedGlyphs;
use crate::{Component, ResolvedVisibility, Resource};
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, ParamSet, Query};
use bevy_ecs::query::With;
use bevy_ecs::world::World;
use std::any::TypeId;
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

#[derive(Resource)]
pub(crate) struct RenderRemoveQueue<R: Clone + Send + Sync + 'static> {
    pub(crate) queue: HashSet<Entity>,
    _phantom: PhantomData<R>,
}

impl<R: Clone + Send + Sync + 'static> RenderRemoveQueue<R> {
    pub(crate) fn new() -> Self {
        Self {
            queue: HashSet::new(),
            _phantom: Default::default(),
        }
    }
}
pub(crate) struct RenderQueueHandle<'a> {
    pub(crate) world: &'a mut World,
}
impl<'a> RenderQueueHandle<'a> {
    pub(crate) fn new(world: &'a mut World) -> Self {
        Self { world }
    }
    pub(crate) fn removes<R: Clone + Send + Sync + 'static>(&mut self) -> Vec<Entity> {
        self.world
            .get_resource_mut::<RenderRemoveQueue<R>>()
            .unwrap()
            .queue
            .drain()
            .collect()
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
    pub(crate) fn attribute<R: Clone + Send + Sync + 'static, RP: Clone + Send + Sync + 'static>(
        &mut self,
    ) -> Vec<(Entity, RP)> {
        self.world
            .get_resource_mut::<RenderQueue<R, RP>>()
            .unwrap()
            .queue
            .drain()
            .collect()
    }
}
