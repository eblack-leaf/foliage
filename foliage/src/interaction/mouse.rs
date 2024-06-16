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
    current: Option<ClickInteraction>,
}
impl MouseAdapter {
    pub(crate) fn cache_different(
        &mut self,
        button: MouseButton,
        state: ElementState,
    ) -> Option<ClickInteraction> {
        if button != MouseButton::Left {
            return None;
        }
        return if let Some(cached) = self.cache.get_mut(&button) {
            if *cached != state {
                let interaction = if state.is_pressed() {
                    let i = ClickInteraction::new(self.cursor);
                    self.current.replace(i);
                    i
                } else {
                    self.current
                        .take()
                        .unwrap_or_default()
                        .ended_at(self.cursor)
                };
                *cached = state;
                Some(interaction)
            } else {
                None
            }
        } else {
            return if state.is_pressed() {
                self.cache.insert(button, state);
                let interaction = ClickInteraction::new(self.cursor);
                self.current.replace(interaction);
                Some(interaction)
            } else {
                None
            };
        };
    }
    pub(crate) fn set_cursor(&mut self, c: Position<LogicalContext>) -> Option<ClickInteraction> {
        if let Some(current) = self.current {
            Some(current.with_current(c))
        } else {
            None
        }
    }
}
