use bevy_ecs::change_detection::Mut;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Commands, Entity};
use bevy_ecs::system::{Query, Res, ResMut};

use crate::action::Signal;
use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinates, LogicalContext};
use crate::element::{IdTable, OnEnd, Opacity, TargetHandle};
use crate::elm::Elm;
use crate::grid::GridPlacement;
use crate::time::{Time, TimeDelta};
use crate::Leaf;

pub(crate) struct EnabledAnimations;
impl Leaf for EnabledAnimations {
    fn attach(elm: &mut Elm) {
        elm.enable_animation::<GridPlacement>();
        elm.enable_animation::<Color>();
        elm.enable_animation::<Opacity>();
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
    animation_target: TargetHandle,
    grid_placement_started: bool,
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
        target: TargetHandle,
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
            animation_target: target,
            grid_placement_started: false,
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
#[derive(Copy, Clone)]
pub struct ControlPoints {
    a: Coordinates,
    b: Coordinates,
}
impl ControlPoints {
    pub fn new<A: Into<Coordinates>, B: Into<Coordinates>>(a: A, b: B) -> Self {
        Self {
            a: a.into().clamped(0.0, 1.0),
            b: b.into().clamped(0.0, 1.0),
        }
    }
}
pub struct Easement {
    behavior: Ease,
}
impl From<Ease> for Easement {
    fn from(value: Ease) -> Self {
        Easement::new(value)
    }
}
pub enum Ease {
    Linear,
    Bezier(ControlPoints),
}
impl Ease {
    pub const DECELERATE: Self = Self::Bezier(ControlPoints {
        a: Coordinates::new(0.05, 0.7),
        b: Coordinates::new(0.1, 1.0),
    });
    pub const ACCELERATE: Self = Self::Bezier(ControlPoints {
        a: Coordinates::new(0.3, 0.0),
        b: Coordinates::new(0.8, 0.15),
    });
    pub const EMPHASIS: Self = Self::Bezier(ControlPoints {
        a: Coordinates::new(0.68, 0.0),
        b: Coordinates::new(0.0, 1.0),
    });
    pub const INWARD: Self = Self::Bezier(ControlPoints {
        a: Coordinates::new(0.29, 0.1),
        b: Coordinates::new(0.36, 0.92),
    });
}
impl Easement {
    pub fn percent_changed(&mut self, d: f32) -> f32 {
        match self.behavior {
            Ease::Linear => d,
            Ease::Bezier(points) => {
                let base = Coordinates::from((0, 0));
                let end = Coordinates::from((1, 1));
                (1f32 - d).powi(3) * base.vertical()
                    + 3f32 * (1f32 - d).powi(2) * d * points.a.vertical()
                    + 3f32 * (1f32 - d) * d.powi(2) * points.b.vertical()
                    + d.powi(3) * end.vertical()
            }
        }
    }
    pub(crate) fn new(behavior: Ease) -> Self {
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
fn start_grid_placement_anim(
    target: Entity,
    mut query: &mut Query<(
        &mut GridPlacement,
        &Position<LogicalContext>,
        &Area<LogicalContext>,
    )>,
    new_grid_placement: GridPlacement,
) {
    let current_pos = *query.get(target).unwrap().1;
    let current_area = *query.get(target).unwrap().2;
    let current = Section::new(current_pos, current_area);
    let mut altered = new_grid_placement.clone();
    altered.queued_offset.replace(current);
    *query.get_mut(target).unwrap().0 = altered;
}

pub(crate) fn animate_grid_placement(
    mut anims: Query<(Entity, &mut Animation<GridPlacement>)>,
    mut anim_targets: Query<(
        &mut GridPlacement,
        &Position<LogicalContext>,
        &Area<LogicalContext>,
    )>,
    time: ResMut<Time>,
    mut sequences: Query<&mut Sequence>,
    id_table: Res<IdTable>,
    mut cmd: Commands,
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
                let mut orphaned = false;
                if let Some(target) = id_table.lookup_target(animation.animation_target.clone()) {
                    if !animation.grid_placement_started {
                        start_grid_placement_anim(
                            id_table
                                .lookup_target(animation.animation_target.clone())
                                .unwrap(),
                            &mut anim_targets,
                            animation.end.clone(),
                        );
                        animation.grid_placement_started = true;
                        continue;
                    }
                    if let Ok(a) = anim_targets.get(target) {
                        animation.interpolations =
                            GridPlacement::interpolations(&a.0, &animation.end);
                        animation.started = true;
                    } else {
                        orphaned = true;
                    }
                } else {
                    orphaned = true;
                }
                if orphaned {
                    despawn_and_update_sequence(
                        &mut sequences,
                        &id_table,
                        &mut cmd,
                        anim_entity,
                        &mut animation,
                    );
                    continue;
                }
            }
            let delta = animation.animation_time.time_delta(frame_diff);
            let percent = animation.easement.percent_changed(delta);
            for i in animation.interpolations.scalars.iter_mut() {
                let d = i.start + i.diff * percent;
                i.current_value.replace(d);
            }
            let mut orphaned = false;
            if let Some(target) = id_table.lookup_target(animation.animation_target.clone()) {
                if let Ok((mut a, _, _)) = anim_targets.get_mut(target) {
                    a.apply(&mut animation.interpolations);
                } else {
                    orphaned = true;
                }
            } else {
                orphaned = true;
            }
            if orphaned {
                despawn_and_update_sequence(
                    &mut sequences,
                    &id_table,
                    &mut cmd,
                    anim_entity,
                    &mut animation,
                );
                cmd.entity(anim_entity).despawn();
                continue;
            }
            if percent >= 1f32 {
                despawn_and_update_sequence(
                    &mut sequences,
                    &id_table,
                    &mut cmd,
                    anim_entity,
                    &mut animation,
                );
                cmd.entity(anim_entity).despawn();
            }
        }
    }
}
pub(crate) fn animate<A: Animate>(
    mut anims: Query<(Entity, &mut Animation<A>)>,
    mut anim_targets: Query<&mut A>,
    time: ResMut<Time>,
    mut sequences: Query<&mut Sequence>,
    id_table: Res<IdTable>,
    mut cmd: Commands,
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
                let mut orphaned = false;
                if let Some(target) = id_table.lookup_target(animation.animation_target.clone()) {
                    if let Ok(a) = anim_targets.get(target) {
                        animation.interpolations = A::interpolations(&a, &animation.end);
                        animation.started = true;
                    } else {
                        orphaned = true;
                    }
                } else {
                    orphaned = true;
                }
                if orphaned {
                    despawn_and_update_sequence(
                        &mut sequences,
                        &id_table,
                        &mut cmd,
                        anim_entity,
                        &mut animation,
                    );
                    continue;
                }
            }
            let delta = animation.animation_time.time_delta(frame_diff);
            let percent = animation.easement.percent_changed(delta);
            for i in animation.interpolations.scalars.iter_mut() {
                let d = i.start + i.diff * percent;
                i.current_value.replace(d);
            }
            let mut orphaned = false;
            if let Some(target) = id_table.lookup_target(animation.animation_target.clone()) {
                if let Ok(mut a) = anim_targets.get_mut(target) {
                    a.apply(&mut animation.interpolations);
                } else {
                    orphaned = true;
                }
            } else {
                orphaned = true;
            }
            if orphaned {
                despawn_and_update_sequence(
                    &mut sequences,
                    &id_table,
                    &mut cmd,
                    anim_entity,
                    &mut animation,
                );
                cmd.entity(anim_entity).despawn();
                continue;
            }
            if percent >= 1f32 {
                despawn_and_update_sequence(
                    &mut sequences,
                    &id_table,
                    &mut cmd,
                    anim_entity,
                    &mut animation,
                );
                cmd.entity(anim_entity).despawn();
            }
        }
    }
}

fn despawn_and_update_sequence<A: Animate>(
    sequences: &mut Query<&mut Sequence>,
    id_table: &Res<IdTable>,
    cmd: &mut Commands,
    anim_entity: Entity,
    animation: &mut Mut<Animation<A>>,
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
        cmd.entity(sequence_entity).despawn();
    }
    cmd.entity(anim_entity).despawn();
}
