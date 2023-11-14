use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Component, Query, Without};
use bevy_ecs::query::Changed;
#[derive(Component, Clone)]
pub struct Differential<T: Component + Clone + PartialEq + Send + Sync + 'static> {
    cache: T,
    differential: Option<T>,
}
impl<T: Component + Clone + PartialEq + Send + Sync + 'static> Differential<T> {
    pub fn new(t: T) -> Self {
        Self {
            cache: t,
            differential: None,
        }
    }
    pub(crate) fn cache_check_and_update(&mut self, t: &T) -> bool {
        if t != &self.cache {
            self.differential.replace(t.clone());
            self.cache = t.clone();
            return true;
        }
        false
    }
    pub(crate) fn differential(&mut self) -> Option<T> {
        self.differential.take()
    }
}
#[derive(Component, Default, Copy, Clone)]
pub struct DifferentialDisable {}

pub(crate) fn differential<T: Component + Clone + PartialEq + Send + Sync + 'static>(
    mut query: Query<
        (Entity, &T, &mut Differential<T>),
        (Changed<T>, Without<DifferentialDisable>),
    >,
) {
    for (entity, t, mut diff) in query.iter_mut() {
        if diff.cache_check_and_update(t) {}
    }
}
#[derive(Bundle, Clone)]
pub struct DifferentialBundle<T: Component + Clone + PartialEq + Send + Sync + 'static> {
    pub component: T,
    pub differential: Differential<T>,
}
impl<T: Component + Clone + PartialEq + Send + Sync + 'static> DifferentialBundle<T> {
    pub fn new(t: T) -> Self {
        Self {
            component: t.clone(),
            differential: Differential::new(t),
        }
    }
}
