use crate::Event;

#[derive(Event, Copy, Clone, Eq, PartialEq, Hash, Debug)]
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
}
