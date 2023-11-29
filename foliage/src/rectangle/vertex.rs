use crate::coordinate::position::CReprPosition;
use crate::texture::TextureCoordinates;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub(crate) struct Vertex {
    position: CReprPosition,
    texture_coordinates: TextureCoordinates,
}

impl Vertex {
    const fn new(position: CReprPosition, texture_coordinates: TextureCoordinates) -> Self {
        Self {
            position,
            texture_coordinates,
        }
    }
}

pub const VERTICES: [Vertex; 6] = [
    Vertex::new(
        CReprPosition::new(1f32, 0f32),
        TextureCoordinates::new(1f32, 0f32),
    ),
    Vertex::new(
        CReprPosition::new(0f32, 0f32),
        TextureCoordinates::new(0f32, 0f32),
    ),
    Vertex::new(
        CReprPosition::new(0f32, 1f32),
        TextureCoordinates::new(0f32, 1f32),
    ),
    Vertex::new(
        CReprPosition::new(1f32, 0f32),
        TextureCoordinates::new(1f32, 0f32),
    ),
    Vertex::new(
        CReprPosition::new(0f32, 1f32),
        TextureCoordinates::new(0f32, 1f32),
    ),
    Vertex::new(
        CReprPosition::new(1f32, 1f32),
        TextureCoordinates::new(1f32, 1f32),
    ),
];
