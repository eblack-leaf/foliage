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
use std::collections::HashMap;
use winit::event::{ElementState, MouseButton, TouchPhase};

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
        elm.main().add_systems((
            (set_interaction_listeners, clear_engaged)
                .chain()
                .in_set(CoreSet::Interaction),
            clear_active.after(ExternalSet::Process),
        ));
        elm.add_event::<InteractionEvent>(EventStage::External);
    }
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
#[derive(Component, Copy, Clone, Default)]
pub struct InteractionListener {
    active: bool,
    pub interaction: Interaction,
    engaged: bool,
}
impl InteractionListener {
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn engaged(&self) -> bool {
        self.engaged
    }
}
fn clear_active(mut active: Query<&mut InteractionListener, Changed<InteractionListener>>) {
    for mut e in active.iter_mut() {
        if e.active {
            e.active = false;
        }
    }
}
fn clear_engaged(
    mut engaged: Query<(Entity, &mut InteractionListener)>,
    primary_interaction_entity: Res<PrimaryInteractionEntity>,
) {
    if primary_interaction_entity.is_changed() {
        for (entity, mut listener) in engaged.iter_mut() {
            if listener.engaged {
                if let Some(prime) = primary_interaction_entity.0 {
                    if prime != entity {
                        listener.engaged = false;
                    }
                } else {
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
            ie.location.to_interface(scale_factor.factor()) + viewport_handle.section.position;
        if primary.0.is_none() {
            primary_entity.0.take();
            if ie.phase != InteractionPhase::Begin {
                continue;
            }
            let mut grabbed = None;
            for (entity, _listener, pos, area, layer) in listeners.iter_mut() {
                let section = Section::new(*pos, *area);
                if section.contains(position) {
                    if grabbed.is_none() {
                        grabbed.replace((entity, *layer));
                    }
                    if grabbed.unwrap().1 >= *layer {
                        grabbed.replace((entity, *layer));
                    }
                }
            }
            if let Some(grab) = grabbed {
                primary.0.replace(ie.id);
                primary_entity.0.replace(grab.0);
                listeners.get_mut(grab.0).unwrap().1.engaged = true;
            }
        } else if ie.id == primary.0.unwrap() {
            match ie.phase {
                InteractionPhase::Begin => {
                    // skip
                }
                InteractionPhase::Moved => {
                    if let Some(prime) = primary_entity.0 {
                        if let Ok((_, mut listener, _, _, _)) = listeners.get_mut(prime) {
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
                        if let Ok((_, mut listener, pos, area, _)) = listeners.get_mut(prime) {
                            let section = Section::new(*pos, *area);
                            if section.contains(position) {
                                listener.interaction.end.replace(position);
                                focused_entity.0.replace(prime);
                                listener.active = true;
                            }
                            listener.engaged = false;
                        } else {
                            focused_entity.0.take();
                        }
                    } else {
                        focused_entity.0.take();
                    }
                    primary.0.take();
                }
                InteractionPhase::Cancel => {
                    primary.0.take();
                    primary_entity.0.take();
                }
            }
        }
    }
}
