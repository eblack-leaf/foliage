use crate::anim::Animate;
use crate::coordinate::Coordinates;
use bevy_ecs::prelude::{Component, DetectChanges};
use std::cmp::PartialOrd;
use std::ops::{Add, Mul, Sub};
use unit::TokenUnit;
pub(crate) mod animation;
pub mod aspect;
pub mod responsive_points;
pub mod responsive_section;
pub mod token;
pub mod unit;

#[derive(Clone, Copy, Component, Debug)]
pub struct Grid {
    columns: u32,
    rows: u32,
    gap: Coordinates,
}
impl Grid {
    pub fn new(columns: u32, rows: u32) -> Grid {
        Self {
            columns,
            rows,
            gap: Coordinates::new(8.0, 8.0),
        }
    }
    pub fn columns(&self) -> f32 {
        self.columns as f32
    }
    pub fn rows(&self) -> f32 {
        self.rows as f32
    }
    pub fn gap<C: Into<Coordinates>>(mut self, g: C) -> Self {
        self.gap = g.into();
        self
    }
}
impl Default for Grid {
    fn default() -> Self {
        Self::new(1, 1)
    }
}
