//! Elm -- change extraction.
//!
//! # Rowan recomputes. Elm decides what changed.
//!
//! Step 8, and the reason recomputing everything costs nothing downstream. Every renderer keeps what
//! the backend is holding and compares this frame's resolved values against it, so an element that
//! did not change costs one comparison and no upload.
//!
//! The cache is *what the backend holds*, not what the element last was. That is what makes a
//! skipped frame unable to lose a change: there is no flag to miss, only a value that still differs
//! at the next comparison. It is also a contract on the backend, which has to apply every batch it
//! is handed -- a dropped batch leaves the cache claiming something the backend does not have, and
//! nothing afterwards will correct it.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use bevy_ecs::component::Component;
use tracing::field::Empty;
use tracing::trace_span;

use crate::aspen::Departed;
use crate::color::Color;
use crate::coordinate::{Position, Section};
use crate::elevation::ResolvedElevation;
use crate::grove::Grove;
use crate::leaf::Leaf;
use crate::palette::Fill;
use crate::panel::PanelInstance;
use crate::rounding::Corners;
use crate::text::TextPigment;
use crate::text::font::Font;

/// Which renderer an element carries, and so which instances it is gathered into.
///
/// Decided when the element is described and never afterwards: what an element draws is part of
/// what it is, and one that is to draw something else is a different element. Nothing writes this
/// -- there is no op that can, and the only place it is set is where the element is grown.
///
/// Extraction walks the tree once and routes on this, so a further renderer costs a variant rather
/// than a pass over everything. It is the *only* thing that says what an element is: a set of
/// components that happens to look like a panel is not a panel.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Chlorophyll {
    /// Nothing. Carrying no renderer is the whole of what makes an element a [`Stem`](crate::Stem).
    #[default]
    None,
    /// A filled rectangle.
    Panel,
    /// A run of monospaced glyphs.
    Text,
}

/// What a renderer was told, for whichever renderer the element carries.
///
/// Carried by the [`Bud`](crate::op::Bud) as one value so that growing an element inserts the
/// pigment its own renderer reads and no other. Past that point each is an ordinary component and
/// nothing asks which kind it was.
pub(crate) enum Pigment {
    Panel(PanelPigment),
    Text(TextPigment),
}

/// What a panel is filled and shaped by: everything the panel renderer was told.
///
/// Grown alongside [`Chlorophyll::Panel`] and by nothing else, so an element carries both or
/// neither. Ordinary declared state, and what [`color`](crate::Grow::color) and
/// [`round`](crate::Grow::round) write -- the decision beside it stays untouched, because an
/// element does not stop being a panel by being repainted.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct PanelPigment {
    pub(crate) fill: Fill,
    pub(crate) rounding: Corners,
}

/// What the backend is holding, one set per renderer.
#[derive(Default)]
pub(crate) struct Elm {
    pub(crate) panels: Instances<PanelInstance>,
    pub(crate) texts: Runs,
    /// Where one run's glyphs are gathered before they are compared against what is held. Kept
    /// between frames for its capacity, and reused by every run in turn: a frame that changes
    /// nothing must not allocate.
    glyphs: Vec<Glyph>,
}

/// One glyph of a run: the cell it occupies, and which character occupies it.
///
/// The cell is in logical pixels and already offset by the run's own box, because where a character
/// lands is what wrapping decided and wrapping is the engine's. Where the *ink* sits inside that
/// cell is not here: that is the rasteriser's, which is the only thing that knows what shape it made
/// and at what density it made it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Glyph {
    pub(crate) cell: Section,
    pub(crate) character: char,
}

/// The runs, and every glyph of each.
///
/// Deliberately not an [`Instances`]: a run is **one** entry in the one stack whose renderer holds
/// many things under it, and that is the whole difference. What is diffed here is the run entire --
/// its glyphs, its fill, its rank and its clip -- because a run's glyphs move, refill and restack
/// together, and finding which of them changed would cost more than rewriting the run.
#[derive(Default)]
pub(crate) struct Runs {
    held: HashMap<Key, Run>,
    /// Runs the backend holds at something other than what is now wanted, or does not hold at all.
    /// Keys rather than values: what the backend is to apply is what is held for them, which it
    /// reads back at [`run`](Runs::run).
    pub(crate) written: Vec<Key>,
    /// Runs the backend holds and should not, in a stable order.
    pub(crate) withdrawn: Vec<Key>,
    /// Which extraction is running. An entry left at an older one is no longer wanted.
    pass: u64,
}

