use crate::EcsExtension;
use crate::coordinate::Logical;
use crate::coordinate::position::Position;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::message::{Message, MessageReader};
use bevy_ecs::prelude::IntoScheduleConfigs;
use bevy_ecs::query::With;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Query, Res, ResMut};
mod adapter;
pub(crate) mod listener;

use crate::ash::clip::ResolvedClip;
use crate::coordinate::elevation::StackKey;
use crate::foliage::{Foliage, MainMarkers};
use crate::grid::view::{Coasting, ScrollInertia, ScrollMomentum, ViewAdjustment};
use crate::{
    Attachment, Component, InteractionShape, Moment, ResolvedElevation, Section, Stem, Tree, View,
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
/// Where an [`Interaction`] sits in a gesture.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum InteractionPhase {
    /// Pointer down, or a wheel notch. Picks the entity the gesture belongs to.
    Start,
    /// Pointer moved while down.
    Moved,
    /// Pointer released -- may resolve to a click, a coast, or nothing.
    End,
    /// The gesture was taken away rather than completed (focus lost, touch cancelled).
    /// Never produces a click.
    Cancel,
}
/// One raw input event, as the platform reported it, queued for `interactive_elements`
/// to resolve against the tree.
#[derive(Message, Debug, Copy, Clone)]
pub struct Interaction {
    click_phase: InteractionPhase,
    position: Position<Logical>,
    method: InteractionMethod,
}
/// What produced an [`Interaction`]. Scroll is kept distinct because it is a discrete
/// pulse rather than continuous tracking: it scales through
/// [`ScrollInertia`](crate::grid::view::ScrollInertia), never drags, and never hands off
/// to a coast.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Default)]
pub enum InteractionMethod {
    ScrollWheel,
    #[default]
    TouchScreen,
    Mouse,
}
impl Interaction {
    /// One input event. Platform adapters queue these; app code rarely builds one
    /// outside tests.
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
/// Where a gesture began, where it is now, and where it ended -- enough for a handler to
/// judge direction and distance without tracking positions itself. `end` is `None` while
/// the pointer is still down.
#[derive(Default, Copy, Clone, Debug)]
pub struct Click {
    pub start: Position<Logical>,
    pub current: Position<Logical>,
    pub end: Option<Position<Logical>>,
}
impl Click {
    /// A gesture beginning at `start`, with `current` there too and no end yet.
    pub fn new(start: Position<Logical>) -> Self {
        Self {
            start,
            current: start,
            end: None,
        }
    }
}
/// The gesture in progress: which entity owns it, where it has travelled, and how fast.
/// One pointer, so one of these for the whole app.
#[derive(Resource, Default)]
pub struct CurrentInteraction {
    pub(crate) primary: Option<Entity>,
    pub(crate) click: Click,
    pub(crate) method: InteractionMethod,
    pub(crate) last_drag: Position<Logical>,
    pub(crate) pass_through: Vec<Entity>,
    pub(crate) focused: Option<Entity>,
    pub(crate) past_drag: bool,
    /// True from `Start` until the matching `End`/`Cancel` -- unlike `primary` (which
    /// deliberately stays set between gestures, cleared only by the *next* `Start`, so a
    /// released entity's `Disengaged` can still fire and its click can still be judged
    /// against where it was originally grabbed) this genuinely reflects "is the pointer
    /// down right now." `coast` needs exactly that: checking `primary.is_some()` alone
    /// is true immediately after every release too (it hasn't been cleared yet), which
    /// would kill a just-started coast on its very first tick.
    pub(crate) pressed: bool,
    /// When the last drag-move's own `ViewAdjustment` diff was computed -- paired with
    /// `velocity` to turn each move's raw px delta into a px/ms rate (see the `moved`
    /// handling in `interactive_elements`).
    pub(crate) last_drag_time: Option<Moment>,
    /// A smoothed (EMA, not instantaneous) px/ms rate of the current drag's own motion --
    /// smoothed so a single noisy/tiny final move right before release doesn't solely
    /// decide whether the release reads as a flick. Read at `ended` to decide whether to
    /// hand off to a [`Coasting`] coast; reset at `started`.
    pub(crate) velocity: Position<Logical>,
}
impl CurrentInteraction {
    /// The current gesture's own start/current/end positions.
    pub fn click(&self) -> Click {
        self.click
    }
}
/// A press and release on the same entity, without exceeding
/// [`DRAG_THRESHOLD`](InteractionListener::DRAG_THRESHOLD). A gesture that became a drag
/// does not also click, even if it happens to end back over the entity it started on.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct OnClick {}
/// This entity has just been grabbed -- pointer down on it. The hook for a pressed
/// visual state.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct Engaged {}
/// The grabbed entity's gesture has passed
/// [`DRAG_THRESHOLD`](InteractionListener::DRAG_THRESHOLD) and is now a drag, not a
/// pending click.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct Dragged {}
/// The gesture that grabbed this entity has ended, however it ended. Fires whether or
/// not an [`OnClick`] also did, so a pressed visual always has somewhere to reset.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct Disengaged {}
/// Whether an entity competes for a gesture or lets it through to whatever is beneath.
#[derive(Component, Copy, Clone)]
pub struct InteractionPropagation {
    grab: bool,
    disable_drag: bool,
}
impl InteractionPropagation {
    /// Competes for the gesture; the topmost grabber under the pointer wins.
    pub fn grab() -> Self {
        Self {
            grab: true,
            disable_drag: false,
        }
    }
    /// Still notified when a gesture crosses it, but never wins one -- for overlays and
    /// decoration that must not eat input.
    pub fn pass_through() -> Self {
        Self {
            grab: false,
            disable_drag: false,
        }
    }
    /// Refuses drag-panning through this entity, so a drag starting here scrolls nothing.
    /// For a knob or slider that owns its own drag. Wheel scrolling is unaffected.
    pub fn disable_drag(mut self) -> Self {
        self.disable_drag = true;
        self
    }
}
/// Whether pressing this entity moves keyboard focus to it.
#[derive(Component, Copy, Clone, Default)]
pub struct FocusBehavior(pub(crate) bool);
impl FocusBehavior {
    /// Takes focus when pressed. The default.
    pub fn grab() -> Self {
        Self(false)
    }
    /// Leaves focus where it is -- for controls pressed alongside a focused field, so a
    /// text input keeps the caret while you press a button next to it.
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
    stack_keys: Query<&StackKey>,
    behaviors: Query<&FocusBehavior>,
    mut listeners: Query<&mut InteractionListener>,
    mut current: ResMut<CurrentInteraction>,
    contexts: Query<&Stem>,
    views: Query<&View>,
    inertias: Query<&ScrollInertia>,
    momentum: Res<ScrollMomentum>,
    mut tree: Tree,
) {
    let events = reader.read().copied().collect::<Vec<_>>();
    if events
        .iter()
        .any(|e| e.click_phase == InteractionPhase::Cancel)
    {
        current.pressed = false;
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
            current.pressed = true;
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
            for (entity, section, _elevation, clip, propagation, shape) in all.iter() {
                // Disabled entities are out of the running entirely, not merely stopped
                // from acting on a grab they won: a disabled overlay sitting on top would
                // otherwise take the gesture on elevation alone and silently swallow
                // input meant for what is underneath.
                if listeners.get(entity).map(|l| l.disabled()).unwrap_or(false) {
                    continue;
                }
                if propagation.grab {
                    if InteractionListener::is_contained(*shape, *section, *clip, event.position) {
                        let wins = match current.primary {
                            None => true,
                            Some(existing) => {
                                stack_keys.get(entity).unwrap() >= stack_keys.get(existing).unwrap()
                            }
                        };
                        if wins {
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
                    .filter(|ps| stack_keys.get(*ps).unwrap() >= stack_keys.get(p).unwrap())
                    .collect::<Vec<_>>();
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
                current.last_drag_time = None;
                current.velocity = Position::default();
            }
            // Focus reconciliation for a non-scroll press: move focus to the grabbed primary if
            // it can take focus, otherwise *clear* focus. Pressing a focus-ignoring element
            // (e.g. Pagination's dots, all `FocusBehavior::ignore()`) or empty space is still
            // "pressing away" from whatever was focused and must blur it -- that's what lets a
            // Dropdown/Popover close when you click outside it onto non-focusable content. Scroll
            // never changes focus at all, so an open overlay stays open while you scroll past it.
            if event.method != InteractionMethod::ScrollWheel {
                let new_focus = current
                    .primary
                    .filter(|p| !behaviors.get(*p).map(|b| b.0).unwrap_or(false));
                if current.focused != new_focus {
                    if let Some(old) = current.focused {
                        tree.trigger_targets(
                            Unfocused {
                                entity: Entity::PLACEHOLDER,
                            },
                            old,
                        );
                    }
                    if let Some(nf) = new_focus {
                        tree.trigger_targets(
                            Focused {
                                entity: Entity::PLACEHOLDER,
                            },
                            nf,
                        );
                    }
                    current.focused = new_focus;
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
                        // seeds a baseline for the *next* Moved event's own velocity
                        // computation -- without this, a fling that only sends a couple of
                        // move samples before release (a real fast flick can) would never
                        // get a `last_drag_time` to diff its first real move against, and
                        // so would never compute a nonzero velocity at all.
                        current.last_drag_time = Some(Moment::now());
                    }
                } else if !all.get(p).unwrap().4.disable_drag {
                    let diff = current.last_drag - event.position;
                    // smoothed (EMA) px/ms rate of this drag's own motion, read at `ended`
                    // to decide whether release velocity is fast enough to hand off to a
                    // `Coasting` coast -- same units/sign convention `diff` (and so
                    // `ViewAdjustment`) already use, so `Coasting::velocity` can be fed
                    // straight back into `ViewAdjustment` unchanged once coasting starts.
                    let now = Moment::now();
                    if let Some(last_time) = current.last_drag_time {
                        let elapsed_ms = now.duration_since(last_time).as_secs_f32() * 1000.0;
                        if elapsed_ms > 0.0 {
                            const SMOOTHING: f32 = 0.35;
                            let instant = diff / elapsed_ms;
                            current.velocity =
                                current.velocity * (1.0 - SMOOTHING) + instant * SMOOTHING;
                        }
                    }
                    current.last_drag_time = Some(now);
                    if let Ok(_) = views.get(p) {
                        // cleared right here, not left to `coast`'s own separate
                        // ancestor-walk check -- this is the one place that knows with
                        // certainty which entity is receiving a *live* pan this frame.
                        // `coast` runs in the same `MainMarkers::Process` set with no
                        // ordering constraint against this system, so relying on it to
                        // notice and cancel independently left a real window where its
                        // own stale, decaying write could land after (and overwrite) the
                        // live one right back, every single frame, for as long as the
                        // coast kept running -- reading as the coast fighting the drag.
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
            current.pressed = false;
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
                    // stale-velocity guard: a fast drag followed by holding perfectly
                    // still (no more Moved events at all -- so `current.velocity` never
                    // gets a fresh EMA sample) for a while before releasing must not
                    // still read as a flick using whatever velocity was last measured,
                    // however long ago that was. A hard recency cutoff, not a continued
                    // exponential decay -- a fast enough swipe held still for a couple of
                    // real seconds would still clear `velocity_threshold` even decayed the
                    // whole time (`decay.powf` never actually reaches zero). Past
                    // `stillness_cutoff_ms` since the last real sample, the pointer had
                    // already stopped, full stop, regardless of how fast it was moving
                    // before that.
                    if let Some(last_time) = current.last_drag_time {
                        let elapsed_ms =
                            Moment::now().duration_since(last_time).as_secs_f32() * 1000.0;
                        if elapsed_ms > momentum.stillness_cutoff_ms {
                            current.velocity = Position::default();
                        }
                    }
                    // wheel scaling: drag stays exactly 1:1 (a raw pointer-drag is
                    // continuous tracking, not a discrete pulse -- this scaling doesn't
                    // apply to it), a wheel tick's delta gets scaled by that target's own
                    // ScrollInertia, which grows the closer together repeated ticks
                    // arrive and resets once they stop -- see ScrollInertia::tick.
                    let wheel_diff = |tree: &mut Tree, target: Entity| -> Position<Logical> {
                        if event.method != InteractionMethod::ScrollWheel {
                            return diff;
                        }
                        let (scale, updated) =
                            inertias.get(target).copied().unwrap_or_default().tick();
                        tree.entity(target).insert(updated);
                        diff * scale
                    };
                    // hands off to a coast only for a real drag/touch release (not a
                    // wheel tick, which already has its own `ScrollInertia`) whose
                    // tracked release velocity clears `ScrollMomentum::velocity_threshold`
                    // -- a slow, deliberate drag that was already settling just stops.
                    //
                    // Attached to the same entity the live pan targeted, so the coast
                    // continues exactly the motion the drag was producing -- including
                    // reaching the real scroller through `OverscrollPropagation` when that
                    // target is a card whose own view has nowhere to go. Picking a
                    // "better" target by walking up to the first view with real scrollable
                    // extent does not work: a card's own content routinely overflows its
                    // box by a hair, so such a walk stops on the card anyway, while the
                    // cases where it does climb higher land the coast somewhere the drag
                    // never touched.
                    let maybe_coast = |tree: &mut Tree, target: Entity| {
                        if event.method == InteractionMethod::ScrollWheel {
                            return;
                        }
                        let speed = current.velocity.left().hypot(current.velocity.top());
                        if speed > momentum.velocity_threshold {
                            tree.entity(target).insert(Coasting {
                                velocity: current.velocity,
                                last_tick: Moment::now(),
                            });
                        }
                    };
                    if let Ok(_) = views.get(p) {
                        let scaled = wheel_diff(&mut tree, p);
                        tree.entity(p).insert(ViewAdjustment(scaled));
                        maybe_coast(&mut tree, p);
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
                                    maybe_coast(&mut tree, id);
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
                    // `!current.past_drag` -- a real drag (crossed `DRAG_THRESHOLD`) that
                    // happens to release back over the same entity's own current bounds
                    // (easy to do on a view that itself just scrolled, e.g. a `ContentsItem`
                    // card that's also a click target) shouldn't *also* count as a click on
                    // it -- position containment alone doesn't distinguish "tapped it" from
                    // "dragged it and let go here by coincidence."
                    if !listener.disabled()
                        && !current.past_drag
                        && event.method != InteractionMethod::ScrollWheel
                    {
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
            let past_drag = current.past_drag;
            for ps in current.pass_through.drain(..) {
                if let Ok(mut listener) = listeners.get_mut(ps) {
                    let data = all.get(ps).unwrap();
                    // same reasoning as the primary's own gate above.
                    if !listener.disabled()
                        && !past_drag
                        && event.method != InteractionMethod::ScrollWheel
                    {
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
/// This entity has taken keyboard focus. Only one entity holds it at a time.
pub struct Focused {}
#[foliage_macros::targeted_event]
#[derive(Copy, Debug)]
/// This entity has lost keyboard focus, to another entity or to a press on empty space.
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
    fn send(
        foliage: &mut Foliage,
        phase: InteractionPhase,
        pos: Position<Logical>,
        method: InteractionMethod,
    ) {
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

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );

        assert_eq!(
            foliage.world.resource::<CurrentInteraction>().primary,
            Some(leaf)
        );
    }

    #[test]
    fn starting_an_interaction_away_from_everything_grabs_nothing() {
        let mut foliage = Foliage::new();
        spawn_grabbable(&mut foliage, 0.0, 0.0, 100.0, 100.0);
        foliage.world.flush();

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(500.0, 500.0),
            InteractionMethod::Mouse,
        );

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

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );

        let primary = foliage.world.resource::<CurrentInteraction>().primary;
        assert_eq!(
            primary,
            Some(front),
            "the more-in-front (up(2)) entity should win over up(1) at the same point"
        );
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
        foliage
            .world
            .trigger_targets(crate::Disable::new(), on_top_disabled);
        foliage.world.flush();

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );

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

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );
        assert!(foliage.world.get::<Marks>(leaf).unwrap().engaged);
        assert!(!foliage.world.get::<Marks>(leaf).unwrap().disengaged);

        send(
            &mut foliage,
            InteractionPhase::End,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );
        assert!(foliage.world.get::<Marks>(leaf).unwrap().disengaged);
    }

    #[test]
    fn releasing_inside_the_grabbed_entitys_own_bounds_fires_onclick() {
        let mut foliage = Foliage::new();
        let leaf = spawn_grabbable(&mut foliage, 0.0, 0.0, 100.0, 100.0);
        foliage.world.flush();

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::End,
            point(55.0, 55.0),
            InteractionMethod::Mouse,
        );

        assert!(foliage.world.get::<Marks>(leaf).unwrap().clicked);
    }

    #[test]
    fn a_real_drag_that_releases_back_inside_the_same_bounds_does_not_fire_onclick() {
        // position containment alone doesn't distinguish "tapped it" from "dragged it and
        // let go here by coincidence" -- a view that scrolled during the drag (a
        // `ContentsItem` card, say, which is also a click target) can easily have its own
        // current bounds still contain the release point even though a real drag happened.
        let mut foliage = Foliage::new();
        let leaf = spawn_grabbable(&mut foliage, 0.0, 0.0, 100.0, 100.0);
        foliage.world.flush();

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(50.0 + InteractionListener::DRAG_THRESHOLD + 5.0, 50.0),
            InteractionMethod::Mouse,
        );
        // released back inside the same 100x100 bounds it started in.
        send(
            &mut foliage,
            InteractionPhase::End,
            point(52.0, 52.0),
            InteractionMethod::Mouse,
        );

        assert!(
            !foliage.world.get::<Marks>(leaf).unwrap().clicked,
            "a gesture that crossed the drag threshold shouldn't also count as a click, \
             regardless of where it happened to release"
        );
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

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::End,
            point(500.0, 500.0),
            InteractionMethod::Mouse,
        );

        assert!(!foliage.world.get::<Marks>(leaf).unwrap().clicked);
        assert!(
            foliage.world.get::<Marks>(leaf).unwrap().disengaged,
            "Disengaged still fires regardless"
        );
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

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(50.0 + InteractionListener::DRAG_THRESHOLD - 1.0, 50.0),
            InteractionMethod::Mouse,
        );

        assert!(
            !foliage.world.resource::<CurrentInteraction>().past_drag,
            "under the threshold"
        );
        assert!(
            foliage.world.get::<Marks>(leaf).unwrap().dragged,
            "but Dragged itself isn't threshold-gated"
        );
    }

    #[test]
    fn crossing_the_drag_threshold_flips_past_drag() {
        let mut foliage = Foliage::new();
        spawn_grabbable(&mut foliage, 0.0, 0.0, 200.0, 200.0);
        foliage.world.flush();

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );
        assert!(
            !foliage.world.resource::<CurrentInteraction>().past_drag,
            "sanity: not yet"
        );

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
        assert_eq!(
            foliage.world.get::<ViewAdjustment>(parent).unwrap().0,
            Position::default()
        );

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(25.0, 25.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(25.0 + InteractionListener::DRAG_THRESHOLD + 5.0, 25.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(80.0, 25.0),
            InteractionMethod::Mouse,
        );

        assert_ne!(
            foliage.world.get::<ViewAdjustment>(parent).unwrap().0,
            Position::default(),
            "the child has no View of its own -- the drag should have walked up its Stem \
             chain and panned its ancestor View instead"
        );
        assert_eq!(
            foliage
                .world
                .get::<ViewAdjustment>(child)
                .unwrap_or(&ViewAdjustment::default())
                .0,
            Position::default(),
            "the child itself isn't a View -- nothing should have been written on it directly"
        );
    }

    fn spawn_grabbable_view(foliage: &mut Foliage) -> Entity {
        let e = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(200.px().as_width()),
                    0.px().as_top().with(200.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with((View::new(), Grid::default(), InteractionListener::new())),
        );
        foliage.world.flush();
        e
    }

    #[test]
    fn touching_a_toc_style_nested_grid_card_stops_the_outer_lists_coast() {
        // reproduces `toc.rs`'s real shape: `content` is a real scrollable `View`; each
        // card branched under it *also* carries its own `Grid` (hence its own View) for
        // purely internal single-cell layout, with a listener (`hepta`) nested one level
        // deeper still inside that. First gesture grabs empty space in `content` itself
        // (nothing else covers that point) and flicks -- `content` should end up
        // coasting. Second gesture touches `hepta`, inside a card, to try to stop it.
        let mut foliage = Foliage::new();
        let content = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(400.px().as_width()),
                    0.px().as_top().with(400.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with((View::new(), Grid::new(1.col().gap(0), 1.row().gap(0)))),
        );
        let card = foliage.world.branch(
            content,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(100.px().as_width()),
                    0.px().as_top().with(100.px().as_height()),
                ))
                .elevate(Elevation::up(2))
                .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
        );
        let hepta = foliage.world.branch(
            card,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(90.px().as_width()),
                    0.px().as_top().with(90.px().as_height()),
                ))
                .elevate(Elevation::up(3))
                .with(InteractionListener::new()),
        );
        let _ = hepta;
        foliage.world.flush();

