use crate::grid::location::CreateDiff;
use crate::time::{OnEnd, Time, TimeDelta};
use crate::{Component, Location, Tree, Update};
use bevy_ecs::change_detection::{Mut, ResMut};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Query;
use ease::Ease;
use interpolation::Interpolations;
use runner::AnimationRunner;
use sequence::{Sequence, SequenceTimeRange};
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
}
impl<A: Animate> Animation<A> {
    pub fn new(a: A) -> Self {
        Self {
            anim_target: Default::default(),
            a,
            sequence_time_range: SequenceTimeRange::default(),
            ease: Ease::DECELERATE,
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
}
pub trait Animate
where
    Self: Sized + Send + Sync + 'static + Clone,
{
    fn interpolations(start: &Self, end: &Self) -> Interpolations;
    fn apply(&mut self, interpolations: &mut Interpolations);
}
pub(crate) fn animate<A: Animate + Component>(
    mut anims: Query<(Entity, &mut AnimationRunner<A>)>,
    mut anim_targets: Query<&mut A>,
    time: ResMut<Time>,
    mut sequences: Query<&mut Sequence>,
    mut tree: Tree,
) {
    let frame_diff = time.frame_diff();
    for (anim_entity, mut animation) in anims.iter_mut() {
        if !animation.animation_time.delay.is_zero() {
            animation.animation_time.delay = animation
                .animation_time
                .delay
                .checked_sub(frame_diff)
                .unwrap_or_default();
        } else {
            if !animation.started {
                let target_entity = animation.animation_target;
                if let Ok(a) = anim_targets.get(target_entity) {
                    animation.interpolations =
                        A::interpolations(&a, animation.finish.as_ref().unwrap());
                    animation.started = true;
                    if TypeId::of::<A>() == TypeId::of::<Location>() {
                        *anim_targets.get_mut(target_entity).unwrap() =
                            animation.finish.clone().unwrap();
                        tree.entity(target_entity).insert(CreateDiff(true));
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

fn despawn_and_update_sequence<A: Animate>(
    sequences: &mut Query<&mut Sequence>,
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
        tree.trigger_targets(OnEnd {}, sequence_entity);
        tree.entity(sequence_entity).despawn();
    }
    tree.entity(anim_entity).despawn();
}
