use crate::time::timer::TIME_SKIP_RESISTANCE;
use crate::time::{Time, TimeDelta};
use bevy_ecs::prelude::{Component, Entity, World};
use bevy_ecs::system::{Command, Commands, Query, Res};

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
    fn animate<I: Interpolate>(
        self,
        start: Option<I>,
        end: I,
        duration: TimeDelta,
    ) -> OverwriteAnimation<I>;
    fn composable_animate<I: Interpolate>(
        self,
        start: Option<I>,
        end: I,
        duration: TimeDelta,
    ) -> ComposableAnimation<I>;
}
impl Animate for Entity {
    fn animate<I: Interpolate>(
        self,
        start: Option<I>,
        end: I,
        duration: TimeDelta,
    ) -> OverwriteAnimation<I> {
        OverwriteAnimation(Animation::<I>::new(
            self,
            start,
            end,
            duration,
            InterpolationMethod::Sinusoidal,
        ))
    }

    fn composable_animate<I: Interpolate>(
        self,
        start: Option<I>,
        end: I,
        duration: TimeDelta,
    ) -> ComposableAnimation<I> {
        ComposableAnimation(Animation::<I>::new(
            self,
            start,
            end,
            duration,
            InterpolationMethod::Sinusoidal,
        ))
    }
}
#[derive(Copy, Clone)]
pub enum InterpolationMethod {
    Sinusoidal,
    Linear,
}
#[derive(Clone)]
struct Interpolator {
    method: InterpolationMethod,
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
impl<I: Interpolate> Command for OverwriteAnimation<I> {
    fn apply(self, world: &mut World) {
        world.entity_mut(self.0.target.0).insert(self.0);
    }
}
#[derive(Clone)]
pub struct OverwriteAnimation<I: Interpolate>(pub Animation<I>);
#[derive(Clone)]
pub struct ComposableAnimation<I: Interpolate>(pub Animation<I>);
impl<I: Interpolate> Command for ComposableAnimation<I> {
    fn apply(self, world: &mut World) {
        world.spawn(self.0);
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
        }
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
            cmd.entity(entity).remove::<Animation<I>>();
        }
    }
}