        // gesture 1: grabbed at (300, 300) -- inside `content`'s own 400x400 box, well
        // outside `card`'s 0..100 corner, so nothing but `content` itself covers this
        // point.
        send(
            &mut foliage,
            InteractionPhase::Start,
            point(300.0, 300.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(300.0 + InteractionListener::DRAG_THRESHOLD + 5.0, 300.0),
            InteractionMethod::Mouse,
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(365.0, 300.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::End,
            point(365.0, 300.0),
            InteractionMethod::Mouse,
        );
        assert!(
            foliage.world.get::<Coasting>(content).is_some(),
            "sanity: the brisk release on empty space should have started coasting on `content`"
        );

        // gesture 2: touch `hepta`, at (50, 50) -- inside the nested card's own listener.
        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );

        assert!(
            foliage.world.get::<Coasting>(content).is_none(),
            "touching the nested card should stop the outer list's own coast"
        );
    }

    #[test]
    fn pressing_a_sibling_card_stops_a_coast_started_from_another_card() {
        // `toc.rs`'s real shape and the real gesture: flick from card A, then press card B
        // to stop it and drag back the other way. A coast attaches where the drag's own
        // pan landed -- card A's own view, the first `View` up from the grab -- so card A
        // and card B are siblings, and no ancestor walk from the press on B reaches the
        // coast on A. Left running, A's decaying `ViewAdjustment` and B's live one both
        // propagate into the same scroller every frame, in opposite directions.
        let mut foliage = Foliage::new();
        let viewport = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(400.px().as_width()),
                    0.px().as_top().with(400.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with((View::new(), Grid::new(1.col().gap(0), 1.row().gap(0)))),
        );
        let content = foliage.world.branch(
            viewport,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(400.px().as_width()),
                    0.px().as_top().with(1200.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(Grid::new(1.col().gap(0), 3.row().gap(0))),
        );
        let mut cards = vec![];
        for row in 1..=3 {
            let card = foliage.world.branch(
                content,
                Leaf::sprout()
                    .at(Location::new().xs(
                        1.col().as_left().with(1.col().as_right()),
                        row.row().as_top().with(row.row().as_bottom()),
                    ))
                    .elevate(Elevation::up(2))
                    .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
            );
            foliage.world.branch(
                card,
                Leaf::sprout()
                    .at(Location::new().xs(
                        1.col().as_left().with(1.col().as_right()),
                        1.row().as_top().with(1.row().as_bottom()),
                    ))
                    .elevate(Elevation::up(3))
                    .with(InteractionListener::new()),
            );
            cards.push(card);
        }
        foliage.world.flush();

        // flick inside card 0 (y 0..400)
        send(&mut foliage, InteractionPhase::Start, point(200.0, 50.0), InteractionMethod::Mouse);
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(200.0, 50.0 + InteractionListener::DRAG_THRESHOLD + 5.0),
            InteractionMethod::Mouse,
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
        send(&mut foliage, InteractionPhase::Moved, point(200.0, 140.0), InteractionMethod::Mouse);
        send(&mut foliage, InteractionPhase::End, point(200.0, 140.0), InteractionMethod::Mouse);
        let coasting: Vec<Entity> = [viewport, content, cards[0], cards[1], cards[2]]
            .into_iter()
            .filter(|e| foliage.world.get::<Coasting>(*e).is_some())
            .collect();
        assert!(
            !coasting.is_empty(),
            "sanity: the flick should have started a coast somewhere"
        );

        // press card 1 (y 400..800) -- a sibling of whatever is coasting
        send(&mut foliage, InteractionPhase::Start, point(200.0, 500.0), InteractionMethod::Mouse);

        for e in coasting {
            assert!(
                foliage.world.get::<Coasting>(e).is_none(),
                "pressing a sibling card must stop the coast on {e:?}, not leave it \
                 fighting the new drag"
            );
        }
    }

    #[test]
    fn a_brisk_release_hands_off_to_a_coast() {
        let mut foliage = Foliage::new();
        let view = spawn_grabbable_view(&mut foliage);

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(25.0, 25.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(25.0 + InteractionListener::DRAG_THRESHOLD + 5.0, 25.0),
            InteractionMethod::Mouse,
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
        // a big move over a short real sleep -- well past the default velocity_threshold
        // (0.15 px/ms), so this should read as a flick, not a settling drag.
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(90.0, 25.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::End,
            point(90.0, 25.0),
            InteractionMethod::Mouse,
        );

        assert!(
            foliage.world.get::<Coasting>(view).is_some(),
            "a brisk release should have started a coast"
        );
    }

    #[test]
    fn a_fast_drag_held_motionless_for_a_while_before_release_does_not_coast() {
        // Drag fast, hold perfectly still, release. No `Moved` events arrive while
        // held, so the velocity EMA keeps its last sample -- the release has to be judged
        // on how long ago that sample was, not on its magnitude alone, or a pointer that
        // has been stationary for seconds still reads as a flick.
        let mut foliage = Foliage::new();
        let view = spawn_grabbable_view(&mut foliage);

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(25.0, 25.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(25.0 + InteractionListener::DRAG_THRESHOLD + 5.0, 25.0),
            InteractionMethod::Mouse,
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
        // same brisk move as `a_brisk_release_hands_off_to_a_coast` -- on its own this
        // would clear the velocity threshold easily.
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(90.0, 25.0),
            InteractionMethod::Mouse,
        );
        // back-dated, same trick `grid::view`'s own coast-decay tests use -- simulates
        // having stood motionless for 3 real seconds without an actual `sleep`.
        foliage.world.resource_mut::<CurrentInteraction>().last_drag_time =
            Some(Moment::now() - crate::TimeDelta::from_secs(3));
        send(
            &mut foliage,
            InteractionPhase::End,
            point(90.0, 25.0),
            InteractionMethod::Mouse,
        );

        assert!(
            foliage.world.get::<Coasting>(view).is_none(),
            "a velocity stale by 3 real seconds of motionless holding shouldn't still coast"
        );
    }

    #[test]
    fn an_arbitrarily_fast_swipe_held_still_past_the_cutoff_never_coasts() {
        // a purely decay-based staleness check (`velocity * decay.powf(elapsed_ms)`) can
        // never guarantee this for *any* fixed pause length: an original velocity high
        // enough always needs proportionally longer to decay under the same fraction, so
        // a fixed real-world pause (2 real seconds, say) is only "enough" for velocities
        // below whatever that specific pause happens to decay past `velocity_threshold` --
        // a fast enough swipe defeats it regardless of how long you wait. A hard recency
        // cutoff doesn't have that problem: past `stillness_cutoff_ms` with no new sample,
        // velocity is zeroed outright, independent of its own original magnitude.
        let mut foliage = Foliage::new();
        let view = spawn_grabbable_view(&mut foliage);

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(25.0, 25.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(25.0 + InteractionListener::DRAG_THRESHOLD + 5.0, 25.0),
            InteractionMethod::Mouse,
        );
        std::thread::sleep(std::time::Duration::from_millis(2));
        // an extreme move over a tiny real sleep -- hundreds of px/ms, far beyond
        // anything a real swipe would produce.
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(525.0, 25.0),
            InteractionMethod::Mouse,
        );
        // just past `ScrollMomentum::default().stillness_cutoff_ms` (150ms) -- nowhere
        // near the multi-second holds the other tests use.
        foliage.world.resource_mut::<CurrentInteraction>().last_drag_time =
            Some(Moment::now() - crate::TimeDelta::from_millis(200));
        send(
            &mut foliage,
            InteractionPhase::End,
            point(525.0, 25.0),
            InteractionMethod::Mouse,
        );

        assert!(
            foliage.world.get::<Coasting>(view).is_none(),
            "past the stillness cutoff, no original velocity should still be able to coast"
        );
    }

    #[test]
    fn a_slow_settling_release_does_not_coast() {
        let mut foliage = Foliage::new();
        let view = spawn_grabbable_view(&mut foliage);

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(25.0, 25.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(25.0 + InteractionListener::DRAG_THRESHOLD + 5.0, 25.0),
            InteractionMethod::Mouse,
        );
        std::thread::sleep(std::time::Duration::from_millis(80));
        // a tiny move over a comparatively long real sleep -- well under the default
        // velocity_threshold, reading as a drag that was already settling before release.
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(37.0, 25.0),
            InteractionMethod::Mouse,
        );
        // a couple px further, not the exact same point as the last Moved -- `ended`
        // recomputes its own `diff` from `current.last_drag` to the release point, so an
        // End at the identical position as the last Moved would (correctly) produce a
        // zero diff there and overwrite the real one, which isn't what this is checking.
        send(
            &mut foliage,
            InteractionPhase::End,
            point(38.0, 25.0),
            InteractionMethod::Mouse,
        );

        assert!(
            foliage.world.get::<Coasting>(view).is_none(),
            "a slow, already-settling release shouldn't hand off to a coast"
        );
        assert_ne!(
            foliage.world.get::<ViewAdjustment>(view).unwrap().0,
            Position::default(),
            "the release's own 1:1 ViewAdjustment should still have been applied"
        );
    }

