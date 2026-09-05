use std::sync::atomic::{AtomicU64, Ordering};

use bevy_ecs::component::Component;
use bevy_ecs::entity::RemoteAllocator;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::world::World;

use crate::coordinate::{Area, Position, Section};
use crate::elevation::{Elevation, ResolvedElevation};
use crate::elm::{Chlorophyll, PanelPigment, Pigment};
use crate::icon::IconPigment;
use crate::image::ImagePigment;
use crate::interaction::Gestures;
use crate::leaf::{Grown, Growth, Leaf, Presence, SpawnedAt};
use crate::lifecycle::{Disabled, Inherited, Opacity, Visible};
use crate::line::{LinePigment, Stretched, Stroke, Traced};
use crate::op::Bud;
use crate::palette::Fill;
use crate::place::{Anchored, Caller, Focusing};
use crate::placement::grid::Grid;
use crate::placement::location::Location;
use crate::placement::point::Point;
use crate::polygon::{PolygonPigment, Shape};
use crate::rounding::Corners;
use crate::rowan::{Cell, Drawn, Intrinsic, Placed};
use crate::text::font::Typeface;
use crate::text::{Lettering, TextPigment, Tints};
use crate::keyboard::Keypad;
use crate::text_input::{Editing, Parts};
use crate::view::{Clipped, Escape, Extent, Floats, Offset, Pinned, Scroll, Scrolls};

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
            // What R1 and R2m write. Present on every element, so no measuring pass has to ask
            // whether the element is the kind of thing that has one.
            Cell::default(),
            Intrinsic::default(),
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
        // The point-mode placement, and the resolved geometry the passes write back into. Both are
        // absent on everything placed by a box, which is what makes "has a trace" the one question
        // the resolver asks to tell the two apart.
        if let Some(traced) = bud.placement.traced {
            entity.insert((traced, Stretched::default()));
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

    /// The character cell of `leaf`'s own font, at its own size, as R1 measured it.
    ///
    /// Per element rather than per engine: an app registers as many fonts as it likes and each
    /// element chooses, so `8.letters()` is eight cells of *that* element's font. An element that
    /// has not been given one has no cell.
    pub(crate) fn cell(&self, leaf: Leaf) -> Area {
        self.read::<Cell>(leaf).unwrap_or_default().0
    }

    /// What `leaf` measured to: max-content across, and the height it wrapped to down.
    ///
    /// What [`content()`](crate::content) reads, of the element itself or of one it names. R1 writes
    /// the width and R2m the height, which is the whole of width-down and height-up. An element with
    /// nothing in it measures to zero.
    pub(crate) fn intrinsic(&self, leaf: Leaf) -> Area {
        self.read::<Intrinsic>(leaf).unwrap_or_default().0
    }

    pub(crate) fn set_cell(&mut self, leaf: Leaf, cell: Area) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(Cell(cell));
        }
    }

    pub(crate) fn set_intrinsic(&mut self, leaf: Leaf, intrinsic: Area) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(Intrinsic(intrinsic));
        }
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
        lettering.0 = value;
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

    /// Where `leaf`'s caret is and what it has selected.
    pub(crate) fn editing(&self, leaf: Leaf) -> Editing {
        self.read::<Editing>(leaf).unwrap_or_default()
    }

    pub(crate) fn set_editing(&mut self, leaf: Leaf, editing: Editing) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(editing);
        }
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
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return false;
        };
        if entity.contains::<Traced>() {
            return false;
        }
        entity.insert(location);
        true
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

    /// Where `leaf`'s two ends are declared to be, or `None` if it is placed by a box.
    ///
    /// The one question that says which of the two placements an element states, which is why every
    /// pass that resolves geometry asks it first.
    pub(crate) fn traced(&self, leaf: Leaf) -> Option<&Traced> {
        self.world.get_entity(leaf.0).ok()?.get::<Traced>()
    }

    /// Moves `leaf`'s two ends, reporting whether it is something placed by ends at all.
    pub(crate) fn set_traced(&mut self, leaf: Leaf, from: Point, to: Point) -> bool {
        let Ok(mut entity) = self.world.get_entity_mut(leaf.0) else {
            return false;
        };
        let Some(mut traced) = entity.get_mut::<Traced>() else {
            return false;
        };
        traced.from = from;
        traced.to = to;
        true
    }

    /// How thick `leaf` is stroked, or `None` if it is not a stroke.
    pub(crate) fn stroke(&self, leaf: Leaf) -> Option<Stroke> {
        self.read::<Stroke>(leaf)
    }

    /// Where `leaf`'s two ends landed, as R2b resolved them and R4 moved them.
    pub(crate) fn stretched(&self, leaf: Leaf) -> Option<Stretched> {
        self.read::<Stretched>(leaf)
    }

    pub(crate) fn set_stretched(&mut self, leaf: Leaf, stretched: Stretched) {
        if let Ok(mut entity) = self.world.get_entity_mut(leaf.0) {
            entity.insert(stretched);
        }
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

    /// Whether `leaf` stays put while its region's content slides under it.
    pub(crate) fn pinned(&self, leaf: Leaf) -> bool {
        self.read::<Pinned>(leaf).is_some()
    }

    /// How far `leaf` floats out of the regions above it, or `None` if it sits in its region like
    /// anything else.
    pub(crate) fn floats(&self, leaf: Leaf) -> Option<Escape> {
        Some(self.read::<Floats>(leaf)?.0)
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
