use crate::place::{Placement, Places};

/// An element that draws nothing.
///
/// It holds children, takes hits, and has a box. Carrying no renderer is the whole of what makes it
/// one, so it is the element to reach for whenever structure is wanted and pixels are not.
#[derive(Clone, Debug, Default)]
pub struct Stem {
    pub(crate) placement: Placement,
}

impl Stem {
    /// An element that draws nothing, to be placed and branched off like any other.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Places for Stem {
    fn placement(&mut self) -> &mut Placement {
        &mut self.placement
    }
}
