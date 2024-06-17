use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::LogicalContext;
use crate::ginkgo::ScaleFactor;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{Event, EventReader};
use bevy_ecs::prelude::Component;
use bevy_ecs::system::{Query, ResMut, Resource};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, Touch, TouchPhase};

pub(crate) struct TouchAdapter {
    primary: Option<u64>,
}
impl TouchAdapter {
    pub(crate) fn parse(
        &mut self,
        touch: Touch,
        viewport_position: Position<LogicalContext>,
        scale_factor: ScaleFactor,
    ) -> Option<ClickInteraction> {
        let position = Position::device((touch.location.x, touch.location.y))
            .to_logical(scale_factor.value())
            + viewport_position;
        if self.primary.is_none() {
            if touch.phase == TouchPhase::Started {
                self.primary.replace(touch.id);
                return Some(ClickInteraction::new(ClickPhase::Start, position));
            }
        } else {
            if self.primary.unwrap() == touch.id {
                match touch.phase {
                    TouchPhase::Started => {}
                    TouchPhase::Moved => {
                        return Some(ClickInteraction::new(ClickPhase::Moved, position));
                    }
                    TouchPhase::Ended => {
                        return Some(ClickInteraction::new(ClickPhase::End, position));
                    }
                    TouchPhase::Cancelled => {
                        return Some(ClickInteraction::new(ClickPhase::Cancel, position));
                    }
                }
            }
        }
        None
    }
}
pub(crate) struct MouseAdapter {
    started: bool,
    cursor: Position<LogicalContext>,
}
impl MouseAdapter {
    pub(crate) fn parse(
        &mut self,
        mouse_button: MouseButton,
        state: ElementState,
    ) -> Option<ClickInteraction> {
        if mouse_button != MouseButton::Left {
            return None;
        }
        if self.started {
            if !state.is_pressed() {
                self.started = false;
                return Some(ClickInteraction::new(ClickPhase::End, self.cursor));
            }
        }
        if !self.started {
            if state.is_pressed() {
                self.started = true;
                return Some(ClickInteraction::new(ClickPhase::Start, self.cursor));
            }
        }
        None
    }
    pub(crate) fn set_cursor(
        &mut self,
        position: PhysicalPosition<f64>,
        viewport_position: Position<LogicalContext>,
        scale_factor: ScaleFactor,
    ) -> Option<ClickInteraction> {
        if self.started {
            return Some(ClickInteraction::new(
                ClickPhase::Moved,
                Position::device((position.x, position.y)).to_logical(scale_factor.value())
                    + viewport_position,
            ));
        }
        None
    }
}
#[derive(Copy, Clone)]
pub enum ClickPhase {
    Start,
    Moved,
    End,
    Cancel,
}
#[derive(Event)]
pub struct ClickInteraction {
    click_phase: ClickPhase,
    position: Position<LogicalContext>,
}
impl ClickInteraction {
    pub fn new(click_phase: ClickPhase, position: Position<LogicalContext>) -> Self {
        Self {
            click_phase,
            position,
        }
    }
}
#[derive(Default, Copy, Clone)]
pub struct Click {
    pub start: Position<LogicalContext>,
    pub current: Position<LogicalContext>,
    pub end: Option<Position<LogicalContext>>,
}
impl Click {
    pub fn new(start: Position<LogicalContext>) -> Self {
        Self {
            start,
            current: start,
            end: None,
        }
    }
}
#[derive(Default, Copy, Clone, Component)]
pub struct ClickInteractionListener {
    pub click: Option<Click>,
    pub focused: bool,
    pub engaged_start: bool,
    pub engaged: bool,
    pub engaged_end: bool,
    pub active: bool,
    pub shape: ClickInteractionShape,
}
impl ClickInteractionListener {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn as_circle(mut self) -> Self {
        self.shape = ClickInteractionShape::Circle;
        self
    }
}
#[derive(Resource)]
pub(crate) struct InteractiveEntity(pub(crate) Option<(Entity)>);
#[derive(Resource)]
pub(crate) struct FocusedEntity(pub(crate) Option<Entity>);
#[derive(Copy, Clone, Default)]
pub enum ClickInteractionShape {
    Circle,
    #[default]
    Rectangle,
}
impl ClickInteractionShape {
    pub fn contains(&self, p: Position<LogicalContext>, section: Section<LogicalContext>) -> bool {
        match self {
            ClickInteractionShape::Circle => section.center().distance(p) <= section.area.width(),
            ClickInteractionShape::Rectangle => section.contains(p),
        }
    }
}
pub(crate) fn listen_for_interactions(
    mut listeners: Query<(
        Entity,
        &mut ClickInteractionListener,
        &Position<LogicalContext>,
        &Area<LogicalContext>,
        &Layer,
    )>,
    mut events: EventReader<ClickInteraction>,
    mut grabbed: ResMut<InteractiveEntity>,
    mut focused: ResMut<FocusedEntity>,
) {
    for event in events.read() {
        match event.click_phase {
            ClickPhase::Start => {
                if grabbed.0.is_none() {
                    let mut grab_info = None;
                    for (entity, listener, pos, area, layer) in listeners.iter_mut() {
                        if listener
                            .shape
                            .contains(event.position, Section::new(*pos, *area))
                        {
                            if grab_info.is_none() {
                                grab_info.replace((entity, *layer));
                            } else {
                                if *layer < grab_info.unwrap().1 {
                                    grab_info.replace((entity, *layer));
                                }
                            }
                        }
                    }
                    if let Some(grab) = grab_info {
                        if let Some(entity) = focused.0.replace(grab.0) {
                            listeners.get_mut(entity).unwrap().1.focused = false;
                        }
                        grabbed.0.replace(grab.0);
                        listeners
                            .get_mut(grab.0)
                            .unwrap()
                            .1
                            .click
                            .replace(Click::new(event.position));
                        listeners.get_mut(grab.0).unwrap().1.focused = true;
                        listeners.get_mut(grab.0).unwrap().1.engaged = true;
                        listeners.get_mut(grab.0).unwrap().1.engaged_start = true;
                    } else {
                        if let Some(entity) = focused.0.take() {
                            listeners.get_mut(entity).unwrap().1.focused = false;
                        }
                    }
                }
            }
            ClickPhase::Moved => {
                if let Some(g) = grabbed.0 {
                    listeners.get_mut(g).unwrap().1.click.unwrap().current = event.position;
                }
            }
            ClickPhase::End => {
                if let Some(g) = grabbed.0.take() {
                    if listeners.get(g).unwrap().1.shape.contains(
                        event.position,
                        Section::new(*listeners.get(g).unwrap().2, *listeners.get(g).unwrap().3),
                    ) {
                        listeners.get_mut(g).unwrap().1.active = true;
                    }
                    listeners
                        .get_mut(g)
                        .unwrap()
                        .1
                        .click
                        .unwrap()
                        .end
                        .replace(event.position);
                    listeners.get_mut(g).unwrap().1.engaged_end = true;
                }
            }
            ClickPhase::Cancel => {
                if let Some(g) = grabbed.0.take() {
                    listeners.get_mut(g).unwrap().1.click.take();
                    listeners.get_mut(g).unwrap().1.engaged_end = true;
                    listeners.get_mut(g).unwrap().1.engaged = false;
                }
            }
        }
    }
}
pub(crate) fn reset_click_listener_flags(mut listeners: Query<(&mut ClickInteractionListener)>) {
    for mut listener in listeners.iter_mut() {
        listener.engaged_start = false;
        listener.engaged_end = false;
        listener.active = false;
    }
}
pub(crate) struct KeyboardAdapter {

}
