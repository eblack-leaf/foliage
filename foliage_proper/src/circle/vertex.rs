use bytemuck::{Pod, Zeroable};

use crate::coordinate::position::CReprPosition;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub(crate) struct Vertex {
    position: CReprPosition,
    texture_index: [u32; 2],
}

impl Vertex {
    const fn new(position: CReprPosition, texture_index: [u32; 2]) -> Self {
        Self {
            position,
            texture_index,
        }
    }
}

pub(crate) const VERTICES: [Vertex; 6] = [
    Vertex::new(CReprPosition::new(1f32, 0f32), [2, 1]),
    Vertex::new(CReprPosition::new(0f32, 0f32), [0, 1]),
    Vertex::new(CReprPosition::new(0f32, 1f32), [0, 3]),
    Vertex::new(CReprPosition::new(1f32, 0f32), [2, 1]),
    Vertex::new(CReprPosition::new(0f32, 1f32), [0, 3]),
    Vertex::new(CReprPosition::new(1f32, 1f32), [2, 3]),
];
