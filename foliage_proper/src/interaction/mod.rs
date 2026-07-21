use crate::EcsExtension;
use crate::coordinate::Logical;
use crate::coordinate::position::Position;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::message::{Message, MessageReader};
use bevy_ecs::prelude::IntoScheduleConfigs;
use bevy_ecs::query::With;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Query, ResMut};
mod adapter;
pub(crate) mod listener;

use crate::ash::clip::ResolvedClip;
use crate::foliage::{Foliage, MainMarkers};
use crate::grid::view::{ScrollMomentum, ViewAdjustment};
use crate::{
    Attachment, Component, InteractionShape, ResolvedElevation, Section, Stem, Tree, View,
};
pub use adapter::{InputSequence, Key, Modifiers, PhysicalInputSequence, PhysicalKey};
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
#[derive(Message, Debug, Copy, Clone)]
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
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct OnClick {}
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct Engaged {}
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct Dragged {}
#[foliage_macros::targeted_event]
#[derive(Copy)]
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
    mut reader: MessageReader<Interaction>,
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
    momentums: Query<&ScrollMomentum>,
    mut tree: Tree,
) {
    let events = reader.read().copied().collect::<Vec<_>>();
    if events
        .iter()
        .any(|e| e.click_phase == InteractionPhase::Cancel)
    {
        if let Some(entity) = current.primary.take() {
            tree.trigger_targets(
                Disengaged {
                    entity: Entity::PLACEHOLDER,
                },
                entity,
            );
        }
        for entity in current.pass_through.drain(..) {
            tree.trigger_targets(
                Disengaged {
                    entity: Entity::PLACEHOLDER,
                },
                entity,
            );
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
                tree.trigger_targets(
                    Disengaged {
                        entity: Entity::PLACEHOLDER,
                    },
                    entity,
                );
            }
            for entity in current.pass_through.drain(..) {
                tree.trigger_targets(
                    Disengaged {
                        entity: Entity::PLACEHOLDER,
                    },
                    entity,
                );
            }
            current.past_drag = false;
            let mut grabbed_elevation = ResolvedElevation::new(101.0);
            for (entity, section, elevation, clip, propagation, shape) in all.iter() {
                // a disabled entity must not compete for the grab at all -- geometry and
                // elevation alone used to decide this, so a disabled-but-still-elevated
                // entity (an app hiding a page-level button behind a modal, say) could win
                // it purely by sitting on top, silently eating clicks/scrolls meant for
                // whatever's actually beneath it even though its own OnClick/Engaged/
                // Dragged were already correctly gated off downstream.
                if listeners.get(entity).map(|l| l.disabled()).unwrap_or(false) {
                    continue;
                }
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
                if !behaviors.get(p).unwrap().0 && event.method != InteractionMethod::ScrollWheel {
                    if let Some(f) = current.focused.replace(p) {
                        if f != p {
                            tree.trigger_targets(
                                Focused {
                                    entity: Entity::PLACEHOLDER,
                                },
                                p,
                            );
                            tree.trigger_targets(
                                Unfocused {
                                    entity: Entity::PLACEHOLDER,
                                },
                                f,
                            );
                        }
                    } else {
                        tree.trigger_targets(
                            Focused {
                                entity: Entity::PLACEHOLDER,
                            },
                            p,
                        );
                    }
                }
                if let Ok(mut listener) = listeners.get_mut(p) {
                    if !listener.disabled() && event.method != InteractionMethod::ScrollWheel {
                        tree.trigger_targets(
                            Engaged {
                                entity: Entity::PLACEHOLDER,
                            },
                            p,
                        );
                    }
                }
                current.click = Click::new(event.position);
                current.last_drag = event.position;
            } else {
                if let Some(f) = current.focused.take() {
                    tree.trigger_targets(
                        Unfocused {
                            entity: Entity::PLACEHOLDER,
                        },
                        f,
                    );
                }
            }
            for ps in current.pass_through.iter() {
                if let Ok(mut listener) = listeners.get_mut(*ps) {
                    if !listener.disabled() && event.method != InteractionMethod::ScrollWheel {
                        tree.trigger_targets(
                            Engaged {
                                entity: Entity::PLACEHOLDER,
                            },
                            *ps,
                        );
                    }
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
                            // `p`'s own `disable_drag` (checked above) only covers "the
                            // grabbed entity refuses to let anything pan" (a knob/cursor
                            // with no view of its own). It says nothing about a *view*
                            // reached by walking up from some other grabbed content (a
                            // Carousel page's own author content, say) that wants to
                            // refuse drag-panning itself regardless of what's grabbed
                            // inside it -- so that view's own `disable_drag` is checked
                            // here too, independently. A disabled view doesn't stop the
                            // search, though -- it keeps walking up for the next ancestor
                            // view instead: touch has no separate wheel channel, so a drag
                            // that isn't meant for this view (Carousel swiping its own
                            // pages, say) must still be able to reach whatever scrollable
                            // ancestor further out IS meant to receive it (the page behind
                            // the Carousel, on mobile where dragging is the only scroll
                            // input there is).
                            let mut wrote = false;
                            if let Ok(_) = views.get(id) {
                                if !all.get(id).unwrap().4.disable_drag {
                                    tree.entity(id).insert(ViewAdjustment(diff));
                                    wrote = true;
                                }
                            }
                            if wrote {
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
                        tree.trigger_targets(
                            Dragged {
                                entity: Entity::PLACEHOLDER,
                            },
                            p,
                        );
                    }
                }
            }
            for ps in current.pass_through.iter() {
                if let Ok(mut listener) = listeners.get_mut(*ps) {
                    if !listener.disabled() && event.method != InteractionMethod::ScrollWheel {
                        tree.trigger_targets(
                            Dragged {
                                entity: Entity::PLACEHOLDER,
                            },
                            *ps,
                        );
                    }
                }
            }
        }
        if let Some(event) = ended.last() {
            if let Some(p) = current.primary {
                // `disable_drag` suppresses *pointer*-drag panning specifically (e.g. the
                // text_input cursor and the slider knob both drag-subscribe themselves and
                // don't want that drag to also pan an ancestor's `View`) -- it was also
                // gating scroll-wheel here, an unrelated interaction method that only ever
                // sends Start+End (no Moved), so `current.past_drag` never becomes true for
                // it either. Together that meant a `disable_drag` entity swallowed every
                // wheel-scroll over it with no `ViewAdjustment` ever inserted, anywhere --
                // e.g. scrolling while the mouse sat over an active (grabbed) text cursor did
                // nothing instead of scrolling the input. Wheel-scroll should always be
                // allowed through regardless of drag suppression.
                if current.past_drag || event.method == InteractionMethod::ScrollWheel {
                    let diff = current.last_drag - event.position;
                    // wheel scaling: drag stays exactly 1:1 (a raw pointer-drag is
                    // continuous tracking, not a discrete pulse -- momentum doesn't apply
                    // to it), a wheel tick's delta gets scaled by that target's own
                    // ScrollMomentum, which grows the closer together repeated ticks
                    // arrive and resets once they stop -- see ScrollMomentum::tick.
                    let wheel_diff = |tree: &mut Tree, target: Entity| -> Position<Logical> {
                        if event.method != InteractionMethod::ScrollWheel {
                            return diff;
                        }
                        let (scale, updated) =
                            momentums.get(target).copied().unwrap_or_default().tick();
                        tree.entity(target).insert(updated);
                        diff * scale
                    };
                    if let Ok(_) = views.get(p) {
                        let scaled = wheel_diff(&mut tree, p);
                        tree.entity(p).insert(ViewAdjustment(scaled));
                    } else {
                        let mut context = *contexts.get(p).unwrap();
                        while let Some(id) = context.id {
                            // same reasoning as the move-phase walk-up above: the specific
                            // view being targeted gets its own disable_drag check,
                            // independent of `p`'s -- wheel still bypasses it here too,
                            // same as it bypasses `p`'s own check -- and a view that
                            // declines the drag doesn't stop the search, it keeps walking
                            // up for the next ancestor view instead (see the move-phase
                            // comment above for why: touch has no separate wheel channel).
                            let mut wrote = false;
                            if let Ok(_) = views.get(id) {
                                if !all.get(id).unwrap().4.disable_drag
                                    || event.method == InteractionMethod::ScrollWheel
                                {
                                    let scaled = wheel_diff(&mut tree, id);
                                    tree.entity(id).insert(ViewAdjustment(scaled));
                                    wrote = true;
                                }
                            }
                            if wrote {
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
                            tree.trigger_targets(OnClick::new(), p);
                        }
                    }
                    tree.trigger_targets(
                        Disengaged {
                            entity: Entity::PLACEHOLDER,
                        },
                        p,
                    );
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
                            tree.trigger_targets(OnClick::new(), ps);
                        }
                    }
                    tree.trigger_targets(
                        Disengaged {
                            entity: Entity::PLACEHOLDER,
                        },
                        ps,
                    );
                }
            }
        }
    }
}
#[foliage_macros::targeted_event]
#[derive(Copy, Debug)]
pub struct Focused {}
#[foliage_macros::targeted_event]
#[derive(Copy, Debug)]
pub struct Unfocused {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Elevation, Grid, GridExt, Sprout};
    use crate::{Leaf, Location, Trigger};

    fn point(x: f32, y: f32) -> Position<Logical> {
        Position::logical((x, y))
    }

    /// Queues an `Interaction` and runs the `main` schedule once -- `interactive_elements`
    /// is a `main`-schedule system driven by a queued `Message`, not an observer, so unlike
    /// every other test in this suite, a bare `world.flush()` never invokes it.
    fn send(foliage: &mut Foliage, phase: InteractionPhase, pos: Position<Logical>, method: InteractionMethod) {
        foliage.queue(Interaction::new(phase, pos, method));
        foliage.main.run(&mut foliage.world);
    }

    #[derive(Component, Default)]
    struct Marks {
        engaged: bool,
        dragged: bool,
        disengaged: bool,
        clicked: bool,
        focused: bool,
        unfocused: bool,
    }
    fn mark_engaged(trigger: Trigger<Engaged>, mut q: Query<&mut Marks>) {
        if let Ok(mut m) = q.get_mut(trigger.event_target()) {
            m.engaged = true;
        }
    }
    fn mark_dragged(trigger: Trigger<Dragged>, mut q: Query<&mut Marks>) {
        if let Ok(mut m) = q.get_mut(trigger.event_target()) {
            m.dragged = true;
        }
    }
    fn mark_disengaged(trigger: Trigger<Disengaged>, mut q: Query<&mut Marks>) {
        if let Ok(mut m) = q.get_mut(trigger.event_target()) {
            m.disengaged = true;
        }
    }
    fn mark_clicked(trigger: Trigger<OnClick>, mut q: Query<&mut Marks>) {
        if let Ok(mut m) = q.get_mut(trigger.event_target()) {
            m.clicked = true;
        }
    }
    fn mark_focused(trigger: Trigger<Focused>, mut q: Query<&mut Marks>) {
        if let Ok(mut m) = q.get_mut(trigger.event_target()) {
            m.focused = true;
        }
    }
    fn mark_unfocused(trigger: Trigger<Unfocused>, mut q: Query<&mut Marks>) {
        if let Ok(mut m) = q.get_mut(trigger.event_target()) {
            m.unfocused = true;
        }
    }

    fn spawn_grabbable(foliage: &mut Foliage, left: f32, top: f32, w: f32, h: f32) -> Entity {
        let e = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    left.px().as_left().with(w.px().as_width()),
                    top.px().as_top().with(h.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with((InteractionListener::new(), Marks::default())),
        );
        foliage.world.flush();
        foliage
            .world
            .entity_mut(e)
            .observe(mark_engaged)
            .observe(mark_dragged)
            .observe(mark_disengaged)
            .observe(mark_clicked)
            .observe(mark_focused)
            .observe(mark_unfocused);
        e
    }

    #[test]
    fn starting_an_interaction_over_a_leaf_grabs_it_as_the_primary() {
        let mut foliage = Foliage::new();
        let leaf = spawn_grabbable(&mut foliage, 0.0, 0.0, 100.0, 100.0);
        foliage.world.flush();

        send(&mut foliage, InteractionPhase::Start, point(50.0, 50.0), InteractionMethod::Mouse);

        assert_eq!(foliage.world.resource::<CurrentInteraction>().primary, Some(leaf));
    }

    #[test]
    fn starting_an_interaction_away_from_everything_grabs_nothing() {
        let mut foliage = Foliage::new();
        spawn_grabbable(&mut foliage, 0.0, 0.0, 100.0, 100.0);
        foliage.world.flush();

        send(&mut foliage, InteractionPhase::Start, point(500.0, 500.0), InteractionMethod::Mouse);

        assert_eq!(foliage.world.resource::<CurrentInteraction>().primary, None);
    }

    #[test]
    fn the_higher_elevation_of_two_overlapping_leaves_wins_the_grab() {
        let mut foliage = Foliage::new();
        let back = spawn_grabbable(&mut foliage, 0.0, 0.0, 100.0, 100.0);
        let front = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(100.px().as_width()),
                    0.px().as_top().with(100.px().as_height()),
                ))
                .elevate(Elevation::up(2))
                .with(InteractionListener::new()),
        );
        foliage.world.flush();

        send(&mut foliage, InteractionPhase::Start, point(50.0, 50.0), InteractionMethod::Mouse);

        let primary = foliage.world.resource::<CurrentInteraction>().primary;
        assert_eq!(primary, Some(front), "the more-in-front (up(2)) entity should win over up(1) at the same point");
        assert_ne!(primary, Some(back));
    }

    #[test]
    fn a_disabled_entity_does_not_compete_for_the_grab_even_when_sitting_on_top() {
        // the exact scenario `interactive_elements`' own comment documents: an app hiding a
        // page-level button behind a modal shouldn't let the (disabled, but still elevated)
        // button silently eat clicks meant for whatever's actually beneath it.
        let mut foliage = Foliage::new();
        let underneath = spawn_grabbable(&mut foliage, 0.0, 0.0, 100.0, 100.0);
        let on_top_disabled = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(100.px().as_width()),
                    0.px().as_top().with(100.px().as_height()),
                ))
                .elevate(Elevation::up(2))
                .with(InteractionListener::new()),
        );
        foliage.world.flush();
        foliage.world.trigger_targets(crate::Disable::new(), on_top_disabled);
        foliage.world.flush();

        send(&mut foliage, InteractionPhase::Start, point(50.0, 50.0), InteractionMethod::Mouse);

        assert_eq!(
            foliage.world.resource::<CurrentInteraction>().primary,
            Some(underneath),
            "the disabled entity sits on top but must be skipped entirely for grab purposes"
        );
    }

    #[test]
    fn engaged_fires_on_start_and_disengaged_fires_on_end_for_the_grabbed_entity() {
        let mut foliage = Foliage::new();
        let leaf = spawn_grabbable(&mut foliage, 0.0, 0.0, 100.0, 100.0);
        foliage.world.flush();

        send(&mut foliage, InteractionPhase::Start, point(50.0, 50.0), InteractionMethod::Mouse);
        assert!(foliage.world.get::<Marks>(leaf).unwrap().engaged);
        assert!(!foliage.world.get::<Marks>(leaf).unwrap().disengaged);

        send(&mut foliage, InteractionPhase::End, point(50.0, 50.0), InteractionMethod::Mouse);
        assert!(foliage.world.get::<Marks>(leaf).unwrap().disengaged);
    }

    #[test]
    fn releasing_inside_the_grabbed_entitys_own_bounds_fires_onclick() {
        let mut foliage = Foliage::new();
        let leaf = spawn_grabbable(&mut foliage, 0.0, 0.0, 100.0, 100.0);
        foliage.world.flush();

        send(&mut foliage, InteractionPhase::Start, point(50.0, 50.0), InteractionMethod::Mouse);
        send(&mut foliage, InteractionPhase::End, point(55.0, 55.0), InteractionMethod::Mouse);

        assert!(foliage.world.get::<Marks>(leaf).unwrap().clicked);
    }

    #[test]
    fn releasing_outside_the_grabbed_entitys_own_bounds_does_not_fire_onclick() {
        // the grabbed entity stays primary for the whole gesture (it doesn't get dropped
        // just because the pointer wandered off it) -- but OnClick specifically requires
        // the release point to still be within its own bounds, e.g. dragging off a button
        // before releasing shouldn't fire its click.
        let mut foliage = Foliage::new();
        let leaf = spawn_grabbable(&mut foliage, 0.0, 0.0, 100.0, 100.0);
        foliage.world.flush();

        send(&mut foliage, InteractionPhase::Start, point(50.0, 50.0), InteractionMethod::Mouse);
        send(&mut foliage, InteractionPhase::End, point(500.0, 500.0), InteractionMethod::Mouse);

        assert!(!foliage.world.get::<Marks>(leaf).unwrap().clicked);
        assert!(foliage.world.get::<Marks>(leaf).unwrap().disengaged, "Disengaged still fires regardless");
    }

    #[test]
    fn a_small_move_stays_under_the_drag_threshold_but_still_fires_dragged() {
        // Dragged fires on every Moved event for the primary entity, unconditionally -- it's
        // only the actual View-pan that's gated behind crossing DRAG_THRESHOLD (`past_drag`).
        // A composite that wants "moved at all" (a custom drag handle, say) and one that
        // wants "moved enough to count as a real drag" (auto-pan) are reading two different
        // signals here, not the same one at two different sensitivities.
        let mut foliage = Foliage::new();
        let leaf = spawn_grabbable(&mut foliage, 0.0, 0.0, 200.0, 200.0);
        foliage.world.flush();

        send(&mut foliage, InteractionPhase::Start, point(50.0, 50.0), InteractionMethod::Mouse);
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(50.0 + InteractionListener::DRAG_THRESHOLD - 1.0, 50.0),
            InteractionMethod::Mouse,
        );

        assert!(!foliage.world.resource::<CurrentInteraction>().past_drag, "under the threshold");
        assert!(foliage.world.get::<Marks>(leaf).unwrap().dragged, "but Dragged itself isn't threshold-gated");
    }

    #[test]
    fn crossing_the_drag_threshold_flips_past_drag() {
        let mut foliage = Foliage::new();
        spawn_grabbable(&mut foliage, 0.0, 0.0, 200.0, 200.0);
        foliage.world.flush();

        send(&mut foliage, InteractionPhase::Start, point(50.0, 50.0), InteractionMethod::Mouse);
        assert!(!foliage.world.resource::<CurrentInteraction>().past_drag, "sanity: not yet");

        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(50.0 + InteractionListener::DRAG_THRESHOLD + 5.0, 50.0),
            InteractionMethod::Mouse,
        );
        assert!(foliage.world.resource::<CurrentInteraction>().past_drag);
    }

    #[test]
    fn dragging_a_view_less_child_walks_up_to_pan_the_nearest_ancestor_view() {
        let mut foliage = Foliage::new();
        let parent = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(200.px().as_width()),
                    0.px().as_top().with(200.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with((View::new(), Grid::default())),
        );
        let child = foliage.world.branch(
            parent,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(50.px().as_width()),
                    0.px().as_top().with(50.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(InteractionListener::new()),
        );
        foliage.world.flush();
        assert_eq!(foliage.world.get::<ViewAdjustment>(parent).unwrap().0, Position::default());

        send(&mut foliage, InteractionPhase::Start, point(25.0, 25.0), InteractionMethod::Mouse);
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(25.0 + InteractionListener::DRAG_THRESHOLD + 5.0, 25.0),
            InteractionMethod::Mouse,
        );
        send(&mut foliage, InteractionPhase::Moved, point(80.0, 25.0), InteractionMethod::Mouse);

        assert_ne!(
            foliage.world.get::<ViewAdjustment>(parent).unwrap().0,
            Position::default(),
            "the child has no View of its own -- the drag should have walked up its Stem \
             chain and panned its ancestor View instead"
        );
        assert_eq!(
            foliage.world.get::<ViewAdjustment>(child).unwrap_or(&ViewAdjustment::default()).0,
            Position::default(),
            "the child itself isn't a View -- nothing should have been written on it directly"
        );
    }

    #[test]
    fn disable_drag_on_the_grabbed_entity_blocks_panning_any_ancestor_view() {
        let mut foliage = Foliage::new();
        let parent = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(200.px().as_width()),
                    0.px().as_top().with(200.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with((View::new(), Grid::default())),
        );
        let child = foliage.world.branch(
            parent,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(50.px().as_width()),
                    0.px().as_top().with(50.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with((InteractionListener::new(), InteractionPropagation::grab().disable_drag())),
        );
        foliage.world.flush();

        send(&mut foliage, InteractionPhase::Start, point(25.0, 25.0), InteractionMethod::Mouse);
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(25.0 + InteractionListener::DRAG_THRESHOLD + 5.0, 25.0),
            InteractionMethod::Mouse,
        );
        send(&mut foliage, InteractionPhase::Moved, point(80.0, 25.0), InteractionMethod::Mouse);

        assert_eq!(
            foliage.world.get::<ViewAdjustment>(parent).unwrap().0,
            Position::default(),
            "the grabbed entity's own disable_drag should have suppressed the pan entirely, \
             mirroring how a slider knob or the text-input cursor keep their own drag local"
        );
    }

    #[test]
    fn starting_on_a_focusable_leaf_sets_it_as_focused() {
        let mut foliage = Foliage::new();
        let leaf = spawn_grabbable(&mut foliage, 0.0, 0.0, 100.0, 100.0);
        foliage.world.flush();

        send(&mut foliage, InteractionPhase::Start, point(50.0, 50.0), InteractionMethod::Mouse);

        assert_eq!(foliage.world.resource::<CurrentInteraction>().focused, Some(leaf));
        assert!(foliage.world.get::<Marks>(leaf).unwrap().focused);
    }

    #[test]
    fn starting_on_a_focus_ignoring_leaf_does_not_change_focus() {
        let mut foliage = Foliage::new();
        let leaf = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(100.px().as_width()),
                    0.px().as_top().with(100.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with((InteractionListener::new(), FocusBehavior::ignore())),
        );
        foliage.world.flush();

        send(&mut foliage, InteractionPhase::Start, point(50.0, 50.0), InteractionMethod::Mouse);

        assert_eq!(foliage.world.resource::<CurrentInteraction>().focused, None);
    }

    #[test]
    fn starting_on_a_new_entity_unfocuses_the_previous_one() {
        let mut foliage = Foliage::new();
        let a = spawn_grabbable(&mut foliage, 0.0, 0.0, 100.0, 100.0);
        let b = spawn_grabbable(&mut foliage, 200.0, 0.0, 100.0, 100.0);
        foliage.world.flush();

        send(&mut foliage, InteractionPhase::Start, point(50.0, 50.0), InteractionMethod::Mouse);
        assert_eq!(foliage.world.resource::<CurrentInteraction>().focused, Some(a));

        send(&mut foliage, InteractionPhase::Start, point(250.0, 50.0), InteractionMethod::Mouse);

        assert_eq!(foliage.world.resource::<CurrentInteraction>().focused, Some(b));
        assert!(foliage.world.get::<Marks>(a).unwrap().unfocused);
        assert!(foliage.world.get::<Marks>(b).unwrap().focused);
    }
}
