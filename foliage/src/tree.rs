use crate::coordinate::{Area, Position};
use crate::elevation::Elevation;
use crate::elm::{Chlorophyll, Outline, PanelPigment, Pigment};
use crate::icon::{Field, IconPigment};
use crate::image::{Fit, ImagePigment, Plate};
use crate::interaction::Gestures;
use crate::keyboard::Keypad;
use crate::layout::{Layout, Short};
use crate::leaf::{Growth, Leaf, Leaves, Named, Presence};
use crate::lifecycle::{Disabled, Opacity, Visible};
use crate::line::{LinePigment, Stroke};
use crate::naming::Naming;
use crate::op::Bud;
use crate::palette::Fill;
use crate::place::{Anchored, Caller, Focusing};
use crate::placement::grid::{Grid, Tracks};
use crate::placement::location::Location;
use crate::placement::trace::{Ends, Trace};
use crate::polygon::{PolygonPigment, Shape};
use crate::rounding::Corners;
use crate::text::font::Typeface;
use crate::text::{Lettering, TextPigment, Tints};
use crate::text_input::{Editing, Parts};
use crate::view::{Escape, Extent, Floats, Offset, Scroll, Shift};

/// No element sits here: the end of a list, or the trunk of an element grown at top level.
const NONE: u32 = u32::MAX;

/// An element is held in the slot.
const LIVE: u32 = 1;
/// The app has not hidden it. See [`Visible`].
const VISIBLE: u32 = 1 << 1;
/// It was disabled in its own right. See [`Disabled`].
const DISABLED: u32 = 1 << 2;
/// It does not travel with its region's content.
///
/// *Moving with the content* and *counting toward extent* are the same question, so they get one
/// answer: a pinned element receives no offset from its nearest scrolling ancestor in R4 and
/// contributes nothing to that region's extent in R3. Two separate flags could drift out of
/// agreement with each other; one cannot.
const PINNED: u32 = 1 << 3;
/// It is a field that refuses the keystrokes that would change its value.
///
/// Carried on the field beside its [`Keypad`], and read wherever a keystroke is about to write:
/// the whole of read-only is that those writes do not happen.
const READ_ONLY: u32 = 1 << 4;
/// It is a run drawn as dots.
///
/// Carried on the field's run rather than on the field, because the run is what R1 shapes and it
/// shapes nothing else: a masked run is shaped as a dot for each character it holds, and that is
/// the whole of what masking changes about the drawing.
const MASKED: u32 = 1 << 5;

/// The tree itself, seen from the inside.
///
/// Holds every element, one slot each at the index of its name, and every field of an element as a
/// column of those slots: what a pass asks of many elements is read from one array rather than
/// gathered from wherever each element is. The only place an element's declarations are held.
///
/// What only some elements state -- a grid, a scroll offset, a font, a run, an anchor -- is held in
/// a side table, and only for those that state it. What an element draws is held packed by
/// renderer, so each renderer's array is as long as what draws with it.
///
/// What is grown under an element is a list threaded through the slots, so letting go of one child
/// of thousands is a few writes rather than a search.
pub(crate) struct Tree {
    /// Where every name came from, and where the index of one that withered goes back to.
    naming: Naming,
    /// The generation each slot is at: that of the element held in it, or of the next one to be.
    /// A name of another generation names something that went, or something still to come.
    generation: Vec<u32>,
    /// Whether an element is held in each slot, what it declared that is one bit, and which of the
    /// side tables hold something for it.
    flags: Vec<u32>,
    /// What each element hangs off, as a slot.
    trunk: Vec<u32>,
    /// What is grown directly under each element, in the order it was grown: the first and last of
    /// them, and for each of them what is before and after it under the same trunk.
    first: Vec<u32>,
    last: Vec<u32>,
    next: Vec<u32>,
    previous: Vec<u32>,
    /// Where each element came in allocation order. See [`Growth`].
    growth: Vec<Growth>,
    /// Where each element was written into existence: what a refusal names, so the panic points at
    /// the call that caused it rather than at an element inside a pass.
    spawned_at: Vec<Option<Caller>>,
    /// Which renderer each element carries, and where in that renderer's array what it was told
    /// sits.
    chlorophyll: Vec<Chlorophyll>,
    pigment: Vec<u32>,
    /// How each element is placed.
    placements: Vec<Option<Declared>>,
    gestures: Vec<Gestures>,
    opacity: Vec<Opacity>,
    elevation: Vec<Elevation>,
    shift: Vec<Shift>,
    scrolls: Vec<Option<Scroll>>,
    panels: Kind<PanelPigment>,
    texts: Kind<TextPigment>,
    polygons: Kind<PolygonPigment>,
    lines: Kind<LinePigment>,
    icons: Kind<IconPigment>,
    images: Kind<ImagePigment>,
    grids: Side<Grid>,
    offsets: Side<Offset>,
    extents: Side<Extent>,
    focusing: Side<Focusing>,
    typefaces: Side<Typeface>,
    anchors: Side<Anchored>,
    floats: Side<Floats>,
    letterings: Side<Lettering>,
    tints: Side<Tints>,
    parts: Side<Parts>,
    editing: Side<Editing>,
    keypads: Side<Keypad>,
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
    /// What was grown since resolution last read this, in the order it was grown, and what withered
    /// and what it drew. Together they are all an order needs to be repaired rather than built
    /// again, unless [`reordered`](Tree::reordered) says an edge moved.
    grown: Vec<Leaf>,
    withered: Vec<(Leaf, Chlorophyll)>,
    /// Whether what scrolls, what is pinned or what floats changed since resolution last read this:
    /// what the regions resolution visits on every frame, and what is not their content, are found
    /// from.
    rescoped: bool,
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

/// How an element is placed, as it declared it.
enum Declared {
    /// By a box.
    Boxed(Location),
    /// By two ends, and the weight of the stroke between them. A stroke is grown with both, and
    /// nothing else is grown with either.
    Traced(Trace, Stroke),
}

/// How an element is placed, borrowed from its declaration.
#[derive(Clone, Copy)]
pub(crate) enum Placement<'a> {
    /// By a box.
    Boxed(&'a Location),
    /// By two ends, and the weight of the stroke between them.
    Traced(&'a Trace, Stroke),
}

/// What one renderer was told, for every element carrying it, packed.
///
/// An element that withers has the last of them moved into its place, and the element that one
/// belongs to is told where it went -- so the array is as long as what draws with this renderer and
/// no longer, and nothing else moves.
struct Kind<P> {
    items: Vec<P>,
    /// The slot each item belongs to.
    owners: Vec<u32>,
}

impl<P> Default for Kind<P> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            owners: Vec::new(),
        }
    }
}

