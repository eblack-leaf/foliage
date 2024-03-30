use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{DeviceContext, InterfaceContext};
use crate::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::{Elm, EventStage};
use crate::ginkgo::viewport::ViewportHandle;
use crate::window::ScaleFactor;
use bevy_ecs::component::Component;
use bevy_ecs::event::{Event, EventReader};
use bevy_ecs::prelude::{DetectChanges, Entity, IntoSystemConfigs};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, Res, ResMut, Resource};
use compact_str::{CompactString, ToCompactString};
use nalgebra::{distance, Point};
use std::collections::HashMap;
use winit::event::{ElementState, Modifiers, MouseButton, TouchPhase};
use winit::keyboard::{ModifiersState, NamedKey};

impl Leaf for Interaction {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.job
            .container
            .insert_resource(PrimaryInteraction::default());
        elm.job
            .container
            .insert_resource(PrimaryInteractionEntity::default());
        elm.job.container.insert_resource(FocusedEntity::default());
        elm.job.container.insert_resource(MouseAdapter::default());
        elm.job
            .container
            .insert_resource(KeyboardAdapter::default());
        elm.main().add_systems((
            (set_interaction_listeners, clear_non_primary)
                .chain()
                .in_set(CoreSet::Interaction),
            clear_active.after(ExternalSet::Configure),
        ));
        elm.add_event::<InteractionEvent>(EventStage::External);
        elm.add_event::<KeyboardEvent>(EventStage::External);
    }
}
#[derive(Event, Debug, Clone)]
pub struct KeyboardEvent {
    pub key: Key,
    pub state: State,
    pub modifiers: Mods,
}
pub type Key = winit::keyboard::Key;
pub type State = ElementState;
pub type Mods = Modifiers;
impl KeyboardEvent {
    pub fn new(key: Key, state: State, mods: Mods) -> Self {
        Self {
            key,
            state,
            modifiers: mods,
        }
    }
    pub fn sequence(&self) -> InputSequence {
        match &self.key {
            Key::Named(name) => match name {
                NamedKey::Backspace => InputSequence::Backspace,
                NamedKey::ArrowLeft => {
                    if self.modifiers.state() == ModifiersState::SHIFT {
                        InputSequence::ArrowLeftShift
                    } else {
                        InputSequence::ArrowLeft
                    }
                }
                NamedKey::ArrowRight => {
                    if self.modifiers.state() == ModifiersState::SHIFT {
                        InputSequence::ArrowRightShift
                    } else {
                        InputSequence::ArrowRight
                    }
                }
                NamedKey::Space => InputSequence::Space,
                NamedKey::Enter => InputSequence::Enter,
                NamedKey::Delete => InputSequence::Delete,
                _ => InputSequence::Unidentified,
            },
            Key::Character(char) => {
                let value = char.to_lowercase();
                if value.contains("a") && self.modifiers.state() == ModifiersState::CONTROL {
                    InputSequence::CtrlA
                } else if value.contains("x") && self.modifiers.state() == ModifiersState::CONTROL {
                    InputSequence::CtrlX
                } else if value.contains("c") && self.modifiers.state() == ModifiersState::CONTROL {
                    InputSequence::CtrlC
                } else if value.contains("v") && self.modifiers.state() == ModifiersState::CONTROL {
                    InputSequence::CtrlV
                } else {
                    InputSequence::Character(char.to_compact_string())
                }
            }
            Key::Unidentified(_) => InputSequence::Unidentified,
            Key::Dead(_) => InputSequence::Unidentified,
        }
    }
}
#[derive(Clone)]
pub enum InputSequence {
    CtrlX,
    CtrlC,
    CtrlA,
    CtrlZ,
    Backspace,
    Enter,
    Character(CompactString),
    ArrowLeft,
    ArrowRight,
    ArrowDown,
    ArrowUp,
    Unidentified,
    Space,
    Delete,
    ArrowLeftShift,
    ArrowRightShift,
    CtrlV,
}
#[derive(Resource, Default)]
pub struct PrimaryInteraction(pub(crate) Option<InteractionId>);
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct InteractionId(pub(crate) i32);
impl InteractionId {
    // Number to offset touch.ids to avoid collision with mouse buttons
    pub(crate) const INTERACTION_ID_COLLISION_AVOIDANCE: i32 = 30;
}
impl From<MouseButton> for InteractionId {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => Self(0),
            MouseButton::Right => Self(1),
            MouseButton::Middle => Self(2),
            MouseButton::Back => Self(3),
            MouseButton::Forward => Self(4),
            MouseButton::Other(o) => Self(4 + o as i32),
        }
    }
}
impl From<u64> for InteractionId {
    fn from(value: u64) -> Self {
        Self(InteractionId::INTERACTION_ID_COLLISION_AVOIDANCE + value as i32)
    }
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InteractionPhase {
    Begin,
    Moved,
    End,
    Cancel,
}
impl From<TouchPhase> for InteractionPhase {
    fn from(value: TouchPhase) -> Self {
        match value {
            TouchPhase::Started => InteractionPhase::Begin,
            TouchPhase::Moved => InteractionPhase::Moved,
            TouchPhase::Ended => InteractionPhase::End,
            TouchPhase::Cancelled => InteractionPhase::Cancel,
        }
    }
}
#[derive(Default, Resource)]
pub struct KeyboardAdapter {
    cache: HashMap<winit::keyboard::Key, ElementState>,
    modifiers: Modifiers,
}
impl KeyboardAdapter {
    pub(crate) fn cache_checked(
        &mut self,
        key: winit::keyboard::Key,
        state: ElementState,
    ) -> Option<KeyboardEvent> {
        if let Some(cached) = self.cache.insert(key.clone(), state) {
            if cached != state {
                return Option::from(KeyboardEvent::new(key, state, self.modifiers));
            }
        } else if state.is_pressed() {
            return Option::from(KeyboardEvent::new(key, state, self.modifiers));
        }
        None
    }
    pub(crate) fn update_modifiers(&mut self, modifiers: Modifiers) {
        self.modifiers = modifiers;
    }
}

#[derive(Resource, Default)]
pub struct MouseAdapter(
    pub HashMap<MouseButton, ElementState>,
    pub Position<DeviceContext>,
);
impl MouseAdapter {
    pub fn button_pressed(&mut self, button: MouseButton, state: ElementState) -> bool {
        let mut r_val = false;
        if let Some(cached) = self.0.get_mut(&button) {
            if !cached.is_pressed() && state.is_pressed() {
                r_val = true;
            }
            *cached = state;
        } else {
            if state.is_pressed() {
                r_val = true;
            }
            self.0.insert(button, state);
        }
        r_val
    }
    pub fn update_location<P: Into<Position<DeviceContext>>>(&mut self, p: P) {
        self.1 = p.into();
    }
}
#[derive(Event)]
pub struct InteractionEvent {
    pub phase: InteractionPhase,
    pub id: InteractionId,
    pub location: Position<DeviceContext>,
}
impl InteractionEvent {
    pub fn new<
        IP: Into<InteractionPhase>,
        ID: Into<InteractionId>,
        P: Into<Position<DeviceContext>>,
    >(
        phase: IP,
        id: ID,
        location: P,
    ) -> Self {
        Self {
            phase: phase.into(),
            id: id.into(),
            location: location.into(),
        }
    }
}
#[derive(Copy, Clone, Default)]
pub struct Interaction {
    pub begin: Position<InterfaceContext>,
    pub current: Position<InterfaceContext>,
    pub end: Option<Position<InterfaceContext>>,
}
impl Interaction {
    pub fn new<P: Into<Position<InterfaceContext>>>(begin: P) -> Self {
        let position = begin.into();
        Self {
            begin: position,
            current: position,
            end: None,
        }
    }
}
#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub enum InteractionShape {
    InteractiveCircle,
    #[default]
    InteractiveRectangle,
}
#[derive(Component, Copy, Clone, Default)]
pub struct InteractionListener {
    active: bool,
    pub interaction: Interaction,
    engaged: bool,
    engaged_start: bool,
    engaged_end: bool,
    shape: InteractionShape,
    lost_focus: bool,
}
impl InteractionListener {
    pub fn with_shape(mut self, shape: InteractionShape) -> Self {
        self.shape = shape;
        self
    }
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn engaged(&self) -> bool {
        self.engaged
    }
    pub fn engaged_start(&self) -> bool {
        self.engaged_start
    }
    pub fn engaged_end(&self) -> bool {
        self.engaged_end
    }
    pub fn lost_focus(&self) -> bool {
        self.lost_focus
    }
    pub(crate) fn shape(&self, section: Section<InterfaceContext>) -> InteractionShapeActualized {
        InteractionShapeActualized(self.shape, section)
    }
}
pub(crate) struct InteractionShapeActualized(InteractionShape, Section<InterfaceContext>);
impl InteractionShapeActualized {
    pub(crate) fn contains(&self, position: Position<InterfaceContext>) -> bool {
        let center = self.1.center();
        match self.0 {
            InteractionShape::InteractiveCircle => {
                if distance(
                    &Point::<f32, 2>::new(position.x, position.y),
                    &Point::<f32, 2>::new(center.x, center.y),
                ) <= self.1.width() / 2f32
                {
                    return true;
                }
                false
            }
            InteractionShape::InteractiveRectangle => {
                if self.1.contains(position) {
                    return true;
                }
                false
            }
        }
    }
}
fn clear_active(mut active: Query<&mut InteractionListener, Changed<InteractionListener>>) {
    for mut e in active.iter_mut() {
        if e.active {
            e.active = false;
        }
        if e.engaged_start {
            e.engaged_start = false;
        }
        if e.engaged_end {
            e.engaged_end = false;
        }
        if e.lost_focus {
            e.lost_focus = false;
        }
    }
}
fn clear_non_primary(
    mut engaged: Query<(Entity, &mut InteractionListener)>,
    primary_interaction_entity: Res<PrimaryInteractionEntity>,
) {
    if primary_interaction_entity.is_changed() {
        for (entity, mut listener) in engaged.iter_mut() {
            if listener.engaged {
                if let Some(prime) = primary_interaction_entity.0 {
                    if prime != entity {
                        tracing::trace!("clearing orphaned-engaged-entity: {:?}", entity);
                        listener.engaged = false;
                    }
                } else {
                    tracing::trace!("clearing engaged-entity: {:?}", entity);
                    listener.engaged = false;
                }
            }
        }
    }
}
#[derive(Resource, Default)]
pub struct PrimaryInteractionEntity(pub Option<Entity>);
#[derive(Resource, Default)]
pub struct FocusedEntity(pub Option<Entity>);
pub fn set_interaction_listeners(
    viewport_handle: Res<ViewportHandle>,
    scale_factor: Res<ScaleFactor>,
    mut events: EventReader<InteractionEvent>,
    mut listeners: Query<(
        Entity,
        &mut InteractionListener,
        &Position<InterfaceContext>,
        &Area<InterfaceContext>,
        &Layer,
    )>,
    mut primary: ResMut<PrimaryInteraction>,
    mut primary_entity: ResMut<PrimaryInteractionEntity>,
    mut focused_entity: ResMut<FocusedEntity>,
) {
    for ie in events.read() {
        let position =
            ie.location.to_interface(scale_factor.factor()) - viewport_handle.section.position;
        if primary.0.is_none() {
            primary_entity.0.take();
            if ie.phase != InteractionPhase::Begin {
                continue;
            }
            let mut grabbed = None;
            for (entity, listener, pos, area, layer) in listeners.iter_mut() {
                let section = Section::new(*pos, *area);
                if listener.shape(section).contains(position) {
                    if grabbed.is_none() {
                        grabbed.replace((entity, *layer));
                    }
                    if grabbed.unwrap().1 >= *layer {
                        grabbed.replace((entity, *layer));
                    }
                }
            }
            if let Some(grab) = grabbed {
                tracing::trace!("grabbing primary: {:?}", grab.0);
                primary.0.replace(ie.id);
                primary_entity.0.replace(grab.0);
                if let Some(f) = focused_entity.0.replace(grab.0) {
                    if let Ok(mut list) = listeners.get_mut(f) {
                        list.1.lost_focus = true;
                    }
                }
                listeners.get_mut(grab.0).unwrap().1.engaged = true;
                listeners.get_mut(grab.0).unwrap().1.engaged_start = true;
                listeners.get_mut(grab.0).unwrap().1.interaction = Interaction::new(position);
            } else {
                if let Some(e) = focused_entity.0.take() {
                    if let Ok(mut list) = listeners.get_mut(e) {
                        list.1.lost_focus = true;
                    }
                }
            }
        } else if ie.id == primary.0.unwrap() {
            match ie.phase {
                InteractionPhase::Begin => {
                    // skip
                }
                InteractionPhase::Moved => {
                    if let Some(prime) = primary_entity.0 {
                        if let Ok((_, mut listener, _, _, _)) = listeners.get_mut(prime) {
                            tracing::trace!("updating prime-current: {:?}-@-{:?}", prime, position);
                            listener.interaction.current = position;
                        } else {
                            primary.0.take();
                            primary_entity.0.take();
                        }
                    } else {
                        primary.0.take();
                    }
                }
                InteractionPhase::End => {
                    if let Some(prime) = primary_entity.0.take() {
                        let mut to_be_unfocused = None;
                        if let Ok((_, mut listener, pos, area, _)) = listeners.get_mut(prime) {
                            let section = Section::new(*pos, *area);
                            if listener.shape(section).contains(position) {
                                listener.interaction.current = position;
                                listener.interaction.end.replace(position);
                                if let Some(old) = focused_entity.0.replace(prime) {
                                    if old != prime {
                                        // defer
                                        to_be_unfocused.replace(old);
                                    }
                                }
                                listener.active = true;
                            }
                            tracing::trace!("ending prime: {:?}-@-{:?}", prime, position);
                            listener.engaged = false;
                            listener.engaged_end = true;
                        } else {
                            if let Some(e) = focused_entity.0.take() {
                                if let Ok(mut list) = listeners.get_mut(e) {
                                    list.1.lost_focus = true;
                                }
                            }
                        }
                        if let Some(e) = to_be_unfocused {
                            if let Ok(mut list) = listeners.get_mut(e) {
                                list.1.lost_focus = true;
                            }
                        }
                    } else {
                        if let Some(e) = focused_entity.0.take() {
                            if let Ok(mut list) = listeners.get_mut(e) {
                                list.1.lost_focus = true;
                            }
                        }
                    }
                    primary.0.take();
                }
                InteractionPhase::Cancel => {
                    primary.0.take();
                    primary_entity.0.take();
                    if let Some(e) = focused_entity.0.take() {
                        if let Ok(mut list) = listeners.get_mut(e) {
                            list.1.lost_focus = true;
                        }
                    }
                }
            }
        }
    }
}
