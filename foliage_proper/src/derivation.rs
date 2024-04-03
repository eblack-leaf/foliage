use std::marker::PhantomData;

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, DetectChanges, Res};
use bevy_ecs::query::{Added, Changed, With, Without};
use bevy_ecs::system::{ParamSet, Query, Resource};

#[derive(Copy, Clone, Component)]
pub struct ResourceDerivedValue<R, V> {
    _a: PhantomData<R>,
    _b: PhantomData<V>,
}
impl<R, V> ResourceDerivedValue<R, V> {
    pub fn new() -> Self {
        Self {
            _a: PhantomData::default(),
            _b: Default::default(),
        }
    }
}
pub(crate) fn resource_derive_value<
    R: Into<V> + Resource + Clone,
    V: Component + 'static + Send + Sync,
>(
    mut dependent: ParamSet<(
        Query<&mut V, With<ResourceDerivedValue<R, V>>>,
        Query<&mut V, Added<ResourceDerivedValue<R, V>>>,
    )>,
    resource: Res<R>,
) {
    if resource.is_changed() {
        for mut v in dependent.p0().iter_mut() {
            *v = resource.as_ref().clone().into();
        }
    }
    for mut a in dependent.p1().iter_mut() {
        *a = resource.as_ref().clone().into();
    }
}
#[derive(Copy, Clone, Component)]
pub struct ComponentDerivedValue<I, D> {
    _a: PhantomData<I>,
    _b: PhantomData<D>,
    pub entity: Entity,
}
impl<I, D> ComponentDerivedValue<I, D> {
    pub fn derive_from(entity: Entity) -> Self {
        Self {
            _a: Default::default(),
            _b: Default::default(),
            entity,
        }
    }
}
pub(crate) fn component_derive_value<
    I: Into<D> + Component + Clone,
    D: Component + 'static + Send + Sync,
>(
    mut independent: ParamSet<(
        Query<&I, (Without<ComponentDerivedValue<I, D>>, Changed<I>)>,
        Query<&I, Without<ComponentDerivedValue<I, D>>>,
    )>,
    mut dependents: ParamSet<(
        Query<(&mut D, &ComponentDerivedValue<I, D>)>,
        Query<(&mut D, &ComponentDerivedValue<I, D>), Added<ComponentDerivedValue<I, D>>>,
    )>,
) {
    for (mut dep, derivation) in dependents.p0().iter_mut() {
        if let Ok(ind) = independent.p0().get(derivation.entity) {
            *dep = ind.clone().into();
        }
    }
    for (mut dep, derivation) in dependents.p1().iter_mut() {
        if let Ok(ind) = independent.p1().get(derivation.entity) {
            *dep = ind.clone().into();
        }
    }
}