impl<P> Kind<P> {
    /// Holds `item` for the element at `slot`, and hands back where.
    fn push(&mut self, slot: usize, item: P) -> u32 {
        self.items.push(item);
        self.owners.push(slot as u32);
        (self.items.len() - 1) as u32
    }

    /// Lets go of the item at `at`, and hands back the slot of the element whose item was moved
    /// into its place, if one was.
    fn remove(&mut self, at: u32) -> Option<u32> {
        let at = at as usize;
        self.items.swap_remove(at);
        self.owners.swap_remove(at);
        self.owners.get(at).copied()
    }
}

/// What only some elements state, held only for those that do.
///
/// Each table owns a bit of a slot's flags, set exactly while it holds something for that slot: an
/// element that states nothing here is answered by the bit rather than by a lookup, and one that
/// withers lets go of what it held and touches no table it held nothing in.
struct Side<V> {
    bit: u32,
    held: Named<u32, V>,
}

impl<V> Side<V> {
    fn new(bit: u32) -> Self {
        Self {
            bit,
            held: Named::default(),
        }
    }

    fn get(&self, flags: &[u32], slot: usize) -> Option<&V> {
        if flags[slot] & self.bit == 0 {
            return None;
        }
        self.held.get(&(slot as u32))
    }

    fn get_mut(&mut self, flags: &[u32], slot: usize) -> Option<&mut V> {
        if flags[slot] & self.bit == 0 {
            return None;
        }
        self.held.get_mut(&(slot as u32))
    }

    fn insert(&mut self, flags: &mut [u32], slot: usize, value: V) {
        flags[slot] |= self.bit;
        self.held.insert(slot as u32, value);
    }

    /// Writes `value` in place where one is held, holding it where none is, and reports whether
    /// what was held changed.
    fn overwrite(&mut self, flags: &mut [u32], slot: usize, value: V) -> bool
    where
        V: PartialEq,
    {
        match self.get_mut(flags, slot) {
            Some(held) if *held == value => false,
            Some(held) => {
                *held = value;
                true
            }
            None => {
                self.insert(flags, slot, value);
                true
            }
        }
    }

