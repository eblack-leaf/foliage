use bevy_ecs::entity::RemoteAllocator;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::world::World;

use crate::coordinate::{Area, Section};
use crate::leaf::{Grown, Leaf, Presence, SpawnedAt};
use crate::op::{Bud, Sown};
use crate::place::{Anchored, Caller};
use crate::placement::grid::Grid;
use crate::placement::location::Location;
use crate::rowan::{LayoutSection, Screen};

/// The tree itself, seen from the inside.
///
/// Owns the world, and the one allocator every name comes from whichever side of the boundary
/// asked for it. The only place raw `bevy_ecs` is touched.
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
        match bud.sown {
            // A stem carries no renderer, which is the whole of what makes it one.
            Sown::Stem => {}
        }
        entity.insert((
            SpawnedAt(bud.at),
            bud.placement.location.unwrap_or_default(),
            bud.placement.grid.unwrap_or_default(),
            LayoutSection::default(),
            Screen::default(),
        ));
        if let Some(anchored) = bud.placement.anchor {
            entity.insert(anchored);
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

    /// Every live element, in a stable order.
    pub(crate) fn leaves(&self) -> Vec<Leaf> {
        let mut leaves = self
            .world
            .iter_entities()
            .map(|entity| Leaf(entity.id()))
            .collect::<Vec<_>>();
        leaves.sort();
        leaves
    }

    pub(crate) fn location(&self, leaf: Leaf) -> Option<&Location> {
        self.world.get_entity(leaf.0).ok()?.get::<Location>()
    }

    pub(crate) fn grid(&self, leaf: Leaf) -> Option<Grid> {
        self.world.get_entity(leaf.0).ok()?.get::<Grid>().copied()
    }

    /// The character cell of `leaf`'s own font, at its own size.
    ///
    /// Per element rather than per engine: an app registers as many fonts as it likes and each
    /// element chooses, so `8.letters()` is eight cells of *that* element's font. An element that
    /// has not been given one has no cell.
    pub(crate) fn cell(&self, _leaf: Leaf) -> Area {
        // Fonts and their metrics land with text.
        Area::default()
    }

    /// The element `leaf`'s placement may read, if it has been given one.
    pub(crate) fn anchor(&self, leaf: Leaf) -> Option<Leaf> {
        Some(self.world.get_entity(leaf.0).ok()?.get::<Anchored>()?.to)
    }

    /// Where `leaf` was written into existence.
    pub(crate) fn spawned_at(&self, leaf: Leaf) -> Option<Caller> {
        Some(self.world.get_entity(leaf.0).ok()?.get::<SpawnedAt>()?.0)
    }

    /// Whether `from` reaches `target` by following anchors.
    ///
    /// Bounded by construction: an anchor is refused if it would close a cycle, so the chain this
    /// walks is always finite.
    pub(crate) fn reaches(&self, from: Leaf, target: Leaf) -> bool {
        let mut step = Some(from);
        while let Some(leaf) = step {
            if leaf == target {
                return true;
            }
            step = self.anchor(leaf);
        }
        false
    }

    pub(crate) fn set_location(&mut self, leaf: Leaf, location: Location) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(location);
        }
    }

    pub(crate) fn set_grid(&mut self, leaf: Leaf, grid: Grid) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(grid);
        }
    }

    pub(crate) fn set_anchor(&mut self, leaf: Leaf, to: Leaf, at: Caller) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(Anchored { to, at });
        }
    }

    pub(crate) fn layout_section(&self, leaf: Leaf) -> Option<Section> {
        Some(self.world.get_entity(leaf.0).ok()?.get::<LayoutSection>()?.0)
    }

    /// Where `leaf` appears, which is what an app reads and what a hit test runs against.
    pub(crate) fn screen(&self, leaf: Leaf) -> Option<Section> {
        Some(self.world.get_entity(leaf.0).ok()?.get::<Screen>()?.0)
    }

    pub(crate) fn settle(&mut self, leaf: Leaf, layout: Section, screen: Section) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert((LayoutSection(layout), Screen(screen)));
        }
    }
}
