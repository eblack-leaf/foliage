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

use crate::grove::Grove;
use crate::leaf::Leaf;
use crate::palette::Palette;
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

/// What a panel is coloured by: everything the panel renderer was told.
///
/// Grown alongside [`Chlorophyll::Panel`] and by nothing else, so an element carries both or
/// neither. Ordinary declared state, and what [`color`](crate::Grow::color) and
/// [`round`](crate::Grow::round) write -- the decision beside it stays untouched, because an
/// element does not stop being a panel by being repainted.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PanelPigment {
    pub(crate) color: Palette,
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
pub(crate) struct Instances<I> {
    held: HashMap<Leaf, Held<I>>,
    /// What should be drawn this frame, gathered before it is compared. Kept between frames for its
    /// capacity: a frame that changes nothing must not allocate.
    wanted: Vec<(Leaf, I)>,
    /// Instances the backend does not hold, or holds at a different value.
    pub(crate) written: Vec<(Leaf, I)>,
    /// Instances the backend holds and should not, in a stable order.
    pub(crate) withdrawn: Vec<Leaf>,
    /// Which extraction is running. An entry left at an older one is no longer wanted.
    pass: u64,
}

/// One instance the backend holds, and the extraction that last asked for it.
struct Held<I> {
    instance: I,
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
    /// Adds one instance to what should be drawn this frame.
    fn want(&mut self, leaf: Leaf, instance: I) {
        self.wanted.push((leaf, instance));
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
            let (leaf, instance) = self.wanted[index];
            match self.held.entry(leaf) {
                Entry::Occupied(mut held) => {
                    let held = held.get_mut();
                    if held.instance != instance {
                        held.instance = instance;
                        self.written.push((leaf, instance));
                    }
                    held.seen = pass;
                }
                Entry::Vacant(slot) => {
                    slot.insert(Held {
                        instance,
                        seen: pass,
                    });
                    self.written.push((leaf, instance));
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
                let instance = PanelInstance::new(
                    grove.tree.drawn(leaf),
                    grove.scheme.color(pigment.color),
                    pigment.rounding,
                    grove.tree.rank(leaf),
                );
                grove.elm.panels.want(leaf, instance);
            }
        }
    }
    grove.elm.panels.extract();
    step.record("written", grove.elm.panels.written.len());
    step.record("withdrawn", grove.elm.panels.withdrawn.len());
}
