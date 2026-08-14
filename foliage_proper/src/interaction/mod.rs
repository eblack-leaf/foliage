use crate::coordinate::Logical;
use crate::coordinate::position::Position;
use bevy_ecs::entity::Entity;
use bevy_ecs::message::{Message, MessageReader};
use bevy_ecs::prelude::IntoScheduleConfigs;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Query, Res, ResMut};
mod adapter;
pub(crate) mod listener;

use crate::ash::clip::ResolvedClip;
use crate::coordinate::elevation::StackKey;
use crate::foliage::{Foliage, MainMarkers};
use crate::grid::view::{Coasting, ScrollMomentum, ViewAdjustment};
use crate::{
    Attachment, Component, InteractionShape, Moment, Parent, ResolvedElevation, Section, Tree, View,
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
/// pulse rather than continuous tracking: it moves its raw delta, never drags, and never
/// hands off to a coast.
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
    /// The current drag's smoothed px/ms rate -- the same value a release hands off as a
    /// coast's starting velocity.
    pub fn velocity(&self) -> Position<Logical> {
        self.velocity
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
/// The pointer moved while this entity held the gesture. Sent once per frame that carries a
/// move, from the first pixel -- what a knob or a slider driving its own drag consumes.
///
/// This is not the threshold: it fires below
/// [`DRAG_THRESHOLD`](InteractionListener::DRAG_THRESHOLD) as well, and a gesture that never
/// crosses it still reports moves and then [`OnClick`]. For "this became a drag", once, take
/// [`DragStarted`].
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct Dragged {}
/// The gesture holding this entity has passed
/// [`DRAG_THRESHOLD`](InteractionListener::DRAG_THRESHOLD) and is a drag rather than a
/// pending click. Sent once per gesture, before the [`Dragged`] for the same move.
///
/// This is the only announcement of the threshold, and the threshold is what decides whether
/// the release also produces an [`OnClick`] -- so this is what a press-and-hold visual, a
/// drag proxy, or anything that has to commit at the moment a click stops being possible
/// hangs off. Nothing else needs to re-measure travel against the constant.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct DragStarted {}
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
    contexts: Query<&Parent>,
    views: Query<&View>,
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
            tree.send_to(
                Disengaged {
                    entity: Entity::PLACEHOLDER,
                },
                entity,
            );
        }
        for entity in current.pass_through.drain(..) {
            tree.send_to(
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
                tree.send_to(
                    Disengaged {
                        entity: Entity::PLACEHOLDER,
                    },
                    entity,
                );
            }
            for entity in current.pass_through.drain(..) {
                tree.send_to(
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
                if let Ok(listener) = listeners.get_mut(p) {
                    if !listener.disabled() && event.method != InteractionMethod::ScrollWheel {
                        tree.send_to(
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
                        tree.send_to(
                            Unfocused {
                                entity: Entity::PLACEHOLDER,
                            },
                            old,
                        );
                    }
                    if let Some(nf) = new_focus {
                        tree.send_to(
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
                if let Ok(listener) = listeners.get_mut(*ps) {
                    if !listener.disabled() && event.method != InteractionMethod::ScrollWheel {
                        tree.send_to(
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
            // Set at the crossing and read at the sends below, which is the only place the
            // grabbed entity and every pass-through are both in hand. The threshold is
            // crossed by the gesture, not by an entity, so all of them hear about it on the
            // same move.
            let mut crossed_threshold = false;
            if let Some(p) = current.primary {
                if !current.past_drag {
                    let scroll_delta = event.position - current.click.start;
                    if scroll_delta.coordinates.a().abs() > InteractionListener::DRAG_THRESHOLD
                        || scroll_delta.coordinates.b().abs() > InteractionListener::DRAG_THRESHOLD
                    {
                        current.past_drag = true;
                        crossed_threshold = true;
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
                        tree.write_to(p, ViewAdjustment(diff));
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
                                    tree.write_to(id, ViewAdjustment(diff));
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
                // Deliberately outside the `past_drag` branch above: `Dragged` is the per-move
                // stream and fires from the first pixel, which is what a knob or a slider
                // driving its own drag consumes. `DragStarted` is the threshold, and goes out
                // first on the move that crosses it -- an entity hearing both on one move
                // learns that this gesture became a drag before it handles the move itself.
                if let Ok(listener) = listeners.get_mut(p) {
                    if !listener.disabled() && event.method != InteractionMethod::ScrollWheel {
                        if crossed_threshold {
                            tree.send_to(
                                DragStarted {
                                    entity: Entity::PLACEHOLDER,
                                },
                                p,
                            );
                        }
                        tree.send_to(
                            Dragged {
                                entity: Entity::PLACEHOLDER,
                            },
                            p,
                        );
                    }
                }
            }
            for ps in current.pass_through.iter() {
                if let Ok(listener) = listeners.get_mut(*ps) {
                    if !listener.disabled() && event.method != InteractionMethod::ScrollWheel {
                        if crossed_threshold {
                            tree.send_to(
                                DragStarted {
                                    entity: Entity::PLACEHOLDER,
                                },
                                *ps,
                            );
                        }
                        tree.send_to(
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
                    // A pointer drag is one continuous gesture, so the newest position is
                    // the whole story and `current.last_drag` is the right baseline for it.
                    // A wheel is not: each notch is its own self-contained Start+End pair
                    // (`photosynthesis.rs`), positioned `delta` apart at whatever the cursor
                    // was, and a single frame routinely carries several of them -- a
                    // free-spinning wheel, high-resolution wheel events that report a
                    // fraction of a notch at a time, or web, where each DOM event tends to
                    // pump its own cycle. `ended.last()` sees only the final pair, so
                    // measuring against `last_drag` would move one notch's worth and drop
                    // however many others arrived alongside it. Summing every pair the frame
                    // carries is what makes the view travel the distance the wheel actually
                    // turned.
                    //
                    // The two lists zip because the pairs are written adjacent and in order,
                    // one End per Start, so the nth wheel End belongs to the nth wheel Start.
                    let diff = if event.method == InteractionMethod::ScrollWheel {
                        started
                            .iter()
                            .filter(|i| i.method == InteractionMethod::ScrollWheel)
                            .zip(
                                ended
                                    .iter()
                                    .filter(|i| i.method == InteractionMethod::ScrollWheel),
                            )
                            .fold(Position::default(), |acc, (start, end)| {
                                acc + (start.position - end.position)
                            })
                    } else {
                        current.last_drag - event.position
                    };
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
                    // hands off to a coast only for a real drag/touch release (not a
                    // wheel tick, which moves its raw delta and then stops) whose
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
                            tree.write_to(
                                target,
                                Coasting {
                                    velocity: current.velocity,
                                },
                            );
                        }
                    };
                    if let Ok(_) = views.get(p) {
                        tree.write_to(p, ViewAdjustment(diff));
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
                                    tree.write_to(id, ViewAdjustment(diff));
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
                if let Ok(listener) = listeners.get_mut(p) {
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
                            tree.send_to(OnClick::new(), p);
                        }
                    }
                    tree.send_to(
                        Disengaged {
                            entity: Entity::PLACEHOLDER,
                        },
                        p,
                    );
                }
            }
            let past_drag = current.past_drag;
            for ps in current.pass_through.drain(..) {
                if let Ok(listener) = listeners.get_mut(ps) {
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
                            tree.send_to(OnClick::new(), ps);
                        }
                    }
                    tree.send_to(
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
