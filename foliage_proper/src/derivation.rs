use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, DetectChanges, Res};
use bevy_ecs::query::{Added, Changed, Without};
use bevy_ecs::system::{ParamSet, Query, Resource};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Component)]
pub struct ResourceDerivedValue<R, V> {
    derivation: Derivation<R, V>,
}
impl<R, V> ResourceDerivedValue<R, V> {
    pub fn new(derivation: Derivation<R, V>) -> Self {
        Self { derivation }
    }
}
pub(crate) fn resource_derive_value<R: Resource + Clone, V: Component + 'static + Send + Sync>(
    mut dependent: ParamSet<(
        Query<(&mut V, &ResourceDerivedValue<R, V>)>,
        Query<(&mut V, &ResourceDerivedValue<R, V>), Added<ResourceDerivedValue<R, V>>>,
    )>,
    resource: Res<R>,
) {
    if resource.is_changed() {
        for (mut v, d) in dependent.p0().iter_mut() {
            (d.derivation)(resource.as_ref(), v.as_mut());
        }
    }
    for (mut a, d) in dependent.p1().iter_mut() {
        (d.derivation)(resource.as_ref(), a.as_mut());
    }
}
#[derive(Clone, Component)]
pub struct ComponentDerivedValue<I, D> {
    pub derivation: Derivation<I, D>,
    pub entity: Entity,
}
impl<I, D> ComponentDerivedValue<I, D> {
    pub fn derive_from(entity: Entity, derivation: Derivation<I, D>) -> Self {
        Self { derivation, entity }
    }
}
pub(crate) fn component_derive_value<I: Component + Clone, D: Component + 'static + Send + Sync>(
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
            (derivation.derivation)(ind, dep.as_mut());
        }
    }
    for (mut dep, derivation) in dependents.p1().iter_mut() {
        if let Ok(ind) = independent.p1().get(derivation.entity) {
            (derivation.derivation)(ind, dep.as_mut());
        }
    }
}
pub type Derivation<I, D> = fn(&I, &mut D);