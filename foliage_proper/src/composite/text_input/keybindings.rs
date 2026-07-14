use crate::composite::text_input::action::TextInputAction;
use crate::{InputSequence, Key, Modifiers, Resource};
use std::collections::HashMap;

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
                    InputSequence::new(Key::Enter, Modifiers::default()),
                    TextInputAction::Enter,
                );
                map.insert(
                    InputSequence::new(Key::Backspace, Modifiers::default()),
                    TextInputAction::Backspace,
                );
                map.insert(
                    InputSequence::new(Key::Delete, Modifiers::default()),
                    TextInputAction::Delete,
                );
                map.insert(
                    InputSequence::new(Key::End, Modifiers::default()),
                    TextInputAction::End,
                );
                map.insert(
                    InputSequence::new(Key::Home, Modifiers::default()),
                    TextInputAction::Home,
                );
                map.insert(
                    InputSequence::new(Key::Copy, Modifiers::default()),
                    TextInputAction::Copy,
                );
                map.insert(
                    InputSequence::new(Key::Paste, Modifiers::default()),
                    TextInputAction::Paste,
                );
                map.insert(
                    InputSequence::new(Key::Character("c".to_string()), Modifiers::CONTROL),
                    TextInputAction::Copy,
                );
                map.insert(
                    InputSequence::new(Key::Character("v".to_string()), Modifiers::CONTROL),
                    TextInputAction::Paste,
                );
                map.insert(
                    InputSequence::new(Key::Character("a".to_string()), Modifiers::CONTROL),
                    TextInputAction::SelectAll,
                );
                map.insert(
                    InputSequence::new(Key::ArrowLeft, Modifiers::SHIFT),
                    TextInputAction::ExtendLeft,
                );
                map.insert(
                    InputSequence::new(Key::ArrowRight, Modifiers::SHIFT),
                    TextInputAction::ExtendRight,
                );
                map.insert(
                    InputSequence::new(Key::ArrowUp, Modifiers::SHIFT),
                    TextInputAction::ExtendUp,
                );
                map.insert(
                    InputSequence::new(Key::ArrowDown, Modifiers::SHIFT),
                    TextInputAction::ExtendDown,
                );
                map.insert(
                    InputSequence::new(Key::ArrowLeft, Modifiers::default()),
                    TextInputAction::Left,
                );
                map.insert(
                    InputSequence::new(Key::ArrowRight, Modifiers::default()),
                    TextInputAction::Right,
                );
                map.insert(
                    InputSequence::new(Key::ArrowUp, Modifiers::default()),
                    TextInputAction::Up,
                );
                map.insert(
                    InputSequence::new(Key::ArrowDown, Modifiers::default()),
                    TextInputAction::Down,
                );
                map.insert(
                    InputSequence::new(Key::Space, Modifiers::default()),
                    TextInputAction::Space,
                );
                map.insert(
                    InputSequence::new(Key::Tab, Modifiers::default()),
                    TextInputAction::Tab,
                );
                map.insert(
                    InputSequence::new(Key::Escape, Modifiers::default()),
                    TextInputAction::Escape,
                );
                map
            },
        }
    }
}
impl KeyBindings {
    pub fn action(&self, i: &InputSequence) -> Option<TextInputAction> {
        // Every entry with `s.mods` a subset of `i.mods` "matches" (see the type's own doc
        // comment on why `contains` is used) -- but `Modifiers::empty()` (the plain, no-
        // modifier bindings like `Left`) is a subset of *everything*, including `SHIFT`, so
        // holding Shift makes both `Left` and `ExtendLeft` match at once. `HashMap` iteration
        // order isn't deterministic across runs (random hash seed per construction), so
        // `.find_map`'s "first" match was effectively a coin flip, per key, per launch --
        // consistent within one run (explaining why some directions "worked" reliably and
        // others never did, in the same session) but different the next time the app
        // started. Taking the match with the most bits set picks the more specific binding
        // deterministically, every time.
        self.bindings
            .iter()
            .filter(|(s, _)| i.key == s.key && i.mods.contains(s.mods))
            .max_by_key(|(s, _)| s.mods.bits().count_ones())
            .map(|(_, a)| *a)
    }
}
