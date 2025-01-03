use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateContext, CoordinateUnit, Logical};
use bevy_ecs::prelude::Component;
use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, Mul, Sub};

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
    pub fn a<P: Into<Position<Context>>>(mut self, a: P) -> Self {
        self.data[0] = a.into();
        self
    }
    pub fn set_a<P: Into<Position<Context>>>(&mut self, a: P) {
        self.data[0] = a.into();
    }
    pub fn b<P: Into<Position<Context>>>(mut self, b: P) -> Self {
        self.data[1] = b.into();
        self
    }
    pub fn set_b<P: Into<Position<Context>>>(&mut self, b: P) {
        self.data[1] = b.into();
    }
    pub fn c<P: Into<Position<Context>>>(mut self, c: P) -> Self {
        self.data[2] = c.into();
        self
    }
    pub fn set_c<P: Into<Position<Context>>>(&mut self, c: P) {
        self.data[2] = c.into();
    }
    pub fn d<P: Into<Position<Context>>>(mut self, d: P) -> Self {
        self.data[3] = d.into();
        self
    }
    pub fn set_d<P: Into<Position<Context>>>(&mut self, d: P) {
        self.data[3] = d.into();
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
        Section::default()
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
impl<Context: CoordinateContext> AddAssign<Points<Context>> for Points<Context> {
    fn add_assign(&mut self, rhs: Points<Context>) {
        *self = *self + rhs;
    }
}
impl<Context: CoordinateContext> Div<CoordinateUnit> for Points<Context> {
    type Output = Self;

    fn div(self, rhs: CoordinateUnit) -> Self::Output {
        todo!()
    }
}
