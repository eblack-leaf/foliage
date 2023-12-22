use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, DeviceContext, InterfaceContext};
use crate::ginkgo::uniform::Uniform;
use bevy_ecs::prelude::Resource;
use nalgebra::{matrix, SMatrix};
use serde::{Deserialize, Serialize};
use wgpu::Queue;

#[derive(Serialize, Deserialize, Copy, Clone, Resource)]
pub struct ViewportHandle {
    pub(crate) section: Section<InterfaceContext>,
    dirty: bool,
}

impl ViewportHandle {
    pub fn new(section: Section<InterfaceContext>) -> Self {
        Self {
            section,
            dirty: false,
        }
    }
    pub fn section(&self) -> Section<InterfaceContext> {
        self.section
    }
    pub fn dirty(&self) -> bool { self.dirty }
    pub(crate) fn changes(&mut self) -> Option<Position<InterfaceContext>> {
        if self.dirty {
            self.dirty = false;
            return Some(self.section.position);
        }
        None
    }
    pub fn adjust_position(&mut self, x: CoordinateUnit, y: CoordinateUnit) {
        self.section.position += Position::new(x, y);
        self.dirty = true;
    }
}

pub struct Viewport {
    pub(crate) section: Section<DeviceContext>,
    pub(crate) near_far: (Layer, Layer),
    pub(crate) repr: nalgebra::Matrix4<CoordinateUnit>,
    pub(crate) gpu_repr: Uniform<GpuRepr>,
}

impl Viewport {
    pub fn far_layer(&self) -> Layer {
        self.near_far.1
    }
    pub(crate) fn new(
        device: &wgpu::Device,
        section: Section<DeviceContext>,
        near_far: (Layer, Layer),
    ) -> Self {
        let repr = Self::matrix(section, near_far);
        let gpu_repr = Uniform::new(device, repr.data.0);
        Self {
            section,
            near_far,
            repr,
            gpu_repr,
        }
    }

    fn matrix(section: Section<DeviceContext>, near_far: (Layer, Layer)) -> SMatrix<f32, 4, 4> {
        let translation = nalgebra::Matrix::new_translation(&nalgebra::vector![
            section.left(),
            section.top(),
            0f32
        ]);
        let projection = matrix![2f32/(section.right() - section.left()), 0.0, 0.0, -1.0;
                                    0.0, 2f32/(section.top() - section.bottom()), 0.0, 1.0;
                                    0.0, 0.0, 1.0/(near_far.1 - near_far.0).z, 0.0;
                                    0.0, 0.0, 0.0, 1.0];
        projection * translation
    }
    pub fn section(&self) -> Section<DeviceContext> {
        self.section
    }
    pub(crate) fn adjust_pos(&mut self, queue: &Queue, position: Position<DeviceContext>) {
        self.adjust(queue, self.section.with_position(position));
    }
    pub(crate) fn adjust(&mut self, queue: &wgpu::Queue, section: Section<DeviceContext>) {
        self.repr = Self::matrix(section, self.near_far);
        self.gpu_repr.update(queue, self.repr.data.0);
    }
}
pub(crate) type GpuRepr = [[CoordinateUnit; 4]; 4];