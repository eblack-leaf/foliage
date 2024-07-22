use crate::coordinate::section::Section;
use crate::coordinate::LogicalContext;
use crate::grid::GridPlacement;

pub(crate) struct PlacementAnimation {
    new: GridPlacement,
    ending_offset: Section<LogicalContext>,
}
pub struct Animation<A: Animate> {
    current: Option<A>,
    end: A,
}
pub trait Animate
where
    Self: Sized + Send + Sync + 'static,
{
    fn interpolations(start: Self, end: Self) -> ();
}
pub(crate) fn animate() {}
pub(crate) fn placement_animate() {}
