use crate::ash::clip::ResolvedClip;
use crate::{Component, CoordinateUnit, Logical, Position, Section};
use bitflags::bitflags;

/// Makes an entity a hit target: pointer and touch events resolve to it, and it can fire
/// [`Engaged`](crate::Engaged)/[`Dragged`](crate::Dragged)/[`OnClick`](crate::OnClick).
///
/// Without one an entity is drawn but never grabbed, and events pass through to whatever
/// is beneath. When several listeners cover the same point the topmost by elevation wins.
///
/// Being enabled is the conjunction of three independent flags -- see
/// [`InteractionState`] -- so a subtree can be disabled wholesale without clobbering an
/// entity's own setting.
#[derive(Component, Copy, Clone)]
pub struct InteractionListener {
    pub(crate) state: InteractionState,
}

impl Default for InteractionListener {
    fn default() -> Self {
        Self::new()
    }
}

impl InteractionListener {
    /// Logical pixels the pointer must travel before a press is read as a drag rather
    /// than a click. Below it, releasing still counts as a click on the grabbed entity.
    pub const DRAG_THRESHOLD: CoordinateUnit = 10.0;
    /// A listener, enabled.
    pub fn new() -> Self {
        Self {
            state: Default::default(),
        }
    }
    /// Whether this listener currently ignores input -- true if *any* of its three
    /// enable flags is clear.
    pub fn disabled(&self) -> bool {
        !(self.state.contains(InteractionState::ENABLED)
            && self.state.contains(InteractionState::AUTO_ENABLED)
            && self.state.contains(InteractionState::INHERIT_ENABLED))
    }
    /// Whether `event` hits this entity: inside its shape *and* inside its resolved clip,
    /// so content scrolled out of a view is not grabbable where it would have been drawn.
    pub(crate) fn is_contained(
        shape: InteractionShape,
        section: Section<Logical>,
        clip: ResolvedClip,
        event: Position<Logical>,
    ) -> bool {
        let section_contained = match shape {
            InteractionShape::Rectangle => section.contains(event),
            InteractionShape::Circle => section.center().distance(event) <= section.width() / 2f32,
        };
        let clip_contained = clip.0.contains(event);
        section_contained && clip_contained
    }
}

/// The shape an entity is hit-tested against, independent of what it draws.
///
/// [`Rounding::Full`](crate::Rounding::Full) sets `Circle` automatically, so a pill or dot
/// only responds where it appears solid.
#[derive(Component, Copy, Clone, Default)]
pub enum InteractionShape {
    /// The entity's whole `Section`.
    #[default]
    Rectangle,
    /// A circle inscribed in the `Section`, using its width as the diameter.
    Circle,
}
/// Three independent reasons a listener may be off, so none can overwrite another:
/// `ENABLED` is the author's own switch, `AUTO_ENABLED` the engine's (cleared while the
/// entity has no resolvable box), and `INHERIT_ENABLED` an ancestor's. Input is accepted
/// only when all three are set.
#[derive(Copy, Clone)]
pub struct InteractionState(u8);
impl Default for InteractionState {
    fn default() -> Self {
        Self::ENABLED | Self::AUTO_ENABLED | Self::INHERIT_ENABLED
    }
}
bitflags! {
    impl InteractionState: u8 {
        const ENABLED = 1 << 0;
        const AUTO_ENABLED = 1 << 1;
        const INHERIT_ENABLED = 1 << 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ash::clip::ResolvedClip;

    fn section(left: f32, top: f32, width: f32, height: f32) -> Section<Logical> {
        Section::new((left, top), (width, height))
    }
    fn unclipped(section: Section<Logical>) -> ResolvedClip {
        ResolvedClip(section)
    }
    fn at(x: f32, y: f32) -> Position<Logical> {
        Position::logical((x, y))
    }

    #[test]
    fn rectangle_shape_is_a_plain_bounds_check() {
        let bounds = section(0.0, 0.0, 100.0, 50.0);
        let clip = unclipped(bounds);
        assert!(InteractionListener::is_contained(
            InteractionShape::Rectangle,
            bounds,
            clip,
            at(50.0, 25.0),
        ));
        assert!(!InteractionListener::is_contained(
            InteractionShape::Rectangle,
            bounds,
            clip,
            at(150.0, 25.0),
        ));
    }

    #[test]
    fn circle_shape_uses_distance_from_center_not_the_bounding_box() {
        // a 100x100 box's circle has radius 50 -- a point in the box's corner is well
        // within the *rectangular* bounds but outside the *circular* hit area, which is
        // exactly the distinction `InteractionShape::Circle` exists to make (round
        // buttons/knobs shouldn't be clickable in their square corners).
        let bounds = section(0.0, 0.0, 100.0, 100.0);
        let clip = unclipped(bounds);
        let corner = at(5.0, 5.0);
        assert!(
            InteractionListener::is_contained(InteractionShape::Rectangle, bounds, clip, corner),
            "sanity: the corner is inside the rectangular bounds"
        );
        assert!(
            !InteractionListener::is_contained(InteractionShape::Circle, bounds, clip, corner),
            "but outside the circle inscribed in those same bounds"
        );
        assert!(InteractionListener::is_contained(
            InteractionShape::Circle,
            bounds,
            clip,
            at(50.0, 50.0), // dead center -- inside any shape
        ));
    }

    #[test]
    fn a_point_inside_the_section_but_outside_the_clip_is_not_contained() {
        // clip is narrower than the section itself -- e.g. a scrolled child peeking past
        // its parent View's visible edge. The point is genuinely inside the entity's own
        // section, but the clip (an ancestor's visible bounds) says it's cut off.
        let section_bounds = section(0.0, 0.0, 100.0, 100.0);
        let clip = unclipped(section(0.0, 0.0, 40.0, 100.0));
        assert!(!InteractionListener::is_contained(
            InteractionShape::Rectangle,
            section_bounds,
            clip,
            at(70.0, 50.0),
        ));
    }

    #[test]
    fn disabled_requires_all_three_enable_flags_together() {
        let mut listener = InteractionListener::new();
        assert!(!listener.disabled(), "default state is fully enabled");

        listener.state.remove(InteractionState::AUTO_ENABLED);
        assert!(
            listener.disabled(),
            "ENABLED and INHERIT_ENABLED alone aren't enough -- it's a conjunction of all three"
        );
    }
}
