use crate::time::{Time, TimeDelta};
use bevy_ecs::prelude::{Component, Entity};
use bevy_ecs::system::{Commands, Query, Res};

pub mod trigger;
#[derive(Component, Copy, Clone)]
pub struct AnimateTarget(pub Entity);
pub struct InterpolationPercent(pub f32);
pub struct InterpolationExtraction(pub f32);
pub struct Interpolation();
pub trait Interpolate {
    fn interpolations() -> Vec<Interpolation>;
    fn apply(&self, extracts: Vec<InterpolationExtraction>) -> Self;
}
pub trait Animate {
    fn animate<I: Interpolate>(i: I) -> Animation<I>;
}
pub struct Animation<I: Interpolate> {}
impl<I: Interpolate> Animation<I> {
    pub fn to(mut self, i: I, duration: TimeDelta) -> Self {
        // end + duration
        self
    }
}
fn apply<I: Interpolate>(
    query: Query<(Entity, &mut Animation<I>)>,
    mut targets: Query<&mut I>,
    time: Res<Time>,
    mut cmd: Commands,
) {
}