/// One run, as the backend is to hold it.
pub(crate) struct Run {
    /// Every glyph that leaves ink, in reading order. A space advances the wrap and is not one.
    pub(crate) glyphs: Vec<Glyph>,
    pub(crate) color: Color,
    /// Which face the glyphs are cut from, and at what size. The backend rasterises against these,
    /// so they are part of what the run is rather than of how it was measured.
    pub(crate) font: Font,
    pub(crate) size: u32,
    pub(crate) rank: ResolvedElevation,
    pub(crate) clip: Section,
    seen: u64,
}

impl Runs {
    /// Opens an extraction, dropping what the last one reported.
    fn open(&mut self) {
        self.written.clear();
        self.withdrawn.clear();
    }

    /// Takes one run as it now stands, and reports it written if the backend is holding it
    /// otherwise.
    #[allow(clippy::too_many_arguments)]
    fn want(
        &mut self,
        key: Key,
        rank: ResolvedElevation,
        clip: Section,
        color: Color,
        font: Font,
        size: u32,
        glyphs: &[Glyph],
    ) {
        let pass = self.pass;
        match self.held.entry(key) {
            Entry::Occupied(mut held) => {
                let held = held.get_mut();
                held.seen = pass;
                if held.color == color
                    && held.font == font
                    && held.size == size
                    && held.rank == rank
                    && held.clip == clip
                    && held.glyphs == glyphs
                {
                    return;
                }
                held.color = color;
                held.font = font;
                held.size = size;
                held.rank = rank;
                held.clip = clip;
                // Rewritten in place, so a run that changed costs its own glyphs and no allocation.
                held.glyphs.clear();
                held.glyphs.extend_from_slice(glyphs);
            }
            Entry::Vacant(slot) => {
                slot.insert(Run {
                    glyphs: glyphs.to_vec(),
                    color,
                    font,
                    size,
                    rank,
                    clip,
                    seen: pass,
                });
            }
        }
        self.written.push(key);
    }

    /// Closes the extraction: what nothing wanted this frame is withdrawn.
    fn extract(&mut self) {
        let pass = self.pass;
        let withdrawn = &mut self.withdrawn;
        self.held.retain(|key, held| {
            if held.seen == pass {
                return true;
            }
            withdrawn.push(*key);
            false
        });
        // Nothing may depend on the order a map iterates, and two identical runs have to extract
        // identically.
        withdrawn.sort();
        self.pass += 1;
    }

    /// What is held for one run, which is what the backend is to be holding for it.
    pub(crate) fn run(&self, key: Key) -> Option<&Run> {
        self.held.get(&key)
    }
}

/// What one instance is held under.
///
/// A plain number, and deliberately not a [`Leaf`]. An element is one of these -- its name's own
/// bits -- and so is a glyph inside a run, so a renderer whose element draws *many* things keeps its
/// own [`Instances`] under its own numbering and hands the one stack a single entry. Nothing about
/// holding, diffing or uploading has to know which of the two it is looking at.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(crate) struct Key(pub(crate) u64);

impl From<Leaf> for Key {
    fn from(leaf: Leaf) -> Self {
        Self(leaf.id())
    }
}

/// One renderer's instances: what the backend holds, and the difference the last extraction found.
///
/// Generic over the instance, because what a renderer sends is the renderer's own business. This
/// owns the comparison and knows nothing about what is being compared.
///
/// The rank is the one thing every renderer has in common, so it is carried here rather than
/// inside each renderer's instance: where an element sits in the one stack is a fact about the
/// element, not about what it happens to draw, and the backend needs it in a different form from
/// the one the resolver produced. Keeping it out is also what leaves an instance free to be
/// exactly the bytes a vertex buffer takes.
pub(crate) struct Instances<I> {
    held: HashMap<Key, Held<I>>,
    /// What should be drawn this frame, gathered before it is compared. Kept between frames for its
    /// capacity: a frame that changes nothing must not allocate.
    wanted: Vec<Stacked<I>>,
    /// Instances the backend does not hold, or holds at a different value or rank.
    pub(crate) written: Vec<Stacked<I>>,
    /// Instances the backend holds and should not, in a stable order.
    pub(crate) withdrawn: Vec<Key>,
    /// Which extraction is running. An entry left at an older one is no longer wanted.
    pass: u64,
}

