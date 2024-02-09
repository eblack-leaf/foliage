use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(
    Component,
    bytemuck::Pod,
    bytemuck::Zeroable,
    Copy,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Debug,
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
    pub const WHITE: Color = Color::rgb(0.95, 0.95, 0.95);
    pub const BLACK: Color = Color::rgb(0.005, 0.005, 0.005);
    pub const GREY: Color = Color::rgb(0.15, 0.15, 0.15);
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { rgba: [r, g, b, a] }
    }
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
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
pub trait Monochromatic {
    const PLUS_THREE: Color;
    const PLUS_TWO: Color;
    const PLUS_ONE: Color;
    const BASE: Color;
    const MINUS_ONE: Color;
    const MINUS_TWO: Color;
    const MINUS_THREE: Color;
}
pub struct Orange {}
impl Monochromatic for Orange {
    const PLUS_THREE: Color = Color::rgb(1.0, 0.51, 0.302);
    const PLUS_TWO: Color = Color::rgb(1.0, 0.439, 0.20);
    const PLUS_ONE: Color = Color::rgb(1.0, 0.369, 0.102);
    const BASE: Color = Color::rgb(1.0, 0.298, 0.0);
    const MINUS_ONE: Color = Color::rgb(0.902, 0.267, 0.0);
    const MINUS_TWO: Color = Color::rgb(0.80, 0.239, 0.0);
    const MINUS_THREE: Color = Color::rgb(0.706, 0.208, 0.0);
}
