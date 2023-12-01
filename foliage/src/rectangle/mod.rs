use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::InterfaceContext;
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::{Elm, Leaf};
use crate::texture::Progress;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::Bundle;
use bytemuck::{Pod, Zeroable};

mod proc_gen;
mod renderer;
mod vertex;
#[derive(Bundle)]
pub struct Rectangle {
    position: Position<InterfaceContext>,
    area: Area<InterfaceContext>,
    c_pos: DifferentialBundle<CReprPosition>,
    c_area: DifferentialBundle<CReprArea>,
    progress: DifferentialBundle<Progress>,
    color: DifferentialBundle<Color>,
    differentiable: Differentiable,
}
impl Rectangle {
    pub fn new(
        position: Position<InterfaceContext>,
        area: Area<InterfaceContext>,
        layer: Layer,
        color: Color,
        progress: Progress,
    ) -> Self {
        Self {
            position,
            area,
            c_pos: DifferentialBundle::new(CReprPosition::default()),
            c_area: DifferentialBundle::new(CReprArea::default()),
            progress: DifferentialBundle::new(progress),
            color: DifferentialBundle::new(color),
            differentiable: Differentiable::new::<Self>(layer),
        }
    }
}
impl Leaf for Rectangle {
    fn attach(elm: &mut Elm) {
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
    for y in 0..Rectangle::TEXTURE_DIMENSIONS {
        for x in 0..Rectangle::TEXTURE_DIMENSIONS {
            filled.push(255u8);
        }
    }
    let size = Rectangle::TEXTURE_DIMENSIONS;
    const PRECISION: u32 = 1000;
    let mut filled_data = vec![0f32; (size * size) as usize];
    for unit in 0..PRECISION {
        for y in 0..size {
            for x in 0..size {
                if x > unit {
                    let index = x + size as u32 * y;
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
    let data_string = serde_json::to_string(&data).unwrap();
    std::fs::write(root.join("rectangle.prog"), data_string).unwrap();
    let filled = serde_json::to_string(&filled).unwrap();
    std::fs::write(root.join("rectangle-texture.cov"), filled).unwrap();
}