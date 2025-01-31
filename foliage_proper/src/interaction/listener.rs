use crate::ash::clip::ResolvedClip;
use crate::{Component, CoordinateUnit, Logical, Position, Section};
use bitflags::bitflags;

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
    pub const DRAG_THRESHOLD: CoordinateUnit = 10.0;
    pub fn new() -> Self {
        Self {
            state: Default::default(),
        }
    }
    pub fn disabled(&self) -> bool {
        !(self.state.contains(InteractionState::ENABLED)
            && self.state.contains(InteractionState::AUTO_ENABLED)
            && self.state.contains(InteractionState::INHERIT_ENABLED))
    }
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

#[derive(Component, Copy, Clone, Default)]
pub enum InteractionShape {
    #[default]
    Rectangle,
    Circle,
}
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
