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
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::observer::Trigger;
use bevy_ecs::prelude::Query;
use bevy_ecs::query::With;

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

pub(crate) fn anim_calc(trigger: Trigger<ResponsiveAnimationCalc>, mut tree: Tree) {
    tree.entity(trigger.entity()).insert(ConfigureFromLayout {});
    tree.entity(trigger.entity()).insert(CalcDiff {});
}

#[derive(Component, Copy, Clone)]
pub(crate) struct CalcDiff {}

pub(crate) fn calc_diff(diff_requested: Query<Entity, With<CalcDiff>>, mut tree: Tree) {
    for e in diff_requested.iter() {
        tree.entity(e).insert(EvaluateLocation::recursive());
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
