use crate::animate::{Interpolate, Interpolation, InterpolationExtraction};
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::Elm;
use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

pub mod monochromatic;

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
    pub const WHITE: Color = Color::rgb(0.90, 0.90, 0.90);
    pub const BLACK: Color = Color::rgb(0.10, 0.10, 0.10);
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

impl Interpolate for Color {
    fn interpolations(&self, end: &Self) -> Vec<Interpolation> {
        vec![
            Interpolation::new(self.red(), end.red()),
            Interpolation::new(self.green(), end.green()),
            Interpolation::new(self.blue(), end.blue()),
            Interpolation::new(self.alpha(), end.alpha()),
        ]
    }

    fn apply(&self, extracts: Vec<InterpolationExtraction>) -> Self {
        Self::rgb(
            extracts
                .get(0)
                .cloned()
                .unwrap_or(InterpolationExtraction(self.red()))
                .0,
            extracts
                .get(0)
                .cloned()
                .unwrap_or(InterpolationExtraction(self.green()))
                .0,
            extracts
                .get(0)
                .cloned()
                .unwrap_or(InterpolationExtraction(self.blue()))
                .0,
        )
        .with_alpha(
            extracts
                .get(0)
                .cloned()
                .unwrap_or(InterpolationExtraction(self.alpha()))
                .0,
        )
    }
}
impl Leaf for Color {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        elm.enable_animation::<Color>();
    }
}