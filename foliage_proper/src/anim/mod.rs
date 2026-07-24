use crate::EcsExtension;
use crate::grid::location::CreateDiff;
use crate::time::{OnEnd, Time, TimeDelta};
use crate::{Component, Location, Tree, Update};
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
pub struct Animation<A: Animate> {
    pub(crate) anim_target: Option<Entity>,
    pub(crate) a: A,
    pub(crate) sequence_time_range: SequenceTimeRange,
    pub(crate) ease: Ease,
    pub(crate) seq: Entity,
}
impl<A: Animate> Animation<A> {
    pub fn new(a: A) -> Self {
        Self {
            anim_target: Default::default(),
            a,
            sequence_time_range: SequenceTimeRange::default(),
            ease: Ease::DECELERATE,
            seq: Entity::PLACEHOLDER,
        }
    }
    pub fn targeting(mut self, lh: Entity) -> Self {
        self.anim_target.replace(lh);
        self
    }
    pub fn start(mut self, s: u64) -> Self {
        self.sequence_time_range.start = TimeDelta::from_millis(s);
        self
    }
    pub fn finish(mut self, e: u64) -> Self {
        self.sequence_time_range.finish = TimeDelta::from_millis(e);
        self
    }
    pub fn eased(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
    pub fn during(mut self, seq: Entity) -> Self {
        self.seq = seq;
        self
    }
}
pub trait Animate
where
    Self: Sized + Send + Sync + 'static + Clone,
{
    fn interpolations(start: &Self, end: &Self) -> Interpolations;
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
    // if a new animation starts on an entity that already has one of the same component
    // type actively running (e.g. a fade-out started while an earlier fade-in on the same
    // entity hasn't finished yet), both `AnimationRunner<A>`s would keep writing the same
    // component every frame -- whichever happens to iterate last each tick wins, which is
    // exactly the nondeterministic "fade-in randomly keeps winning" race this guards
    // against. Only "actively running" (delay already expired) animations compete here --
    // one still waiting on its own `.start(delay)` hasn't touched the component at all yet,
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
                        // resolve can cascade an `Update::<Location>` into this entity at any
                        // point once its `Location` is replaced (e.g. it's someone else's
                        // anchor target too), and if that lands after the `Location` swap but
                        // before a *deferred* `CreateDiff(true)` actually applied, it resolves
                        // with the new target and the stale (usually zero) cached diff --
                        // permanently poisoning this animation's diff to zero. Deferred insert
                        // used to open exactly that window; a direct query mutation, right next
                        // to the `Location` swap, closes it.
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
                tree.trigger_targets(Update::<Animation<A>>::new(), animation.animation_target);
            } else {
                despawn_and_update_sequence(&mut sequences, &mut tree, anim_entity, &mut animation);
                tree.entity(anim_entity).despawn();
                continue;
            }
            if percent >= 1f32 {
                despawn_and_update_sequence(&mut sequences, &mut tree, anim_entity, &mut animation);
                tree.entity(anim_entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EcsExtension, Elevation, Foliage, Leaf, Opacity, Sprout};
    use std::thread::sleep;
    use std::time::Duration;

    /// Replicates a page-content entrance fade-in still mid-flight when the navigator
    /// starts its own fade-out on the same entity: two `AnimationRunner<Opacity>`s would
    /// otherwise both write `Opacity` every frame, and whichever iterates last each tick
    /// wins nondeterministically -- reading as "the fade-in randomly keeps winning". Only
    /// the newer (fade-out) animation should survive and drive the value.
    #[test]
    fn a_newer_animation_supersedes_an_older_one_on_the_same_entity() {
        let mut foliage = Foliage::new();
        let entity = foliage
            .world
            .leaf(Leaf::sprout().elevate(Elevation::up(1)).with(Opacity::new(0.0)));
        foliage.world.flush();

        let fade_in_seq = foliage.world.sequence();
        let fade_in = foliage.world.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(entity)
                .during(fade_in_seq)
                .start(0)
                .finish(1000)
                .eased(Ease::Linear),
        );
        foliage.world.flush();

        // let the fade-in actually start and make some partial progress.
        sleep(Duration::from_millis(16));
        foliage.main.run(&mut foliage.world);
        foliage.world.flush();
        assert!(
            foliage.world.get::<AnimationRunner<Opacity>>(fade_in).is_some(),
            "fade-in should still be alive and mid-flight"
        );

        // now start a fade-out on the same entity, before the fade-in has finished.
        let fade_out_seq = foliage.world.sequence();
        let fade_out = foliage.world.animate(
            Animation::new(Opacity::new(0.0))
                .targeting(entity)
                .during(fade_out_seq)
                .start(0)
                .finish(100)
                .eased(Ease::Linear),
        );
        foliage.world.flush();

        sleep(Duration::from_millis(16));
        foliage.main.run(&mut foliage.world);
        foliage.world.flush();

        assert!(
            foliage.world.get::<AnimationRunner<Opacity>>(fade_in).is_none(),
            "the older fade-in should have been superseded and torn down"
        );
        assert!(
            foliage.world.get::<AnimationRunner<Opacity>>(fade_out).is_some(),
            "the newer fade-out should still be driving the entity"
        );

        for _ in 0..10 {
            sleep(Duration::from_millis(16));
            foliage.main.run(&mut foliage.world);
            foliage.world.flush();
        }
        let value = foliage.world.get::<Opacity>(entity).unwrap().value;
        assert!(
            value < 0.5,
            "opacity should have followed the fade-out toward 0, not the superseded fade-in toward 1 (got {value})"
        );
    }

