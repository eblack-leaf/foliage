use bevy_ecs::prelude::Bundle;

use crate::color::Color;
use crate::coordinate::area::CReprArea;
use crate::coordinate::position::CReprPosition;
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::config::ElmConfiguration;
use crate::elm::Elm;
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::texture::factors::Progress;

mod proc_gen;
mod renderer;
mod vertex;
#[derive(Bundle, Clone)]
pub struct Rectangle {
    progress: DifferentialBundle<Progress>,
    color: DifferentialBundle<Color>,
    differentiable: Differentiable,
}
impl Rectangle {
    pub fn new<C: Into<Color>>(color: C, progress: Progress) -> Self {
        Self {
            progress: DifferentialBundle::new(progress),
            color: DifferentialBundle::new(color.into()),
            differentiable: Differentiable::new::<Self>(),
        }
    }
}

impl Leaf for Rectangle {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.enable_conditional::<Rectangle>();
        differential_enable!(elm, CReprPosition, CReprArea, Progress, Color);
    }
}

#[test]
fn textures() {
    use std::path::Path;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("rectangle")
        .join("texture_resources");
    let mut filled = vec![];
    for _y in 0..Rectangle::TEXTURE_DIMENSIONS {
        for _x in 0..Rectangle::TEXTURE_DIMENSIONS {
            filled.push(255u8);
        }
    }
    let size = Rectangle::TEXTURE_DIMENSIONS;
    const PRECISION: u32 = 10000;
    let mut filled_data = vec![0f32; (size * size) as usize];
    for unit in 0..PRECISION {
        for y in 0..size {
            for x in 0..size {
                if x > unit {
                    let index = x + size * y;
                    *filled_data.get_mut(index as usize).unwrap() += 1f32;
                }
            }
        }
    }
    let data = filled_data
        .drain(..)
        .map(|p| {
            let normalized = p / size as f32;
            let scaled = normalized * 255f32;
            scaled as u8
        })
        .collect::<Vec<u8>>();
    let data_string = rmp_serde::to_vec(&data).unwrap();
    std::fs::write(root.join("rectangle.prog"), data_string).unwrap();
    let filled = rmp_serde::to_vec(&filled).unwrap();
    std::fs::write(root.join("rectangle-texture.cov"), filled).unwrap();
}