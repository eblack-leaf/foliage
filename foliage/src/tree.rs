use bevy_ecs::entity::RemoteAllocator;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::world::World;

use crate::leaf::{Grown, Leaf, Presence};
use crate::op::Bud;

/// The tree itself, seen from the inside.
///
/// Owns the world, and the one allocator every name comes from whichever side of the boundary
/// asked for it.
pub(crate) struct Tree {
    world: World,
    allocator: RemoteAllocator,
}

impl Tree {
    pub(crate) fn new() -> Self {
        let world = World::new();
        let allocator = world.entity_allocator().build_remote_allocator();
        Self { world, allocator }
    }

    pub(crate) fn allocate(&self) -> Leaf {
        Leaf(self.allocator.alloc())
    }

    /// What `leaf` names right now.
    pub(crate) fn presence(&self, leaf: Leaf) -> Presence {
        let entities = self.world.entities();
        if entities.contains_spawned(leaf.0) {
            Presence::Live
        } else if entities.contains(leaf.0) {
            Presence::Planted
        } else {
            Presence::Withered
        }
    }

    pub(crate) fn is_live(&self, leaf: Leaf) -> bool {
        self.presence(leaf) == Presence::Live
    }

    /// Grows `leaf`, reporting whether the name was still free to grow into.
    pub(crate) fn grow(&mut self, leaf: Leaf, under: Option<Leaf>, bud: Bud) -> bool {
        let Ok(mut entity) = self.world.spawn_at(leaf.0, Grown) else {
            return false;
        };
        match bud {
            Bud::Stem(stem) => {
                entity.insert(stem);
            }
        }
        if let Some(under) = under {
            entity.insert(ChildOf(under.0));
        }
        true
    }

    /// Takes `leaf` and everything beneath it down, and reports every name that went.
    pub(crate) fn wither(&mut self, leaf: Leaf) -> Vec<Leaf> {
        let mut gone = Vec::new();
        self.gather(leaf, &mut gone);
        if let Ok(entity) = self.world.get_entity_mut(leaf.0) {
            entity.despawn();
        }
        gone
    }

    fn gather(&self, leaf: Leaf, into: &mut Vec<Leaf>) {
        into.push(leaf);
        let Ok(entity) = self.world.get_entity(leaf.0) else {
            return;
        };
        let Some(children) = entity.get::<Children>() else {
            return;
        };
        for child in children.iter().copied() {
            self.gather(Leaf(child), into);
        }
    }

    /// The elements the app branched directly off `leaf`, in the order they were grown.
    pub(crate) fn branches(&self, leaf: Leaf) -> Vec<Leaf> {
        let Ok(entity) = self.world.get_entity(leaf.0) else {
            return Vec::new();
        };
        let Some(children) = entity.get::<Children>() else {
            return Vec::new();
        };
        children
            .iter()
            .copied()
            .filter(|child| {
                self.world
                    .get_entity(*child)
                    .is_ok_and(|child| child.contains::<Grown>())
            })
            .map(Leaf)
            .collect()
    }

    /// The element `leaf` was branched off, or `None` if it was planted at top level.
    pub(crate) fn trunk(&self, leaf: Leaf) -> Option<Leaf> {
        let entity = self.world.get_entity(leaf.0).ok()?;
        entity.get::<ChildOf>().map(|trunk| Leaf(trunk.0))
    }
}
