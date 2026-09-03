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
use crate::coordinate::{Position, Section};
use crate::elevation::ResolvedElevation;
use crate::grove::Grove;
use crate::leaf::Leaf;
use crate::palette::Fill;
use crate::panel::PanelInstance;
use crate::rounding::Corners;

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
    held: HashMap<Leaf, Held<I>>,
    /// What should be drawn this frame, gathered before it is compared. Kept between frames for its
    /// capacity: a frame that changes nothing must not allocate.
    wanted: Vec<Stacked<I>>,
    /// Instances the backend does not hold, or holds at a different value or rank.
    pub(crate) written: Vec<Stacked<I>>,
    /// Instances the backend holds and should not, in a stable order.
    pub(crate) withdrawn: Vec<Leaf>,
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
    pub(crate) leaf: Leaf,
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
    fn want(&mut self, leaf: Leaf, rank: ResolvedElevation, clip: Section, instance: I) {
        self.wanted.push(Stacked {
            leaf,
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
            match self.held.entry(wanted.leaf) {
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
        self.held.retain(|leaf, held| {
            if held.seen == pass {
                return true;
            }
            withdrawn.push(*leaf);
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

    /// What the backend is holding for `leaf`.
    pub(crate) fn holding(&self, leaf: Leaf) -> Option<I> {
        self.held.get(&leaf).map(|held| held.instance)
    }
}

/// Step 8. Resolved state becomes instances, and only where it differs from what is already drawn.
pub(crate) fn run(grove: &mut Grove) {
    let step = trace_span!("extract", written = Empty, withdrawn = Empty);
    let _entered = step.enter();
    for leaf in grove.tree.leaves() {
        match grove.tree.chlorophyll(leaf) {
            Chlorophyll::None => {}
            Chlorophyll::Panel => {
                // Grown together and by nothing else, so a panel always has one.
                let Some(pigment) = grove.tree.pigment(leaf) else {
                    continue;
                };
                let inherited = grove.tree.inherited(leaf);
                let section = grove.tree.drawn(leaf);
                // What a scrolling ancestor leaves visible, never wider than the surface: a clip is
                // what the backend scissors the pass to, and nothing outside the surface is painted
                // whatever the rect says.
                let clip = grove
                    .tree
                    .clip(leaf)
                    .intersect(Section::new(Position::default(), grove.viewport));
                // Hidden is the app's intent and culled is this pass's decision, taken here from
                // the clip rect and recorded nowhere: an element scrolled out of its region is
                // absent from the batch and unchanged in every other respect, so scrolling back to
                // it needs nothing to be undone.
                if !inherited.visible || section.intersect(clip).is_empty() {
                    continue;
                }
                // A blend of two fills is a color rather than a fill, so a motion on one is applied
                // here -- where a fill becomes a color -- and not written onto the element. Both
                // ends are read against the same scheme at the same instant, so a repaint
                // mid-motion moves whichever of them is a role.
                let target = pigment.fill.color(&grove.scheme);
                let color = match grove.aspen.fill(leaf) {
                    Some((Departed::Declared(fill), at)) => {
                        fill.color(&grove.scheme).blend(target, at)
                    }
                    Some((Departed::Snapshot(color), at)) => color.blend(target, at),
                    None => target,
                };
                let instance =
                    PanelInstance::new(section, color.faded(inherited.opacity), pigment.rounding);
                grove
                    .elm
                    .panels
                    .want(leaf, grove.tree.rank(leaf), clip, instance);
            }
        }
    }
    grove.elm.panels.extract();
    step.record("written", grove.elm.panels.written.len());
    step.record("withdrawn", grove.elm.panels.withdrawn.len());
}