/// One instance, where in the one stack it is to be drawn, and what it is clipped to.
///
/// The clip is beside the instance rather than inside it for the same reason the rank is: it is not
/// the renderer's data. It says where the backend is allowed to paint, which is a property of the
/// region the element sits in, and the backend applies it to the pass rather than to the panel.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Stacked<I> {
    pub(crate) key: Key,
    pub(crate) rank: ResolvedElevation,
    pub(crate) clip: Section,
    pub(crate) instance: I,
}

/// One instance the backend holds, and the extraction that last asked for it.
struct Held<I> {
    instance: I,
    rank: ResolvedElevation,
    clip: Section,
    seen: u64,
}

impl<I> Default for Instances<I> {
    fn default() -> Self {
        Self {
            held: HashMap::new(),
            wanted: Vec::new(),
            written: Vec::new(),
            withdrawn: Vec::new(),
            pass: 0,
        }
    }
}

impl<I: Copy + PartialEq> Instances<I> {
    /// Adds one instance to what should be drawn this frame, at the rank it resolved to and inside
    /// the clip it resolved under.
    fn want(&mut self, key: impl Into<Key>, rank: ResolvedElevation, clip: Section, instance: I) {
        self.wanted.push(Stacked {
            key: key.into(),
            rank,
            clip,
            instance,
        });
    }

    /// Diffs what should be drawn against what the backend holds, and takes the result as the new
    /// holding.
    ///
    /// The holding is updated in place. Rebuilding it would allocate a map per renderer per frame,
    /// which is a cost on every frame including the ones with nothing in them -- and an unchanged
    /// frame costing nothing is the whole claim of this phase.
    fn extract(&mut self) {
        self.written.clear();
        self.withdrawn.clear();
        self.pass += 1;
        let pass = self.pass;
        for index in 0..self.wanted.len() {
            let wanted = self.wanted[index];
            match self.held.entry(wanted.key) {
                Entry::Occupied(mut held) => {
                    let held = held.get_mut();
                    if held.instance != wanted.instance
                        || held.rank != wanted.rank
                        || held.clip != wanted.clip
                    {
                        held.instance = wanted.instance;
                        held.rank = wanted.rank;
                        held.clip = wanted.clip;
                        self.written.push(wanted);
                    }
                    held.seen = pass;
                }
                Entry::Vacant(slot) => {
                    slot.insert(Held {
                        instance: wanted.instance,
                        rank: wanted.rank,
                        clip: wanted.clip,
                        seen: pass,
                    });
                    self.written.push(wanted);
                }
            }
        }
        self.wanted.clear();
        let withdrawn = &mut self.withdrawn;
        self.held.retain(|key, held| {
            if held.seen == pass {
                return true;
            }
            withdrawn.push(*key);
            false
        });
        // Nothing may depend on the order a map iterates, and two identical runs have to extract
        // identically.
        withdrawn.sort();
    }

    /// How many instances the backend is holding.
    pub(crate) fn len(&self) -> usize {
        self.held.len()
    }

    /// What the backend is holding for `key`.
    pub(crate) fn holding(&self, key: impl Into<Key>) -> Option<I> {
        self.held.get(&key.into()).map(|held| held.instance)
    }
}

