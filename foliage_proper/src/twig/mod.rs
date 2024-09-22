use crate::coordinate::elevation::Elevation;
use crate::grid::{ContextUnit, GridLocation};
use crate::leaf::LeafHandle;

pub mod button;
#[derive(Clone)]
pub struct Twig<T: Clone> {
    handle: LeafHandle,
    elevation: Elevation,
    location: GridLocation,
    t: T,
    stem: Option<LeafHandle>,
}
impl<T: Clone> Twig<T> {
    pub fn new(t: T) -> Self {
        Self {
            handle: Default::default(),
            elevation: Default::default(),
            location: GridLocation::new(),
            t,
            stem: None,
        }
    }
    pub fn named<LH: Into<LeafHandle>>(mut self, lh: LH) -> Self {
        self.handle = lh.into();
        self
    }
    pub fn elevation<E: Into<Elevation>>(mut self, e: E) -> Self {
        self.elevation = e.into();
        self
    }
    pub fn located<GL: Into<GridLocation>>(mut self, gl: GL) -> Self {
        self.location = gl.into();
        self
    }
    pub fn stem_from<LH: Into<LeafHandle>>(mut self, lh: LH) -> Self {
        self.stem = Some(lh.into());
        self
    }
}
