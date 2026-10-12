use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::{Component, Mutable};
use bevy_ecs::entity::RemoteAllocator;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::query::{Has, QueryState};
use bevy_ecs::world::World;

use crate::coordinate::{Area, Position};
use crate::elevation::Elevation;
use crate::elm::{Chlorophyll, Outline, PanelPigment, Pigment};
use crate::icon::{Field, IconPigment};
use crate::image::{Fit, ImagePigment, Plate};
use crate::interaction::Gestures;
use crate::keyboard::Keypad;
use crate::leaf::{Grown, Growth, Leaf, Leaves, Named, Presence, SpawnedAt};
use crate::lifecycle::{Disabled, Opacity, Visible};
use crate::line::{LinePigment, Stroke};
use crate::op::Bud;
use crate::palette::Fill;
use crate::place::{Anchored, Caller, Focusing};
use crate::placement::grid::Grid;
use crate::placement::location::Location;
use crate::placement::trace::{Ends, Trace};
use crate::polygon::{PolygonPigment, Shape};
use crate::rounding::Corners;
use crate::rowan::Standing;
use crate::text::font::Typeface;
use crate::text::{Lettering, TextPigment, Tints};
use crate::text_input::{Editing, Masked, Parts, ReadOnly};
use crate::view::{Extent, Floats, Offset, Pinned, Scroll, Scrolls, Shift};

/// The tree itself, seen from the inside.
///
/// Owns the world. The only place raw `bevy_ecs` is touched.
pub(crate) struct Tree {
    world: World,
    /// How each element is placed, fetched from its row in one step rather than looked up one
    /// component at a time. Resolution asks it of every element it resolves, on each axis and again
    /// when measuring, so on a frame that resolves tens of thousands of elements the lookups are
    /// most of what the asking costs. See [`refresh`](Tree::refresh).
    placements: QueryState<(
        Option<&'static Trace>,
        Option<&'static Stroke>,
        Option<&'static Location>,
    )>,
    /// Everything an element's [`Standing`] is read from, fetched the same way: an element grown or
    /// restyled has all of it read, and a frame that grows tens of thousands reads it for each.
    standings: QueryState<Stands>,
    /// Every element written to since resolution last read this.
    ///
    /// A declaration is what resolution reads, so an element nothing has written to resolves to what
    /// it resolved to last frame. A placement, a trace, a grid or a run written back at what the
    /// element already held is not recorded -- the setter compares first -- because resolution would
    /// answer the same box for it, and asking is the cost of the element and of everything resolved
    /// against it.
    touched: Leaves,
    /// Every element written to in a way no pass that places anything reads.
    ///
    /// How an element is filled, rounded, tinted or shaped is read by extraction alone, and whether
    /// it is shown, how opaque it is, whether it is disabled and how far forward it sits are read by
    /// the passes that run over every element anyway. None of it moves a box, so none of it is a
    /// reason to resolve one again: an element here is read afresh and drawn again, and its subtree
    /// is left where it was.
    restyled: Leaves,
    /// Every element whose run was written, or that was grown: what R1 measures again.
    ///
    /// Apart from `touched`, because R1 reads nothing but the element's own run and face. An element
    /// that moved says what it said, in the cell it said it in, and has nothing to measure again.
    lettered: Leaves,
    /// Every element that has to measure again because what is under it went away.
    ///
    /// Separate from `touched`, because the two spread differently: what an element was written to
    /// travels to everything resolved against it, and what an element measures to travels no further
    /// than the measure -- unless the measure moves, which R2m finds out and says so.
    reached: Leaves,
    /// Whether any run stopped being stated -- a run rewritten, or an element carrying one withered
    /// -- which is the only thing that can leave the shaping cache holding what nothing wants.
    restated: bool,
    /// Whether the elements, or the edges between them, changed -- which is what the order is built
    /// from and the only thing that makes it worth building again.
    restructured: bool,
    /// What was grown since resolution last read this, in the order it was grown, and what withered.
    /// Together they are all an order needs to be repaired rather than built again, unless
    /// [`reordered`](Tree::reordered) says an edge moved.
    grown: Vec<Leaf>,
    withered: Vec<Leaf>,
    /// Whether an edge between elements that were already there changed -- an anchor written --
    /// which no repair of the order can follow, so it is built again.
    reordered: bool,
    /// Whether something every element is read against has changed, which is every element written.
    invalidated: bool,
    /// Whether that something is also what a measure across reads -- a breakpoint, which chooses
    /// every placement and every size, or a face, which sizes every cell -- so every element is
    /// measured across again as well as laid out again.
    remeasured: bool,
    /// Whether something every element is drawn against has changed -- the scheme, or an asset --
    /// which is every element drawn again and none of them placed again.
    repainted: bool,
}

/// Everything an element's [`Standing`] is read from, as one row of the world.
type Stands = (
    Option<&'static Chlorophyll>,
    Option<&'static Growth>,
    Option<&'static Gestures>,
    Option<&'static Scrolls>,
    Has<Pinned>,
    Option<&'static Floats>,
    Option<&'static Visible>,
    Option<&'static Opacity>,
    Option<&'static Disabled>,
    Option<&'static Elevation>,
    Option<&'static Shift>,
);