/// Step 8. Resolved state becomes instances, and only where it differs from what is already drawn.
pub(crate) fn run(grove: &mut Grove) {
    let step = trace_span!("extract", written = Empty, withdrawn = Empty, glyphs = Empty);
    let _entered = step.enter();
    grove.elm.texts.open();
    // Detached for the walk so that gathering a run's glyphs -- which reads the shaping cache -- and
    // handing them over -- which writes what is held -- are not the same borrow. It goes back below,
    // with whatever capacity the widest run this frame gave it.
    let mut glyphs = core::mem::take(&mut grove.elm.glyphs);
    let mut total = 0;
    for leaf in grove.tree.leaves() {
        let chlorophyll = grove.tree.chlorophyll(leaf);
        if chlorophyll == Chlorophyll::None {
            continue;
        }
        let Some(painted) = painted(grove, leaf) else {
            continue;
        };
        match chlorophyll {
            // Answered above: carrying no renderer is the whole of what makes an element a stem.
            Chlorophyll::None => {}
            Chlorophyll::Panel => {
                // Grown together and by nothing else, so a panel always has one.
                let Some(pigment) = grove.tree.panel_pigment(leaf) else {
                    continue;
                };
                let instance = PanelInstance::new(
                    painted.section,
                    tint(grove, leaf, pigment.fill).faded(painted.opacity),
                    pigment.rounding,
                );
                grove
                    .elm
                    .panels
                    .want(leaf, grove.tree.rank(leaf), painted.clip, instance);
            }
            // A run's box, fill, rank and clip resolve exactly as a panel's do. What is different is
            // that it draws *many* things at one rank: it is one entry in the one stack, and its
            // renderer holds its glyphs under its own numbering -- which is what a [`Key`] is a
            // number rather than a [`Leaf`] for.
            Chlorophyll::Text => {
                // Grown together and by nothing else, so a run always has both.
                let (Some(pigment), Some(typeface)) =
                    (grove.tree.text_pigment(leaf), grove.tree.typeface(leaf))
                else {
                    continue;
                };
                let color = tint(grove, leaf, pigment.fill).faded(painted.opacity);
                let size = typeface.size.at(grove.layout, grove.short);
                let Some(value) = grove.tree.lettering(leaf) else {
                    continue;
                };
                // R1 shapes every run that is measured at all, so one that is not held is one
                // nothing is laying out. Extraction reads that cache and never adds to it.
                let Some(shaped) = grove.shaping.shaped(typeface.font, size, value) else {
                    continue;
                };
                let origin = painted.section.position;
                let cell = shaped.cell();
                glyphs.clear();
                // Wrapped at the width the run resolved to, which is the width it was measured at.
                shaped.place(painted.section.width(), |character, at| {
                    glyphs.push(Glyph {
                        cell: Section::new(origin.moved(at), cell),
                        character,
                    });
                });
                total += glyphs.len();
                grove.elm.texts.want(
                    leaf.into(),
                    grove.tree.rank(leaf),
                    painted.clip,
                    color,
                    typeface.font,
                    size,
                    &glyphs,
                );
            }
        }
    }
    grove.elm.glyphs = glyphs;
    grove.elm.panels.extract();
    grove.elm.texts.extract();
    step.record(
        "written",
        grove.elm.panels.written.len() + grove.elm.texts.written.len(),
    );
    step.record(
        "withdrawn",
        grove.elm.panels.withdrawn.len() + grove.elm.texts.withdrawn.len(),
    );
    step.record("glyphs", total);
}

/// Where an element is painted, and inside what.
struct Painted {
    section: Section,
    clip: Section,
    opacity: f32,
}

/// What is painted of `leaf`, or `None` if nothing is.
///
/// Hidden is the app's intent and culled is this pass's decision, taken here from the clip rect and
/// recorded nowhere: an element scrolled out of its region is absent from the batch and unchanged in
/// every other respect, so scrolling back to it needs nothing to be undone.
fn painted(grove: &Grove, leaf: Leaf) -> Option<Painted> {
    let inherited = grove.tree.inherited(leaf);
    let section = grove.tree.drawn(leaf);
    // What a scrolling ancestor leaves visible, never wider than the surface: a clip is what the
    // backend scissors the pass to, and nothing outside the surface is painted whatever the rect
    // says.
    let clip = grove
        .tree
        .clip(leaf)
        .intersect(Section::new(Position::default(), grove.viewport));
    if !inherited.visible || section.intersect(clip).is_empty() {
        return None;
    }
    Some(Painted {
        section,
        clip,
        opacity: inherited.opacity,
    })
}

/// What a fill currently paints as.
///
/// A blend of two fills is a color rather than a fill, so a motion on one is applied here -- where a
/// fill becomes a color -- and not written onto the element. Both ends are read against the same
/// scheme at the same instant, so a repaint mid-motion moves whichever of them is a role.
fn tint(grove: &Grove, leaf: Leaf, fill: Fill) -> Color {
    let target = fill.color(&grove.scheme);
    match grove.aspen.fill(leaf) {
        Some((Departed::Declared(fill), at)) => fill.color(&grove.scheme).blend(target, at),
        Some((Departed::Snapshot(color), at)) => color.blend(target, at),
        None => target,
    }
}
