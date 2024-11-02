use crate::anim::{Animate, Interpolations};
use crate::color::Color;
use crate::leaf::{Dependents, Stem};
use bevy_ecs::component::StorageType::SparseSet;
use bevy_ecs::component::{Component, ComponentHooks, ComponentId, StorageType};
use bevy_ecs::entity::Entity;
use bevy_ecs::world::DeferredWorld;

impl Animate for Opacity {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        Interpolations::new().with(start.value, end.value)
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(o) = interpolations.read(0) {
            self.value = o;
        }
    }
}

#[derive(Copy, Clone, Component)]
pub struct Opacity {
    value: f32,
}
impl From<f32> for Opacity {
    fn from(value: f32) -> Self {
        Opacity { value }
    }
}
impl Default for Opacity {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl Opacity {
    pub fn new(o: f32) -> Self {
        Self {
            value: o.clamp(0.0, 1.0),
        }
    }
}
#[derive(Copy, Clone, Default)]
pub struct EvaluateOpacity {
    recursive: bool,
    is_first: bool,
}
impl EvaluateOpacity {
    pub fn recursive() -> Self {
        Self { recursive: true, is_first: true }
    }
    pub fn no_deps() -> Self {
        Self { recursive: false, is_first: true }
    }
    pub(crate) fn on_insert(mut world: DeferredWorld, entity: Entity, _c: ComponentId) {
        let event = world.get::<EvaluateOpacity>(entity).copied().unwrap();
        if !event.is_first {
            let inherited = if let Some(stem) = world.get::<Stem>(entity).copied() {
                if let Some(s) = stem.0 {
                    if let Some(o) = world.get::<Opacity>(s).copied() {
                        o.value
                    } else { 1.0 }
                } else { 1.0 }
            } else { 1.0 };
            world.commands().entity(entity).insert(Opacity::new(inherited));
            if let Some(color) = world.get::<Color>(entity).copied() {
                let blended = inherited;
                world
                    .commands()
                    .entity(entity)
                    .insert(color.with_alpha(blended));
            }
        } else {
            let value = world.get::<Opacity>(entity).copied().unwrap().value;
            if let Some(color) = world.get::<Color>(entity).copied() {
                world
                    .commands()
                    .entity(entity)
                    .insert(color.with_alpha(value));
            }
        }
        if !world
            .get::<EvaluateOpacity>(entity)
            .copied()
            .unwrap()
            .recursive
        {
            return;
        }
        if let Some(ds) = world.get::<Dependents>(entity).cloned() {
            for d in ds.0 {
                world
                    .commands()
                    .entity(d)
                    .insert(EvaluateOpacity {
                        recursive: true,
                        is_first: false,
                    });
            }
        }
    }
}
impl Component for EvaluateOpacity {
    const STORAGE_TYPE: StorageType = SparseSet;
    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        _hooks.on_insert(EvaluateOpacity::on_insert);
    }
}
