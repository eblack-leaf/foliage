use std::cmp::Ordering;

use bevy_ecs::entity::Entity;

use crate::ash::instruction::RenderInstructionHandle;
use crate::ash::render::Render;
use crate::coordinate::layer::Layer;

pub(crate) struct RenderPackageStorage<T: Render>(
    pub(crate) Vec<(Entity, Layer, RenderPackage<T>)>,
);

impl<T: Render> RenderPackageStorage<T> {
    pub(crate) fn new() -> Self {
        Self(vec![])
    }
    pub(crate) fn index(&self, entity: Entity) -> Option<usize> {
        let mut index = None;
        for (current, (package_entity, _layer, _package)) in self.0.iter().enumerate() {
            if &entity == package_entity {
                index.replace(current);
            }
        }
        index
    }
    pub(crate) fn order_by_layer(&mut self) {
        self.0
            .sort_by(|lhs, rhs| -> Ordering { lhs.1.partial_cmp(&rhs.1).unwrap() });
        self.0.reverse();
    }
}

pub struct RenderPackage<T: Render> {
    pub(crate) instruction_handle: Option<RenderInstructionHandle>,
    pub package_data: T::RenderPackage,
    pub(crate) should_record: bool,
}

impl<T: Render> RenderPackage<T> {
    pub(crate) fn new(data: T::RenderPackage) -> Self {
        Self {
            instruction_handle: None,
            package_data: data,
            should_record: true,
        }
    }
    pub fn signal_record(&mut self) {
        self.should_record = true;
    }
}
