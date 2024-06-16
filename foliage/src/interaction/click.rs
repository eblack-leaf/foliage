use bevy_ecs::event::Event;
use bevy_ecs::system::Resource;
use winit::event::{Touch, TouchPhase};

use crate::coordinate::position::Position;
use crate::coordinate::LogicalContext;
use crate::ginkgo::ScaleFactor;
#[derive(Event, Copy, Clone)]
pub struct CancelInteraction {}
#[derive(Event, Copy, Clone, Default)]
pub struct ClickInteraction {
    start: Position<LogicalContext>,
    current: Position<LogicalContext>,
    end: Option<Position<LogicalContext>>,
}
impl ClickInteraction {
    pub fn new(start: Position<LogicalContext>) -> Self {
        Self {
            start,
            current: start,
            end: None,
        }
    }
    pub(crate) fn with_current(mut self, p: Position<LogicalContext>) -> Self {
        self.current = p;
        self
    }
    pub(crate) fn ended_at(mut self, p: Position<LogicalContext>) -> Self {
        self.end.replace(p);
        self
    }
    pub fn start(&self) -> Position<LogicalContext> {
        self.start
    }
    pub fn current(&self) -> Position<LogicalContext> {
        self.current
    }
    pub fn end(&self) -> Option<Position<LogicalContext>> {
        self.end
    }
}
#[derive(Resource)]
pub(crate) struct TouchAdapter {
    primary: Option<TouchIdentifier>,
    primary_interaction: ClickInteraction,
}
impl TouchAdapter {
    pub(crate) fn read_touch(
        &mut self,
        touch: Touch,
        scale_factor: &ScaleFactor,
    ) -> (Option<ClickInteraction>, Option<CancelInteraction>) {
        let position =
            Position::device((touch.location.x, touch.location.y)).to_logical(scale_factor.value());
        if self.primary.is_none() {
            self.primary.replace(TouchIdentifier::new(touch.id));
            let interaction = ClickInteraction::new(position);
            self.primary_interaction = interaction;
            return (Some(interaction), None);
        } else {
            if touch.id != self.primary.unwrap().id {
                return (None, None);
            } else {
                match touch.phase {
                    TouchPhase::Started => (None, None),
                    TouchPhase::Moved => {
                        return (Some(self.primary_interaction.with_current(position)), None)
                    }
                    TouchPhase::Ended => {
                        return (Some(self.primary_interaction.ended_at(position)), None)
                    }
                    TouchPhase::Cancelled => {
                        self.primary.take();
                        return (None, Some(CancelInteraction {}));
                    }
                }
            }
        }
    }
}
#[derive(Copy, Clone)]
pub(crate) struct TouchIdentifier {
    id: u64,
}
impl TouchIdentifier {
    pub(crate) fn new(id: u64) -> Self {
        Self { id }
    }
}
