use crate::coordinate::position::Position;
use crate::coordinate::Logical;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{Event, EventReader};
use bevy_ecs::prelude::IntoSystemConfigs;
use bevy_ecs::system::{Query, ResMut, Resource};
mod adapter;
pub(crate) mod listener;

use crate::ash::clip::ResolvedClip;
use crate::foliage::{Foliage, MainMarkers};
use crate::{Attachment, Component, InteractionShape, ResolvedElevation, Section, Tree};
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
    pub(crate) pass_through: Vec<Entity>,
    pub(crate) focused: Option<Entity>,
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
}
impl InteractionPropagation {
    pub fn grab() -> Self {
        Self { grab: true }
    }
    pub fn pass_through() -> Self {
        Self { grab: false }
    }
}
impl Default for InteractionPropagation {
    fn default() -> Self {
        Self { grab: true }
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
    mut ls: Query<&mut InteractionListener>,
    mut current: ResMut<CurrentInteraction>,
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
            current.primary.take();
            current.pass_through.clear();
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
                    current.pass_through.push(entity);
                }
            }
            if let Some(p) = current.primary {
                if let Ok(mut listener) = ls.get_mut(p) {
                    if !listener.disabled() {
                        if event.from_scroll && !listener.listen_scroll_wheel {
                            // no trigger Engaged / Disengaged stuff but still process for overscroll
                        }
                    }
                } else {
                    // TODO keep for doing scroll stuff in moved but no listener process
                }
                // all.get_mut(p).unwrap().1.click = Click::new(event.position);
                // all.get_mut(p).unwrap().1.last_drag = event.position;
                // tree.trigger_targets(Engaged {}, p);
                // if let Some(f) = current.focused.replace(p) {
                //     if f != p {
                //         tree.trigger_targets(Focused {}, p);
                //         tree.trigger_targets(Unfocused {}, f);
                //     }
                // } else {
                //     tree.trigger_targets(Focused {}, p);
                // }
            }
            for ps in current.pass_through.iter() {
                // let mut listener = all.get_mut(ps).unwrap().1;
                // listener.click = Click::new(event.position);
                // listener.last_drag = event.position;
                // tree.trigger_targets(Engaged {}, ps);
                // if current.primary.is_none() {
                //     if let Some(f) = current.focused.replace(ps) {
                //         if f != ps {
                //             tree.trigger_targets(Focused {}, ps);
                //             tree.trigger_targets(Unfocused {}, f);
                //         }
                //     } else {
                //         tree.trigger_targets(Focused {}, ps);
                //     }
                // }
            }
            if current.primary.is_none() {
                if let Some(f) = current.focused.take() {
                    tree.trigger_targets(Unfocused {}, f);
                }
            }
        }
        if let Some(event) = moved.last() {
            // TODO if no View(primary) => recursive-up stems to find one w/ View to do last_drag + ViewAdjustment(diff) to
            // TODO above runs regardless of if let Ok(listener) ...
            // if let Some(p) = current.primary {
            //     let scroll_delta = event.position - all.get(p).unwrap().1.click.start;
            //     if scroll_delta.coordinates.a().abs() > InteractionListener::DRAG_THRESHOLD
            //         || scroll_delta.coordinates.b().abs() > InteractionListener::DRAG_THRESHOLD
            //     {
            //         tree.trigger_targets(Disengaged {}, p);
            //         if let Some(f) = current.focused.take() {
            //             if f == p {
            //                 tree.trigger_targets(Unfocused {}, p);
            //             }
            //         }
            //         tree.trigger_targets(Unfocused {}, p);
            //         current.primary.take();
            //         if let Some(ps) = current.pass_through {
            //             if let Ok(mut listener) = all.get_mut(ps) {
            //                 listener.1.click = Click::new(event.position);
            //                 listener.1.last_drag = event.position;
            //                 if let Some(f) = current.focused.replace(ps) {
            //                     if f != ps {
            //                         tree.trigger_targets(Focused {}, ps);
            //                         tree.trigger_targets(Unfocused {}, f);
            //                     }
            //                 } else {
            //                     tree.trigger_targets(Focused {}, ps);
            //                 }
            //             }
            //         }
            //     }
            // }
            // if let Some(p) = current.primary {
            //     if let Ok(mut listener) = all.get_mut(p) {
            //         listener.1.click.current = event.position;
            //         if listener.1.scroll {
            //             let diff = listener.1.last_drag - event.position;
            //             tree.entity(listener.0).insert(ViewAdjustment(diff));
            //         }
            //         tree.trigger_targets(Dragged {}, p);
            //     }
            // } else {
            //     if let Some(ps) = current.pass_through {
            //         if let Ok(mut listener) = all.get_mut(ps) {
            //             listener.1.click.current = event.position;
            //             if listener.1.scroll {
            //                 let diff = listener.1.last_drag - event.position;
            //                 tree.entity(listener.0).insert(ViewAdjustment(diff));
            //             }
            //             listener.1.last_drag = event.position;
            //             tree.trigger_targets(Dragged {}, ps);
            //         }
            //     }
            // }
        }
        if let Some(event) = ended.last() {
            // if let Some(p) = current.primary {
            //     if let Ok(mut listener) = all.get_mut(p) {
            //         if event.from_scroll && listener.1.scroll {
            //             let diff = listener.1.last_drag - event.position;
            //             tree.entity(p).insert(ViewAdjustment(diff));
            //         }
            //         if listener
            //             .1
            //             .is_contained(*listener.2, *listener.4, event.position)
            //         {
            //             listener.1.click.end.replace(event.position);
            //             tree.trigger_targets(OnClick::default(), p);
            //         }
            //         tree.trigger_targets(Disengaged {}, p);
            //     }
            // }
            // if let Some(ps) = current.pass_through.take() {
            //     if let Ok(mut listener) = all.get_mut(ps) {
            //         if event.from_scroll && listener.1.scroll {
            //             let diff = listener.1.last_drag - event.position;
            //             tree.entity(ps).insert(ViewAdjustment(diff));
            //         }
            //         listener.1.click.end.replace(event.position);
            //         tree.trigger_targets(OnClick::default(), ps);
            //         tree.trigger_targets(Disengaged {}, ps);
            //     }
            // }
        }
    }
}
#[derive(Event, Copy, Clone, Debug)]
pub struct Focused {}
#[derive(Event, Copy, Clone, Debug)]
pub struct Unfocused {}
