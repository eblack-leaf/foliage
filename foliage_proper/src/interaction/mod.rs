use crate::coordinate::position::Position;
use crate::coordinate::LogicalContext;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::{Component, IntoSystemConfigs};
use bevy_ecs::system::Resource;
mod adapter;
use crate::{Attachment, Foliage};
pub use adapter::InputSequence;
pub(crate) use adapter::{KeyboardAdapter, MouseAdapter, TouchAdapter};

impl Attachment for Interaction {
    fn attach(foliage: &mut Foliage) {
        foliage.world.insert_resource(KeyboardAdapter::default());
        foliage.world.insert_resource(MouseAdapter::default());
        foliage.world.insert_resource(TouchAdapter::default());
    }
}
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum InteractionPhase {
    Start,
    Moved,
    End,
    Cancel,
}
#[derive(Event, Debug, Copy, Clone)]
pub struct Interaction {
    click_phase: InteractionPhase,
    position: Position<LogicalContext>,
    from_scroll: bool,
}
impl Interaction {
    pub fn new(
        click_phase: InteractionPhase,
        position: Position<LogicalContext>,
        from_scroll: bool,
    ) -> Self {
        Self {
            click_phase,
            position,
            from_scroll,
        }
    }
}
#[derive(Default, Copy, Clone, Debug)]
pub struct Click {
    pub start: Position<LogicalContext>,
    pub current: Position<LogicalContext>,
    pub end: Option<Position<LogicalContext>>,
}
impl Click {
    pub fn new(start: Position<LogicalContext>) -> Self {
        Self {
            start,
            current: start,
            end: None,
        }
    }
}
#[derive(Resource, Default)]
pub(crate) struct InteractiveEntity(pub(crate) Option<Entity>);
#[derive(Resource, Default)]
pub(crate) struct FocusedEntity(pub(crate) Option<Entity>);
#[derive(Resource, Default)]
pub(crate) struct PassThroughInteractions {
    ps: Vec<Entity>,
}
