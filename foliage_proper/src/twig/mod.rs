use crate::coordinate::elevation::Elevation;
use crate::grid::GridLocation;
use crate::tree::Tree;
use bevy_ecs::entity::Entity;

pub mod button;
pub struct Twig<T> {
    elevation: Elevation,
    location: GridLocation,
    t: T,
    stem: Option<Entity>,
}
impl<T> Twig<T> {
    pub fn new(t: T) -> Self {
        Self {
            elevation: Default::default(),
            location: GridLocation::new(),
            t,
            stem: None,
        }
    }
    pub fn elevation<E: Into<Elevation>>(mut self, e: E) -> Self {
        self.elevation = e.into();
        self
    }
    pub fn located<GL: Into<GridLocation>>(mut self, gl: GL) -> Self {
        self.location = gl.into();
        self
    }
    pub fn stem_from(mut self, lh: Entity) -> Self {
        self.stem = Some(lh);
        self
    }
}

pub trait Branch
where
    Self: Sized,
{
    type Handle;
    fn grow(twig: Twig<Self>, tree: &mut Tree) -> Self::Handle;
}
