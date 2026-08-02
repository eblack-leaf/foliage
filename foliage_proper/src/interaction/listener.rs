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
