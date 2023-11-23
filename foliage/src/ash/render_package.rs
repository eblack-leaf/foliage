use crate::ash::instruction::RenderInstructionHandle;
use crate::ash::render::Render;
use bevy_ecs::entity::Entity;

pub(crate) struct RenderPackageStorage<T: Render>(pub(crate) Vec<(Entity, RenderPackage<T>)>);

impl<T: Render> RenderPackageStorage<T> {
    pub(crate) fn new() -> Self {
        Self(vec![])
    }
    pub(crate) fn index(&self, entity: Entity) -> Option<usize> {
        let mut index = None;
        let mut current = 0;
        for (package_entity, _package) in self.0.iter() {
            if &entity == package_entity {
                index.replace(current);
            }
            current += 1;
        }
        index
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