/// How an element is placed, borrowed from its declaration.
#[derive(Clone, Copy)]
pub(crate) enum Placement<'a> {
    /// By a box.
    Boxed(&'a Location),
    /// By two ends, and the weight of the stroke between them.
    Traced(&'a Trace, Stroke),
}

/// What the tree has been written to since resolution last read it.
///
/// Borrowed rather than handed over, so a frame that writes to a great many elements is not also a
/// frame that allocates a set the size of what it wrote.
pub(crate) struct Written<'a> {
    /// The elements whose own declarations were written.
    pub(crate) declared: &'a Leaves,
    /// The elements written to in a way that moves nothing.
    pub(crate) restyled: &'a Leaves,
    /// The elements whose run was written, or that were grown.
    pub(crate) lettered: &'a Leaves,
    /// The elements that have to measure again because what is under them went away.
    pub(crate) measures: &'a Leaves,
    /// Whether the order has to be built again, or repaired.
    pub(crate) restructured: bool,
    /// What was grown, in the order it was grown, and what withered: what repairing it reads.
    pub(crate) grown: &'a [Leaf],
    pub(crate) withered: &'a [Leaf],
    /// Whether it can only be built again: an edge between elements already there moved.
    pub(crate) reordered: bool,
    /// Whether a run stopped being stated, so the shaping cache has to be swept.
    pub(crate) restated: bool,
    /// Whether everything has to be resolved again.
    pub(crate) all: bool,
    /// Whether everything has to be measured across again, as against only laid out again.
    pub(crate) across: bool,
    /// Whether everything has to be drawn again.
    pub(crate) repainted: bool,
}

impl Tree {
    pub(crate) fn new() -> Self {
        let mut world = World::new();
        let placements = QueryState::new(&mut world);
        let standings = QueryState::new(&mut world);
        Self {
            world,
            placements,
            standings,
            touched: Leaves::default(),
            restyled: Leaves::default(),
            lettered: Leaves::default(),
            reached: Leaves::default(),
            restated: true,
            restructured: true,
            grown: Vec::new(),
            withered: Vec::new(),
            // Nothing has been ordered yet, so there is nothing to repair.
            reordered: true,
            // Nothing has resolved yet, so the first frame has everything to do.
            invalidated: true,
            remeasured: true,
            repainted: false,
        }
    }

    /// Records that `leaf`'s declared state was written.
    ///
    /// Every write of something resolution reads passes through here. What resolution *writes* does
    /// not: those go through [`overwrite`](Tree::overwrite), which is the whole of the difference
    /// between a declaration and a resolved value.
    ///
    /// [`aspen`](crate::aspen) states it directly, because a motion is the one thing resolution
    /// reads that is not held on the element at all.
    pub(crate) fn declared(&mut self, leaf: Leaf) {
        self.touched.insert(leaf);
    }

    /// Records that something about how `leaf` is drawn was written, and nothing about where.
    ///
    /// [`aspen`](crate::aspen) states it directly for a fill in motion, which is blended at
    /// extraction and never written to the element.
    pub(crate) fn restyled(&mut self, leaf: Leaf) {
        self.restyled.insert(leaf);
    }

    /// Records that what is under `leaf` changed, so what it measures to has to be found again.
    fn measures(&mut self, leaf: Leaf) {
        self.reached.insert(leaf);
    }

    /// Records that everything has to be resolved again.
    ///
    /// The viewport, the breakpoint and the short-side reading are each read by every placement in
    /// the tree, and none of them belongs to an element that could be marked. A breakpoint can move
    /// what size a run is set at, which leaves the run it was set at unstated.
    ///
    /// The order is not built again: none of this changes which elements there are or what any of
    /// them hangs off.
    pub(crate) fn invalidate(&mut self) {
        self.invalidated = true;
        self.remeasured = true;
        self.restated = true;
    }

    /// Records that everything has to be laid out again, and nothing measured across again.
    ///
    /// The viewport alone, moved inside one breakpoint: every placement reads it, so every element
    /// is written, but no placement and no size is chosen differently for it and no cell is a
    /// different size -- which is all a measure across reads. So every box is resolved again and
    /// what each one measures to across stands, which is what keeps dragging a window's edge from
    /// measuring the whole tree on every frame of the drag.
    pub(crate) fn relay(&mut self) {
        self.invalidated = true;
    }

    /// Records that everything has to be drawn again, and nothing placed again.
    ///
    /// The scheme every role resolves against, and an asset an element draws, are read by extraction
    /// alone -- so a repaint or an arrival is every element's fill written, and no box's.
    pub(crate) fn repaint(&mut self) {
        self.repainted = true;
    }

