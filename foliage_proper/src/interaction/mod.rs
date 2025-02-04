use crate::coordinate::position::Position;
use crate::coordinate::Logical;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{Event, EventReader};
use bevy_ecs::prelude::IntoSystemConfigs;
use bevy_ecs::query::With;
use bevy_ecs::system::{Query, ResMut, Resource};
mod adapter;
pub(crate) mod listener;

use crate::ash::clip::ResolvedClip;
use crate::foliage::{Foliage, MainMarkers};
use crate::grid::view::ViewAdjustment;
use crate::{
    Attachment, Component, InteractionShape, ResolvedElevation, Section, Stem, Tree, View,
};
pub use adapter::InputSequence;
pub(crate) use adapter::{KeyboardAdapter, MouseAdapter, TouchAdapter};
use listener::InteractionListener;

impl Attachment for Interaction {
    fn attach(foliage: &mut Foliage) {
        foliage
            .main
            .add_systems(interactive_elements.in_set(MainMarkers::Process));
        foliage.world.insert_resource(KeyboardAdapter::default());
        foliage.world.insert_resource(MouseAdapter::default());
        foliage.world.insert_resource(TouchAdapter::default());
        foliage.world.insert_resource(CurrentInteraction::default());
        foliage.enable_queued_event::<Interaction>();
    }
}
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum InteractionPhase {
    Start,
    Moved,
    End,
    Cancel,
}
#[derive(Event, Debug, Copy, Clone)]
pub struct Interaction {
    click_phase: InteractionPhase,
    position: Position<Logical>,
    method: InteractionMethod,
}
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Default)]
pub enum InteractionMethod {
    ScrollWheel,
    #[default]
    TouchScreen,
    Mouse,
}
impl Interaction {
    pub fn new(
        click_phase: InteractionPhase,
        position: Position<Logical>,
        method: InteractionMethod,
    ) -> Self {
        Self {
            click_phase,
            position,
            method,
        }
    }
}
#[derive(Default, Copy, Clone, Debug)]
pub struct Click {
    pub start: Position<Logical>,
    pub current: Position<Logical>,
    pub end: Option<Position<Logical>>,
}
impl Click {
    pub fn new(start: Position<Logical>) -> Self {
        Self {
            start,
            current: start,
            end: None,
        }
    }
}
#[derive(Resource, Default)]
pub struct CurrentInteraction {
    pub(crate) primary: Option<Entity>,
    pub(crate) click: Click,
    pub(crate) method: InteractionMethod,
    pub(crate) last_drag: Position<Logical>,
    pub(crate) pass_through: Vec<Entity>,
    pub(crate) focused: Option<Entity>,
    pub(crate) past_drag: bool,
}
impl CurrentInteraction {
    pub fn click(&self) -> Click {
        self.click
    }
}
#[derive(Event, Copy, Clone, Default)]
pub struct OnClick {}
#[derive(Event, Copy, Clone, Default)]
pub struct Engaged {}
#[derive(Event, Copy, Clone, Default)]
pub struct Dragged {}
#[derive(Event, Copy, Clone, Default)]
pub struct Disengaged {}
#[derive(Component, Copy, Clone)]
pub struct InteractionPropagation {
    grab: bool,
    disable_drag: bool,
}
impl InteractionPropagation {
    pub fn grab() -> Self {
        Self {
            grab: true,
            disable_drag: false,
        }
    }
    pub fn pass_through() -> Self {
        Self {
            grab: false,
            disable_drag: false,
        }
    }
    pub fn disable_drag(mut self) -> Self {
        self.disable_drag = true;
        self
    }
}
#[derive(Component, Copy, Clone, Default)]
pub struct FocusBehavior(pub(crate) bool);
impl FocusBehavior {
    pub fn grab() -> Self {
        Self(false)
    }
    pub fn ignore() -> Self {
        Self(true)
    }
}
impl Default for InteractionPropagation {
    fn default() -> Self {
        Self {
            grab: true,
            disable_drag: false,
        }
    }
}
pub(crate) fn interactive_elements(
    mut reader: EventReader<Interaction>,
    all: Query<(
        Entity,
        &Section<Logical>,
        &ResolvedElevation,
        &ResolvedClip,
        &InteractionPropagation,
        &InteractionShape,
    )>,
    behaviors: Query<&FocusBehavior>,
    mut listeners: Query<&mut InteractionListener>,
    mut current: ResMut<CurrentInteraction>,
    contexts: Query<&Stem>,
    views: Query<Entity, With<View>>,
    mut tree: Tree,
) {
    let events = reader.read().copied().collect::<Vec<_>>();
    if events
        .iter()
        .any(|e| e.click_phase == InteractionPhase::Cancel)
    {
        if let Some(entity) = current.primary.take() {
            tree.trigger_targets(Disengaged {}, entity);
        }
        for entity in current.pass_through.drain(..) {
            tree.trigger_targets(Disengaged {}, entity);
        }
    } else {
        let started = events
            .iter()
            .copied()
            .filter(|i| i.click_phase == InteractionPhase::Start)
            .collect::<Vec<_>>();
        let moved = events
            .iter()
            .copied()
            .filter(|i| i.click_phase == InteractionPhase::Moved)
            .collect::<Vec<_>>();
        let ended = events
            .iter()
            .copied()
            .filter(|i| i.click_phase == InteractionPhase::End)
            .collect::<Vec<_>>();
        if let Some(event) = started.last() {
            if let Some(entity) = current.primary.take() {
                tree.trigger_targets(Disengaged {}, entity);
            }
            for entity in current.pass_through.drain(..) {
                tree.trigger_targets(Disengaged {}, entity);
            }
            current.past_drag = false;
            let mut grabbed_elevation = ResolvedElevation::new(101.0);
            for (entity, section, elevation, clip, propagation, shape) in all.iter() {
                if propagation.grab {
                    if elevation >= &grabbed_elevation {
                        if InteractionListener::is_contained(
                            *shape,
                            *section,
                            *clip,
                            event.position,
                        ) {
                            grabbed_elevation = *elevation;
                            current.primary.replace(entity);
                        }
                    }
                } else {
                    if InteractionListener::is_contained(*shape, *section, *clip, event.position) {
                        current.pass_through.push(entity);
                    }
                }
            }
            if let Some(p) = current.primary {
                current.method = event.method;
                current.pass_through = current
                    .pass_through
                    .drain(..)
                    .filter(|ps| all.get(*ps).unwrap().2 >= &grabbed_elevation)
                    .collect::<Vec<_>>();
                if let Ok(mut listener) = listeners.get_mut(p) {
                    if !listener.disabled() && event.method != InteractionMethod::ScrollWheel {
                        tree.trigger_targets(Engaged {}, p);
                    }
                }
                if !behaviors.get(p).unwrap().0 && event.method != InteractionMethod::ScrollWheel {
                    if let Some(f) = current.focused.replace(p) {
                        if f != p {
                            tree.trigger_targets(Focused {}, p);
                            tree.trigger_targets(Unfocused {}, f);
                        }
                    } else {
                        tree.trigger_targets(Focused {}, p);
                    }
                }
                current.click = Click::new(event.position);
                current.last_drag = event.position;
            }
            for ps in current.pass_through.iter() {
                if let Ok(mut listener) = listeners.get_mut(*ps) {
                    if !listener.disabled() && event.method != InteractionMethod::ScrollWheel {
                        tree.trigger_targets(Engaged {}, *ps);
                    }
                }
            }
            if current.primary.is_none() {
                if let Some(f) = current.focused.take() {
                    tree.trigger_targets(Unfocused {}, f);
                }
            }
        }
        if let Some(event) = moved.last() {
            if let Some(p) = current.primary {
                if !current.past_drag {
                    let scroll_delta = event.position - current.click.start;
                    if scroll_delta.coordinates.a().abs() > InteractionListener::DRAG_THRESHOLD
                        || scroll_delta.coordinates.b().abs() > InteractionListener::DRAG_THRESHOLD
                    {
                        current.past_drag = true;
                        current.last_drag = event.position;
                    }
                } else if !all.get(p).unwrap().4.disable_drag {
                    let diff = current.last_drag - event.position;
                    if let Ok(_) = views.get(p) {
                        tree.entity(p).insert(ViewAdjustment(diff));
                    } else {
                        let mut context = *contexts.get(p).unwrap();
                        while let Some(id) = context.id {
                            if let Ok(_) = views.get(id) {
                                tree.entity(id).insert(ViewAdjustment(diff));
                                break;
                            }
                            if let Ok(up) = contexts.get(id) {
                                context = *up;
                            } else {
                                break;
                            }
                        }
                    }
                }
                current.last_drag = event.position;
                current.click.current = event.position;
                if let Ok(mut listener) = listeners.get_mut(p) {
                    if !listener.disabled() && event.method != InteractionMethod::ScrollWheel {
                        tree.trigger_targets(Dragged {}, p);
                    }
                }
            }
            for ps in current.pass_through.iter() {
                if let Ok(mut listener) = listeners.get_mut(*ps) {
                    if !listener.disabled() && event.method != InteractionMethod::ScrollWheel {
                        tree.trigger_targets(Dragged {}, *ps);
                    }
                }
            }
        }
        if let Some(event) = ended.last() {
            if let Some(p) = current.primary {
                if current.past_drag
                    || event.method == InteractionMethod::ScrollWheel
                        && !all.get(p).unwrap().4.disable_drag
                {
                    let diff = current.last_drag - event.position;
                    if let Ok(_) = views.get(p) {
                        tree.entity(p).insert(ViewAdjustment(diff));
                    } else {
                        let mut context = *contexts.get(p).unwrap();
                        while let Some(id) = context.id {
                            if let Ok(_) = views.get(id) {
                                tree.entity(id).insert(ViewAdjustment(diff));
                                break;
                            }
                            if let Ok(up) = contexts.get(id) {
                                context = *up;
                            } else {
                                break;
                            }
                        }
                    }
                }
                current.click.end.replace(event.position);
                if let Ok(mut listener) = listeners.get_mut(p) {
                    let data = all.get(p).unwrap();
                    if !listener.disabled() && event.method != InteractionMethod::ScrollWheel {
                        if InteractionListener::is_contained(
                            *data.5,
                            *data.1,
                            *data.3,
                            event.position,
                        ) {
                            tree.trigger_targets(OnClick::default(), p);
                        }
                    }
                    tree.trigger_targets(Disengaged {}, p);
                }
            }
            for ps in current.pass_through.drain(..) {
                if let Ok(mut listener) = listeners.get_mut(ps) {
                    let data = all.get(ps).unwrap();
                    if !listener.disabled() && event.method != InteractionMethod::ScrollWheel {
                        if InteractionListener::is_contained(
                            *data.5,
                            *data.1,
                            *data.3,
                            event.position,
                        ) {
                            tree.trigger_targets(OnClick::default(), ps);
                        }
                    }
                    tree.trigger_targets(Disengaged {}, ps);
                }
            }
        }
    }
}
#[derive(Event, Copy, Clone, Debug)]
pub struct Focused {}
#[derive(Event, Copy, Clone, Debug)]
pub struct Unfocused {}
