use crate::coordinate::position::Position;
use crate::coordinate::Logical;
use crate::ginkgo::ScaleFactor;
use crate::interaction::{Interaction, InteractionMethod, InteractionPhase};
use crate::{Event, Resource};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, Touch, TouchPhase};
use winit::keyboard::{Key, ModifiersState};

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

#[derive(Resource, Default)]
pub(crate) struct KeyboardAdapter {
    pub(crate) mods: ModifiersState,
}

impl KeyboardAdapter {
    pub(crate) fn parse(
        &mut self,
        key: Key,
        state: ElementState,
        repeat: bool,
    ) -> Option<InputSequence> {
        if state.is_pressed() && !repeat {
            Some(InputSequence::new(key, self.mods))
        } else {
            None
        }
    }
}