    /// What has been written to since resolution last read this.
    pub(crate) fn written(&self) -> Written<'_> {
        Written {
            declared: &self.touched,
            restyled: &self.restyled,
            lettered: &self.lettered,
            measures: &self.reached,
            restructured: self.restructured,
            grown: &self.grown,
            withered: &self.withered,
            reordered: self.reordered,
            restated: self.restated,
            all: self.invalidated,
            across: self.remeasured,
            repainted: self.repainted,
        }
    }

    /// Forgets it, once resolution has taken it.
    ///
    /// Called the moment the read is made rather than at the end of the frame, so a write made by a
    /// pass after that point belongs to the next frame -- which is the frame that would resolve it.
    /// The sets keep the room they had, because a frame that writes a great deal is usually followed
    /// by another one.
    pub(crate) fn taken(&mut self) {
        self.touched.clear();
        self.restyled.clear();
        self.lettered.clear();
        self.reached.clear();
        self.restated = false;
        self.restructured = false;
        self.grown.clear();
        self.withered.clear();
        self.reordered = false;
        self.invalidated = false;
        self.remeasured = false;
        self.repainted = false;
    }

    /// The world's entity allocator, in the form that can be carried away from it.
    ///
    /// What [`Naming`](crate::naming::Naming) is built from, so a name minted off the frame is one
    /// this world can still grow.
    pub(crate) fn remote(&self) -> RemoteAllocator {
        self.world.entity_allocator().build_remote_allocator()
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
        mut bud: Bud,
    ) -> bool {
        // Everything an element declares about how it behaves. Each is present on every element, so
        // nothing that reads one has to ask whether the element is the kind of thing that has it.
        // What resolution makes of them is not here: that is held in resolution's own columns
        // ([`Elements`](crate::rowan::Elements)), which is where every pass reads it.
        let manner = bud.placement.manner;
        let common = (
            (
                Grown,
                SpawnedAt(bud.at),
                growth,
                bud.chlorophyll,
                bud.placement.grid.unwrap_or_default(),
                bud.placement.elevation.unwrap_or_default(),
            ),
            (
                manner.gestures,
                manner.focusing,
                manner.visible,
                manner.opacity,
                Disabled::default(),
                Offset::default(),
                Extent::default(),
            ),
        );
        // Spawned with what every element has, what it hangs off, and what its renderer reads, in
        // one bundle. Each insert after the spawn moves the element to another archetype and copies
        // every component it holds, and growing tens of thousands of elements in a frame is mostly
        // those copies when they are made one group at a time. A stroke always carries a trace and
        // a weight, so they go in with its pigment.
        let parent = under.map(|under| ChildOf(under.0));
        // An element placed by its ends has no box placement, and is not given an empty one: it
        // would be held, and carried, and read by nothing, since a trace is what resolution asks
        // for first and a `Location` written onto a stroke is refused.
        let location = match bud.placement.traced {
            Some(_) => None,
            None => Some(bud.placement.location.take().unwrap_or_default()),
        };
        let spawned = match bud.pigment.take() {
            Some(Pigment::Panel(pigment)) => self.spawn(leaf, common, parent, location, pigment),
            Some(Pigment::Text(pigment)) => self.spawn(leaf, common, parent, location, pigment),
            Some(Pigment::Polygon(pigment)) => self.spawn(leaf, common, parent, location, pigment),
            Some(Pigment::Line(pigment)) => {
                match (bud.placement.traced.take(), bud.placement.stroke.take()) {
                    (Some(trace), Some(stroke)) => {
                        self.spawn(leaf, common, parent, location, (pigment, trace, stroke))
                    }
                    (trace, stroke) => {
                        bud.placement.traced = trace;
                        bud.placement.stroke = stroke;
                        self.spawn(leaf, common, parent, location, pigment)
                    }
                }
            }
            Some(Pigment::Icon(pigment)) => self.spawn(leaf, common, parent, location, pigment),
            Some(Pigment::Image(pigment)) => self.spawn(leaf, common, parent, location, pigment),
            None => self.spawn(leaf, common, parent, location, ()),
        };
        if !spawned {
            return false;
        }
        let mut entity = self.world.entity_mut(leaf.0);
        if let Some(scrolls) = manner.scrolls {
            entity.insert(scrolls);
        }
        // Absent unless declared: an element that travels with its region's content and is clipped
        // by it is the ordinary case, and neither is a value to hold.
        if manner.pinned {
            entity.insert(Pinned);
        }
        if let Some(escape) = manner.floats {
            entity.insert(Floats(escape));
        }
        if let Some(lettering) = bud.lettering {
            entity.insert(lettering);
        }
        if let Some(tints) = bud.tints {
            entity.insert(tints);
        }
        // The point-mode placement. Absent on everything placed by a box, which is what makes "has a
        // trace" the one question the resolver asks to tell the two apart.
        if let Some(trace) = bud.placement.traced {
            entity.insert(trace);
        }
        if let Some(stroke) = bud.placement.stroke {
            entity.insert(stroke);
        }
        if let Some(typeface) = bud.placement.typeface {
            entity.insert(typeface);
        }
        if let Some(anchored) = bud.placement.anchor {
            entity.insert(anchored);
        }
        // What it was grown under has one more element to reach over, and finds that out by the
        // walk R2m makes from this one toward it.
        self.declared(leaf);
        self.lettered.insert(leaf);
        self.restructured = true;
        self.grown.push(leaf);
        true
    }

    /// Spawns `leaf` holding `common` and `kind`, hanging off `parent` and placed by `location`
    /// where it has either, reporting whether the name was still free to grow into.
    fn spawn<C: Bundle, K: Bundle>(
        &mut self,
        leaf: Leaf,
        common: C,
        parent: Option<ChildOf>,
        location: Option<Location>,
        kind: K,
    ) -> bool {
        let world = &mut self.world;
        match (parent, location) {
            (Some(parent), Some(location)) => world
                .spawn_at(leaf.0, (common, kind, parent, location))
                .is_ok(),
            (Some(parent), None) => world.spawn_at(leaf.0, (common, kind, parent)).is_ok(),
            (None, Some(location)) => world.spawn_at(leaf.0, (common, kind, location)).is_ok(),
            (None, None) => world.spawn_at(leaf.0, (common, kind)).is_ok(),
        }
    }

    /// Takes each of `leaves` down, and everything beneath each, and reports every name that went.
    ///
    /// Each trunk they hang off has them taken out of its branches in one pass, before any of them
    /// goes. One at a time, each would be searched for among its trunk's branches and everything
    /// grown after it moved down a place -- so letting go of many off one trunk, as a list cut short
    /// does, would cost the trunk's length many times over.
    pub(crate) fn wither(&mut self, leaves: &[Leaf]) -> Vec<Leaf> {
        let mut gone = Vec::new();
        let mut taken = Leaves::default();
        let mut trunks: Named<Leaf, Leaves> = Named::default();
        for &leaf in leaves {
            // Already going, under one of the others.
            if taken.contains(&leaf) {
                continue;
            }
            let from = gone.len();
            self.gather(leaf, &mut gone);
            taken.extend(gone[from..].iter().copied());
            if let Some(trunk) = self.trunk(leaf) {
                trunks.entry(trunk).or_default().insert(leaf);
            }
        }
        for (trunk, going) in trunks {
            if !taken.contains(&trunk) {
                let staying: Vec<_> = self
                    .world
                    .get::<Children>(trunk.0)
                    .into_iter()
                    .flat_map(|children| children.iter().copied())
                    .filter(|child| !going.contains(&Leaf(*child)))
                    .collect();
                self.world
                    .entity_mut(trunk.0)
                    .replace_related::<ChildOf>(&staying);
            }
            // What it hung off has one fewer element to reach over, and nothing left below it to
            // say so: the element that went is what the walk would have started from.
            self.measures(trunk);
        }
        for leaf in gone.iter().rev() {
            if let Ok(entity) = self.world.get_entity_mut(leaf.0) {
                entity.despawn();
            }
        }
        self.restructured = true;
        self.withered.extend_from_slice(&gone);
        // Whatever it said, it does not say any more.
        self.restated = true;
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
        self.branched(leaf).collect()
    }

    /// The same, borrowed rather than collected.
    ///
    /// Resolution asks this of every element on more than one of its passes, so the vector
    /// [`branches`](Tree::branches) hands back would be an allocation per element per pass.
    pub(crate) fn branched(&self, leaf: Leaf) -> impl Iterator<Item = Leaf> + '_ {
        self.world
            .get_entity(leaf.0)
            .ok()
            .and_then(|entity| entity.get::<Children>())
            .into_iter()
            .flat_map(|children| children.iter().copied())
            .filter(|child| {
                self.world
                    .get_entity(*child)
                    .is_ok_and(|child| child.contains::<Grown>())
            })
            .map(Leaf)
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

    /// How `leaf` is placed: by its two ends and the weight between them, or by a box. `None` for
    /// an element that is not live.
    ///
    /// One read of the element for the question every pass that resolves geometry asks first, and
    /// the declaration it goes on to read -- which a pass resolving every element on two axes would
    /// otherwise ask the world for separately.
    pub(crate) fn placement(&self, leaf: Leaf) -> Option<Placement<'_>> {
        let (trace, stroke, location) = match self.placements.get_manual(&self.world, leaf.0) {
            Ok(row) => row,
            // Grown into a shape the query has not been shown yet, since the last refresh. Asked
            // of the element itself, which is always right and only slower.
            Err(_) => {
                let entity = self.world.get_entity(leaf.0).ok()?;
                (
                    entity.get::<Trace>(),
                    entity.get::<Stroke>(),
                    entity.get::<Location>(),
                )
            }
        };
        match trace {
            Some(trace) => Some(Placement::Traced(
                trace,
                stroke.copied().unwrap_or_default(),
            )),
            None => location.map(Placement::Boxed),
        }
    }

    /// Shows [`placement`](Tree::placement)'s query every shape an element has been grown in since
    /// it was last shown, so every element is fetched in one step. Before resolution, which is
    /// after everything this frame grows has been grown.
    pub(crate) fn refresh(&mut self) {
        self.placements.update_archetypes(&self.world);
        self.standings.update_archetypes(&self.world);
    }

    pub(crate) fn grid(&self, leaf: Leaf) -> Option<Grid> {
        self.world.get_entity(leaf.0).ok()?.get::<Grid>().cloned()
    }

    /// Which font `leaf` composes in and at what size, or `None` if it was never given one.
    pub(crate) fn typeface(&self, leaf: Leaf) -> Option<Typeface> {
        self.read::<Typeface>(leaf)
    }

    /// What `leaf` says, if it is a run of glyphs.
    pub(crate) fn lettering(&self, leaf: Leaf) -> Option<&str> {
        Some(
            self.world
                .get_entity(leaf.0)
                .ok()?
                .get::<Lettering>()?
                .0
                .as_str(),
        )
    }

    /// Rewrites what `leaf` says, reporting whether it is something with a run to write.
    pub(crate) fn set_lettering(&mut self, leaf: Leaf, value: String) -> bool {
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return false;
        };
        let Some(mut lettering) = entity.get_mut::<Lettering>() else {
            return false;
        };
        // The same words again are the same run, measured and drawn as it already is.
        if lettering.0 == value {
            return true;
        }
        lettering.0 = value;
        self.declared(leaf);
        self.lettered.insert(leaf);
        // The run it stated is not stated any more, whatever else still states one like it.
        self.restated = true;
        true
    }

    /// The four parts of `leaf`, if it is a [`TextInput`](crate::TextInput).
    ///
    /// The one question that tells a field from anything else, so every verb addressed to a field
    /// asks it first.
    pub(crate) fn parts(&self, leaf: Leaf) -> Option<Parts> {
        self.read::<Parts>(leaf)
    }

    pub(crate) fn set_parts(&mut self, leaf: Leaf, parts: Parts) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(parts);
        }
    }

    /// Every field, with what each is made of.
    pub(crate) fn fields(&mut self) -> Vec<(Leaf, Parts)> {
        self.world
            .query::<(bevy_ecs::entity::Entity, &Parts)>()
            .iter(&self.world)
            .map(|(entity, parts)| (Leaf(entity), *parts))
            .collect()
    }

    /// Which soft keyboard `leaf` asks for, if it is something that is typed into at all.
    ///
    /// `None` for everything but a field, which is what makes focus alone decide the keyboard: a
    /// button can hold focus and has no keypad, so nothing is raised for it.
    pub(crate) fn keypad(&self, leaf: Leaf) -> Option<Keypad> {
        self.read::<Keypad>(leaf)
    }

    pub(crate) fn set_keypad(&mut self, leaf: Leaf, keypad: Keypad) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(keypad);
        }
    }

    /// Whether `leaf` is a field that refuses the keystrokes that would change its value. `false`
    /// for everything that is not a field, which has no value to keep.
    pub(crate) fn read_only(&self, leaf: Leaf) -> bool {
        self.read::<ReadOnly>(leaf)
            .is_some_and(|read_only| read_only.0)
    }

    pub(crate) fn set_read_only(&mut self, leaf: Leaf, read_only: bool) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(ReadOnly(read_only));
        }
    }

    /// Whether `leaf` is a run drawn as dots. `false` for everything but a masked field's run.
    pub(crate) fn masked(&self, leaf: Leaf) -> bool {
        self.read::<Masked>(leaf).is_some_and(|masked| masked.0)
    }

    /// Masks a run or shows it, and has it shaped again if that changed what it draws -- which is
    /// the same statement a new value is, so it is marked the way [`set_lettering`](Self::set_lettering)
    /// marks one.
    pub(crate) fn set_masked(&mut self, leaf: Leaf, masked: bool) {
        if self.masked(leaf) == masked {
            if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
                entity.insert(Masked(masked));
            }
            return;
        }
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return;
        };
        entity.insert(Masked(masked));
        self.declared(leaf);
        self.lettered.insert(leaf);
        self.restated = true;
    }

    /// Where `leaf`'s caret is and what it has selected.
    pub(crate) fn editing(&self, leaf: Leaf) -> Editing {
        self.read::<Editing>(leaf).unwrap_or_default()
    }

    pub(crate) fn set_editing(&mut self, leaf: Leaf, editing: Editing) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(editing);
        }
        self.declared(leaf);
    }

    /// The element `leaf`'s placement may read, if it has been given one.
    pub(crate) fn anchor(&self, leaf: Leaf) -> Option<Leaf> {
        Some(self.world.get_entity(leaf.0).ok()?.get::<Anchored>()?.to)
    }

    /// Where `leaf` was written into existence.
    pub(crate) fn spawned_at(&self, leaf: Leaf) -> Option<Caller> {
        Some(self.world.get_entity(leaf.0).ok()?.get::<SpawnedAt>()?.0)
    }

    /// Whether `leaf` is grown somewhere under `trunk`, however deep.
    ///
    /// What [`ScrollTo::show`](crate::ScrollTo::show) is asked, because bringing an element into
    /// view means nothing unless the region is what it is inside.
    pub(crate) fn grown_under(&self, leaf: Leaf, trunk: Leaf) -> bool {
        let mut step = self.trunk(leaf);
        while let Some(above) = step {
            if above == trunk {
                return true;
            }
            step = self.trunk(above);
        }
        false
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

    /// Moves `leaf`, reporting whether it is something with a box to place.
    ///
    /// An element placed by its ends has none, and refuses the write rather than taking a
    /// declaration nothing would read: the resolver asks for a trace first, so a `Location` written
    /// onto a stroke would sit there being ignored.
    pub(crate) fn set_location(&mut self, leaf: Leaf, location: Location) -> bool {
        let Ok(entity) = self.world.get_entity(leaf.0) else {
            return false;
        };
        if entity.contains::<Trace>() {
            return false;
        }
        // Written back at what it already holds is no write at all. Resolution would answer the
        // same box, and asking it to is the whole cost of the element and everything under it.
        if self.overwrite(leaf, location) {
            self.declared(leaf);
        }
        true
    }

    pub(crate) fn set_grid(&mut self, leaf: Leaf, grid: Grid) {
        if self.overwrite(leaf, grid) {
            self.declared(leaf);
        }
    }

    pub(crate) fn set_anchor(&mut self, leaf: Leaf, to: Leaf, at: Caller) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(Anchored { to, at });
        }
        self.declared(leaf);
        // An anchor is an edge of the graph the order is built from, between elements already there.
        self.restructured = true;
        self.reordered = true;
    }

    /// How far in front of its trunk `leaf` was told to sit.
    pub(crate) fn elevation(&self, leaf: Leaf) -> Elevation {
        self.read::<Elevation>(leaf).unwrap_or_default()
    }

    /// Where `leaf` came in allocation order.
    pub(crate) fn growth(&self, leaf: Leaf) -> Growth {
        self.read::<Growth>(leaf).unwrap_or_default()
    }

    pub(crate) fn set_elevation(&mut self, leaf: Leaf, elevation: Elevation) {
        if self.overwrite(leaf, elevation) {
            self.restyled(leaf);
        }
    }

    /// What the panel renderer on `leaf` was told, or `None` if `leaf` is not a panel.
    pub(crate) fn panel_pigment(&self, leaf: Leaf) -> Option<PanelPigment> {
        self.read::<PanelPigment>(leaf)
    }

    /// What the text renderer on `leaf` was told, or `None` if `leaf` is not a run.
    pub(crate) fn text_pigment(&self, leaf: Leaf) -> Option<TextPigment> {
        self.read::<TextPigment>(leaf)
    }

    /// What the polygon renderer on `leaf` was told, or `None` if `leaf` is not a polygon.
    pub(crate) fn polygon_pigment(&self, leaf: Leaf) -> Option<PolygonPigment> {
        self.read::<PolygonPigment>(leaf)
    }

    /// What the line renderer on `leaf` was told, or `None` if `leaf` is not a stroke.
    pub(crate) fn line_pigment(&self, leaf: Leaf) -> Option<LinePigment> {
        self.read::<LinePigment>(leaf)
    }

    /// What the icon renderer on `leaf` was told, or `None` if `leaf` is not a mark.
    pub(crate) fn icon_pigment(&self, leaf: Leaf) -> Option<IconPigment> {
        self.read::<IconPigment>(leaf)
    }

    /// What the image renderer on `leaf` was told, or `None` if `leaf` is not a picture.
    pub(crate) fn image_pigment(&self, leaf: Leaf) -> Option<ImagePigment> {
        self.read::<ImagePigment>(leaf)
    }

    /// Where `leaf`'s two ends are declared to be, per breakpoint, or `None` if it is placed by a
    /// box.
    ///
    /// The one question that says which of the two placements an element states, which is why every
    /// pass that resolves geometry asks it first.
    pub(crate) fn trace(&self, leaf: Leaf) -> Option<&Trace> {
        self.world.get_entity(leaf.0).ok()?.get::<Trace>()
    }

    /// Moves `leaf`'s two ends, reporting whether it is something placed by ends at all.
    ///
    /// A box-placed element has none, and refuses the write for the reason
    /// [`set_location`](Tree::set_location) refuses the other: a trace on a box would sit there
    /// being ignored.
    pub(crate) fn set_trace(&mut self, leaf: Leaf, trace: Trace) -> bool {
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return false;
        };
        let Some(mut held) = entity.get_mut::<Trace>() else {
            return false;
        };
        if *held != trace {
            *held = trace;
            self.declared(leaf);
        }
        true
    }

    /// Writes both of `leaf`'s ends at every breakpoint into the trace it holds, reporting whether
    /// it is something placed by its ends. Ends it already holds, and nothing else, are no write.
    pub(crate) fn set_ends(&mut self, leaf: Leaf, ends: Ends) -> bool {
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return false;
        };
        let Some(mut held) = entity.get_mut::<Trace>() else {
            return false;
        };
        if held.0.only() != Some(&ends) {
            held.0.reset(ends);
            self.declared(leaf);
        }
        true
    }

    /// Writes how thick `leaf` is stroked, reporting whether it is a stroke. The same weight is no
    /// write.
    pub(crate) fn set_stroke(&mut self, leaf: Leaf, stroke: Stroke) -> bool {
        let Ok(entity) = self.world.get_entity(leaf.0) else {
            return false;
        };
        if !entity.contains::<Stroke>() {
            return false;
        }
        if self.overwrite(leaf, stroke) {
            self.declared(leaf);
        }
        true
    }

    /// How thick `leaf` is stroked, or `None` if it is not a stroke.
    pub(crate) fn stroke(&self, leaf: Leaf) -> Option<Stroke> {
        self.read::<Stroke>(leaf)
    }

    /// How parts of `leaf`'s run are filled differently from the rest of it.
    pub(crate) fn tints(&self, leaf: Leaf) -> Option<&Tints> {
        self.world.get_entity(leaf.0).ok()?.get::<Tints>()
    }

    /// Refills parts of `leaf`'s run, reporting whether it is something with a run to tint.
    ///
    /// Replaces every tint rather than adding one, for the reason a placement is one value: there is
    /// no half-written state between two of these and no question of which range a later write meant.
    pub(crate) fn set_tints(&mut self, leaf: Leaf, tints: Tints) -> bool {
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return false;
        };
        if !entity.contains::<Lettering>() {
            return false;
        }
        entity.insert(tints);
        self.restyled(leaf);
        true
    }

    /// Reshapes `leaf`, reporting whether it is something with a shape to reshape.
    pub(crate) fn set_shape(&mut self, leaf: Leaf, shape: Shape) -> bool {
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return false;
        };
        let Some(mut pigment) = entity.get_mut::<PolygonPigment>() else {
            return false;
        };
        pigment.shape = shape;
        self.restyled(leaf);
        true
    }

    /// Swaps the mark `leaf` draws, reporting whether it is an icon at all.
    pub(crate) fn set_mark(&mut self, leaf: Leaf, field: Field) -> bool {
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return false;
        };
        let Some(mut pigment) = entity.get_mut::<IconPigment>() else {
            return false;
        };
        pigment.field = field;
        self.restyled(leaf);
        true
    }

    /// Swaps the picture `leaf` draws, reporting whether it is an image at all.
    pub(crate) fn set_plate(&mut self, leaf: Leaf, plate: Plate) -> bool {
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return false;
        };
        let Some(mut pigment) = entity.get_mut::<ImagePigment>() else {
            return false;
        };
        pigment.plate = plate;
        self.restyled(leaf);
        true
    }

    /// Changes how `leaf`'s pixels are fitted into its box, reporting whether it is an image at all.
    pub(crate) fn set_fit(&mut self, leaf: Leaf, fit: Fit) -> bool {
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return false;
        };
        let Some(mut pigment) = entity.get_mut::<ImagePigment>() else {
            return false;
        };
        pigment.fit = fit;
        self.restyled(leaf);
        true
    }

    /// What `leaf` is shaped as, or `None` if it is not a polygon.
    pub(crate) fn shape(&self, leaf: Leaf) -> Option<Shape> {
        Some(self.read::<PolygonPigment>(leaf)?.shape)
    }

    /// What `leaf` is filled with, whichever renderer holds the fill, or `None` if it has none.
    ///
    /// One question across the renderers, because a fill is one property: the same
    /// [`color`](crate::Grow::color) refills a panel, a run, a shape, a stroke and a mark, and the
    /// same motion moves any of them. An [`Image`](crate::Image) is the one element with nothing to
    /// answer -- it carries its own colour, and a fill would be a second opinion about it.
    pub(crate) fn fill(&self, leaf: Leaf) -> Option<Fill> {
        if let Some(pigment) = self.read::<PanelPigment>(leaf) {
            return Some(pigment.fill);
        }
        if let Some(pigment) = self.read::<TextPigment>(leaf) {
            return Some(pigment.fill);
        }
        if let Some(pigment) = self.read::<PolygonPigment>(leaf) {
            return Some(pigment.fill);
        }
        if let Some(pigment) = self.read::<LinePigment>(leaf) {
            return Some(pigment.fill);
        }
        Some(self.read::<IconPigment>(leaf)?.fill)
    }

    /// Refills `leaf`, reporting whether it is something with a fill to write.
    pub(crate) fn set_fill(&mut self, leaf: Leaf, fill: Fill) -> bool {
        let filled = self.filled(leaf, fill);
        if filled {
            self.restyled(leaf);
        }
        filled
    }

    fn filled(&mut self, leaf: Leaf, fill: Fill) -> bool {
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return false;
        };
        if let Some(mut pigment) = entity.get_mut::<PanelPigment>() {
            pigment.fill = fill;
            return true;
        }
        if let Some(mut pigment) = entity.get_mut::<TextPigment>() {
            pigment.fill = fill;
            return true;
        }
        if let Some(mut pigment) = entity.get_mut::<PolygonPigment>() {
            pigment.fill = fill;
            return true;
        }
        if let Some(mut pigment) = entity.get_mut::<LinePigment>() {
            pigment.fill = fill;
            return true;
        }
        if let Some(mut pigment) = entity.get_mut::<IconPigment>() {
            pigment.fill = fill;
            return true;
        }
        false
    }

    /// The line drawn around the inside of `leaf`, or `None` if it is not a panel.
    pub(crate) fn outline(&self, leaf: Leaf) -> Option<Outline> {
        Some(self.read::<PanelPigment>(leaf)?.outline)
    }

    /// Outlines `leaf`, reporting whether it is a panel.
    ///
    /// Read by extraction alone, so it is a restyle: the panel is drawn again and nothing is placed
    /// again. A panel and nothing else, because an outline is drawn by the field that rounds a
    /// panel, and the other renderers have no such field to draw one with.
    pub(crate) fn set_outline(&mut self, leaf: Leaf, outline: Outline) -> bool {
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return false;
        };
        let Some(mut pigment) = entity.get_mut::<PanelPigment>() else {
            return false;
        };
        pigment.outline = outline;
        self.restyled(leaf);
        true
    }

    /// How `leaf`'s corners are rounded, or `None` if it has no box to round.
    pub(crate) fn rounding(&self, leaf: Leaf) -> Option<Corners> {
        if let Some(pigment) = self.read::<PanelPigment>(leaf) {
            return Some(pigment.rounding);
        }
        Some(self.read::<ImagePigment>(leaf)?.rounding)
    }

    /// Rounds `leaf`'s corners, reporting whether it is something with corners to round.
    ///
    /// The two elements that are a rectangle: a panel and a picture, which round through the same
    /// field so a full-bleed picture sits flush inside a rounded card. A run of glyphs, a stroke and
    /// a regular polygon have no rectangle of their own -- a polygon's corners are its own, and are
    /// [`Shape::rounding`](crate::Shape::rounding) -- so an op naming one is dropped like any other
    /// that named something it does not apply to.
    pub(crate) fn set_rounding(&mut self, leaf: Leaf, rounding: Corners) -> bool {
        let rounded = self.rounded(leaf, rounding);
        if rounded {
            self.restyled(leaf);
        }
        rounded
    }

    fn rounded(&mut self, leaf: Leaf, rounding: Corners) -> bool {
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return false;
        };
        if let Some(mut pigment) = entity.get_mut::<PanelPigment>() {
            pigment.rounding = rounding;
            return true;
        }
        if let Some(mut pigment) = entity.get_mut::<ImagePigment>() {
            pigment.rounding = rounding;
            return true;
        }
        false
    }

    /// What `leaf` declared about gestures.
    pub(crate) fn gestures(&self, leaf: Leaf) -> Gestures {
        self.read::<Gestures>(leaf).unwrap_or_default()
    }

    /// What `leaf` declared about scrolling, or `None` if it does not scroll.
    pub(crate) fn scrolls(&self, leaf: Leaf) -> Option<Scroll> {
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
        self.overwrite(leaf, Offset(offset));
    }

    /// How far `leaf`'s content reaches, as R3 last measured it.
    pub(crate) fn extent(&self, leaf: Leaf) -> Area {
        self.read::<Extent>(leaf).unwrap_or_default().0
    }

    pub(crate) fn set_extent(&mut self, leaf: Leaf, extent: Area) {
        self.overwrite(leaf, Extent(extent));
    }

    /// Whether the app has hidden `leaf` itself, as against an ancestor of it.
    pub(crate) fn visible(&self, leaf: Leaf) -> Visible {
        self.read::<Visible>(leaf).unwrap_or_default()
    }

    pub(crate) fn set_visible(&mut self, leaf: Leaf, visible: bool) {
        if self.overwrite(leaf, Visible(visible)) {
            self.restyled(leaf);
        }
    }

    /// How opaque `leaf` was told to be, before its ancestry is taken into account.
    pub(crate) fn opacity(&self, leaf: Leaf) -> Opacity {
        self.read::<Opacity>(leaf).unwrap_or_default()
    }

    pub(crate) fn set_opacity(&mut self, leaf: Leaf, opacity: f32) {
        if self.overwrite(leaf, Opacity::new(opacity)) {
            self.restyled(leaf);
        }
    }

    /// Whether `leaf` was disabled in its own right.
    pub(crate) fn disabled(&self, leaf: Leaf) -> Disabled {
        self.read::<Disabled>(leaf).unwrap_or_default()
    }

    pub(crate) fn set_disabled(&mut self, leaf: Leaf, disabled: bool) {
        if self.overwrite(leaf, Disabled(disabled)) {
            self.restyled(leaf);
        }
    }

    /// How far `leaf` is drawn from where the layout put it.
    pub(crate) fn shift(&self, leaf: Leaf) -> Position {
        self.read::<Shift>(leaf).unwrap_or_default().0
    }

    pub(crate) fn set_shift(&mut self, leaf: Leaf, by: Position) {
        if self.overwrite(leaf, Shift(by)) {
            self.restyled(leaf);
        }
    }

    /// Everything the passes after the axes read about `leaf`, in one read of the element.
    ///
    /// What resolution holds beside the order for every element, and reads again only where one of
    /// these was written -- so the passes that run over every element on every frame index a column
    /// rather than look each of these up.
    pub(crate) fn standing(&self, leaf: Leaf) -> Standing {
        let row = match self.standings.get_manual(&self.world, leaf.0) {
            Ok(row) => row,
            // A shape the query has not been shown since the last refresh. Asked of the element
            // itself, which is always right and only slower.
            Err(_) => {
                let Ok(entity) = self.world.get_entity(leaf.0) else {
                    return Standing::default();
                };
                (
                    entity.get::<Chlorophyll>(),
                    entity.get::<Growth>(),
                    entity.get::<Gestures>(),
                    entity.get::<Scrolls>(),
                    entity.contains::<Pinned>(),
                    entity.get::<Floats>(),
                    entity.get::<Visible>(),
                    entity.get::<Opacity>(),
                    entity.get::<Disabled>(),
                    entity.get::<Elevation>(),
                    entity.get::<Shift>(),
                )
            }
        };
        let (
            chlorophyll,
            growth,
            gestures,
            scrolls,
            pinned,
            floats,
            visible,
            opacity,
            disabled,
            elevation,
            shift,
        ) = row;
        Standing {
            chlorophyll: chlorophyll.copied().unwrap_or_default(),
            growth: growth.map_or(0, |growth| growth.0),
            gestures: gestures.copied().unwrap_or_default(),
            scrolls: scrolls.map(|scrolls| scrolls.0),
            pinned,
            floats: floats.map(|floats| floats.0),
            visible: visible.copied().unwrap_or_default().0,
            opacity: opacity.copied().unwrap_or_default().0,
            disabled: disabled.copied().unwrap_or_default().0,
            elevation: elevation.copied().unwrap_or_default(),
            shift: shift.copied().unwrap_or_default().0,
        }
    }

    /// Writes a component that is already there in place, inserting it only the first time, and
    /// reports whether what it held changed.
    ///
    /// `insert` goes through the bundle machinery whatever it is handed, which is what a component
    /// arriving for the first time needs and what one being overwritten does not. And a write of
    /// what is already held is reported as none, which is what lets a setter mark an element only
    /// when something about it actually changed.
    fn overwrite<C: Component<Mutability = Mutable> + PartialEq>(
        &mut self,
        leaf: Leaf,
        value: C,
    ) -> bool {
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return false;
        };
        match entity.get_mut::<C>() {
            Some(mut held) => {
                if *held == value {
                    return false;
                }
                *held = value;
            }
            None => {
                entity.insert(value);
            }
        }
        true
    }

    fn read<C: Component + Copy>(&self, leaf: Leaf) -> Option<C> {
        self.world.get_entity(leaf.0).ok()?.get::<C>().copied()
    }
}
