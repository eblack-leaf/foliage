use crate::coordinate::position::CReprPosition;
use crate::coordinate::CoordinateUnit;
use crate::texture::TextureCoordinates;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub struct Vertex {
    position: CReprPosition,
    texture_coordinates: TextureCoordinates,
    listen_hook: [CoordinateUnit; 2],
}

impl Vertex {
    const fn new(
        position: CReprPosition,
        texture_coordinates: TextureCoordinates,
        hook: [CoordinateUnit; 2],
    ) -> Self {
        Self {
            position,
            texture_coordinates,
            listen_hook: hook,
        }
    }
}

pub const CORNER_DEPTH: CoordinateUnit = 5f32;
const CORNER_TEXTURE_EXTENT: CoordinateUnit = 0.49f32;
const CORNER_SPACING: CoordinateUnit = 0.02f32;
pub const VERTICES: [Vertex; 16] = [
    // left-top
    Vertex::new(
        CReprPosition::new(0f32, 0f32),
        TextureCoordinates::new(0f32, 0f32),
        [0.0, 0.0],
    ),
    Vertex::new(
        CReprPosition::new(0f32, CORNER_DEPTH),
        TextureCoordinates::new(0f32, CORNER_TEXTURE_EXTENT),
        [0.0, 0.0],
    ),
    Vertex::new(
        CReprPosition::new(CORNER_DEPTH, CORNER_DEPTH),
        TextureCoordinates::new(CORNER_TEXTURE_EXTENT, CORNER_TEXTURE_EXTENT),
        [0.0, 0.0],
    ),
    Vertex::new(
        CReprPosition::new(CORNER_DEPTH, 0f32),
        TextureCoordinates::new(CORNER_TEXTURE_EXTENT, 0f32),
        [0.0, 0.0],
    ),
    // left-bottom
    Vertex::new(
        CReprPosition::new(0f32, CORNER_DEPTH),
        TextureCoordinates::new(0f32, CORNER_TEXTURE_EXTENT + CORNER_SPACING),
        [0.0, 1.0],
    ),
    Vertex::new(
        CReprPosition::new(0f32, CORNER_DEPTH * 2f32),
        TextureCoordinates::new(0f32, 1f32),
        [0.0, 1.0],
    ),
    Vertex::new(
        CReprPosition::new(CORNER_DEPTH, CORNER_DEPTH * 2f32),
        TextureCoordinates::new(CORNER_TEXTURE_EXTENT + CORNER_SPACING, 1f32),
        [0.0, 1.0],
    ),
    Vertex::new(
        CReprPosition::new(CORNER_DEPTH, CORNER_DEPTH),
        TextureCoordinates::new(
            CORNER_TEXTURE_EXTENT + CORNER_SPACING,
            CORNER_TEXTURE_EXTENT + CORNER_SPACING,
        ),
        [0.0, 1.0],
    ),
    // right-bottom
    Vertex::new(
        CReprPosition::new(CORNER_DEPTH, CORNER_DEPTH),
        TextureCoordinates::new(
            CORNER_TEXTURE_EXTENT + CORNER_SPACING,
            CORNER_TEXTURE_EXTENT + CORNER_SPACING,
        ),
        [1.0, 1.0],
    ),
    Vertex::new(
        CReprPosition::new(CORNER_DEPTH, CORNER_DEPTH * 2f32),
        TextureCoordinates::new(CORNER_TEXTURE_EXTENT + CORNER_SPACING, 1f32),
        [1.0, 1.0],
    ),
    Vertex::new(
        CReprPosition::new(CORNER_DEPTH * 2f32, CORNER_DEPTH * 2f32),
        TextureCoordinates::new(1f32, 1f32),
        [1.0, 1.0],
    ),
    Vertex::new(
        CReprPosition::new(CORNER_DEPTH * 2f32, CORNER_DEPTH),
        TextureCoordinates::new(1f32, CORNER_TEXTURE_EXTENT + CORNER_SPACING),
        [1.0, 1.0],
    ),
    // right-top
    Vertex::new(
        CReprPosition::new(CORNER_DEPTH, 0f32),
        TextureCoordinates::new(CORNER_TEXTURE_EXTENT + CORNER_SPACING, 0f32),
        [1.0, 0.0],
    ),
    Vertex::new(
        CReprPosition::new(CORNER_DEPTH, CORNER_DEPTH),
        TextureCoordinates::new(
            CORNER_TEXTURE_EXTENT + CORNER_SPACING,
            CORNER_TEXTURE_EXTENT,
        ),
        [1.0, 0.0],
    ),
    Vertex::new(
        CReprPosition::new(CORNER_DEPTH * 2f32, CORNER_DEPTH),
        TextureCoordinates::new(1f32, CORNER_TEXTURE_EXTENT),
        [1.0, 0.0],
    ),
    Vertex::new(
        CReprPosition::new(CORNER_DEPTH * 2f32, 0f32),
        TextureCoordinates::new(1f32, 0f32),
        [1.0, 0.0],
    ),
];
pub const INDICES: [u16; 54] = [
    3, 0, 1, 3, 1, 2, 2, 1, 4, 2, 4, 7, 7, 4, 5, 7, 5, 6, 8, 7, 6, 8, 6, 9, 11, 8, 9, 11, 9, 10,
    14, 13, 8, 14, 8, 11, 15, 12, 13, 15, 13, 14, 12, 3, 2, 12, 2, 13, 13, 2, 7, 13, 7, 8,
];