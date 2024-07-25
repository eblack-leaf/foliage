use std::ops::Mul;

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::anim::{Animate, Interpolations};

#[repr(C)]
#[derive(
    bytemuck::Pod,
    bytemuck::Zeroable,
    Copy,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Debug,
    Component,
)]
pub struct Color {
    rgba: [f32; 4],
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl Color {
    pub const WHITE: Color = Color::rgb_unchecked(0.90, 0.90, 0.90);
    pub const BLACK: Color = Color::rgb_unchecked(0.10, 0.10, 0.10);
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            rgba: [
                r.clamp(0.0, 1.0),
                g.clamp(0.0, 1.0),
                b.clamp(0.0, 1.0),
                a.clamp(0.0, 1.0),
            ],
        }
    }
    pub const fn rgba_unchecked(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { rgba: [r, g, b, a] }
    }
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self {
            rgba: [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), 1.0],
        }
    }
    pub const fn rgb_unchecked(r: f32, g: f32, b: f32) -> Self {
        Self {
            rgba: [r, g, b, 1.0],
        }
    }
    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.rgba[3] = alpha;
        self
    }
    pub fn set_red(&mut self, r: f32) {
        self.rgba[0] = r.clamp(0.0, 1.0);
    }
    pub fn red(&self) -> f32 {
        self.rgba[0]
    }
    pub fn set_green(&mut self, g: f32) {
        self.rgba[1] = g.clamp(0.0, 1.0);
    }
    pub fn green(&self) -> f32 {
        self.rgba[1]
    }
    pub fn set_blue(&mut self, b: f32) {
        self.rgba[2] = b.clamp(0.0, 1.0);
    }
    pub fn blue(&self) -> f32 {
        self.rgba[2]
    }
    pub fn set_alpha(&mut self, a: f32) {
        self.rgba[3] = a.clamp(0.0, 1.0);
    }
    pub fn alpha(&self) -> f32 {
        self.rgba[3]
    }
}

impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self {
        Self {
            r: color.red() as f64,
            g: color.green() as f64,
            b: color.blue() as f64,
            a: color.alpha() as f64,
        }
    }
}

impl Animate for Color {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        Interpolations::new()
            .with(start.red(), end.red())
            .with(start.green(), end.green())
            .with(start.blue(), end.blue())
            .with(start.alpha(), end.alpha())
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(r) = interpolations.read(0) {
            self.set_red(r);
        }
        if let Some(g) = interpolations.read(1) {
            self.set_green(g);
        }
        if let Some(b) = interpolations.read(2) {
            self.set_blue(b);
        }
        if let Some(a) = interpolations.read(3) {
            self.set_alpha(a);
        }
    }
}
impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::rgb(self.red() * rhs, self.green() * rhs, self.blue() * rhs).with_alpha(self.alpha())
    }
}

pub trait Monochromatic {
    fn minus_one() -> Color {
        Self::BASE * 0.65
    }
    fn minus_two() -> Color {
        Self::BASE * 0.45
    }
    fn minus_three() -> Color {
        Self::BASE * 0.25
    }
    fn base() -> Color {
        Self::BASE
    }
    const BASE: Color;
    fn plus_one() -> Color {
        Self::BASE * 1.25
    }
    fn plus_two() -> Color {
        Self::BASE * 1.45
    }
    fn plus_three() -> Color {
        Self::BASE * 1.65
    }
}

pub struct Grey;
impl Monochromatic for Grey {
    const BASE: Color = Color::rgb_unchecked(0.5, 0.5, 0.5);
}
