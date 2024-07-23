use std::collections::HashSet;
use std::slice::IterMut;

use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Commands, Entity};
use bevy_ecs::system::{Query, Res, ResMut};

use crate::action::Signal;
use crate::element::{ActionHandle, IdTable};
use crate::time::{Time, TimeDelta};

// when make => get current section of entity + set in new.queued_offset
// read ending-offset from new.offset (default == 0/0/0/0)
#[derive(Component)]
pub(crate) struct Animation<A: Animate> {
    started: bool,
    end: A,
    interpolations: Interpolations,
    easement: Easement,
    sequence_entity: Entity,
}
impl<A: Animate> Animation<A> {
    pub(crate) fn new<EASE: Into<Easement>>(end: A, ease: EASE, se: Entity) -> Self {
        Self {
            started: false,
            end,
            interpolations: Interpolations::default(),
            easement: ease.into(),
            sequence_entity: se,
        }
    }
}
#[derive(Clone, Default)]
pub struct Interpolations {
    scalars: Vec<Interpolation>,
}
impl Interpolations {
    pub fn read(&mut self) -> IterMut<'_, Interpolation> {
        self.scalars.iter_mut()
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
    pub fn current_value(&mut self) -> Option<f32> {
        self.current_value.take()
    }
}
pub struct Easement {
    // bezier curve
    accumulated_time: TimeDelta, // use these two to get linear % => use Bézier curve 0-1 to get actual %
    total_time: TimeDelta,
}
impl Easement {
    pub fn delta(&mut self, fd: TimeDelta) -> f32 {
        // clamp to 0/100% or 0-1
        todo!()
    }
}
pub trait Animate
where
    Self: Sized + Send + Sync + 'static + Component,
{
    fn interpolations(start: &Self, end: &Self) -> Interpolations;
    fn apply(&mut self, interpolations: &mut Interpolations);
}
#[derive(Component)]
pub struct Sequence {
    animations_to_finish: i32,
    on_end: OnEnd,
}
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
    for (anim_entity, mut animation, mut a) in anims.iter_mut() {
        if !animation.started {
            animation.interpolations = A::interpolations(&a, &animation.end);
            animation.started = true;
        }
        let frame_diff = time.frame_diff();
        let percent = animation.easement.delta(frame_diff);
        for i in animation.interpolations.scalars.iter_mut() {
            let d = i.start + i.diff * percent;
            i.current_value.replace(d);
        }
        a.apply(&mut animation.interpolations);
        if percent == 1f32 {
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
