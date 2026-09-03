use std::sync::atomic::{AtomicU64, Ordering};

use bevy_ecs::component::Component;
use bevy_ecs::entity::RemoteAllocator;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::world::World;

use crate::coordinate::{Area, Axes, Position, Section};
use crate::elevation::{Elevation, ResolvedElevation};
use crate::elm::{Chlorophyll, PanelPigment};
use crate::interaction::Gestures;
use crate::leaf::{Grown, Growth, Leaf, Presence, SpawnedAt};
use crate::lifecycle::{Disabled, Inherited, Opacity, Visible};
use crate::op::Bud;
use crate::palette::Fill;
use crate::place::{Anchored, Caller, Focusing};
use crate::placement::grid::Grid;
use crate::placement::location::Location;
use crate::rounding::Corners;
use crate::rowan::{Drawn, Placed};
use crate::view::{Clipped, Extent, Offset, Scrolls};

/// The tree itself, seen from the inside.
///
/// Owns the world, and the one allocator every name comes from whichever side of the boundary
/// asked for it. The only place raw `bevy_ecs` is touched.
pub(crate) struct Tree {
    world: World,
    allocator: RemoteAllocator,
    growth: AtomicU64,
}

impl Tree {
    pub(crate) fn new() -> Self {
        let world = World::new();
        let allocator = world.entity_allocator().build_remote_allocator();
        Self {
            world,
            allocator,
            growth: AtomicU64::new(0),
        }
    }

