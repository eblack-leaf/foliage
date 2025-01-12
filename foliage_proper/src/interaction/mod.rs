use crate::coordinate::position::Position;
use crate::coordinate::Logical;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{Event, EventReader};
use bevy_ecs::prelude::IntoSystemConfigs;
use bevy_ecs::system::{Query, ResMut, Resource};
mod adapter;
pub(crate) mod listener;

use crate::ash::clip::ClipSection;
use crate::foliage::{Foliage, MainMarkers};
use crate::grid::view::ViewAdjustment;
use crate::{Attachment, ResolvedElevation, Section, Tree};
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
    from_scroll: bool,
}
impl Interaction {
    pub fn new(
        click_phase: InteractionPhase,
        position: Position<Logical>,
        from_scroll: bool,
    ) -> Self {
        Self {
            click_phase,
            position,
            from_scroll,
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
pub(crate) struct CurrentInteraction {
    pub(crate) primary: Option<Entity>,
    pub(crate) pass_through: Option<Entity>,
}
#[derive(Event, Copy, Clone, Default)]
pub struct OnClick {}
pub(crate) fn interactive_elements(
    mut reader: EventReader<Interaction>,
    mut listeners: Query<(
        Entity,
        &mut InteractionListener,
        &Section<Logical>,
        &ResolvedElevation,
        Option<&ClipSection>,
    )>,
    mut current: ResMut<CurrentInteraction>,
    mut tree: Tree,
) {
    let events = reader.read().copied().collect::<Vec<_>>();
    if events
        .iter()
        .any(|e| e.click_phase == InteractionPhase::Cancel)
    {
        current.primary.take();
        current.pass_through.take();
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
            current.primary.take();
            current.pass_through.take();
            let mut grabbed_elevation = ResolvedElevation::new(101.0);
            let mut pass_through_elevation = ResolvedElevation::new(101.0);
            for (entity, listener, section, elevation, clip) in listeners.iter_mut() {
                if !listener.scroll && event.from_scroll || listener.disabled() {
                    continue;
                }
                if listener.is_contained(*section, clip.copied(), event.position) {
                    if listener.pass_through {
                        if elevation >= &pass_through_elevation {
                            current.pass_through.replace(entity);
                            pass_through_elevation = *elevation;
                        }
                    } else if elevation >= &grabbed_elevation {
                        current.primary.replace(entity);
                        grabbed_elevation = *elevation;
                    }
                }
            }
            if let Some(p) = current.primary {
                listeners.get_mut(p).unwrap().1.click = Click::new(event.position);
                // TODO tree.trigger_targets(EngagedBegin::default(), p);
            }
            if let Some(ps) = current.pass_through {
                let mut listener = listeners.get_mut(ps).unwrap().1;
                listener.click = Click::new(event.position);
                listener.last_drag = event.position;
            }
        }
        if let Some(event) = moved.last() {
            if let Some(p) = current.primary {
                let scroll_delta = event.position - listeners.get(p).unwrap().1.click.start;
                if scroll_delta.coordinates.a() > InteractionListener::DRAG_THRESHOLD
                    || scroll_delta.coordinates.b() > InteractionListener::DRAG_THRESHOLD
                {
                    // TODO tree.trigger_targets(EngagedEnd::default(), p);
                    current.primary.take();
                    if let Some(ps) = current.pass_through {
                        let mut listener = listeners.get_mut(ps).unwrap();
                        listener.1.click = Click::new(event.position);
                        listener.1.last_drag = event.position;
                    }
                }
            }
            if let Some(p) = current.primary {
                listeners.get_mut(p).unwrap().1.click.current = event.position;
            } else {
                if let Some(ps) = current.pass_through {
                    let mut listener = listeners.get_mut(ps).unwrap();
                    listener.1.click.current = event.position;
                    if listener.1.scroll {
                        let diff = listener.1.last_drag - event.position;
                        tree.entity(listener.0).insert(ViewAdjustment(diff));
                    }
                    listener.1.last_drag = event.position;
                }
            }
        }
        if let Some(event) = ended.last() {
            if let Some(p) = current.primary {
                let mut listener = listeners.get_mut(p).unwrap();
                if listener
                    .1
                    .is_contained(*listener.2, listener.4.copied(), event.position)
                {
                    listener.1.click.end.replace(event.position);
                    tree.trigger_targets(OnClick::default(), p);
                } else {
                    // TODO tree.trigger_targets(EngagedEnd::default(), p);
                }
            }
            if let Some(ps) = current.pass_through.take() {
                let mut listener = listeners.get_mut(ps).unwrap();
                if event.from_scroll && listener.1.scroll {
                    let diff = listener.1.last_drag - event.position;
                    tree.entity(ps).insert(ViewAdjustment(diff));
                }
                listener.1.click.end.replace(event.position);
                tree.trigger_targets(OnClick::default(), ps);
            }
        }
    }
}
