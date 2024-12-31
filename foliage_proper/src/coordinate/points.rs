use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateContext, CoordinateUnit, Logical};
use bevy_ecs::prelude::Component;
use std::fmt::Display;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Default, Component, Copy, PartialEq)]
pub struct Points<Context: CoordinateContext> {
    pub data: [Position<Context>; 4],
}
impl<Context: CoordinateContext> Points<Context> {
    pub fn new() -> Self {
        Self {
            data: [Position::<Context>::default(); 4],
        }
    }
}
impl<Context: CoordinateContext> Display for Points<Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}:{}:{}:{}",
            self.data[0], self.data[1], self.data[2], self.data[3]
        ))
    }
}
impl<Context: CoordinateContext> Points<Context> {
    pub fn bbox(&self) -> Section<Logical> {
        let bbox = Section::default();

        bbox
    }
}
impl<Context: CoordinateContext> Add for Points<Context> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut new = Points::new();
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
            new.data[i] = self.data[i] * rhs;
        }
        new
    }
}
