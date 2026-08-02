use crate::grid::location::CreateDiff;
use crate::time::{OnEnd, Time, TimeDelta};
use crate::{Component, Location, Resolve, Tree};
use bevy_ecs::change_detection::{Mut, ResMut};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Query;
use ease::Ease;
use interpolation::Interpolations;
use runner::AnimationRunner;
use sequence::{SequenceMarker, SequenceTimeRange};
use std::any::TypeId;

pub(crate) mod ease;
pub(crate) mod interpolation;
pub(crate) mod runner;
pub(crate) mod sequence;
#[derive(Clone)]
/// A tween of one [`Animate`] component from its current value to a target, over a
/// window of time.
///
/// ```ignore
/// tree.animate(
///     Animation::new(Opacity::new(1.0))
///         .targeting(entity)
///         .during(seq)
///         .start(0)
///         .finish(300)
///         .eased(Ease::DECELERATE),
/// );
/// ```
///
/// The start value is whatever the component holds when the animation begins, not
/// something declared here -- so a tween interrupted partway resumes from where it
/// actually is. Starting a second animation of the same component type on the same entity
/// supersedes the first rather than fighting it.
pub struct Animation<A: Animate> {
    pub(crate) anim_target: Option<Entity>,
    pub(crate) a: A,
    pub(crate) sequence_time_range: SequenceTimeRange,
    pub(crate) ease: Ease,
    pub(crate) seq: Entity,
    pub(crate) repeat: Repeat,
    pub(crate) backtrack: bool,
}
/// How many times a tween replays after its first pass -- see [`Animation::loops`] and
/// [`Animation::forever`].
#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub enum Repeat {
    #[default]
    Once,
    Times(u32),
    Forever,
}
impl<A: Animate> Animation<A> {
    /// A tween ending at `a`. Defaults to [`Ease::DECELERATE`], and still needs a target,
    /// a time range, and a sequence -- [`during`](Self::during) is required, not optional.
    /// A single animation with no other timing to coordinate against still gets one of its
    /// own: `.during(tree.sequence())`.
    pub fn new(a: A) -> Self {
        Self {
            anim_target: Default::default(),
            a,
            sequence_time_range: SequenceTimeRange::default(),
            ease: Ease::DECELERATE,
            seq: Entity::PLACEHOLDER,
            repeat: Repeat::Once,
            backtrack: false,
        }
    }
    /// Runs each replay back the way it came, instead of snapping to the start value and
    /// replaying forward: `A -> B -> A -> B`, not `A -> B, A -> B`.
    ///
    /// Which one is right depends on the values, not on taste. A rotation from `0` to `360`
    /// wants the default -- the snap is invisible because the ends look identical, and
    /// backtracking would visibly unwind it. A pulse from `0.4` to `1.0` opacity wants this,
    /// because snapping back is a jump the eye catches every cycle.
    ///
    /// A pass is one traversal either way, so `.loops(1).backtrack()` runs `A -> B` then
    /// `B -> A` and ends where it began, while `.loops(1)` alone ends at `B`.
    pub fn backtrack(mut self) -> Self {
        self.backtrack = true;
        self
    }
    /// Replays this tween `n` more times after its first pass, snapping back to the start
    /// value each time.
    ///
    /// Looping lives inside the animation rather than being rebuilt from a
    /// [`sequence_end`](crate::EcsExtension::sequence_end) chain, and that is the safety
    /// difference: a runner checks its target every frame and tears itself down when the
    /// entity is gone, so a loop *cannot* outlive what it animates. A hand-rolled chain
    /// keeps firing after its page is despawned.
    ///
    /// Stop one early by removing the entity [`animate`](crate::EcsExtension::animate)
    /// returned, or just start another animation on the same target -- the newer one
    /// supersedes it.
    ///
    /// A looping tween settles its sequence on its *first* completed pass, so
    /// [`OnEnd`](crate::OnEnd) still fires once and chains behind it still run; it simply
    /// keeps going afterwards.
    pub fn loops(mut self, n: u32) -> Self {
        self.repeat = Repeat::Times(n);
        self
    }
    /// Replays this tween until it is stopped or its target is despawned -- see
    /// [`loops`](Self::loops) for how it ends and how to stop it.
    pub fn forever(mut self) -> Self {
        self.repeat = Repeat::Forever;
        self
    }
    /// The entity whose component is tweened. Implied when going through
    /// [`Graft::animate`](crate::Graft::animate).
    pub fn targeting(mut self, lh: Entity) -> Self {
        self.anim_target.replace(lh);
        self
    }
    /// Milliseconds from the sequence's own beginning before this tween starts -- a
    /// delay, not a duration.
    pub fn start(mut self, s: u64) -> Self {
        self.sequence_time_range.start = TimeDelta::from_millis(s);
        self
    }
    /// Milliseconds from the sequence's own beginning at which this tween ends. Measured
    /// from the same origin as [`start`](Self::start), so a 0→300 tween runs 300ms and a
    /// 200→300 one runs 100ms after a 200ms wait.
    pub fn finish(mut self, e: u64) -> Self {
        self.sequence_time_range.finish = TimeDelta::from_millis(e);
        self
    }
    /// The curve shaping the motion. [`Ease::DECELERATE`] unless set.
    pub fn eased(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
    /// Joins the sequence from [`sequence`](crate::EcsExtension::sequence), so
    /// [`start`](Self::start)/[`finish`](Self::finish) are measured from its origin and
    /// its [`OnEnd`](crate::OnEnd) waits for this tween.
    ///
    /// Required. Every animation is counted against a sequence, so one that never joins a
    /// real sequence entity has nothing to register with and panics when it starts.
    pub fn during(mut self, seq: Entity) -> Self {
        self.seq = seq;
        self
    }
}
/// Lets a component be tweened by [`Animation`].
///
/// Implementors decompose themselves into scalar channels and reassemble from them. The
/// channel *order* is the contract between the two halves -- `apply` must read the same
/// indices `interpolations` wrote, since nothing else enforces the correspondence.
///
/// Register a custom implementation with
/// [`enable_animation`](crate::Foliage::enable_animation); the built-in animatable types
/// do this themselves.
pub trait Animate
where
    Self: Sized + Send + Sync + 'static + Clone,
{
    /// Decomposes the tween into scalar channels, in a fixed order.
    fn interpolations(start: &Self, end: &Self) -> Interpolations;
    /// Reassembles this frame's value, reading the same channel indices
    /// [`interpolations`](Self::interpolations) wrote.
    fn apply(&mut self, interpolations: &mut Interpolations);
}
pub(crate) fn animate<A: Animate + Component<Mutability = bevy_ecs::component::Mutable>>(
    mut anims: Query<(Entity, &mut AnimationRunner<A>)>,
    mut anim_targets: Query<&mut A>,
    mut create_diffs: Query<&mut CreateDiff>,
    time: ResMut<Time>,
    mut sequences: Query<&mut SequenceMarker>,
    mut tree: Tree,
) {
    let frame_diff = time.frame_diff();
    let now = time.mark().since_beginning();
    // Two `AnimationRunner<A>`s on one entity would both write the same component every
    // frame, and whichever iterated last would win -- so a newer animation supersedes any
    // older one already running. Only animations past their own `.start(delay)` compete:
    // one still waiting has not touched the component yet,
    // so it isn't racing anything; a whole staggered batch pre-created on one entity (e.g.
    // `build_morph`'s square -> pentagon -> heptagon stages, all queued up front with
    // increasing delays) must NOT get torn down to just the first stage just because they
    // share a target and were created in the same tick. Among the actual competitors, the
    // most-recently-created one (by `created_at`, stamped from `Time` on an animation's
    // first tick here -- not `Entity`, whose `Ord` doesn't correlate with spawn order) wins;
    // any other active one targeting the same entity is superseded and torn down
    // immediately, same as if it had finished normally.
    let mut latest_for_target: std::collections::HashMap<Entity, (Entity, TimeDelta)> =
        std::collections::HashMap::new();
    for (anim_entity, mut animation) in anims.iter_mut() {
        if !animation.animation_time.delay.is_zero() {
            continue;
        }
        let created_at = *animation.created_at.get_or_insert(now);
        latest_for_target
            .entry(animation.animation_target)
            .and_modify(|(current_entity, current_created)| {
                if created_at > *current_created {
                    *current_entity = anim_entity;
                    *current_created = created_at;
                }
            })
            .or_insert((anim_entity, created_at));
    }
    for (anim_entity, mut animation) in anims.iter_mut() {
        if !animation.animation_time.delay.is_zero() {
            animation.animation_time.delay = animation
                .animation_time
                .delay
                .checked_sub(frame_diff)
                .unwrap_or_default();
        } else if latest_for_target[&animation.animation_target].0 != anim_entity {
            despawn_and_update_sequence(&mut sequences, &mut tree, anim_entity, &mut animation);
            continue;
        } else {
            if !animation.started {
                let target_entity = animation.animation_target;
                if let Ok(a) = anim_targets.get(target_entity) {
                    animation.interpolations =
                        A::interpolations(&a, animation.finish.as_ref().unwrap());
                    animation.started = true;
                    if TypeId::of::<A>() == TypeId::of::<Location>() {
                        // both writes must land in the same instant: an anchor-target's own
                        // resolve can cascade an `Resolve::<Location>` into this entity at any
                        // point once its `Location` is replaced (e.g. it's someone else's
                        // anchor target too), and if that lands after the `Location` swap but
                        // before a *deferred* `CreateDiff(true)` actually applied, it resolves
                        // with the new target and the stale (usually zero) cached diff --
                        // permanently poisoning this animation's diff to zero. Mutated
                        // directly through the query, right next to the `Location` swap,
                        // so no window exists between the two.
                        *anim_targets.get_mut(target_entity).unwrap() =
                            animation.finish.clone().unwrap();
                        *create_diffs.get_mut(target_entity).unwrap() = CreateDiff(true);
                    }
                } else {
                    despawn_and_update_sequence(
                        &mut sequences,
                        &mut tree,
                        anim_entity,
                        &mut animation,
                    );
                    continue;
                }
            }
            let delta = animation.animation_time.time_delta(frame_diff);
            let percent = animation.easement.percent_changed(delta);
            for i in animation.interpolations.scalars.iter_mut() {
                let d = if percent >= 1.0 {
                    i.finish
                } else {
                    i.start + i.diff * percent
                };
                i.current_value.replace(d);
            }
            if let Ok(mut a) = anim_targets.get_mut(animation.animation_target) {
                a.apply(&mut animation.interpolations);
                tree.send_to(Resolve::<Animation<A>>::new(), animation.animation_target);
            } else {
                despawn_and_update_sequence(&mut sequences, &mut tree, anim_entity, &mut animation);
                tree.despawn(anim_entity);
                continue;
            }
            if percent >= 1f32 {
                let replay = match animation.repeat {
                    Repeat::Once => false,
                    Repeat::Forever => true,
                    Repeat::Times(0) => false,
                    Repeat::Times(n) => {
                        animation.repeat = Repeat::Times(n - 1);
                        true
                    }
                };
                // A pass finished either way, so the sequence settles now -- but only once,
                // since it is despawned when its count reaches zero and later teardowns must
                // not touch it.
                settle_sequence(&mut sequences, &mut tree, &mut animation);
                if replay {
                    // Interpolations carry absolute start/diff, so rewinding the clock
                    // replays from the start value without re-deriving them -- and flipping
                    // them first is the whole of `backtrack`.
                    if animation.backtrack {
                        animation.interpolations.reverse();
                    }
                    animation.animation_time.rewind();
                } else {
                    tree.despawn(anim_entity);
                }
            }
        }
    }
}

/// Reports this animation as done to its sequence, firing [`OnEnd`] and despawning the
/// sequence once nothing is left pending. Idempotent: a loop settles on its first completed
/// pass and then keeps running, and the sequence entity is gone after that -- so a later
/// teardown must not look it up again.
fn settle_sequence<A: Animate>(
    sequences: &mut Query<&mut SequenceMarker>,
    tree: &mut Tree,
    animation: &mut Mut<AnimationRunner<A>>,
) {
    if animation.sequence_settled {
        return;
    }
    animation.sequence_settled = true;
    let sequence_entity = animation.sequence_entity;
    let remaining = {
        let mut marker = sequences.get_mut(sequence_entity).unwrap();
        marker.animations_to_finish -= 1;
        marker.animations_to_finish
    };
    if remaining <= 0 {
        tree.send_to(OnEnd::new(), sequence_entity);
        tree.despawn(sequence_entity);
    }
}
fn despawn_and_update_sequence<A: Animate>(
    sequences: &mut Query<&mut SequenceMarker>,
    tree: &mut Tree,
    anim_entity: Entity,
    animation: &mut Mut<AnimationRunner<A>>,
) {
    settle_sequence(sequences, tree, animation);
    tree.despawn(anim_entity);
}
