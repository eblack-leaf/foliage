use crate::coordinate::Area;
use crate::leaf::Leaf;
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

/// What the tree put out this frame.
///
/// A set you interrogate, not a list you walk: ask it about the elements you own. There is no
/// order to read, apart from the sequences that are ordered in their own right.
#[derive(Clone, Default)]
pub struct Pollen(Arc<Drift>);

impl Pollen {
    /// Whether `leaf` was taken down. Reported once, in the frame after it went.
    pub fn withered(&self, leaf: Leaf) -> bool {
        self.0.withered.contains(&leaf)
    }

    /// The surface's new size, if it changed this frame.
    pub fn resized(&self) -> Option<Area> {
        self.0.resized
    }

    pub(crate) fn seal(drift: Drift) -> Self {
        Self(Arc::new(drift))
    }
}

impl fmt::Debug for Pollen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// What the frame is collecting, before it is sealed into a [`Pollen`] and handed over.
#[derive(Default, Debug)]
pub(crate) struct Drift {
    pub(crate) withered: HashSet<Leaf>,
    pub(crate) resized: Option<Area>,
}
