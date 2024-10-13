use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateContext, CoordinateUnit, LogicalContext};
use bevy_ecs::prelude::Component;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Default, Component, Copy)]
pub struct Points<Context: CoordinateContext> {
    pub data: [Position<Context>; 4],
}
impl<Context: CoordinateContext> Points<Context> {
    pub fn bbox(&self) -> Section<LogicalContext> {
        let bbox = Section::default();

        bbox
    }
}
impl<Context: CoordinateContext> Add for Points<Context> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut new = Points::default();
        for i in 0..4 {
            new.data[i] = self.data[i] + rhs.data[i];
        }
        new
    }
}
impl<Context: CoordinateContext> Sub for Points<Context> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut new = Points::default();
        for i in 0..4 {
            new.data[i] = self.data[i] - rhs.data[i];
        }
        new
    }
}
impl<Context: CoordinateContext> Mul<CoordinateUnit> for Points<Context> {
    type Output = Self;
    fn mul(self, rhs: CoordinateUnit) -> Self::Output {
        let mut new = Points::default();
        for i in 0..4 {
            new.data[i] = new.data[i] * rhs;
        }
        new
    }
}
