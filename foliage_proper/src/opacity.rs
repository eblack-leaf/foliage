use crate::anim::interpolation::Interpolations;
use crate::{Animate, Attachment, Branch, Component, Foliage, Stem, Tree};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{OnInsert, Trigger};
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;

#[derive(Component, Copy, Clone, PartialEq)]
#[component(on_insert = Opacity::on_insert)]
#[require(InheritedOpacity, BlendedOpacity)]
pub struct Opacity {
    pub value: f32,
}
impl Attachment for Opacity {
    fn attach(foliage: &mut Foliage) {
        foliage.enable_animation::<Self>();
        foliage.define(Opacity::stem_insert);
    }
}
impl Opacity {
    pub fn new(value: f32) -> Opacity {
        Opacity { value }
    }
    fn stem_insert(trigger: Trigger<OnInsert, Stem>, mut tree: Tree, stems: Query<&Stem>, blended: Query<&BlendedOpacity>) {
        let this = trigger.entity();
        let stem = stems.get(this).unwrap();
        if let Some(entity) = stem.id {
            let resolved = *blended.get(entity).unwrap();
            tree.entity(this).insert(InheritedOpacity {
                value: resolved.value,
            });
        }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let inherited = world.get::<InheritedOpacity>(this).unwrap();
        let current = world.get::<Opacity>(this).unwrap();
        let blended = BlendedOpacity::new(inherited.value * current.value);
        world.commands().entity(this).insert(blended);
        let deps = world.get::<Branch>(this).unwrap().ids.clone();
        for d in deps.iter() {
            world
                .commands()
                .entity(*d)
                .insert(InheritedOpacity::new(blended.value));
        }
    }
}
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
impl Default for Opacity {
    fn default() -> Self {
        Self::new(1.0)
    }
}
#[derive(Component, Copy, Clone, PartialEq)]
#[component(on_insert = Opacity::on_insert)]
pub struct InheritedOpacity {
    pub value: f32,
}
impl InheritedOpacity {
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}
impl Default for InheritedOpacity {
    fn default() -> Self {
        Self::new(1.0)
    }
}
#[repr(C)]
#[derive(Component, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlendedOpacity {
    pub value: f32,
}
impl BlendedOpacity {
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}
impl Default for BlendedOpacity {
    fn default() -> Self {
        Self::new(1.0)
    }
}