    #[test]
    fn grabbing_a_coasting_view_again_cancels_the_coast() {
        let mut foliage = Foliage::new();
        let view = spawn_grabbable_view(&mut foliage);
        foliage.world.entity_mut(view).insert(Coasting {
            velocity: point(1.0, 0.0),
            last_tick: Moment::now(),
        });
        foliage.world.flush();

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(25.0, 25.0),
            InteractionMethod::Mouse,
        );

        assert!(
            foliage.world.get::<Coasting>(view).is_none(),
            "touching the view again should cancel any in-flight coast immediately"
        );
    }

    #[test]
    fn a_live_drag_move_on_a_coasting_view_is_not_overwritten_by_the_stale_coast() {
        // `coast` and `interactive_elements` have no ordering between them, so a live
        // pan and a still-running coast can both write the same view's `ViewAdjustment`
        // in one frame, in either order. Cancellation therefore has to happen inline,
        // where the pan is written, rather than being left to a system that might not run
        // until after the stale write has already landed.
        let mut foliage = Foliage::new();
        let view = spawn_grabbable_view(&mut foliage);
        foliage.world.entity_mut(view).insert(Coasting {
            velocity: point(5.0, 0.0),
            last_tick: Moment::now(),
        });
        foliage.world.flush();

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(25.0, 25.0),
            InteractionMethod::Mouse,
        );
        assert!(
            foliage.world.get::<Coasting>(view).is_none(),
            "sanity: Start alone already cancels it"
        );
        // re-insert it, simulating `coast` not yet having gotten around to removing it
        // (or a fresh coast starting back up) by the time the real drag-move lands --
        // the point of this test is that the *move* handler itself is also a removal
        // site, not just Start.
        foliage.world.entity_mut(view).insert(Coasting {
            velocity: point(5.0, 0.0),
            last_tick: Moment::now(),
        });
        foliage.world.flush();

        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(25.0 + InteractionListener::DRAG_THRESHOLD + 5.0, 25.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(60.0, 25.0),
            InteractionMethod::Mouse,
        );

        assert!(
            foliage.world.get::<Coasting>(view).is_none(),
            "a live drag-move on the exact view that's coasting must clear Coasting itself"
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
                .with((
                    InteractionListener::new(),
                    InteractionPropagation::grab().disable_drag(),
                )),
        );
        foliage.world.flush();

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(25.0, 25.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(25.0 + InteractionListener::DRAG_THRESHOLD + 5.0, 25.0),
            InteractionMethod::Mouse,
        );
        send(
            &mut foliage,
            InteractionPhase::Moved,
            point(80.0, 25.0),
            InteractionMethod::Mouse,
        );

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

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );

        assert_eq!(
            foliage.world.resource::<CurrentInteraction>().focused,
            Some(leaf)
        );
        assert!(foliage.world.get::<Marks>(leaf).unwrap().focused);
    }

    #[test]
    fn a_focus_ignoring_leaf_never_takes_focus_itself() {
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

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );

        assert_eq!(foliage.world.resource::<CurrentInteraction>().focused, None);
    }

    #[test]
    fn pressing_a_focus_ignoring_leaf_blurs_a_previously_focused_one() {
        // pressing a `FocusBehavior::ignore()` element (Pagination's dots, say) doesn't take
        // focus itself, but it IS "pressing away" from whatever was focused, so it must blur it.
        // This is exactly what lets an open Dropdown/Popover close when you click outside it onto
        // non-focusable content.
        let mut foliage = Foliage::new();
        let focusable = spawn_grabbable(&mut foliage, 0.0, 0.0, 100.0, 100.0);
        let ignoring = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    200.px().as_left().with(100.px().as_width()),
                    0.px().as_top().with(100.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with((InteractionListener::new(), FocusBehavior::ignore())),
        );
        foliage.world.flush();

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );
        assert_eq!(
            foliage.world.resource::<CurrentInteraction>().focused,
            Some(focusable)
        );

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(250.0, 50.0),
            InteractionMethod::Mouse,
        );
        assert_eq!(
            foliage.world.resource::<CurrentInteraction>().focused,
            None,
            "pressing the focus-ignoring leaf should have cleared focus, not left it on the old one"
        );
        assert!(foliage.world.get::<Marks>(focusable).unwrap().unfocused);
        let _ = ignoring;
    }

    #[test]
    fn scrolling_does_not_change_focus() {
        // an open overlay must stay open while you scroll past it -- scroll never blurs.
        let mut foliage = Foliage::new();
        let focusable = spawn_grabbable(&mut foliage, 0.0, 0.0, 100.0, 100.0);
        foliage.world.flush();

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );
        assert_eq!(
            foliage.world.resource::<CurrentInteraction>().focused,
            Some(focusable)
        );

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(500.0, 500.0),
            InteractionMethod::ScrollWheel,
        );
        assert_eq!(
            foliage.world.resource::<CurrentInteraction>().focused,
            Some(focusable),
            "a scroll should not blur the focused entity"
        );
    }

    #[test]
    fn starting_on_a_new_entity_unfocuses_the_previous_one() {
        let mut foliage = Foliage::new();
        let a = spawn_grabbable(&mut foliage, 0.0, 0.0, 100.0, 100.0);
        let b = spawn_grabbable(&mut foliage, 200.0, 0.0, 100.0, 100.0);
        foliage.world.flush();

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );
        assert_eq!(
            foliage.world.resource::<CurrentInteraction>().focused,
            Some(a)
        );

        send(
            &mut foliage,
            InteractionPhase::Start,
            point(250.0, 50.0),
            InteractionMethod::Mouse,
        );

        assert_eq!(
            foliage.world.resource::<CurrentInteraction>().focused,
            Some(b)
        );
        assert!(foliage.world.get::<Marks>(a).unwrap().unfocused);
        assert!(foliage.world.get::<Marks>(b).unwrap().focused);
    }

    /// Reproduces the exact structure that caused the real Carousel bug: one branch
    /// (`viewport` -> `slot` -> `deep_decorative`) reaches the same *raw* elevation as a
    /// completely different branch (`chrome` -> `chrome_child` -> `interactive`) purely
    /// because the two chains' `up(n)` amounts happen to sum to the same total, in a
    /// different order. `deep_decorative` has no `InteractionListener` (it's a stand-in for
    /// Carousel's own page `Text`); `interactive` does (a stand-in for a pagination dot).
    /// Under the old flat-elevation comparison this was a genuine tie, resolved arbitrarily
    /// by ECS iteration order -- sometimes the decorative branch won and silently ate the
    /// click. `chrome`'s own elevation (`up(2)`) is deliberately more in front than
    /// `viewport`'s (`up(1)`), so under the branch-local comparison this must be
    /// deterministic every time, regardless of how deep either side's own content nests.
    #[test]
    fn a_chrome_branch_beats_deeply_nested_content_in_a_different_branch_despite_a_raw_elevation_tie()
     {
        let mut foliage = Foliage::new();
        let root = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(200.px().as_width()),
                    0.px().as_top().with(200.px().as_height()),
                ))
                .elevate(Elevation::abs(0))
                .with(Grid::default()),
        );
        let viewport = foliage.world.branch(
            root,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(200.px().as_width()),
                    0.px().as_top().with(200.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(Grid::default()),
        );
        let slot = foliage.world.branch(
            viewport,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(200.px().as_width()),
                    0.px().as_top().with(200.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(Grid::default()),
        );
        let deep_decorative = foliage.world.branch(
            slot,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(200.px().as_width()),
                    0.px().as_top().with(200.px().as_height()),
                ))
                .elevate(Elevation::up(2)),
        );
        let chrome = foliage.world.branch(
            root,
            Leaf::sprout()
                .at(Location::new().xs(
                    50.px().as_left().with(50.px().as_width()),
                    50.px().as_top().with(50.px().as_height()),
                ))
                .elevate(Elevation::up(2))
                .with(Grid::default()),
        );
        let chrome_child = foliage.world.branch(
            chrome,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(50.px().as_width()),
                    0.px().as_top().with(50.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(Grid::default()),
        );
        let interactive = foliage.world.branch(
            chrome_child,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(50.px().as_width()),
                    0.px().as_top().with(50.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(InteractionListener::new()),
        );
        foliage.world.flush();

        // sanity: confirm the tie is real before asserting the fix resolves it correctly.
        let deep_raw = foliage
            .world
            .get::<ResolvedElevation>(deep_decorative)
            .unwrap()
            .value();
        let interactive_raw = foliage
            .world
            .get::<ResolvedElevation>(interactive)
            .unwrap()
            .value();
        assert_eq!(
            deep_raw, interactive_raw,
            "sanity: this must actually be a raw-elevation tie for the test to mean anything"
        );

        // both entities occupy the same click point (50, 50) -- inside chrome/interactive's
        // small box, and also inside viewport/slot/deep_decorative's larger covering box.
        send(
            &mut foliage,
            InteractionPhase::Start,
            point(50.0, 50.0),
            InteractionMethod::Mouse,
        );

        assert_eq!(
            foliage.world.resource::<CurrentInteraction>().primary,
            Some(interactive),
            "chrome's own branch (up(2)) is more in front than viewport's (up(1)) -- it must \
             win regardless of how deep either branch's own content nests, not by iteration-\
             order luck on a coincidental raw-elevation tie"
        );
    }
}
