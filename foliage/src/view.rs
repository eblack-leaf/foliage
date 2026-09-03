//! Scrolling regions -- what a gesture hands itself to.
//!
//! An element scrolls because it said so. A grid divides an element's box for its children and says
//! nothing about scrolling, and an axis that was not declared does not scroll, has no extent, and
//! cannot be moved: it is not a scrolling axis with a range of zero, which is a different and
//! simpler thing to reason about.
//!
//! Three values, each written by one pass:
//!
//! | | Pass | Holds |
//! |---|---|---|
//! | [`Extent`] | R3, bottom-up | how far the content reaches from the region's own near edges |
//! | [`Offset`] | R4, top-down | how far the region has been moved, clamped to what it can reach |
//! | [`Clipped`] | R5, top-down | the box a scrolling ancestor leaves visible |
//!
//! What is here is what a gesture needs of a region: where it can still move, and what moving it
//! does to everything inside it.

use bevy_ecs::component::Component;

use crate::coordinate::{Area, Axes, Axis, Position, Section};

/// Which axes an element scrolls.
///
/// Absent, it does not scroll, and a drag inside it goes on outward to something that does.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Scrolls(pub(crate) Axes);

/// How far a region has been moved from its content's origin, in logical pixels.
///
/// One unit throughout, read and written the same way. Positive is content moved toward the near
/// edge -- what a drag away from that edge produces.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Offset(pub(crate) Position);

/// How far a region's content reaches, measured outward from the region's own near edges.
///
/// Derived from where the children landed, never from what is currently drawn: a child scrolled out
/// of sight is the content the extent exists to describe, so culling has no say in it. Never
/// smaller than the region's own box, so an empty region has a range of zero rather than a negative
/// one.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Extent(pub(crate) Area);

/// The box a scrolling ancestor leaves visible: the intersection of every one of them.
///
/// A rect, and only a rect. Whether an element is *culled* is a decision extraction makes by
/// comparing this against the element's box, and is never recorded on the element -- which is what
/// keeps culling out of anything that reads state, extent first among them.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub(crate) struct Clipped(pub(crate) Section);

impl Clipped {
    /// Nothing clipped: what an element with no scrolling ancestor carries.
    pub(crate) fn unbounded() -> Self {
        Self(Section::from_edges(
            f32::MIN / 2.0,
            f32::MIN / 2.0,
            f32::MAX / 2.0,
            f32::MAX / 2.0,
        ))
    }
}

impl Default for Clipped {
    fn default() -> Self {
        Self::unbounded()
    }
}

/// How far a region can be moved on one axis: what its content reaches, less the box it is seen
/// through.
pub(crate) fn range(extent: Area, own: Area, axis: Axis) -> f32 {
    let (reach, seen) = match axis {
        Axis::Horizontal => (extent.width, own.width),
        Axis::Vertical => (extent.height, own.height),
    };
    (reach - seen).max(0.0)
}

/// How much of `delta` a region sitting at `offset` can still take.
///
/// The whole of what "can no longer consume" means, and the reason it needs no flag: a region at its
/// end returns zero for a delta going further out and the full delta for one coming back, so the
/// answer is about where the region currently sits rather than about what it was told at spawn.
pub(crate) fn consumable(offset: f32, range: f32, delta: f32) -> f32 {
    (offset + delta).clamp(0.0, range) - offset
}
