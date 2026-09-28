use bevy_ecs::component::{Component, Mutable};
use bevy_ecs::entity::RemoteAllocator;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::world::World;

use crate::coordinate::{Area, Position};
use crate::elevation::Elevation;
use crate::elm::{Chlorophyll, PanelPigment, Pigment};
use crate::icon::{Field, IconPigment};
use crate::image::{Fit, ImagePigment, Plate};
use crate::interaction::Gestures;
use crate::keyboard::Keypad;
use crate::leaf::{Grown, Growth, Leaf, Leaves, Presence, SpawnedAt};
use crate::lifecycle::{Disabled, Opacity, Visible};
use crate::line::{LinePigment, Stroke};
use crate::op::Bud;
use crate::palette::Fill;
use crate::place::{Anchored, Caller, Focusing};
use crate::placement::grid::Grid;
use crate::placement::location::Location;
use crate::placement::trace::Trace;
use crate::polygon::{PolygonPigment, Shape};
use crate::rounding::Corners;
use crate::rowan::Standing;
use crate::text::font::Typeface;
use crate::text::{Lettering, TextPigment, Tints};
use crate::text_input::{Editing, Masked, Parts, ReadOnly};
use crate::view::{Extent, Floats, Offset, Pinned, Scroll, Scrolls};

/// The tree itself, seen from the inside.
///
/// Owns the world. The only place raw `bevy_ecs` is touched.
pub(crate) struct Tree {
    world: World,
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
    /// Whether something every element is read against has changed, which is every element written.
    invalidated: bool,
    /// Whether something every element is drawn against has changed -- the scheme, or an asset --
    /// which is every element drawn again and none of them placed again.
    repainted: bool,
}

/// How an element is placed, borrowed from its declaration.
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
    /// Whether the order has to be built again.
    pub(crate) restructured: bool,
    /// Whether a run stopped being stated, so the shaping cache has to be swept.
    pub(crate) restated: bool,
    /// Whether everything has to be resolved again.
    pub(crate) all: bool,
    /// Whether everything has to be drawn again.
    pub(crate) repainted: bool,
}

impl Tree {
    pub(crate) fn new() -> Self {
        Self {
            world: World::new(),
            touched: Leaves::default(),
            restyled: Leaves::default(),
            lettered: Leaves::default(),
            reached: Leaves::default(),
            restated: true,
            restructured: true,
            // Nothing has resolved yet, so the first frame has everything to do.
            invalidated: true,
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
        self.restated = true;
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
            restated: self.restated,
            all: self.invalidated,
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
        self.invalidated = false;
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
        ));
        // Everything an element declares about how it behaves. Each is present on every element, so
        // nothing that reads one has to ask whether the element is the kind of thing that has it.
        // What resolution makes of them is not here: that is held in resolution's own columns
        // ([`Elements`](crate::rowan::Elements)), which is where every pass reads it.
        let manner = bud.placement.manner;
        entity.insert((
            manner.gestures,
            manner.focusing,
            manner.visible,
            manner.opacity,
            Disabled::default(),
            Offset::default(),
            Extent::default(),
        ));
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
        match bud.pigment {
            Some(Pigment::Panel(pigment)) => {
                entity.insert(pigment);
            }
            Some(Pigment::Text(pigment)) => {
                entity.insert(pigment);
            }
            Some(Pigment::Polygon(pigment)) => {
                entity.insert(pigment);
            }
            Some(Pigment::Line(pigment)) => {
                entity.insert(pigment);
            }
            Some(Pigment::Icon(pigment)) => {
                entity.insert(pigment);
            }
            Some(Pigment::Image(pigment)) => {
                entity.insert(pigment);
            }
            None => {}
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
        if let Some(under) = under {
            entity.insert(ChildOf(under.0));
        }
        // What it was grown under has one more element to reach over, and finds that out by the
        // walk R2m makes from this one toward it.
        self.declared(leaf);
        self.lettered.insert(leaf);
        self.restructured = true;
        true
    }

    /// Takes `leaf` and everything beneath it down, and reports every name that went.
    pub(crate) fn wither(&mut self, leaf: Leaf) -> Vec<Leaf> {
        let mut gone = Vec::new();
        self.gather(leaf, &mut gone);
        let trunk = self.trunk(leaf);
        if let Ok(entity) = self.world.get_entity_mut(leaf.0) {
            entity.despawn();
        }
        // What it hung off has one fewer element to reach over, and nothing left below it to say so:
        // the element that went is what the walk would have started from.
        if let Some(trunk) = trunk {
            self.measures(trunk);
        }
        self.restructured = true;
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
        let entity = self.world.get_entity(leaf.0).ok()?;
        match entity.get::<Trace>() {
            Some(trace) => Some(Placement::Traced(
                trace,
                entity.get::<Stroke>().copied().unwrap_or_default(),
            )),
            None => entity.get::<Location>().map(Placement::Boxed),
        }
    }

    pub(crate) fn grid(&self, leaf: Leaf) -> Option<Grid> {
        self.world.get_entity(leaf.0).ok()?.get::<Grid>().copied()
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
        self.read::<ReadOnly>(leaf).is_some_and(|read_only| read_only.0)
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
        // An anchor is an edge of the graph the order is built from.
        self.restructured = true;
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

    /// Everything the passes after the axes read about `leaf`, in one read of the element.
    ///
    /// What resolution holds beside the order for every element, and reads again only where one of
    /// these was written -- so the passes that run over every element on every frame index a column
    /// rather than look each of these up.
    pub(crate) fn standing(&self, leaf: Leaf) -> Standing {
        let Ok(entity) = self.world.get_entity(leaf.0) else {
            return Standing::default();
        };
        Standing {
            chlorophyll: entity.get::<Chlorophyll>().copied().unwrap_or_default(),
            growth: entity.get::<Growth>().map_or(0, |growth| growth.0),
            gestures: entity.get::<Gestures>().copied().unwrap_or_default(),
            scrolls: entity.get::<Scrolls>().map(|scrolls| scrolls.0),
            pinned: entity.contains::<Pinned>(),
            floats: entity.get::<Floats>().map(|floats| floats.0),
            visible: entity.get::<Visible>().copied().unwrap_or_default().0,
            opacity: entity.get::<Opacity>().copied().unwrap_or_default().0,
            disabled: entity.get::<Disabled>().copied().unwrap_or_default().0,
            elevation: entity.get::<Elevation>().copied().unwrap_or_default(),
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
