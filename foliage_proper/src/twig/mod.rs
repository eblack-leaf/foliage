use crate::coordinate::elevation::Elevation;
use crate::tree::Tree;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;

pub mod button;
pub struct Twig<T> {
    pub elevation: Elevation,
    pub location: GridLocation,
    pub t: T,
    pub stem: Option<Entity>,
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
    pub fn location<GL: Into<GridLocation>>(mut self, gl: GL) -> Self {
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