    /// A name, and its place in allocation order.
    ///
    /// Both are taken here rather than at the drain, so the order is the order `plant` and `branch`
    /// were called in. The counter is atomic because allocation takes `&self` on either side of the
    /// boundary, and an op issued off-thread is ordered against the frame's own by nothing but when
    /// it arrived.
    pub(crate) fn allocate(&self) -> (Leaf, Growth) {
        let leaf = Leaf(self.allocator.alloc());
        (leaf, Growth(self.growth.fetch_add(1, Ordering::Relaxed)))
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
    pub(crate) fn grow(
        &mut self,
        leaf: Leaf,
        growth: Growth,
        under: Option<Leaf>,
        bud: Bud,
    ) -> bool {
        let Ok(mut entity) = self.world.spawn_at(leaf.0, Grown) else {
            return false;
        };
        entity.insert((
            SpawnedAt(bud.at),
            growth,
            bud.chlorophyll,
            bud.placement.location.unwrap_or_default(),
            bud.placement.grid.unwrap_or_default(),
            bud.placement.elevation.unwrap_or_default(),
            ResolvedElevation::default(),
            Placed::default(),
            Drawn::default(),
        ));
        // Everything an element declares about how it behaves, and the values the passes that read
        // those declarations write back. Each is present on every element, so no pass has to ask
        // whether an element is the kind of thing that has one.
        let manner = bud.placement.manner;
        entity.insert((
            manner.gestures,
            manner.focusing,
            manner.visible,
            manner.opacity,
            Disabled::default(),
            Inherited::default(),
            Offset::default(),
            Extent::default(),
            Clipped::default(),
        ));
        if let Some(scrolls) = manner.scrolls {
            entity.insert(scrolls);
        }
        if let Some(pigment) = bud.pigment {
            entity.insert(pigment);
        }
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

    /// What `leaf` measured to: max-content across, and the height it wrapped to down.
    ///
    /// What [`content()`](crate::content) reads, of the element itself or of one it names. An
    /// element with nothing in it measures to zero.
    pub(crate) fn intrinsic(&self, _leaf: Leaf) -> Area {
        // Measuring lands with text.
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

    /// Where the layout put `leaf`, which is what its children resolve against.
    ///
    /// Every grown element carries one, so this is the answer for anything live and a zero box for
    /// anything else.
    pub(crate) fn placed(&self, leaf: Leaf) -> Section {
        self.read::<Placed>(leaf).unwrap_or_default().0
    }

    /// Where `leaf` is on screen, which is what an app reads and what a hit test runs against.
    pub(crate) fn drawn(&self, leaf: Leaf) -> Section {
        self.read::<Drawn>(leaf).unwrap_or_default().0
    }

    pub(crate) fn settle(&mut self, leaf: Leaf, placed: Section, drawn: Section) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert((Placed(placed), Drawn(drawn)));
        }
    }

    /// What `leaf` draws, and what the renderer drawing it was told.
    pub(crate) fn chlorophyll(&self, leaf: Leaf) -> Chlorophyll {
        self.read::<Chlorophyll>(leaf).unwrap_or_default()
    }

    /// How far in front of its trunk `leaf` was told to sit.
    pub(crate) fn elevation(&self, leaf: Leaf) -> Elevation {
        self.read::<Elevation>(leaf).unwrap_or_default()
    }

    /// Where `leaf` sits in the one stack, as R6 last resolved it.
    pub(crate) fn rank(&self, leaf: Leaf) -> ResolvedElevation {
        self.read::<ResolvedElevation>(leaf).unwrap_or_default()
    }

    /// Where `leaf` came in allocation order.
    pub(crate) fn growth(&self, leaf: Leaf) -> Growth {
        self.read::<Growth>(leaf).unwrap_or_default()
    }

    pub(crate) fn set_elevation(&mut self, leaf: Leaf, elevation: Elevation) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(elevation);
        }
    }

    pub(crate) fn set_rank(&mut self, leaf: Leaf, rank: ResolvedElevation) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(rank);
        }
    }

    /// What the panel renderer on `leaf` was told, or `None` if it draws nothing.
    pub(crate) fn pigment(&self, leaf: Leaf) -> Option<PanelPigment> {
        self.read::<PanelPigment>(leaf)
    }

    /// Refills `leaf`, reporting whether it is something with a fill to write.
    pub(crate) fn set_fill(&mut self, leaf: Leaf, fill: Fill) -> bool {
        self.pigment_mut(leaf, |pigment| pigment.fill = fill)
    }

    /// Rounds `leaf`'s corners, reporting whether it is something with corners to round.
    pub(crate) fn set_rounding(&mut self, leaf: Leaf, rounding: Corners) -> bool {
        self.pigment_mut(leaf, |pigment| pigment.rounding = rounding)
    }

    /// An element that draws nothing has no pigment, so there is nothing to write and the op that
    /// asked is dropped like any other that named something it does not apply to.
    fn pigment_mut(&mut self, leaf: Leaf, write: impl FnOnce(&mut PanelPigment)) -> bool {
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return false;
        };
        let Some(mut pigment) = entity.get_mut::<PanelPigment>() else {
            return false;
        };
        write(&mut pigment);
        true
    }

    /// What `leaf` declared about gestures.
    pub(crate) fn gestures(&self, leaf: Leaf) -> Gestures {
        self.read::<Gestures>(leaf).unwrap_or_default()
    }

    /// Which axes `leaf` scrolls, or `None` if it does not scroll.
    pub(crate) fn scrolls(&self, leaf: Leaf) -> Option<Axes> {
        Some(self.read::<Scrolls>(leaf)?.0)
    }

    /// Where `leaf` was told to sit in focus order, relative to the elements around it.
    pub(crate) fn focus_order(&self, leaf: Leaf) -> i32 {
        self.read::<Focusing>(leaf).unwrap_or_default().order
    }

    /// Whether focus cycles inside `leaf`.
    pub(crate) fn focus_scope(&self, leaf: Leaf) -> bool {
        self.read::<Focusing>(leaf).unwrap_or_default().scope
    }

    /// How far `leaf` has been scrolled.
    pub(crate) fn offset(&self, leaf: Leaf) -> Position {
        self.read::<Offset>(leaf).unwrap_or_default().0
    }

    pub(crate) fn set_offset(&mut self, leaf: Leaf, offset: Position) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(Offset(offset));
        }
    }

    /// How far `leaf`'s content reaches, as R3 last measured it.
    pub(crate) fn extent(&self, leaf: Leaf) -> Area {
        self.read::<Extent>(leaf).unwrap_or_default().0
    }

    pub(crate) fn set_extent(&mut self, leaf: Leaf, extent: Area) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(Extent(extent));
        }
    }

    /// What a scrolling ancestor leaves visible of `leaf`.
    pub(crate) fn clip(&self, leaf: Leaf) -> Section {
        self.read::<Clipped>(leaf).unwrap_or_default().0
    }

    pub(crate) fn set_clip(&mut self, leaf: Leaf, clip: Section) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(Clipped(clip));
        }
    }

    /// Whether the app has hidden `leaf` itself, as against an ancestor of it.
    pub(crate) fn visible(&self, leaf: Leaf) -> Visible {
        self.read::<Visible>(leaf).unwrap_or_default()
    }

    pub(crate) fn set_visible(&mut self, leaf: Leaf, visible: bool) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(Visible(visible));
        }
    }

    /// How opaque `leaf` was told to be, before its ancestry is taken into account.
    pub(crate) fn opacity(&self, leaf: Leaf) -> Opacity {
        self.read::<Opacity>(leaf).unwrap_or_default()
    }

    pub(crate) fn set_opacity(&mut self, leaf: Leaf, opacity: f32) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(Opacity::new(opacity));
        }
    }

    /// Whether `leaf` was disabled in its own right.
    pub(crate) fn disabled(&self, leaf: Leaf) -> Disabled {
        self.read::<Disabled>(leaf).unwrap_or_default()
    }

    pub(crate) fn set_disabled(&mut self, leaf: Leaf, disabled: bool) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(Disabled(disabled));
        }
    }

    /// What the three off-states resolved to over `leaf`'s whole ancestry, as R7 last computed it.
    pub(crate) fn inherited(&self, leaf: Leaf) -> Inherited {
        self.read::<Inherited>(leaf).unwrap_or_default()
    }

    pub(crate) fn set_inherited(&mut self, leaf: Leaf, inherited: Inherited) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(inherited);
        }
    }

    fn read<C: Component + Copy>(&self, leaf: Leaf) -> Option<C> {
        self.world.get_entity(leaf.0).ok()?.get::<C>().copied()
    }
}
