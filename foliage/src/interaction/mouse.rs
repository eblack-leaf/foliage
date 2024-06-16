use crate::coordinate::position::Position;
use crate::coordinate::LogicalContext;
use crate::interaction::click::ClickInteraction;
use bevy_ecs::prelude::Resource;
use std::collections::HashMap;
use winit::event::{ElementState, MouseButton};

#[derive(Resource)]
pub(crate) struct MouseAdapter {
    cache: HashMap<MouseButton, ElementState>,
    pub(crate) cursor: Position<LogicalContext>,
}
impl MouseAdapter {
    pub(crate) fn cache_different(
        &mut self,
        button: MouseButton,
        state: ElementState,
    ) -> Option<ClickInteraction> {
        if let Some(cached) = self.cache.get_mut(&button) {
            if *cached != state {
                *cached = state;
                return Some(ClickInteraction::new(self.cursor));
            } else {
                return None;
            }
        } else {
            self.cache.insert(button, state);
            return Some(ClickInteraction::new(self.cursor));
        }
    }
}