    fn vacate(&mut self, flags: &mut [u32], slot: usize) {
        if flags[slot] & self.bit != 0 {
            flags[slot] &= !self.bit;
            self.held.remove(&(slot as u32));
        }
    }
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
    /// What was grown, in the order it was grown, and what withered and what it drew: what
    /// repairing it reads.
    pub(crate) grown: &'a [Leaf],
    pub(crate) withered: &'a [(Leaf, Chlorophyll)],
    /// Whether what scrolls, what is pinned or what floats changed, so the regions and what is not
    /// their content are found again.
    pub(crate) rescoped: bool,
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
    pub(crate) fn new(naming: Naming) -> Self {
        Self {
            naming,
            generation: Vec::new(),
            flags: Vec::new(),
            trunk: Vec::new(),
            first: Vec::new(),
            last: Vec::new(),
            next: Vec::new(),
            previous: Vec::new(),
            growth: Vec::new(),
            spawned_at: Vec::new(),
            chlorophyll: Vec::new(),
            pigment: Vec::new(),
            placements: Vec::new(),
            gestures: Vec::new(),
            opacity: Vec::new(),
            elevation: Vec::new(),
            shift: Vec::new(),
            scrolls: Vec::new(),
            panels: Kind::default(),
            texts: Kind::default(),
            polygons: Kind::default(),
            lines: Kind::default(),
            icons: Kind::default(),
            images: Kind::default(),
            grids: Side::new(1 << 8),
            offsets: Side::new(1 << 9),
            extents: Side::new(1 << 10),
            focusing: Side::new(1 << 11),
            typefaces: Side::new(1 << 12),
            anchors: Side::new(1 << 13),
            floats: Side::new(1 << 14),
            letterings: Side::new(1 << 15),
            tints: Side::new(1 << 16),
            parts: Side::new(1 << 17),
            editing: Side::new(1 << 18),
            keypads: Side::new(1 << 19),
            touched: Leaves::default(),
            restyled: Leaves::default(),
            lettered: Leaves::default(),
            reached: Leaves::default(),
            restated: true,
            restructured: true,
            grown: Vec::new(),
            withered: Vec::new(),
            rescoped: false,
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
    /// not: those are held in resolution's own columns, or written without a mark, which is the
    /// whole of the difference between a declaration and a resolved value.
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
            rescoped: self.rescoped,
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
        self.rescoped = false;
        self.reordered = false;
        self.invalidated = false;
        self.remeasured = false;
        self.repainted = false;
    }

    /// The slot `leaf` is held in, or `None` if it is not live.
    fn held(&self, leaf: Leaf) -> Option<usize> {
        let slot = leaf.slot();
        (self.generation.get(slot) == Some(&leaf.generation) && self.flags[slot] & LIVE != 0)
            .then_some(slot)
    }

    /// The name of the element held in `slot`.
    fn named(&self, slot: u32) -> Leaf {
        Leaf {
            index: slot,
            generation: self.generation[slot as usize],
        }
    }

    /// What `leaf` names right now.
    ///
    /// A name of the generation its slot is at names what is held there, or what is still to be
    /// grown there; one of another names what went. A name past every slot was handed out and has
    /// not been grown, since nothing has been grown that far yet.
    pub(crate) fn presence(&self, leaf: Leaf) -> Presence {
        let slot = leaf.slot();
        match self.generation.get(slot) {
            None => Presence::Planted,
            Some(&generation) if generation != leaf.generation => Presence::Withered,
            Some(_) if self.flags[slot] & LIVE != 0 => Presence::Live,
            Some(_) => Presence::Planted,
        }
    }

    pub(crate) fn is_live(&self, leaf: Leaf) -> bool {
        self.held(leaf).is_some()
    }

    /// Grows `leaf`, reporting whether the name was still free to grow into.
    pub(crate) fn grow(
        &mut self,
        leaf: Leaf,
        growth: Growth,
        under: Option<Leaf>,
        mut bud: Bud,
    ) -> bool {
        let slot = leaf.slot();
        if slot >= self.generation.len() {
            self.reach(slot + 1);
        }
        if self.generation[slot] != leaf.generation || self.flags[slot] & LIVE != 0 {
            return false;
        }
        // Everything an element declares about how it behaves. What resolution makes of it is not
        // here: that is held in resolution's own columns ([`Elements`](crate::rowan::Elements)),
        // which is where every pass reads it.
        let manner = bud.placement.manner;
        let mut flags = LIVE;
        if manner.visible.0 {
            flags |= VISIBLE;
        }
        // Absent unless declared: an element that travels with its region's content and is clipped
        // by it is the ordinary case.
        if manner.pinned {
            flags |= PINNED;
        }
        self.flags[slot] = flags;
        self.growth[slot] = growth;
        self.spawned_at[slot] = Some(bud.at);
        self.chlorophyll[slot] = bud.chlorophyll;
        self.gestures[slot] = manner.gestures;
        self.opacity[slot] = manner.opacity;
        self.elevation[slot] = bud.placement.elevation.unwrap_or_default();
        self.shift[slot] = Shift::default();
        self.scrolls[slot] = manner.scrolls.map(|scrolls| scrolls.0);
        // Grown under something that went before the drain reached this is grown at top level,
        // since there is nothing left to hang it off.
        match under.and_then(|under| self.held(under)) {
            Some(trunk) => self.attach(slot, trunk),
            None => self.trunk[slot] = NONE,
        }
        // A stroke is placed by its ends and nothing else is, and is not given a box as well: it
        // would be held, and carried, and read by nothing, since a trace is what resolution asks
        // for first and a `Location` written onto a stroke is refused.
        self.placements[slot] = Some(match bud.placement.traced.take() {
            Some(trace) => Declared::Traced(trace, bud.placement.stroke.take().unwrap_or_default()),
            None => Declared::Boxed(bud.placement.location.take().unwrap_or_default()),
        });
        self.pigment[slot] = match bud.pigment.take() {
            // What a renderer was told is held where that renderer reads it, so it is held only
            // where the element carries that renderer.
            Some(pigment) if chlorophyll(&pigment) != bud.chlorophyll => NONE,
            Some(Pigment::Panel(pigment)) => self.panels.push(slot, pigment),
            Some(Pigment::Text(pigment)) => self.texts.push(slot, pigment),
            Some(Pigment::Polygon(pigment)) => self.polygons.push(slot, pigment),
            Some(Pigment::Line(pigment)) => self.lines.push(slot, pigment),
            Some(Pigment::Icon(pigment)) => self.icons.push(slot, pigment),
            Some(Pigment::Image(pigment)) => self.images.push(slot, pigment),
            None => NONE,
        };
        let flags = &mut self.flags;
        if let Some(grid) = bud.placement.grid {
            self.grids.insert(flags, slot, grid);
        }
        if manner.focusing != Focusing::default() {
            self.focusing.insert(flags, slot, manner.focusing);
        }
        if let Some(escape) = manner.floats {
            self.floats.insert(flags, slot, Floats(escape));
        }
        if let Some(lettering) = bud.lettering {
            self.letterings.insert(flags, slot, lettering);
        }
        if let Some(tints) = bud.tints {
            self.tints.insert(flags, slot, tints);
        }
        if let Some(typeface) = bud.placement.typeface {
            self.typefaces.insert(flags, slot, typeface);
        }
        if let Some(anchored) = bud.placement.anchor {
            self.anchors.insert(flags, slot, anchored);
        }
        // What it was grown under has one more element to reach over, and finds that out by the
        // walk R2m makes from this one toward it.
        self.declared(leaf);
        self.lettered.insert(leaf);
        self.restructured = true;
        self.grown.push(leaf);
        true
    }

    /// Makes every column `len` slots long, each new slot empty.
    fn reach(&mut self, len: usize) {
        self.generation.resize(len, 0);
        self.flags.resize(len, 0);
        self.trunk.resize(len, NONE);
        self.first.resize(len, NONE);
        self.last.resize(len, NONE);
        self.next.resize(len, NONE);
        self.previous.resize(len, NONE);
        self.growth.resize(len, Growth::default());
        self.spawned_at.resize(len, None);
        self.chlorophyll.resize(len, Chlorophyll::None);
        self.pigment.resize(len, NONE);
        self.placements.resize_with(len, || None);
        self.gestures.resize(len, Gestures::default());
        self.opacity.resize(len, Opacity::default());
        self.elevation.resize(len, Elevation::default());
        self.shift.resize(len, Shift::default());
        self.scrolls.resize(len, None);
    }

    /// Hangs the element at `slot` off the one at `trunk`, after everything already grown there.
    fn attach(&mut self, slot: usize, trunk: usize) {
        let last = self.last[trunk];
        self.trunk[slot] = trunk as u32;
        self.previous[slot] = last;
        self.next[slot] = NONE;
        match last {
            NONE => self.first[trunk] = slot as u32,
            last => self.next[last as usize] = slot as u32,
        }
        self.last[trunk] = slot as u32;
    }

    /// Takes the element at `slot` out from under what it hangs off, leaving the rest in order.
    fn detach(&mut self, slot: usize) {
        let trunk = self.trunk[slot];
        if trunk == NONE {
            return;
        }
        let (previous, next) = (self.previous[slot], self.next[slot]);
        match previous {
            NONE => self.first[trunk as usize] = next,
            previous => self.next[previous as usize] = next,
        }
        match next {
            NONE => self.last[trunk as usize] = previous,
            next => self.previous[next as usize] = previous,
        }
        self.trunk[slot] = NONE;
        self.previous[slot] = NONE;
        self.next[slot] = NONE;
    }

    /// Takes each of `leaves` down, and everything beneath each, and reports every name that went.
    ///
    /// Each is taken out from under what it hangs off where it lies, which leaves what was grown
    /// beside it in order and costs the same however many were grown there -- so letting go of many
    /// off one trunk, as a list cut short does, costs what went and not the trunk's length for each.
    pub(crate) fn wither(&mut self, leaves: &[Leaf]) -> Vec<Leaf> {
        let mut gone = Vec::new();
        let mut beneath = Vec::new();
        for &leaf in leaves {
            // Already gone, under one of the others.
            let Some(slot) = self.held(leaf) else {
                continue;
            };
            if let Some(trunk) = self.trunk(leaf) {
                self.detach(slot);
                // What it hung off has one fewer element to reach over, and nothing left below it
                // to say so: the element that went is what the walk would have started from.
                self.measures(trunk);
            }
            // Everything beneath it, each before what was grown under it and in the order it was
            // grown -- and each let go of once found, so one of `leaves` under another is not
            // found twice.
            beneath.push(slot as u32);
            while let Some(slot) = beneath.pop() {
                gone.push(self.named(slot));
                // What it drew goes with it, and extraction has to know which renderer to tell.
                self.withered
                    .push((self.named(slot), self.chlorophyll[slot as usize]));
                let mut child = self.last[slot as usize];
                while child != NONE {
                    beneath.push(child);
                    child = self.previous[child as usize];
                }
                self.vacate(slot as usize);
            }
        }
        // Back to be named again, each a generation on from the name that went.
        self.naming.release(gone.iter().map(|leaf| Leaf {
            index: leaf.index,
            generation: self.generation[leaf.slot()],
        }));
        self.restructured = true;
        // Whatever it said, it does not say any more.
        self.restated = true;
        gone
    }

    /// Empties `slot`, and moves it a generation on.
    fn vacate(&mut self, slot: usize) {
        let at = self.pigment[slot];
        if at != NONE {
            let moved = match self.chlorophyll[slot] {
                Chlorophyll::None => None,
                Chlorophyll::Panel => self.panels.remove(at),
                Chlorophyll::Text => self.texts.remove(at),
                Chlorophyll::Polygon => self.polygons.remove(at),
                Chlorophyll::Line => self.lines.remove(at),
                Chlorophyll::Icon => self.icons.remove(at),
                Chlorophyll::Image => self.images.remove(at),
            };
            if let Some(moved) = moved {
                self.pigment[moved as usize] = at;
            }
        }
        let flags = &mut self.flags;
        self.grids.vacate(flags, slot);
        self.offsets.vacate(flags, slot);
        self.extents.vacate(flags, slot);
        self.focusing.vacate(flags, slot);
        self.typefaces.vacate(flags, slot);
        self.anchors.vacate(flags, slot);
        self.floats.vacate(flags, slot);
        self.letterings.vacate(flags, slot);
        self.tints.vacate(flags, slot);
        self.parts.vacate(flags, slot);
        self.editing.vacate(flags, slot);
        self.keypads.vacate(flags, slot);
        self.generation[slot] = self.generation[slot].wrapping_add(1);
        self.flags[slot] = 0;
        self.trunk[slot] = NONE;
        self.first[slot] = NONE;
        self.last[slot] = NONE;
        self.next[slot] = NONE;
        self.previous[slot] = NONE;
        self.growth[slot] = Growth::default();
        self.spawned_at[slot] = None;
        self.chlorophyll[slot] = Chlorophyll::None;
        self.pigment[slot] = NONE;
        self.placements[slot] = None;
        self.gestures[slot] = Gestures::default();
        self.opacity[slot] = Opacity::default();
        self.elevation[slot] = Elevation::default();
        self.shift[slot] = Shift::default();
        self.scrolls[slot] = None;
    }

    /// The elements the app branched directly off `leaf`, in the order they were grown.
    pub(crate) fn branches(&self, leaf: Leaf) -> Vec<Leaf> {
        self.branched(leaf).collect()
    }

    /// The same, borrowed rather than collected.
    pub(crate) fn branched(&self, leaf: Leaf) -> impl Iterator<Item = Leaf> + '_ {
        let first = self.held(leaf).map_or(NONE, |slot| self.first[slot]);
        core::iter::successors((first != NONE).then_some(first), |&slot| {
            let next = self.next[slot as usize];
            (next != NONE).then_some(next)
        })
        .map(|slot| self.named(slot))
    }

