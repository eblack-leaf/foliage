use std::collections::HashSet;

use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Commands, Entity};
use bevy_ecs::system::{Query, Res, ResMut};

use crate::action::Signal;
use crate::element::{ActionHandle, IdTable};
use crate::elm::Elm;
use crate::grid::GridPlacement;
use crate::time::{Time, TimeDelta};
use crate::Leaf;

pub(crate) struct EnabledAnimations;
impl Leaf for EnabledAnimations {
    fn attach(elm: &mut Elm) {
        elm.enable_animation::<GridPlacement>();
        // elm.enable_animation::<Color>();
    }
}
#[derive(Component)]
pub(crate) struct Animation<A: Animate> {
    started: bool,
    end: A,
    interpolations: Interpolations,
    easement: Easement,
    sequence_entity: Entity,
    animation_time: AnimationTime,
}
pub(crate) struct AnimationTime {
    accumulated_time: TimeDelta, // use these two to get linear % => use Bézier curve 0-1 to get actual %
    total_time: TimeDelta,
    delay: TimeDelta,
}
impl AnimationTime {
    pub(crate) fn time_delta(&mut self, fd: TimeDelta) -> f32 {
        self.accumulated_time += fd;
        let delta = self.accumulated_time.as_millis() as f32 / self.total_time.as_millis() as f32;
        println!("delta:{}", delta);
        delta.clamp(0.0, 1.0)
    }
}
impl From<SequenceTimeRange> for AnimationTime {
    fn from(value: SequenceTimeRange) -> Self {
        AnimationTime {
            accumulated_time: Default::default(),
            total_time: value.end - value.start,
            delay: value.start,
        }
    }
}
impl<A: Animate> Animation<A> {
    pub(crate) fn new<EASE: Into<Easement>>(
        end: A,
        ease: EASE,
        se: Entity,
        animation_time: AnimationTime,
    ) -> Self {
        Self {
            started: false,
            end,
            interpolations: Interpolations::default(),
            easement: ease.into(),
            sequence_entity: se,
            animation_time,
        }
    }
}
#[derive(Clone, Default)]
pub struct Interpolations {
    scalars: Vec<Interpolation>,
}
impl Interpolations {
    pub fn new() -> Self {
        Self { scalars: vec![] }
    }
    pub fn with(mut self, s: f32, e: f32) -> Self {
        self.scalars.push(Interpolation::new(s, e));
        self
    }
    pub fn read(&mut self, i: usize) -> Option<f32> {
        self.scalars.get_mut(i).unwrap().current_value()
    }
}
#[derive(Copy, Clone)]
pub struct Interpolation {
    start: f32,
    end: f32,
    diff: f32,
    current_value: Option<f32>,
}
impl Interpolation {
    pub fn new(s: f32, e: f32) -> Self {
        Self {
            start: s,
            end: e,
            diff: e - s,
            current_value: None,
        }
    }
    pub fn current_value(&mut self) -> Option<f32> {
        self.current_value.take()
    }
}
pub struct Easement {
    behavior: EasementBehavior,
    // bezier curve else x = y (linear)
}
impl From<EasementBehavior> for Easement {
    fn from(value: EasementBehavior) -> Self {
        Easement::new(value)
    }
}
pub enum EasementBehavior {
    Linear,
    Bezier(/* control points? */),
}
#[derive(Copy, Clone)]
pub struct SequenceTimeRange {
    start: TimeDelta,
    end: TimeDelta,
}
#[derive(Copy, Clone)]
pub struct SequenceTime {
    val: TimeDelta,
}
impl SequenceTime {
    pub fn to(self, end: Self) -> SequenceTimeRange {
        SequenceTimeRange {
            start: self.val,
            end: end.val,
        }
    }
}
pub trait SequenceTiming {
    fn sec(self) -> SequenceTime;
    fn millis(self) -> SequenceTime;
}
impl SequenceTiming for u32 {
    fn sec(self) -> SequenceTime {
        SequenceTime {
            val: TimeDelta::from_secs(self as u64),
        }
    }

    fn millis(self) -> SequenceTime {
        SequenceTime {
            val: TimeDelta::from_millis(self as u64),
        }
    }
}
impl Easement {
    pub fn percent_changed(&mut self, d: f32) -> f32 {
        match self.behavior {
            EasementBehavior::Linear => {
                d
            }
            EasementBehavior::Bezier(/* curve */) => {
                todo!()
            }
        }
    }
    pub(crate) fn new(behavior: EasementBehavior) -> Self {
        Self { behavior }
    }
}
pub trait Animate
where
    Self: Sized + Send + Sync + 'static + Component,
{
    fn interpolations(start: &Self, end: &Self) -> Interpolations;
    fn apply(&mut self, interpolations: &mut Interpolations);
}
#[derive(Component, Default)]
pub struct Sequence {
    pub(crate) animations_to_finish: i32,
    pub(crate) on_end: OnEnd,
}
#[derive(Default)]
pub struct OnEnd {
    actions: HashSet<ActionHandle>,
}
pub(crate) fn animate<A: Animate>(
    mut anims: Query<(Entity, &mut Animation<A>, &mut A)>,
    time: ResMut<Time>,
    mut sequences: Query<&mut Sequence>,
    id_table: Res<IdTable>,
    mut cmd: Commands,
) {
    let frame_diff = time.frame_diff();
    for (anim_entity, mut animation, mut a) in anims.iter_mut() {
        if !animation.animation_time.delay.is_zero() {
            animation.animation_time.delay = animation
                .animation_time
                .delay
                .checked_sub(frame_diff)
                .unwrap_or_default();
        } else {
            if !animation.started {
                animation.interpolations = A::interpolations(&a, &animation.end);
                animation.started = true;
            }

            let delta = animation.animation_time.time_delta(frame_diff);
            let percent = animation.easement.percent_changed(delta);
            for i in animation.interpolations.scalars.iter_mut() {
                let d = i.start + i.diff * percent;
                i.current_value.replace(d);
            }
            a.apply(&mut animation.interpolations);
            if percent >= 1f32 {
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
                    for handle in sequences
                        .get_mut(sequence_entity)
                        .unwrap()
                        .on_end
                        .actions
                        .iter()
                    {
                        let e = id_table.lookup_action(handle.clone()).unwrap();
                        cmd.entity(e).insert(Signal::active());
                    }
                    cmd.entity(anim_entity).remove::<Animation<A>>();
                    cmd.entity(sequence_entity).despawn();
                }
            }
        }
    }
}