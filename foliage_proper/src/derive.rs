use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Component};
use bevy_ecs::system::{Commands, Query, ResMut, Resource};

pub type DeriveFn<D, B> = fn(&mut D) -> B;
#[derive(Component, Clone)]
pub struct DerivedValue<
    D: Resource + Send + Sync + 'static + Clone,
    B: Bundle + Send + Sync + 'static + Clone,
> {
    d_fn: Box<DeriveFn<D, B>>,
    listening: bool,
}
impl<D: Resource + Send + Sync + 'static + Clone, B: Bundle + Send + Sync + 'static + Clone>
    DerivedValue<D, B>
{
    pub fn new(func: DeriveFn<D, B>) -> Self {
        Self {
            d_fn: Box::new(func),
            listening: true,
        }
    }
    pub fn listen(&mut self) {
        self.listening = true;
    }
}
// TODO change to Changed<B> => read Resource + if ResourceChanged => all B
pub(crate) fn on_derive<
    D: Resource + Send + Sync + 'static + Clone,
    B: Bundle + Send + Sync + 'static + Clone,
>(
    mut derivees: Query<(Entity, &mut DerivedValue<D, B>)>,
    mut resource: ResMut<D>,
    mut cmd: Commands,
) {
    for (entity, mut derivee) in derivees.iter_mut() {
        if derivee.listening {
            cmd.entity(entity).insert((derivee.d_fn)(resource.as_mut()));
            derivee.listening = false;
        }
    }
}
