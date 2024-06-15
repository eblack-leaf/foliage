use bevy_ecs::prelude::{Component, Event, Resource};
use std::collections::HashMap;
use winit::event::ElementState;
use winit::keyboard::{Key, ModifiersState};
use crate::view::SignalHandle;

#[derive(Event)]
pub struct KeyboardInteraction {
    pub input_sequence: InputSequence,
}
#[derive(PartialEq, Hash)]
pub struct InputSequence {
    key: Key,
    state: ElementState,
    modifiers_state: ModifiersState,
}
#[derive(Resource)]
pub(crate) struct KeyboardAdapter {
    cache: HashMap<Key, ElementState>,
    pub(crate) current_modifiers: ModifiersState,
}
#[derive(Component)]
pub struct KeyBindings {
    bindings: HashMap<InputSequence, SignalHandle>,
}
impl KeyboardAdapter {
    pub(crate) fn cache_different(
        &mut self,
        key: Key,
        state: ElementState,
    ) -> Option<KeyboardInteraction> {
        return if let Some(cached) = self.cache.get_mut(&key) {
            if *cached != state {
                *cached = state;
                Some(KeyboardInteraction {
                    input_sequence: InputSequence {
                        key,
                        state,
                        modifiers_state: self.current_modifiers,
                    },
                })
            } else {
                None
            }
        } else {
            self.cache.insert(key.clone(), state);
            Some(KeyboardInteraction {
                input_sequence: InputSequence {
                    key,
                    state,
                    modifiers_state: self.current_modifiers,
                },
            })
        };
    }
}
