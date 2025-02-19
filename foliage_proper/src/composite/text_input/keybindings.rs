use crate::composite::text_input::action::TextInputAction;
use crate::{InputSequence, Resource};
use std::collections::HashMap;
use winit::keyboard::{Key, ModifiersState, NamedKey, SmolStr};

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
                    InputSequence::new(Key::Named(NamedKey::Enter), ModifiersState::default()),
                    TextInputAction::Enter,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::Backspace), ModifiersState::default()),
                    TextInputAction::Backspace,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::Delete), ModifiersState::default()),
                    TextInputAction::Delete,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::End), ModifiersState::default()),
                    TextInputAction::End,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::Home), ModifiersState::default()),
                    TextInputAction::Home,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::Copy), ModifiersState::default()),
                    TextInputAction::Copy,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::Paste), ModifiersState::default()),
                    TextInputAction::Paste,
                );
                map.insert(
                    InputSequence::new(Key::Character(SmolStr::new("c")), ModifiersState::CONTROL),
                    TextInputAction::Copy,
                );
                map.insert(
                    InputSequence::new(Key::Character(SmolStr::new("v")), ModifiersState::CONTROL),
                    TextInputAction::Paste,
                );
                map.insert(
                    InputSequence::new(Key::Character(SmolStr::new("a")), ModifiersState::CONTROL),
                    TextInputAction::SelectAll,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::ArrowLeft), ModifiersState::SHIFT),
                    TextInputAction::ExtendLeft,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::ArrowRight), ModifiersState::SHIFT),
                    TextInputAction::ExtendRight,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::ArrowUp), ModifiersState::SHIFT),
                    TextInputAction::ExtendUp,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::ArrowDown), ModifiersState::SHIFT),
                    TextInputAction::ExtendDown,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::ArrowLeft), ModifiersState::default()),
                    TextInputAction::Left,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::ArrowRight), ModifiersState::default()),
                    TextInputAction::Right,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::ArrowUp), ModifiersState::default()),
                    TextInputAction::Up,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::ArrowDown), ModifiersState::default()),
                    TextInputAction::Down,
                );
                map.insert(
                    InputSequence::new(Key::Named(NamedKey::Space), ModifiersState::default()),
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
