use crate::ash::clip::ClipSection;
use crate::interaction::CurrentInteraction;
use crate::Elevation;
use crate::Logical;
use crate::Opacity;
use crate::Section;
use crate::Visibility;
use crate::{
    Animation, Component, FocusBehavior, InteractionPropagation, InteractionShape, Location, Tree,
    Update,
};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;
use std::collections::HashSet;

#[derive(Component)]
#[require(Stem, Branch)]
#[require(Opacity, Visibility, ClipSection)]
#[require(Section<Logical>, Elevation, InteractionShape, InteractionPropagation, FocusBehavior)]
#[component(on_add = Self::on_add)]
#[component(on_remove = Self::on_remove)]
pub struct Leaf {}

impl Default for Leaf {
    fn default() -> Self {
        Self::new()
    }
}

impl Leaf {
    pub fn new() -> Leaf {
        Leaf {}
    }
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .entity(this)
            .observe(Self::anim_opacity)
            .observe(Self::anim_elevation)
            .observe(Self::anim_location);
    }
    fn anim_opacity(
        trigger: Trigger<Update<Animation<Opacity>>>,
        opacities: Query<&Opacity>,
        mut tree: Tree,
    ) {
        if let Ok(o) = opacities.get(trigger.entity()) {
            tree.entity(trigger.entity()).insert(*o);
        }
    }
    fn anim_elevation(
        trigger: Trigger<Update<Animation<Elevation>>>,
        mut tree: Tree,
        elevation: Query<&Elevation>,
    ) {
        if let Ok(e) = elevation.get(trigger.entity()) {
            tree.entity(trigger.entity()).insert(*e);
        }
    }
    fn anim_location(trigger: Trigger<Update<Animation<Location>>>, mut tree: Tree) {
        tree.trigger_targets(Update::<Location>::new(), trigger.entity());
    }
    fn on_remove(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        if let Some(mut current) = world.get_resource_mut::<CurrentInteraction>() {
            if let Some(p) = current.primary {
                if p == this {
                    current.primary.take();
                    return;
                }
            }
            let mut found = false;
            for ps in current.pass_through.iter() {
                if *ps == this {
                    found = true;
                    break;
                }
            }
            if found {
                current.pass_through.retain(|p| *p != this);
            }
        }
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[component(on_insert = Stem::on_insert)]
#[component(on_replace = Stem::on_replace)]
pub struct Stem {
    pub id: Option<Entity>,
}
impl Default for Stem {
    fn default() -> Self {
        Stem::none()
    }
}

impl Stem {
    pub fn new(id: Option<Entity>) -> Self {
        Self { id }
    }
    pub fn some(entity: Entity) -> Self {
        Self { id: Some(entity) }
    }
    pub fn none() -> Self {
        Self { id: None }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let stem = world.get::<Stem>(this).copied().unwrap();
        if let Some(s) = stem.id {
            if let Some(mut deps) = world.get_mut::<Branch>(s) {
                deps.ids.insert(this);
            }
        }
    }
    fn on_replace(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let stem = world.get::<Stem>(this).copied().unwrap();
        if let Some(s) = stem.id {
            if let Some(mut deps) = world.get_mut::<Branch>(s) {
                deps.ids.remove(&this);
            }
        }
    }
}

#[derive(Component, Clone, Default)]
pub struct Branch {
    pub ids: HashSet<Entity>,
}
