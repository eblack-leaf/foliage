use crate::{Branch, Component, Stem};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::DeferredWorld;

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Component)]
#[component(on_add = Visibility::on_add)]
#[component(on_insert = Visibility::on_insert)]
#[require(InheritedVisibility, ResolvedVisibility)]
pub struct Visibility {
    visible: bool,
}
impl Visibility {
    pub fn new(v: bool) -> Self {
        Self { visible: v }
    }
    pub fn visible(&self) -> bool {
        self.visible
    }
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let stem = world.get::<Stem>(this).unwrap();
        if let Some(s) = stem.id {
            let resolved = *world.get::<ResolvedVisibility>(s).unwrap();
            world.commands().entity(this).insert(InheritedVisibility {
                visible: resolved.visible,
            });
        }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let inherited = world.get::<InheritedVisibility>(this).unwrap();
        let current = world.get::<Visibility>(this).unwrap();
        let resolved = ResolvedVisibility {
            visible: inherited.visible && current.visible,
        };
        world.commands().entity(this).insert(resolved);
        let deps = world.get::<Branch>(this).unwrap().ids.clone();
        for d in deps {
            world.commands().entity(d).insert(InheritedVisibility {
                visible: resolved.visible,
            });
        }
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Self::new(true)
    }
}
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Component)]
#[component(on_insert = Visibility::on_insert)]
pub struct InheritedVisibility {
    visible: bool,
}
impl Default for InheritedVisibility {
    fn default() -> Self {
        Self { visible: true }
    }
}
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Component)]
pub struct ResolvedVisibility {
    visible: bool,
}
impl Default for ResolvedVisibility {
    fn default() -> Self {
        Self { visible: true }
    }
}