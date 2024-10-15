use crate::anim::{Animate, Interpolations};
use crate::coordinate::points::Points;
use crate::coordinate::section::Section;
use crate::coordinate::LogicalContext;
use crate::grid::responsive::configure::ConfigureFromLayout;
use crate::grid::responsive::evaluate::EvaluateLocation;
use crate::grid::responsive::{
    PointExceptions, ResponsiveConfigurationException, ResponsiveLocation, ResponsivePointBundle,
    ResponsivePoints, ResponsiveSection,
};
use crate::tree::Tree;
use bevy_ecs::component::Component;
use bevy_ecs::event::Event;
use bevy_ecs::observer::Trigger;
use bevy_ecs::prelude::Query;

#[derive(Component, Clone, Default)]
pub(crate) struct ResponsiveLocationAnimPackage {
    pub(crate) base: ResponsiveSection,
    pub(crate) exceptions: ResponsiveConfigurationException,
}

#[derive(Component, Default, Copy, Clone)]
pub struct ResponsiveAnimationHook {
    pub section: Section<LogicalContext>,
    pub percent: f32,
}

impl ResponsiveAnimationHook {
    pub(crate) fn value(&self) -> Section<LogicalContext> {
        self.section * self.percent
    }
}

#[derive(Event)]
pub(crate) struct ResponsiveAnimationCalc {}

pub(crate) fn anim_calc(
    trigger: Trigger<ResponsiveAnimationCalc>,
    actual: Query<&Section<LogicalContext>>,
    pts: Query<&Points<LogicalContext>>,
    mut tree: Tree,
) {
    let last = actual.get(trigger.entity()).copied().unwrap();
    let last_pts = pts.get(trigger.entity()).copied().unwrap();
    tree.entity(trigger.entity()).insert(ConfigureFromLayout {});
    tree.entity(trigger.entity())
        .insert(EvaluateLocation::no_deps());
    tree.trigger_targets(CalcDiff { last, last_pts }, trigger.entity());
}

#[derive(Event, Copy, Clone)]
pub(crate) struct CalcDiff {
    last: Section<LogicalContext>,
    last_pts: Points<LogicalContext>,
}

pub(crate) fn calc_diff(
    trigger: Trigger<CalcDiff>,
    mut diffs: Query<&mut ResponsiveAnimationHook>,
    mut pt_diffs: Query<&mut ResponsivePointsAnimationHook>,
    calculated: Query<&Section<LogicalContext>>,
    calc_pts: Query<&Points<LogicalContext>>,
) {
    if let Ok(mut diff) = diffs.get_mut(trigger.entity()) {
        let last = trigger.event().last;
        let new = calculated.get(trigger.entity()).copied().unwrap();
        let delta = last - new;
        tracing::trace!(
            "last: {} - new: {} = d: {} @ {:?}",
            last,
            new,
            delta,
            trigger.entity()
        );
        diff.section = delta;
    }
    if let Ok(mut diff) = pt_diffs.get_mut(trigger.entity()) {
        let last = trigger.event().last_pts;
        let new = calc_pts.get(trigger.entity()).copied().unwrap();
        let delta = last - new;
        diff.points = delta;
    }
}

impl Animate for ResponsiveAnimationHook {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        todo!()
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(s) = interpolations.read(0) {
            self.percent = s;
        }
    }
}

#[derive(Component)]
pub struct ResponsivePointsAnimPackage {
    pub(crate) base_points: ResponsivePoints,
    pub(crate) exceptions: PointExceptions,
}

#[derive(Component, Copy, Clone, Default)]
pub(crate) struct ResponsivePointsAnimationHook {
    pub(crate) points: Points<LogicalContext>,
    pub(crate) percent: f32,
}

impl ResponsivePointsAnimationHook {
    pub(crate) fn value(&self) -> Points<LogicalContext> {
        self.points * self.percent
    }
}

impl Animate for ResponsivePointsAnimationHook {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        todo!()
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(s) = interpolations.read(0) {
            self.percent = s;
        }
    }
}

impl Animate for ResponsivePointBundle {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        todo!()
    }
    fn apply(&mut self, interpolations: &mut Interpolations) {
        todo!()
    }
}

impl Animate for ResponsiveLocation {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        todo!()
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        todo!()
    }
}
