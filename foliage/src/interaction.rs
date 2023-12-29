use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{DeviceContext, InterfaceContext};
use crate::ginkgo::viewport::ViewportHandle;
use crate::window::ScaleFactor;
use bevy_ecs::component::Component;
use bevy_ecs::event::{Event, EventReader};
use bevy_ecs::prelude::{Entity, IntoSystemConfigs};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, Res, ResMut, Resource};
use winit::event::{MouseButton, Touch};
use crate::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use crate::elm::{Elm, EventStage};
use crate::elm::leaf::{EmptySetDescriptor, Leaf};

impl Leaf for Interaction {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {
    }

    fn attach(elm: &mut Elm) {
        elm.job.container.insert_resource(PrimaryInteraction::default());
        elm.job.container.insert_resource(PrimaryInteractionEntity::default());
        elm.job.container.insert_resource(FocusedEntity::default());
        elm.main().add_systems(
            (set_interaction_listeners.in_set(CoreSet::Interaction),
             clear_engaged.after(ExternalSet::Process),)
        );
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
impl From<Touch> for InteractionId {
    fn from(value: Touch) -> Self {
        Self(InteractionId::INTERACTION_ID_COLLISION_AVOIDANCE + value.id as i32)
    }
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InteractionPhase {
    Begin,
    Update,
    End,
    Cancel,
}
#[derive(Event)]
pub struct InteractionEvent {
    pub phase: InteractionPhase,
    pub id: InteractionId,
    pub location: Position<DeviceContext>,
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
    engaged: bool,
    pub interaction: Interaction,
}
fn clear_engaged(mut engaged: Query<&mut InteractionListener, Changed<InteractionListener>>) {
    for mut e in engaged.iter_mut() {
        if e.engaged {
            e.engaged = false;
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
            }
        } else {
            if ie.id == primary.0.unwrap() {
                match ie.phase {
                    InteractionPhase::Begin => {
                        // skip
                    }
                    InteractionPhase::Update => {
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
                                    listener.engaged = true;
                                }
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
}