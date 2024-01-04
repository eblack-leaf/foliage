use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, DeviceContext, InterfaceContext};
use crate::elm::config::{CoreSet, ElmConfiguration};
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::Elm;
use crate::ginkgo::uniform::Uniform;
use bevy_ecs::prelude::{IntoSystemConfigs, Resource};
use bevy_ecs::system::ResMut;
use nalgebra::{matrix, SMatrix};
use serde::{Deserialize, Serialize};
use wgpu::Queue;

#[derive(Serialize, Deserialize, Copy, Clone, Resource)]
pub struct ViewportHandle {
    pub(crate) section: Section<InterfaceContext>,
    changes_present: bool,
    pub(crate) area_updated: bool,
}

impl ViewportHandle {
    pub fn new(section: Section<InterfaceContext>) -> Self {
        Self {
            section,
            changes_present: false,
            area_updated: true,
        }
    }
    pub fn section(&self) -> Section<InterfaceContext> {
        self.section
    }
    pub fn area_updated(&self) -> bool {
        self.area_updated
    }
    pub(crate) fn changes(&mut self) -> Option<Position<InterfaceContext>> {
        if self.changes_present {
            self.changes_present = false;
            return Some(self.section.position);
        }
        None
    }
    pub(crate) fn adjust_area(&mut self, area: Area<InterfaceContext>) {
        self.area_updated = true;
        self.section.area = area;
    }
    pub fn adjust_position(&mut self, x: CoordinateUnit, y: CoordinateUnit) {
        self.section.position += Position::new(x, y);
        self.changes_present = true;
    }
}
impl Leaf for ViewportHandle {
    type SetDescriptor = EmptySetDescriptor;

    fn config(elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.main()
            .add_systems(clear_viewport_handle.after(CoreSet::Differential));
    }
}
fn clear_viewport_handle(mut viewport_handle: ResMut<ViewportHandle>) {
    if viewport_handle.area_updated() {
        viewport_handle.area_updated = false;
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
        tracing::trace!("matrix-section: {:?}", section);
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