    /// The element `leaf` was branched off, or `None` if it was planted at top level.
    pub(crate) fn trunk(&self, leaf: Leaf) -> Option<Leaf> {
        let trunk = self.trunk[self.held(leaf)?];
        (trunk != NONE).then(|| self.named(trunk))
    }

    /// Every live element, in the order of the slots they are held in.
    pub(crate) fn leaves(&self) -> Vec<Leaf> {
        (0..self.flags.len() as u32)
            .filter(|&slot| self.flags[slot as usize] & LIVE != 0)
            .map(|slot| self.named(slot))
            .collect()
    }

    pub(crate) fn location(&self, leaf: Leaf) -> Option<&Location> {
        match self.placements[self.held(leaf)?].as_ref()? {
            Declared::Boxed(location) => Some(location),
            Declared::Traced(..) => None,
        }
    }

    /// How `leaf` is placed: by its two ends and the weight between them, or by a box. `None` for
    /// an element that is not live.
    ///
    /// One read of the element for the question every pass that resolves geometry asks first, and
    /// the declaration it goes on to read.
    pub(crate) fn placement(&self, leaf: Leaf) -> Option<Placement<'_>> {
        Some(match self.placements[self.held(leaf)?].as_ref()? {
            Declared::Boxed(location) => Placement::Boxed(location),
            Declared::Traced(trace, stroke) => Placement::Traced(trace, *stroke),
        })
    }

    /// Which font `leaf` composes in and at what size, or `None` if it was never given one.
    pub(crate) fn typeface(&self, leaf: Leaf) -> Option<Typeface> {
        self.typefaces.get(&self.flags, self.held(leaf)?).copied()
    }

    /// What `leaf` says, if it is a run of glyphs.
    pub(crate) fn lettering(&self, leaf: Leaf) -> Option<&str> {
        let lettering = self.letterings.get(&self.flags, self.held(leaf)?)?;
        Some(lettering.0.as_str())
    }

    /// Rewrites what `leaf` says, reporting whether it is something with a run to write.
    pub(crate) fn set_lettering(&mut self, leaf: Leaf, value: String) -> bool {
        let Some(slot) = self.held(leaf) else {
            return false;
        };
        let Some(lettering) = self.letterings.get_mut(&self.flags, slot) else {
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
        self.parts.get(&self.flags, self.held(leaf)?).copied()
    }

    pub(crate) fn set_parts(&mut self, leaf: Leaf, parts: Parts) {
        if let Some(slot) = self.held(leaf) {
            self.parts.insert(&mut self.flags, slot, parts);
        }
    }

    /// Every field, with what each is made of, in the order of the slots they are held in.
    pub(crate) fn fields(&mut self) -> Vec<(Leaf, Parts)> {
        let mut fields: Vec<_> = self
            .parts
            .held
            .iter()
            .map(|(&slot, &parts)| (self.named(slot), parts))
            .collect();
        fields.sort_unstable_by_key(|(field, _)| field.index);
        fields
    }

    /// Which soft keyboard `leaf` asks for, if it is something that is typed into at all.
    ///
    /// `None` for everything but a field, which is what makes focus alone decide the keyboard: a
    /// button can hold focus and has no keypad, so nothing is raised for it.
    pub(crate) fn keypad(&self, leaf: Leaf) -> Option<Keypad> {
        self.keypads.get(&self.flags, self.held(leaf)?).copied()
    }

    pub(crate) fn set_keypad(&mut self, leaf: Leaf, keypad: Keypad) {
        if let Some(slot) = self.held(leaf) {
            self.keypads.insert(&mut self.flags, slot, keypad);
        }
    }

    /// Whether `leaf` is a field that refuses the keystrokes that would change its value. `false`
    /// for everything that is not a field, which has no value to keep.
    pub(crate) fn read_only(&self, leaf: Leaf) -> bool {
        self.held(leaf)
            .is_some_and(|slot| self.flags[slot] & READ_ONLY != 0)
    }

    pub(crate) fn set_read_only(&mut self, leaf: Leaf, read_only: bool) {
        if let Some(slot) = self.held(leaf) {
            self.flag(slot, READ_ONLY, read_only);
        }
    }

    /// Whether `leaf` is a run drawn as dots. `false` for everything but a masked field's run.
    pub(crate) fn masked(&self, leaf: Leaf) -> bool {
        self.held(leaf)
            .is_some_and(|slot| self.flags[slot] & MASKED != 0)
    }

    /// Masks a run or shows it, and has it shaped again if that changed what it draws -- which is
    /// the same statement a new value is, so it is marked the way [`set_lettering`](Self::set_lettering)
    /// marks one.
    pub(crate) fn set_masked(&mut self, leaf: Leaf, masked: bool) {
        let Some(slot) = self.held(leaf) else {
            return;
        };
        if !self.flag(slot, MASKED, masked) {
            return;
        }
        self.declared(leaf);
        self.lettered.insert(leaf);
        self.restated = true;
    }

    /// Where `leaf`'s caret is and what it has selected.
    pub(crate) fn editing(&self, leaf: Leaf) -> Editing {
        self.held(leaf)
            .and_then(|slot| self.editing.get(&self.flags, slot).copied())
            .unwrap_or_default()
    }

    pub(crate) fn set_editing(&mut self, leaf: Leaf, editing: Editing) {
        if let Some(slot) = self.held(leaf) {
            self.editing.insert(&mut self.flags, slot, editing);
        }
        self.declared(leaf);
    }

    /// The element `leaf`'s placement may read, if it has been given one.
    pub(crate) fn anchor(&self, leaf: Leaf) -> Option<Leaf> {
        Some(self.anchors.get(&self.flags, self.held(leaf)?)?.to)
    }

    /// Where `leaf` was written into existence.
    pub(crate) fn spawned_at(&self, leaf: Leaf) -> Option<Caller> {
        self.spawned_at[self.held(leaf)?]
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
        let Some(slot) = self.held(leaf) else {
            return false;
        };
        match &mut self.placements[slot] {
            Some(Declared::Traced(..)) => return false,
            // Written back at what it already holds is no write at all. Resolution would answer the
            // same box, and asking it to is the whole cost of the element and everything under it.
            Some(Declared::Boxed(held)) if *held == location => return true,
            declared => *declared = Some(Declared::Boxed(location)),
        }
        self.declared(leaf);
        true
    }

    pub(crate) fn set_grid(&mut self, leaf: Leaf, grid: Grid) {
        let Some(slot) = self.held(leaf) else {
            return;
        };
        if self.grids.overwrite(&mut self.flags, slot, grid) {
            self.declared(leaf);
        }
    }

    pub(crate) fn set_anchor(&mut self, leaf: Leaf, to: Leaf, at: Caller) {
        if let Some(slot) = self.held(leaf) {
            self.anchors
                .insert(&mut self.flags, slot, Anchored { to, at });
        }
        self.declared(leaf);
        // An anchor is an edge of the graph the order is built from, between elements already there.
        self.restructured = true;
        self.reordered = true;
    }

    /// How far in front of its trunk `leaf` was told to sit.
    pub(crate) fn elevation(&self, leaf: Leaf) -> Elevation {
        self.held(leaf)
            .map(|slot| self.elevation[slot])
            .unwrap_or_default()
    }

    /// Where `leaf` came in allocation order.
    pub(crate) fn growth(&self, leaf: Leaf) -> Growth {
        self.held(leaf)
            .map(|slot| self.growth[slot])
            .unwrap_or_default()
    }

    pub(crate) fn set_elevation(&mut self, leaf: Leaf, elevation: Elevation) {
        let Some(slot) = self.held(leaf) else {
            return;
        };
        if self.elevation[slot] != elevation {
            self.elevation[slot] = elevation;
            self.restyled(leaf);
        }
    }

    /// Where in the array of `chlorophyll`'s renderer what `leaf` told it sits, or `None` if `leaf`
    /// does not carry that renderer.
    fn pigmented(&self, leaf: Leaf, chlorophyll: Chlorophyll) -> Option<usize> {
        let slot = self.held(leaf)?;
        let at = self.pigment[slot];
        (self.chlorophyll[slot] == chlorophyll && at != NONE).then_some(at as usize)
    }

    /// What the panel renderer on `leaf` was told, or `None` if `leaf` is not a panel.
    pub(crate) fn panel_pigment(&self, leaf: Leaf) -> Option<PanelPigment> {
        Some(self.panels.items[self.pigmented(leaf, Chlorophyll::Panel)?])
    }

    /// What the text renderer on `leaf` was told, or `None` if `leaf` is not a run.
    pub(crate) fn text_pigment(&self, leaf: Leaf) -> Option<TextPigment> {
        Some(self.texts.items[self.pigmented(leaf, Chlorophyll::Text)?])
    }

    /// What the polygon renderer on `leaf` was told, or `None` if `leaf` is not a polygon.
    pub(crate) fn polygon_pigment(&self, leaf: Leaf) -> Option<PolygonPigment> {
        Some(self.polygons.items[self.pigmented(leaf, Chlorophyll::Polygon)?])
    }

    /// What the line renderer on `leaf` was told, or `None` if `leaf` is not a stroke.
    pub(crate) fn line_pigment(&self, leaf: Leaf) -> Option<LinePigment> {
        Some(self.lines.items[self.pigmented(leaf, Chlorophyll::Line)?])
    }

    /// What the icon renderer on `leaf` was told, or `None` if `leaf` is not a mark.
    pub(crate) fn icon_pigment(&self, leaf: Leaf) -> Option<IconPigment> {
        Some(self.icons.items[self.pigmented(leaf, Chlorophyll::Icon)?])
    }

    /// What the image renderer on `leaf` was told, or `None` if `leaf` is not a picture.
    pub(crate) fn image_pigment(&self, leaf: Leaf) -> Option<ImagePigment> {
        Some(self.images.items[self.pigmented(leaf, Chlorophyll::Image)?])
    }

    /// Where `leaf`'s two ends are declared to be, per breakpoint, or `None` if it is placed by a
    /// box.
    ///
    /// The one question that says which of the two placements an element states, which is why every
    /// pass that resolves geometry asks it first.
    pub(crate) fn trace(&self, leaf: Leaf) -> Option<&Trace> {
        match self.placements[self.held(leaf)?].as_ref()? {
            Declared::Traced(trace, _) => Some(trace),
            Declared::Boxed(_) => None,
        }
    }

    /// The trace `leaf` holds and its weight, to be written in place, or `None` if it is placed by
    /// a box.
    fn traced(&mut self, leaf: Leaf) -> Option<(&mut Trace, &mut Stroke)> {
        let slot = self.held(leaf)?;
        match self.placements[slot].as_mut()? {
            Declared::Traced(trace, stroke) => Some((trace, stroke)),
            Declared::Boxed(_) => None,
        }
    }

    /// Moves `leaf`'s two ends, reporting whether it is something placed by ends at all.
    ///
    /// A box-placed element has none, and refuses the write for the reason
    /// [`set_location`](Tree::set_location) refuses the other: a trace on a box would sit there
    /// being ignored.
    pub(crate) fn set_trace(&mut self, leaf: Leaf, trace: Trace) -> bool {
        let Some((held, _)) = self.traced(leaf) else {
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
        let Some((held, _)) = self.traced(leaf) else {
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
        let Some((_, held)) = self.traced(leaf) else {
            return false;
        };
        if *held != stroke {
            *held = stroke;
            self.declared(leaf);
        }
        true
    }

    /// How thick `leaf` is stroked, or `None` if it is not a stroke.
    pub(crate) fn stroke(&self, leaf: Leaf) -> Option<Stroke> {
        match self.placements[self.held(leaf)?].as_ref()? {
            Declared::Traced(_, stroke) => Some(*stroke),
            Declared::Boxed(_) => None,
        }
    }

    /// How parts of `leaf`'s run are filled differently from the rest of it.
    pub(crate) fn tints(&self, leaf: Leaf) -> Option<&Tints> {
        self.tints.get(&self.flags, self.held(leaf)?)
    }

    /// Refills parts of `leaf`'s run, reporting whether it is something with a run to tint.
    ///
    /// Replaces every tint rather than adding one, for the reason a placement is one value: there is
    /// no half-written state between two of these and no question of which range a later write meant.
    pub(crate) fn set_tints(&mut self, leaf: Leaf, tints: Tints) -> bool {
        let Some(slot) = self.held(leaf) else {
            return false;
        };
        if self.letterings.get(&self.flags, slot).is_none() {
            return false;
        }
        self.tints.insert(&mut self.flags, slot, tints);
        self.restyled(leaf);
        true
    }

    /// Reshapes `leaf`, reporting whether it is something with a shape to reshape.
    pub(crate) fn set_shape(&mut self, leaf: Leaf, shape: Shape) -> bool {
        let Some(at) = self.pigmented(leaf, Chlorophyll::Polygon) else {
            return false;
        };
        self.polygons.items[at].shape = shape;
        self.restyled(leaf);
        true
    }

    /// Swaps the mark `leaf` draws, reporting whether it is an icon at all.
    pub(crate) fn set_mark(&mut self, leaf: Leaf, field: Field) -> bool {
        let Some(at) = self.pigmented(leaf, Chlorophyll::Icon) else {
            return false;
        };
        self.icons.items[at].field = field;
        self.restyled(leaf);
        true
    }

    /// Swaps the picture `leaf` draws, reporting whether it is an image at all.
    pub(crate) fn set_plate(&mut self, leaf: Leaf, plate: Plate) -> bool {
        let Some(at) = self.pigmented(leaf, Chlorophyll::Image) else {
            return false;
        };
        self.images.items[at].plate = plate;
        self.restyled(leaf);
        true
    }

    /// Changes how `leaf`'s pixels are fitted into its box, reporting whether it is an image at all.
    pub(crate) fn set_fit(&mut self, leaf: Leaf, fit: Fit) -> bool {
        let Some(at) = self.pigmented(leaf, Chlorophyll::Image) else {
            return false;
        };
        self.images.items[at].fit = fit;
        self.restyled(leaf);
        true
    }

    /// What `leaf` is shaped as, or `None` if it is not a polygon.
    pub(crate) fn shape(&self, leaf: Leaf) -> Option<Shape> {
        Some(self.polygon_pigment(leaf)?.shape)
    }

    /// What `leaf` is filled with, whichever renderer holds the fill, or `None` if it has none.
    ///
    /// One question across the renderers, because a fill is one property: the same
    /// [`color`](crate::Grow::color) refills a panel, a run, a shape, a stroke and a mark, and the
    /// same motion moves any of them. An [`Image`](crate::Image) is the one element with nothing to
    /// answer -- it carries its own colour, and a fill would be a second opinion about it.
    pub(crate) fn fill(&self, leaf: Leaf) -> Option<Fill> {
        let slot = self.held(leaf)?;
        let at = self.pigment[slot] as usize;
        if self.pigment[slot] == NONE {
            return None;
        }
        match self.chlorophyll[slot] {
            Chlorophyll::Panel => Some(self.panels.items[at].fill),
            Chlorophyll::Text => Some(self.texts.items[at].fill),
            Chlorophyll::Polygon => Some(self.polygons.items[at].fill),
            Chlorophyll::Line => Some(self.lines.items[at].fill),
            Chlorophyll::Icon => Some(self.icons.items[at].fill),
            Chlorophyll::Image | Chlorophyll::None => None,
        }
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
        let Some(slot) = self.held(leaf) else {
            return false;
        };
        let at = self.pigment[slot] as usize;
        if self.pigment[slot] == NONE {
            return false;
        }
        match self.chlorophyll[slot] {
            Chlorophyll::Panel => self.panels.items[at].fill = fill,
            Chlorophyll::Text => self.texts.items[at].fill = fill,
            Chlorophyll::Polygon => self.polygons.items[at].fill = fill,
            Chlorophyll::Line => self.lines.items[at].fill = fill,
            Chlorophyll::Icon => self.icons.items[at].fill = fill,
            Chlorophyll::Image | Chlorophyll::None => return false,
        }
        true
    }

    /// The line drawn around the inside of `leaf`, or `None` if it is not a panel.
    pub(crate) fn outline(&self, leaf: Leaf) -> Option<Outline> {
        Some(self.panel_pigment(leaf)?.outline)
    }

    /// Outlines `leaf`, reporting whether it is a panel.
    ///
    /// Read by extraction alone, so it is a restyle: the panel is drawn again and nothing is placed
    /// again. A panel and nothing else, because an outline is drawn by the field that rounds a
    /// panel, and the other renderers have no such field to draw one with.
    pub(crate) fn set_outline(&mut self, leaf: Leaf, outline: Outline) -> bool {
        let Some(at) = self.pigmented(leaf, Chlorophyll::Panel) else {
            return false;
        };
        self.panels.items[at].outline = outline;
        self.restyled(leaf);
        true
    }

    /// How `leaf`'s corners are rounded, or `None` if it has no box to round.
    pub(crate) fn rounding(&self, leaf: Leaf) -> Option<Corners> {
        if let Some(pigment) = self.panel_pigment(leaf) {
            return Some(pigment.rounding);
        }
        Some(self.image_pigment(leaf)?.rounding)
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
        if let Some(at) = self.pigmented(leaf, Chlorophyll::Panel) {
            self.panels.items[at].rounding = rounding;
            return true;
        }
        if let Some(at) = self.pigmented(leaf, Chlorophyll::Image) {
            self.images.items[at].rounding = rounding;
            return true;
        }
        false
    }

    /// What `leaf` declared about gestures.
    pub(crate) fn gestures(&self, leaf: Leaf) -> Gestures {
        self.held(leaf)
            .map(|slot| self.gestures[slot])
            .unwrap_or_default()
    }

    /// Pins `leaf` in its region, or lets it travel with the content again, reporting whether that
    /// changed it. A restyle, for the reason [`set_scrolls`](Tree::set_scrolls) is one.
    pub(crate) fn set_pinned(&mut self, leaf: Leaf, pinned: bool) -> bool {
        let Some(slot) = self.held(leaf) else {
            return false;
        };
        if !self.flag(slot, PINNED, pinned) {
            return false;
        }
        self.restyled(leaf);
        self.rescoped = true;
        true
    }

    /// Floats `leaf` out of the regions above it as far as `escape` says, or puts it back in its
    /// own, reporting whether that changed it. A restyle, for the reason
    /// [`set_scrolls`](Tree::set_scrolls) is one.
    pub(crate) fn set_floats(&mut self, leaf: Leaf, escape: Option<Escape>) -> bool {
        let Some(slot) = self.held(leaf) else {
            return false;
        };
        let held = self.floats.get(&self.flags, slot).map(|floats| floats.0);
        if held == escape {
            return false;
        }
        match escape {
            Some(escape) => self.floats.insert(&mut self.flags, slot, Floats(escape)),
            None => self.floats.vacate(&mut self.flags, slot),
        }
        self.restyled(leaf);
        self.rescoped = true;
        true
    }

    /// What `leaf` declared about scrolling, or `None` if it does not scroll.
    pub(crate) fn scrolls(&self, leaf: Leaf) -> Option<Scroll> {
        self.scrolls[self.held(leaf)?]
    }

    /// Makes `leaf` a region scrolling as `scroll` says, or no region at all, reporting whether that
    /// changed what it is.
    ///
    /// A restyle: no box moves for it, and what an element scrolled by is taken off what is drawn
    /// rather than off what is placed. What the regions are is found again from it. An element that
    /// stops scrolling lets go of where it was scrolled to and how far its content reached, as an
    /// axis that is not declared has neither.
    pub(crate) fn set_scrolls(&mut self, leaf: Leaf, scroll: Option<Scroll>) -> bool {
        let Some(slot) = self.held(leaf) else {
            return false;
        };
        if self.scrolls[slot] == scroll {
            return false;
        }
        self.scrolls[slot] = scroll;
        if scroll.is_none() {
            self.offsets.vacate(&mut self.flags, slot);
            self.extents.vacate(&mut self.flags, slot);
        }
        self.restyled(leaf);
        self.rescoped = true;
        true
    }

    /// Where `leaf` was told to sit in focus order, and whether focus cycles inside it.
    fn focusing(&self, leaf: Leaf) -> Focusing {
        self.held(leaf)
            .and_then(|slot| self.focusing.get(&self.flags, slot).copied())
            .unwrap_or_default()
    }

    /// Where `leaf` was told to sit in focus order, relative to the elements around it.
    pub(crate) fn focus_order(&self, leaf: Leaf) -> i32 {
        self.focusing(leaf).order
    }

    /// Whether focus cycles inside `leaf`.
    pub(crate) fn focus_scope(&self, leaf: Leaf) -> bool {
        self.focusing(leaf).scope
    }

    /// How far `leaf` has been scrolled.
    pub(crate) fn offset(&self, leaf: Leaf) -> Position {
        self.held(leaf)
            .and_then(|slot| self.offsets.get(&self.flags, slot))
            .map_or_else(Position::default, |offset| offset.0)
    }

    /// Moves `leaf`'s content, if it is a region: nothing else has an offset to hold.
    pub(crate) fn set_offset(&mut self, leaf: Leaf, offset: Position) {
        if let Some(slot) = self.held(leaf)
            && self.scrolls[slot].is_some()
        {
            self.offsets
                .overwrite(&mut self.flags, slot, Offset(offset));
        }
    }

    /// How far `leaf`'s content reaches, as R3 last measured it.
    pub(crate) fn extent(&self, leaf: Leaf) -> Area {
        self.held(leaf)
            .and_then(|slot| self.extents.get(&self.flags, slot))
            .map_or_else(Area::default, |extent| extent.0)
    }

    /// Holds how far `leaf`'s content reaches, if it is a region: nothing else has an extent to
    /// hold.
    pub(crate) fn set_extent(&mut self, leaf: Leaf, extent: Area) {
        if let Some(slot) = self.held(leaf)
            && self.scrolls[slot].is_some()
        {
            self.extents
                .overwrite(&mut self.flags, slot, Extent(extent));
        }
    }

    /// Sets or clears `bit` of `slot`'s flags, reporting whether that changed them.
    fn flag(&mut self, slot: usize, bit: u32, set: bool) -> bool {
        let held = self.flags[slot];
        let flags = match set {
            true => held | bit,
            false => held & !bit,
        };
        self.flags[slot] = flags;
        flags != held
    }

    /// Whether the app has hidden `leaf` itself, as against an ancestor of it.
    pub(crate) fn visible(&self, leaf: Leaf) -> Visible {
        self.held(leaf).map_or_else(Visible::default, |slot| {
            Visible(self.flags[slot] & VISIBLE != 0)
        })
    }

    pub(crate) fn set_visible(&mut self, leaf: Leaf, visible: bool) {
        if let Some(slot) = self.held(leaf)
            && self.flag(slot, VISIBLE, visible)
        {
            self.restyled(leaf);
        }
    }

    /// How opaque `leaf` was told to be, before its ancestry is taken into account.
    pub(crate) fn opacity(&self, leaf: Leaf) -> Opacity {
        self.held(leaf)
            .map(|slot| self.opacity[slot])
            .unwrap_or_default()
    }

    pub(crate) fn set_opacity(&mut self, leaf: Leaf, opacity: f32) {
        let Some(slot) = self.held(leaf) else {
            return;
        };
        let opacity = Opacity::new(opacity);
        if self.opacity[slot] != opacity {
            self.opacity[slot] = opacity;
            self.restyled(leaf);
        }
    }

    /// Whether `leaf` was disabled in its own right.
    pub(crate) fn disabled(&self, leaf: Leaf) -> Disabled {
        self.held(leaf).map_or_else(Disabled::default, |slot| {
            Disabled(self.flags[slot] & DISABLED != 0)
        })
    }

    pub(crate) fn set_disabled(&mut self, leaf: Leaf, disabled: bool) {
        if let Some(slot) = self.held(leaf)
            && self.flag(slot, DISABLED, disabled)
        {
            self.restyled(leaf);
        }
    }

    /// How far `leaf` is drawn from where the layout put it.
    pub(crate) fn shift(&self, leaf: Leaf) -> Position {
        self.held(leaf)
            .map_or_else(Position::default, |slot| self.shift[slot].0)
    }

    pub(crate) fn set_shift(&mut self, leaf: Leaf, by: Position) {
        let Some(slot) = self.held(leaf) else {
            return;
        };
        if self.shift[slot].0 != by {
            self.shift[slot] = Shift(by);
            self.restyled(leaf);
        }
    }

    /// Everything the passes after the axes read about `leaf`, read where it is held.
    ///
    /// Only of an element resolution holds in its order, which is live: what is asked of it on
    /// every frame is not asked again whether it is there.
    pub(crate) fn standing(&self, leaf: Leaf) -> Standing<'_> {
        debug_assert!(self.is_live(leaf), "{leaf:?} is not live");
        Standing {
            tree: self,
            slot: leaf.slot(),
        }
    }
}

/// Everything the passes after the axes read about one element, and nothing that places it.
///
/// Read where the tree holds it rather than copied out beside the order, so what an element
/// declared is held once and a write to it is what the next pass to ask reads.
#[derive(Clone, Copy)]
pub(crate) struct Standing<'a> {
    tree: &'a Tree,
    slot: usize,
}

