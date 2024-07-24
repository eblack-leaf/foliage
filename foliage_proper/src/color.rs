use std::ops::Mul;

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

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
    pub fn red(&self) -> f32 {
        self.rgba[0]
    }
    pub fn green(&self) -> f32 {
        self.rgba[1]
    }
    pub fn blue(&self) -> f32 {
        self.rgba[2]
    }
    pub fn alpha(&self) -> f32 {
        self.rgba[3]
    }
    pub fn alpha_mut(&mut self) -> &mut f32 {
        &mut self.rgba[3]
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

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::rgb(self.red() * rhs, self.green() * rhs, self.blue() * rhs).with_alpha(self.alpha())
    }
}

pub trait Monochromatic {
    fn minus_one() -> Color {
        Self::BASE * 0.75
    }
    fn minus_two() -> Color {
        Self::BASE * 0.5
    }
    fn minus_three() -> Color {
        Self::BASE * 0.25
    }
    const BASE: Color;
    fn plus_one() -> Color {
        Self::BASE * 1.15
    }
    fn plus_two() -> Color {
        Self::BASE * 1.35
    }
    fn plus_three() -> Color {
        Self::BASE * 1.5
    }
}

pub struct Grey;
impl Monochromatic for Grey {
    const BASE: Color = Color::rgb_unchecked(0.5, 0.5, 0.5);
}
