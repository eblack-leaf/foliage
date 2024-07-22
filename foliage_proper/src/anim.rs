use crate::coordinate::section::Section;
use crate::coordinate::LogicalContext;
use crate::grid::GridPlacement;

// when make => get current section of entity + set in new.queued_offset
// read ending-offset from new.offset (default == 0/0/0/0)
//
pub(crate) struct PlacementAnimation {
    new: GridPlacement,
    ending_offset: Section<LogicalContext>, // read from new grid-placement
}
pub struct Animation<A: Animate> {
    start: Option<A>,
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