impl Standing<'_> {
    fn flag(self, bit: u32) -> bool {
        self.tree.flags[self.slot] & bit != 0
    }

    /// What it draws.
    pub(crate) fn chlorophyll(self) -> Chlorophyll {
        self.tree.chlorophyll[self.slot]
    }

    /// Where it came in allocation order, which is what settles a tie in the stack.
    pub(crate) fn growth(self) -> u64 {
        self.tree.growth[self.slot].0
    }

    /// What it declared about gestures.
    pub(crate) fn gestures(self) -> Gestures {
        self.tree.gestures[self.slot]
    }

    /// What it declared about scrolling, if it scrolls.
    pub(crate) fn scrolls(self) -> Option<Scroll> {
        self.tree.scrolls[self.slot]
    }

    /// Whether it stays put while its region's content slides under it.
    pub(crate) fn pinned(self) -> bool {
        self.flag(PINNED)
    }

    /// How far it floats out of the regions above it.
    pub(crate) fn floats(self) -> Option<Escape> {
        let tree = self.tree;
        tree.floats
            .get(&tree.flags, self.slot)
            .map(|floats| floats.0)
    }

    /// Whether the app hid it, as against an ancestor of it.
    pub(crate) fn visible(self) -> bool {
        self.flag(VISIBLE)
    }

    /// How opaque it was told to be, before its ancestry is taken into account.
    pub(crate) fn opacity(self) -> f32 {
        self.tree.opacity[self.slot].0
    }

    /// Whether it was disabled in its own right.
    pub(crate) fn disabled(self) -> bool {
        self.flag(DISABLED)
    }

    /// How far in front of its trunk it was told to sit.
    pub(crate) fn elevation(self) -> Elevation {
        self.tree.elevation[self.slot]
    }

    /// How far it is drawn from where the layout put it, and everything under it with it.
    pub(crate) fn shift(self) -> Position {
        self.tree.shift[self.slot].0
    }

    /// Its own grid, divided at the breakpoint in force: what a child reads a track of. One column
    /// and one row where it states none.
    pub(crate) fn tracks(self, layout: Layout, short: Short) -> Tracks {
        let tree = self.tree;
        tree.grids
            .get(&tree.flags, self.slot)
            .map_or_else(Tracks::default, |grid| grid.tracks(layout, short))
    }
}

/// Which renderer reads `pigment`.
fn chlorophyll(pigment: &Pigment) -> Chlorophyll {
    match pigment {
        Pigment::Panel(_) => Chlorophyll::Panel,
        Pigment::Text(_) => Chlorophyll::Text,
        Pigment::Polygon(_) => Chlorophyll::Polygon,
        Pigment::Line(_) => Chlorophyll::Line,
        Pigment::Icon(_) => Chlorophyll::Icon,
        Pigment::Image(_) => Chlorophyll::Image,
    }
}
