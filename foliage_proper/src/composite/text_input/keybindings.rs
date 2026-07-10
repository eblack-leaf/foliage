use crate::composite::text_input::action::TextInputAction;
use crate::{InputSequence, Key, Resource};
use std::collections::HashMap;
use winit::keyboard::ModifiersState;

#[derive(Resource)]
pub struct KeyBindings {
    pub bindings: HashMap<InputSequence, TextInputAction>,
}
impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            bindings: {
                let mut map = HashMap::new();
                map.insert(
                    InputSequence::new(Key::Enter, ModifiersState::default()),
                    TextInputAction::Enter,
                );
                map.insert(
                    InputSequence::new(Key::Backspace, ModifiersState::default()),
                    TextInputAction::Backspace,
                );
                map.insert(
                    InputSequence::new(Key::Delete, ModifiersState::default()),
                    TextInputAction::Delete,
                );
                map.insert(
                    InputSequence::new(Key::End, ModifiersState::default()),
                    TextInputAction::End,
                );
                map.insert(
                    InputSequence::new(Key::Home, ModifiersState::default()),
                    TextInputAction::Home,
                );
                map.insert(
                    InputSequence::new(Key::Copy, ModifiersState::default()),
                    TextInputAction::Copy,
                );
                map.insert(
                    InputSequence::new(Key::Paste, ModifiersState::default()),
                    TextInputAction::Paste,
                );
                map.insert(
                    InputSequence::new(Key::Character("c".to_string()), ModifiersState::CONTROL),
                    TextInputAction::Copy,
                );
                map.insert(
                    InputSequence::new(Key::Character("v".to_string()), ModifiersState::CONTROL),
                    TextInputAction::Paste,
                );
                map.insert(
                    InputSequence::new(Key::Character("a".to_string()), ModifiersState::CONTROL),
                    TextInputAction::SelectAll,
                );
                map.insert(
                    InputSequence::new(Key::ArrowLeft, ModifiersState::SHIFT),
                    TextInputAction::ExtendLeft,
                );
                map.insert(
                    InputSequence::new(Key::ArrowRight, ModifiersState::SHIFT),
                    TextInputAction::ExtendRight,
                );
                map.insert(
                    InputSequence::new(Key::ArrowUp, ModifiersState::SHIFT),
                    TextInputAction::ExtendUp,
                );
                map.insert(
                    InputSequence::new(Key::ArrowDown, ModifiersState::SHIFT),
                    TextInputAction::ExtendDown,
                );
                map.insert(
                    InputSequence::new(Key::ArrowLeft, ModifiersState::default()),
                    TextInputAction::Left,
                );
                map.insert(
                    InputSequence::new(Key::ArrowRight, ModifiersState::default()),
                    TextInputAction::Right,
                );
                map.insert(
                    InputSequence::new(Key::ArrowUp, ModifiersState::default()),
                    TextInputAction::Up,
                );
                map.insert(
                    InputSequence::new(Key::ArrowDown, ModifiersState::default()),
                    TextInputAction::Down,
                );
                map.insert(
                    InputSequence::new(Key::Space, ModifiersState::default()),
                    TextInputAction::Space,
                );
                map
            },
        }
    }
}
impl KeyBindings {
    pub fn action(&self, i: &InputSequence) -> Option<TextInputAction> {
        self.bindings.iter().find_map(|(s, a)| {
            if i.key == s.key && i.mods.contains(s.mods) {
                Some(*a)
            } else {
                None
            }
        })
    }
}
