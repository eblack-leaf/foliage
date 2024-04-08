use crate::animate::trigger::Trigger;
use crate::time::timer::TIME_SKIP_RESISTANCE;
use crate::time::{Time, TimeDelta};
use bevy_ecs::prelude::{Component, Entity};
use bevy_ecs::system::{Commands, Query, Res};
use std::collections::HashSet;

pub mod trigger;

#[derive(Copy, Clone)]
struct AnimateTarget(pub Entity);
#[derive(Copy, Clone)]
struct InterpolationPercent(pub f32);
impl InterpolationPercent {
    pub fn remaining() -> Self {
        Self(1.0)
    }
}
#[derive(Default, Copy, Clone)]
pub struct InterpolationExtraction(pub f32);
#[derive(Copy, Clone)]
pub struct Interpolation {
    total: f32,
    remaining: f32,
    factor: f32,
}
impl Interpolation {
    pub fn new(begin: f32, end: f32) -> Self {
        let diff = end - begin;
        Self {
            total: diff.abs(),
            remaining: diff.abs(),
            factor: if diff.is_sign_positive() { 1.0 } else { -1.0 },
        }
    }
}
pub trait Interpolate
where
    Self: Clone + Component,
{
    fn interpolations(&self, end: &Self) -> Vec<Interpolation>;
    fn apply(&self, extracts: Vec<InterpolationExtraction>) -> Self;
}
pub trait Animate {
    fn animate<I: Interpolate>(self, start: Option<I>, end: I, duration: TimeDelta)
        -> Animation<I>;
}
impl Animate for Entity {
    fn animate<I: Interpolate>(
        self,
        start: Option<I>,
        end: I,
        duration: TimeDelta,
    ) -> Animation<I> {
        Animation::<I>::new(self, start, end, duration, InterpolationMethod::Sinusoidal)
    }
}
#[derive(Copy, Clone)]
pub enum InterpolationMethod {
    Sinusoidal,
    Linear,
}
#[derive(Clone)]
struct Interpolator {
    #[allow(unused)]
    method: InterpolationMethod,
    #[allow(unused)]
    current: InterpolationPercent,
    total: TimeDelta,
}
impl Interpolator {
    fn new(interpolation_method: InterpolationMethod, total: TimeDelta) -> Self {
        Self {
            method: interpolation_method,
            current: InterpolationPercent(0.0),
            total,
        }
    }
    fn interpolate(&mut self, elapsed: TimeDelta) -> InterpolationPercent {
        let linear = 1f32
            - self
                .total
                .checked_sub(elapsed)
                .unwrap_or_default()
                .as_millis() as f32
                / self.total.as_millis() as f32;
        InterpolationPercent(linear)
    }
}
#[derive(Component, Clone)]
pub struct Animation<I: Interpolate> {
    started: bool,
    interpolations: Vec<Interpolation>,
    current_value: Option<I>,
    end_value: I,
    duration: TimeDelta,
    target: AnimateTarget,
    interpolator: Interpolator,
    on_end: HashSet<Entity>,
}
impl<I: Interpolate> Animation<I> {
    fn new(
        target: Entity,
        start: Option<I>,
        end: I,
        duration: TimeDelta,
        interpolation_method: InterpolationMethod,
    ) -> Self {
        Self {
            started: false,
            interpolations: vec![],
            current_value: start,
            end_value: end,
            duration,
            target: AnimateTarget(target),
            interpolator: Interpolator::new(interpolation_method, duration),
            on_end: HashSet::new(),
        }
    }
    pub fn with_on_end(mut self, entity: Entity) -> Self {
        self.on_end.insert(entity);
        self
    }
}
pub(crate) fn apply<I: Interpolate>(
    mut query: Query<(Entity, &mut Animation<I>)>,
    mut targets: Query<&mut I>,
    time: Res<Time>,
    mut cmd: Commands,
) {
    let time_elapsed = time.frame_diff().min(TIME_SKIP_RESISTANCE);
    for (entity, mut animation) in query.iter_mut() {
        if !animation.started {
            let start = if let Some(start) = animation.current_value.clone() {
                start
            } else {
                targets.get(animation.target.0).unwrap().clone()
            };
            animation.interpolations = I::interpolations(&start, &animation.end_value);
            tracing::trace!(
                "starting animation: {:?}, w/ length: {:?}",
                entity,
                animation.duration
            );
            animation.current_value.replace(start);
            animation.started = true;
            continue;
        }
        let time_remaining = animation
            .duration
            .checked_sub(time_elapsed)
            .unwrap_or_default();
        animation.duration = time_remaining;
        let percent = if time_remaining.is_zero() {
            tracing::trace!("extract-remaining from {:?}", entity);
            InterpolationPercent::remaining()
        } else {
            tracing::trace!("get interpolation percent for {:?}", entity);
            animation.interpolator.interpolate(time_elapsed)
        };
        let mut extracts = vec![];
        let mut all_done = true;
        for (i, interpolation) in animation.interpolations.iter_mut().enumerate() {
            let amount = interpolation.total * percent.0;
            let extract = amount.min(interpolation.remaining);
            tracing::trace!("extracting {:?} from {:?}", extract, i);
            interpolation.remaining -= extract;
            extracts.push(InterpolationExtraction(extract * interpolation.factor));
            if interpolation.remaining != 0.0 {
                all_done = false;
            }
        }
        if !extracts.is_empty() {
            let new_value = animation.current_value.as_ref().unwrap().apply(extracts);
            animation.current_value.replace(new_value);
            if let Ok(mut value) = targets.get_mut(animation.target.0) {
                *value = animation.current_value.as_ref().unwrap().clone();
            } else {
                // orphaned
                tracing::trace!("orphaned-animation: {:?}", entity);
                cmd.entity(entity).remove::<Animation<I>>();
            }
        }
        if all_done {
            // end anim and not already done from orphaned?
            tracing::trace!("animation done: {:?}", entity);
            for trigger in animation.on_end.iter() {
                cmd.entity(*trigger).insert(Trigger::active());
            }
            cmd.entity(entity).remove::<Animation<I>>();
        }
    }
}