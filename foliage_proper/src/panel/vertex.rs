use crate::Coordinates;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub(crate) struct Vertex {
    position: Coordinates,
    segment: f32,
}

impl Vertex {
    pub(crate) const fn new(position: Coordinates, segment: f32) -> Self {
        Self { position, segment }
    }
}

pub(crate) const VERTICES: [Vertex; 6 * 9] = [
    Vertex::new(Coordinates::new(1f32, 0f32), 0f32),
    Vertex::new(Coordinates::new(0f32, 0f32), 0f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 0f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 0f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 0f32),
    Vertex::new(Coordinates::new(1f32, 1f32), 0f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 1f32),
    Vertex::new(Coordinates::new(0f32, 0f32), 1f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 1f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 1f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 1f32),
    Vertex::new(Coordinates::new(1f32, 1f32), 1f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 2f32),
    Vertex::new(Coordinates::new(0f32, 0f32), 2f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 2f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 2f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 2f32),
    Vertex::new(Coordinates::new(1f32, 1f32), 2f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 3f32),
    Vertex::new(Coordinates::new(0f32, 0f32), 3f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 3f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 3f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 3f32),
    Vertex::new(Coordinates::new(1f32, 1f32), 3f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 4f32),
    Vertex::new(Coordinates::new(0f32, 0f32), 4f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 4f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 4f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 4f32),
    Vertex::new(Coordinates::new(1f32, 1f32), 4f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 5f32),
    Vertex::new(Coordinates::new(0f32, 0f32), 5f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 5f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 5f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 5f32),
    Vertex::new(Coordinates::new(1f32, 1f32), 5f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 6f32),
    Vertex::new(Coordinates::new(0f32, 0f32), 6f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 6f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 6f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 6f32),
    Vertex::new(Coordinates::new(1f32, 1f32), 6f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 7f32),
    Vertex::new(Coordinates::new(0f32, 0f32), 7f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 7f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 7f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 7f32),
    Vertex::new(Coordinates::new(1f32, 1f32), 7f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 8f32),
    Vertex::new(Coordinates::new(0f32, 0f32), 8f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 8f32),
    Vertex::new(Coordinates::new(1f32, 0f32), 8f32),
    Vertex::new(Coordinates::new(0f32, 1f32), 8f32),
    Vertex::new(Coordinates::new(1f32, 1f32), 8f32),
];
