use crate::coordinate::position::Position;
use crate::coordinate::Logical;
use crate::ginkgo::ScaleFactor;
use crate::interaction::{Interaction, InteractionMethod, InteractionPhase};
use crate::{Event, Resource};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, Touch, TouchPhase};
use winit::keyboard::{
    Key as WinitKey, KeyCode, ModifiersState, NamedKey, PhysicalKey as WinitPhysicalKey,
};

/// Foliage's own key representation — converted once, right here in `KeyboardAdapter::parse`,
/// from winit's `Key` so no winit type (or its version churn) ever reaches a consumer. Mirrors
/// the same boundary `MouseAdapter`/`TouchAdapter` already draw for pointer input (raw
/// `MouseButton`/`Touch` never escape past them either) — keyboard was the one gap. `Other` is
/// the deliberate escape hatch: a future winit `NamedKey` addition falls in here instead of
/// forcing a breaking change on every consumer matching on this enum.
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Key {
    Character(String),
    Enter,
    Escape,
    Tab,
    Backspace,
    Delete,
    Space,
    End,
    Home,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Copy,
    Paste,
    Other,
}
impl From<WinitKey> for Key {
    fn from(key: WinitKey) -> Self {
        match key {
            WinitKey::Character(s) => Key::Character(s.to_string()),
            WinitKey::Named(NamedKey::Enter) => Key::Enter,
            WinitKey::Named(NamedKey::Escape) => Key::Escape,
            WinitKey::Named(NamedKey::Tab) => Key::Tab,
            WinitKey::Named(NamedKey::Backspace) => Key::Backspace,
            WinitKey::Named(NamedKey::Delete) => Key::Delete,
            WinitKey::Named(NamedKey::Space) => Key::Space,
            WinitKey::Named(NamedKey::End) => Key::End,
            WinitKey::Named(NamedKey::Home) => Key::Home,
            WinitKey::Named(NamedKey::ArrowLeft) => Key::ArrowLeft,
            WinitKey::Named(NamedKey::ArrowRight) => Key::ArrowRight,
            WinitKey::Named(NamedKey::ArrowUp) => Key::ArrowUp,
            WinitKey::Named(NamedKey::ArrowDown) => Key::ArrowDown,
            WinitKey::Named(NamedKey::Copy) => Key::Copy,
            WinitKey::Named(NamedKey::Paste) => Key::Paste,
            _ => Key::Other,
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct TouchAdapter {
    primary: Option<u64>,
}

impl TouchAdapter {
    pub(crate) fn parse(
        &mut self,
        touch: Touch,
        viewport_position: Position<Logical>,
        scale_factor: ScaleFactor,
    ) -> Option<Interaction> {
        let position = Position::physical((touch.location.x, touch.location.y))
            .to_logical(scale_factor.value())
            + viewport_position;
        if self.primary.is_none() {
            if touch.phase == TouchPhase::Started {
                self.primary.replace(touch.id);
                return Some(Interaction::new(
                    InteractionPhase::Start,
                    position,
                    InteractionMethod::TouchScreen,
                ));
            }
        } else if self.primary.unwrap() == touch.id {
            match touch.phase {
                TouchPhase::Started => {}
                TouchPhase::Moved => {
                    return Some(Interaction::new(
                        InteractionPhase::Moved,
                        position,
                        InteractionMethod::TouchScreen,
                    ));
                }
                TouchPhase::Ended => {
                    self.primary.take();
                    return Some(Interaction::new(
                        InteractionPhase::End,
                        position,
                        InteractionMethod::TouchScreen,
                    ));
                }
                TouchPhase::Cancelled => {
                    self.primary.take();
                    return Some(Interaction::new(
                        InteractionPhase::Cancel,
                        position,
                        InteractionMethod::TouchScreen,
                    ));
                }
            }
        }
        None
    }
}

#[derive(Resource, Default)]
pub(crate) struct MouseAdapter {
    started: bool,
    pub(crate) cursor: Position<Logical>,
}

impl MouseAdapter {
    pub(crate) fn parse(
        &mut self,
        mouse_button: MouseButton,
        state: ElementState,
    ) -> Option<Interaction> {
        if mouse_button != MouseButton::Left {
            return None;
        }
        if self.started && !state.is_pressed() {
            self.started = false;
            return Some(Interaction::new(
                InteractionPhase::End,
                self.cursor,
                InteractionMethod::Mouse,
            ));
        }
        if !self.started && state.is_pressed() {
            self.started = true;
            return Some(Interaction::new(
                InteractionPhase::Start,
                self.cursor,
                InteractionMethod::Mouse,
            ));
        }
        None
    }
    pub(crate) fn set_cursor(
        &mut self,
        position: PhysicalPosition<f64>,
        viewport_position: Position<Logical>,
        scale_factor: ScaleFactor,
    ) -> Option<Interaction> {
        let adjusted_position =
            Position::physical((position.x, position.y)).to_logical(scale_factor.value());
        self.cursor = adjusted_position;
        if self.started {
            return Some(Interaction::new(
                InteractionPhase::Moved,
                adjusted_position + viewport_position,
                InteractionMethod::Mouse,
            ));
        }
        None
    }
}

#[derive(Event, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct InputSequence {
    pub key: Key,
    pub mods: ModifiersState,
}

impl InputSequence {
    pub fn new(key: Key, mods: ModifiersState) -> Self {
        Self { key, mods }
    }
}

/// Winit's `PhysicalKey` mirrored as an owned type, same reasoning as `Key` above. This is the
/// *other* half of a winit key event — `Key`/`InputSequence` already cover the logical (produced
/// character) half, but a layout-independent "which physical key" scheme (e.g. a spatial WASD-
/// style keymap that must land on the same physical keys under Dvorak too) needs this instead,
/// and previously had no way to reach a consumer at all. Kept as a wholly separate event
/// (`PhysicalInputSequence`) rather than an extra field on `InputSequence`, so existing
/// logical-key consumers (text input keybindings, which must match regardless of physical
/// position) are unaffected. `Other` is the same escape hatch as `Key::Other`.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum PhysicalKey {
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Comma,
    Period,
    Semicolon,
    Quote,
    Slash,
    Backslash,
    Minus,
    Equal,
    BracketLeft,
    BracketRight,
    Backquote,
    Space,
    Enter,
    Escape,
    Tab,
    Backspace,
    Delete,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
    Other,
}
impl From<WinitPhysicalKey> for PhysicalKey {
    fn from(key: WinitPhysicalKey) -> Self {
        let code = match key {
            WinitPhysicalKey::Code(code) => code,
            WinitPhysicalKey::Unidentified(_) => return PhysicalKey::Other,
        };
        match code {
            KeyCode::KeyA => PhysicalKey::KeyA,
            KeyCode::KeyB => PhysicalKey::KeyB,
            KeyCode::KeyC => PhysicalKey::KeyC,
            KeyCode::KeyD => PhysicalKey::KeyD,
            KeyCode::KeyE => PhysicalKey::KeyE,
            KeyCode::KeyF => PhysicalKey::KeyF,
            KeyCode::KeyG => PhysicalKey::KeyG,
            KeyCode::KeyH => PhysicalKey::KeyH,
            KeyCode::KeyI => PhysicalKey::KeyI,
            KeyCode::KeyJ => PhysicalKey::KeyJ,
            KeyCode::KeyK => PhysicalKey::KeyK,
            KeyCode::KeyL => PhysicalKey::KeyL,
            KeyCode::KeyM => PhysicalKey::KeyM,
            KeyCode::KeyN => PhysicalKey::KeyN,
            KeyCode::KeyO => PhysicalKey::KeyO,
            KeyCode::KeyP => PhysicalKey::KeyP,
            KeyCode::KeyQ => PhysicalKey::KeyQ,
            KeyCode::KeyR => PhysicalKey::KeyR,
            KeyCode::KeyS => PhysicalKey::KeyS,
            KeyCode::KeyT => PhysicalKey::KeyT,
            KeyCode::KeyU => PhysicalKey::KeyU,
            KeyCode::KeyV => PhysicalKey::KeyV,
            KeyCode::KeyW => PhysicalKey::KeyW,
            KeyCode::KeyX => PhysicalKey::KeyX,
            KeyCode::KeyY => PhysicalKey::KeyY,
            KeyCode::KeyZ => PhysicalKey::KeyZ,
            KeyCode::Digit0 => PhysicalKey::Digit0,
            KeyCode::Digit1 => PhysicalKey::Digit1,
            KeyCode::Digit2 => PhysicalKey::Digit2,
            KeyCode::Digit3 => PhysicalKey::Digit3,
            KeyCode::Digit4 => PhysicalKey::Digit4,
            KeyCode::Digit5 => PhysicalKey::Digit5,
            KeyCode::Digit6 => PhysicalKey::Digit6,
            KeyCode::Digit7 => PhysicalKey::Digit7,
            KeyCode::Digit8 => PhysicalKey::Digit8,
            KeyCode::Digit9 => PhysicalKey::Digit9,
            KeyCode::Comma => PhysicalKey::Comma,
            KeyCode::Period => PhysicalKey::Period,
            KeyCode::Semicolon => PhysicalKey::Semicolon,
            KeyCode::Quote => PhysicalKey::Quote,
            KeyCode::Slash => PhysicalKey::Slash,
            KeyCode::Backslash => PhysicalKey::Backslash,
            KeyCode::Minus => PhysicalKey::Minus,
            KeyCode::Equal => PhysicalKey::Equal,
            KeyCode::BracketLeft => PhysicalKey::BracketLeft,
            KeyCode::BracketRight => PhysicalKey::BracketRight,
            KeyCode::Backquote => PhysicalKey::Backquote,
            KeyCode::Space => PhysicalKey::Space,
            KeyCode::Enter => PhysicalKey::Enter,
            KeyCode::Escape => PhysicalKey::Escape,
            KeyCode::Tab => PhysicalKey::Tab,
            KeyCode::Backspace => PhysicalKey::Backspace,
            KeyCode::Delete => PhysicalKey::Delete,
            KeyCode::ArrowLeft => PhysicalKey::ArrowLeft,
            KeyCode::ArrowRight => PhysicalKey::ArrowRight,
            KeyCode::ArrowUp => PhysicalKey::ArrowUp,
            KeyCode::ArrowDown => PhysicalKey::ArrowDown,
            KeyCode::Home => PhysicalKey::Home,
            KeyCode::End => PhysicalKey::End,
            _ => PhysicalKey::Other,
        }
    }
}

#[derive(Event, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct PhysicalInputSequence {
    pub code: PhysicalKey,
    pub mods: ModifiersState,
}
impl PhysicalInputSequence {
    pub fn new(code: PhysicalKey, mods: ModifiersState) -> Self {
        Self { code, mods }
    }
}

#[derive(Resource, Default)]
pub(crate) struct KeyboardAdapter {
    pub(crate) mods: ModifiersState,
}

impl KeyboardAdapter {
    pub(crate) fn parse(
        &mut self,
        key: WinitKey,
        state: ElementState,
        repeat: bool,
    ) -> Option<InputSequence> {
        if state.is_pressed() {
            Some(InputSequence::new(key.into(), self.mods))
        } else {
            None
        }
    }
    pub(crate) fn parse_physical(
        &mut self,
        physical: WinitPhysicalKey,
        state: ElementState,
    ) -> Option<PhysicalInputSequence> {
        if state.is_pressed() {
            Some(PhysicalInputSequence::new(physical.into(), self.mods))
        } else {
            None
        }
    }
}
