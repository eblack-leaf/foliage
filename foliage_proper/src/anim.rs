use bevy_ecs::change_detection::Mut;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Commands, Entity};
use bevy_ecs::system::{Query, ResMut};
use std::any::TypeId;

use crate::color::Color;
use crate::coordinate::elevation::Elevation;
use crate::coordinate::Coordinates;
use crate::elm::Elm;
use crate::grid::responsive::anim::{
    ResponsiveAnimationCalc, ResponsiveAnimationHook, ResponsiveLocationAnimPackage,
    ResponsivePointsAnimPackage, ResponsivePointsAnimationHook,
};
use crate::grid::responsive::evaluate::EvaluateLocation;
use crate::leaf::EvaluateElevation;
use crate::opacity::{EvaluateOpacity, Opacity};
use crate::panel::Rounding;
use crate::time::{OnEnd, Time, TimeDelta};
use crate::tree::Tree;
use crate::Root;

pub(crate) struct EnabledAnimations;
impl Root for EnabledAnimations {
    fn attach(elm: &mut Elm) {
        elm.enable_animation::<Color>();
        elm.enable_animation::<Opacity>();
        elm.enable_animation::<Rounding>();
        elm.enable_animation::<ResponsiveAnimationHook>();
        elm.enable_animation::<ResponsivePointsAnimationHook>();
    }
}
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
    pub fn end(mut self, e: u64) -> Self {
        self.sequence_time_range.end = TimeDelta::from_millis(e);
        self
    }
    pub fn eased(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
}
#[derive(Component)]
pub(crate) struct AnimationRunner<A: Animate> {
    started: bool,
    end: Option<A>,
    interpolations: Interpolations,
    easement: Easement,
    sequence_entity: Entity,
    animation_time: AnimationTime,
    animation_target: Entity,
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
impl<A: Animate> AnimationRunner<A> {
    pub(crate) fn new<EASE: Into<Easement>>(
        target: Entity,
        end: A,
        ease: EASE,
        se: Entity,
        animation_time: AnimationTime,
    ) -> Self {
        Self {
            started: false,
            end: Some(end),
            interpolations: Interpolations::default(),
            easement: ease.into(),
            sequence_entity: se,
            animation_time,
            animation_target: target,
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
        self.scalars.get_mut(i)?.current_value()
    }
    pub fn read_percent(&mut self, i: usize) -> Option<f32> {
        self.scalars.get_mut(i)?.percent()
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
    pub fn percent(&self) -> Option<f32> {
        self.current_value.and_then(|v| Option::from(v / self.diff))
    }
}
#[derive(Copy, Clone, Default)]
pub struct SequenceTimeRange {
    pub start: TimeDelta,
    pub end: TimeDelta,
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
#[derive(Clone)]
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
    Self: Sized + Send + Sync + 'static + Clone,
{
    fn interpolations(start: &Self, end: &Self) -> Interpolations;
    fn apply(&mut self, interpolations: &mut Interpolations);
}
#[derive(Component, Default, Copy, Clone)]
pub struct Sequence {
    pub(crate) animations_to_finish: i32,
    pub(crate) on_end: Option<OnEnd>,
}
pub(crate) fn animate<A: Animate + Component>(
    mut anims: Query<(Entity, &mut AnimationRunner<A>)>,
    package: Query<&ResponsiveLocationAnimPackage>,
    pt_package: Query<&ResponsivePointsAnimPackage>,
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
                let mut orphaned = false;
                let target_entity = animation.animation_target;
                let is_section_diff = TypeId::of::<A>() == TypeId::of::<ResponsiveAnimationHook>();
                let is_point_diff =
                    TypeId::of::<A>() == TypeId::of::<ResponsivePointsAnimationHook>();
                if is_section_diff || is_point_diff {
                    if is_section_diff {
                        if let Ok(pkg) = package.get(anim_entity) {
                            tree.entity(animation.animation_target)
                                .insert(pkg.base.clone())
                                .insert(pkg.exceptions.clone());
                        }
                    }
                    if is_point_diff {
                        if let Ok(pkg) = pt_package.get(anim_entity) {
                            tree.entity(animation.animation_target)
                                .insert(pkg.base_points.clone())
                                .insert(pkg.exceptions.clone());
                        }
                    }
                    tree.trigger_targets(ResponsiveAnimationCalc {}, animation.animation_target);
                    animation.interpolations = Interpolations::new().with(1.0, 0.0);
                    animation.started = true;
                } else {
                    if let Ok(a) = anim_targets.get(target_entity) {
                        animation.interpolations =
                            A::interpolations(&a, animation.end.as_ref().unwrap());
                        animation.started = true;
                    } else {
                        orphaned = true;
                    }
                }
                if orphaned {
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
                    i.end
                } else {
                    i.start + i.diff * percent
                };
                i.current_value.replace(d);
            }
            let mut orphaned = false;
            if let Ok(mut a) = anim_targets.get_mut(animation.animation_target) {
                a.apply(&mut animation.interpolations);
                if TypeId::of::<A>() == TypeId::of::<ResponsiveAnimationHook>()
                    || TypeId::of::<A>() == TypeId::of::<ResponsivePointsAnimationHook>()
                {
                    tree.entity(animation.animation_target)
                        .insert(EvaluateLocation::recursive());
                }
                if TypeId::of::<A>() == TypeId::of::<Opacity>()
                    || TypeId::of::<A>() == TypeId::of::<Color>()
                {
                    tree.entity(animation.animation_target)
                        .insert(EvaluateOpacity {});
                }
                if TypeId::of::<A>() == TypeId::of::<Elevation>() {
                    tree.entity(animation.animation_target)
                        .insert(EvaluateElevation {});
                }
            } else {
                orphaned = true;
            }
            if orphaned {
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
    cmd: &mut Commands,
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
        cmd.trigger_targets(OnEnd {}, sequence_entity);
        cmd.entity(sequence_entity).despawn();
    }
    cmd.entity(anim_entity).despawn();
}
