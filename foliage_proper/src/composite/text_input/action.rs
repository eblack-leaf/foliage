use crate::Event;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TextInputAction {
    Enter,
    Backspace,
    Delete,
    End,
    Home,
    Copy,
    Paste,
    SelectAll,
    ExtendLeft,
    ExtendRight,
    ExtendUp,
    ExtendDown,
    Up,
    Down,
    Left,
    Right,
    Space,
    /// No text mutation — forwarded (like every action) as an [`InputAction`] for enclosing
    /// composites (Prompt commits the highlighted suggestion on Tab).
    Tab,
    /// No text mutation — Prompt dismisses its suggestions on Escape.
    Escape,
}

/// The targeted event form of a [`TextInputAction`]: triggered at the `TextInput` root so
/// enclosing composites (e.g. a search prompt) can subscribe to submitted/updated actions.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct InputAction {
    pub action: TextInputAction,
}
