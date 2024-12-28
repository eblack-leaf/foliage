use crate::Component;
use crate::Elevation;
use crate::LogicalContext;
use crate::Opacity;
use crate::Section;
use crate::Visibility;
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::DeferredWorld;
use std::collections::HashSet;
#[derive(Component)]
#[require(Stem, Branch)]
#[require(Opacity, Visibility)]
#[require(Section<LogicalContext>, Elevation)]
pub struct Leaf {}

impl Leaf {
    pub fn new() -> Leaf {
        Leaf {}
    }
}

#[derive(Component, Copy, Clone)]
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

#[derive(Component, Clone)]
pub struct Branch {
    pub ids: HashSet<Entity>,
}

impl Default for Branch {
    fn default() -> Self {
        Self {
            ids: HashSet::new(),
        }
    }
}