    /// Replicates `build_morph`'s square -> pentagon -> heptagon batch: several
    /// non-overlapping, staggered animations on the *same* entity, all created up front in
    /// one go via increasing `.start(t)`. None of these are racing each other -- only one
    /// is ever actually active (delay expired) at a time -- so the supersede logic must
    /// leave the later, still-delayed stages alone rather than tearing them down just
    /// because they share a target and were created in the same tick as an earlier stage.
    #[test]
    fn a_staggered_batch_of_animations_on_the_same_entity_all_get_their_turn() {
        let mut foliage = Foliage::new();
        let entity = foliage
            .world
            .leaf(Leaf::sprout().elevate(Elevation::up(1)).with(Opacity::new(0.0)));
        foliage.world.flush();

        let seq = foliage.world.sequence();
        foliage.world.animate(
            Animation::new(Opacity::new(0.3))
                .targeting(entity)
                .during(seq)
                .start(0)
                .finish(50)
                .eased(Ease::Linear),
        );
        foliage.world.animate(
            Animation::new(Opacity::new(0.6))
                .targeting(entity)
                .during(seq)
                .start(50)
                .finish(100)
                .eased(Ease::Linear),
        );
        foliage.world.animate(
            Animation::new(Opacity::new(0.9))
                .targeting(entity)
                .during(seq)
                .start(100)
                .finish(150)
                .eased(Ease::Linear),
        );
        foliage.world.flush();

        for _ in 0..30 {
            sleep(Duration::from_millis(16));
            foliage.main.run(&mut foliage.world);
            foliage.world.flush();
        }

        let value = foliage.world.get::<Opacity>(entity).unwrap().value;
        assert!(
            (value - 0.9).abs() < 0.05,
            "the final staggered stage should have actually run and landed near 0.9, got {value} \
             -- if an earlier stage got superseded/torn down instead of waiting its turn, later \
             stages would never activate and this would stall at an earlier target"
        );
    }
}

fn despawn_and_update_sequence<A: Animate>(
    sequences: &mut Query<&mut SequenceMarker>,
    tree: &mut Tree,
    anim_entity: Entity,
    animation: &mut Mut<AnimationRunner<A>>,
) {
    let sequence_entity = animation.sequence_entity;
    sequences
        .get_mut(sequence_entity)
        .unwrap()
        .animations_to_finish -= 1;
    if sequences
        .get_mut(sequence_entity)
        .unwrap()
        .animations_to_finish
        <= 0
    {
        tree.trigger_targets(OnEnd::new(), sequence_entity);
        tree.entity(sequence_entity).despawn();
    }
    tree.entity(anim_entity).despawn();
}
