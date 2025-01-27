use crate::ash::clip::ResolvedClip;
use crate::interaction::{Click, CurrentInteraction};
use crate::{Component, CoordinateUnit, Logical, Position, Section};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::DeferredWorld;
use bitflags::bitflags;

#[derive(Component, Copy, Clone)]
#[component(on_replace = Self::on_replace)]
pub struct InteractionListener {
    pub(crate) click: Click,
    pub(crate) scroll: bool,
    pub(crate) pass_through: bool,
    pub(crate) shape: InteractionShape,
    pub(crate) last_drag: Position<Logical>,
    pub(crate) state: InteractionState,
}

impl Default for InteractionListener {
    fn default() -> Self {
        Self::new()
    }
}

impl InteractionListener {
    pub const DRAG_THRESHOLD: CoordinateUnit = 40.0;
    pub fn new() -> Self {
        Self {
            click: Default::default(),
            scroll: false,
            pass_through: false,
            shape: Default::default(),
            last_drag: Default::default(),
            state: Default::default(),
        }
    }
    pub fn circle(mut self) -> Self {
        self.shape = InteractionShape::Circle;
        self
    }
    pub fn scroll(mut self, s: bool) -> Self {
        self.pass_through = s;
        self.scroll = s;
        self
    }
    pub fn click(&self) -> Click {
        self.click
    }
    pub fn disabled(&self) -> bool {
        !(self.state.contains(InteractionState::ENABLED)
            && self.state.contains(InteractionState::AUTO_ENABLED)
            && self.state.contains(InteractionState::INHERIT_ENABLED))
    }
    pub(crate) fn is_contained(
        &self,
        section: Section<Logical>,
        clip: ResolvedClip,
        event: Position<Logical>,
    ) -> bool {
        let section_contained = match self.shape {
            InteractionShape::Rectangle => section.contains(event),
            InteractionShape::Circle => section.center().distance(event) <= section.width() / 2f32,
        };
        let clip_contained = clip.0.contains(event);
        section_contained && clip_contained
    }
    fn on_replace(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        if let Some(mut current) = world.get_resource_mut::<CurrentInteraction>() {
            if let Some(p) = current.primary {
                if p == this {
                    current.primary.take();
                }
            }
        }
    }
}

#[derive(Copy, Clone, Default)]
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
