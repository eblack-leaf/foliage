use crate::coordinate::elevation::Elevation;
use crate::coordinate::points::Points;
use crate::coordinate::section::Section;
use crate::coordinate::LogicalContext;
use crate::grid::responsive::{ResponsiveLocation, ResponsivePoints};
use crate::tree::Tree;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;

pub mod button;

#[derive(Event, Copy, Clone, Default)]
pub struct Configure {}
pub struct Twig<T> {
    pub elevation: Elevation,
    pub res: ResponsiveLocation,
    pub section: Section<LogicalContext>,
    pub pts: Option<Points<LogicalContext>>,
    pub res_pts: Option<ResponsivePoints>,
    pub t: T,
    pub stem: Option<Entity>,
}
impl<T> Twig<T> {
    pub fn new(t: T) -> Self {
        Self {
            elevation: Default::default(),
            res: ResponsiveLocation::new(),
            section: Default::default(),
            pts: None,
            res_pts: None,
            t,
            stem: None,
        }
    }
    pub fn elevation<E: Into<Elevation>>(mut self, e: E) -> Self {
        self.elevation = e.into();
        self
    }
    pub fn stem_from(mut self, lh: Entity) -> Self {
        self.stem = Some(lh);
        self
    }
    pub fn responsive(mut self, res_loc: ResponsiveLocation) -> Self {
        self.res = res_loc;
        self
    }
    pub fn responsive_points(mut self, res_pts: ResponsivePoints) -> Self {
        self.res_pts = Some(res_pts);
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