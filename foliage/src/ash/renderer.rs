use crate::ash::instruction::{
    RenderInstructionGroup, RenderInstructionHandle, RenderInstructionsRecorder,
};
use crate::ash::render::Render;
use crate::ash::render_packet::RenderPacketQueue;
use crate::ginkgo::viewport::Viewport;
use crate::ginkgo::Ginkgo;
use anymap::AnyMap;
use bevy_ecs::entity::Entity;
use std::collections::HashMap;
pub(crate) type EntityMapping = HashMap<Entity, usize>;
pub(crate) struct Renderer<T: Render> {
    resources: T::Resources,
    packages: RenderPackageStorage<T>,
    pub(crate) instructions: RenderInstructionGroup,
    entity_to_package: EntityMapping,
    per_renderer_record_hook: bool,
    record_behavior: RenderRecordBehavior<T>,
    updated: bool,
}

impl<T: Render> Renderer<T> {
    pub(crate) fn new(ginkgo: &Ginkgo) -> Self {
        Self {
            resources: T::resources(ginkgo),
            packages: RenderPackageStorage::default(),
            instructions: RenderInstructionGroup::default(),
            entity_to_package: EntityMapping::new(),
            per_renderer_record_hook: true,
            record_behavior: T::record_behavior(),
            updated: true,
        }
    }
    pub(crate) fn prepare_packages(&mut self, ginkgo: &Ginkgo, queue: RenderPacketQueue) {
        Self::inner_prepare_packages(
            &mut self.resources,
            &mut self.packages,
            ginkgo,
            queue,
            &mut self.entity_to_package,
            &mut self.updated,
        );
    }
    fn inner_prepare_packages(
        resources: &mut T::Resources,
        packages: &mut RenderPackageStorage<T>,
        ginkgo: &Ginkgo,
        mut queue: RenderPacketQueue,
        mut entity_mapping: &mut EntityMapping,
        updated_hook: &mut bool,
    ) {
        for entity in queue.retrieve_removals() {
            if let Some(index) = entity_mapping.remove(&entity) {
                packages.0.remove(index);
                *updated_hook = true;
            }
        }
        for (entity, package) in packages.0.iter_mut() {
            if let Some(packet) = queue.retrieve_packet(*entity) {
                T::prepare_package(ginkgo, resources, package, packet);
                *updated_hook = true;
            }
        }
        if !queue.0.is_empty() {
            for (entity, packet) in queue.0.drain() {
                packages.0.push((
                    entity,
                    RenderPackage::new(T::package(ginkgo, &resources, packet)),
                ));
                let index = packages.0.len() - 1;
                entity_mapping.insert(entity, index);
            }
            // order by z after insertion
            // update indices in entity_to_package
            *updated_hook = true;
        }
    }
    pub(crate) fn resource_preparation(&mut self, ginkgo: &Ginkgo) {
        Self::inner_resource_preparation(
            &mut self.resources,
            ginkgo,
            &mut self.per_renderer_record_hook,
        );
    }
    fn inner_resource_preparation(
        resources: &mut <T as Render>::Resources,
        ginkgo: &Ginkgo,
        per_renderer_record_hook: &mut bool,
    ) {
        T::prepare_resources(resources, ginkgo, per_renderer_record_hook);
    }
    pub(crate) fn record(&mut self, ginkgo: &Ginkgo) -> bool {
        if self.updated {
            self.instructions.0.clear();
            Self::inner_record(
                &self.resources,
                &mut self.packages,
                ginkgo,
                &mut self.instructions,
                &self.record_behavior,
                self.per_renderer_record_hook,
            );
            self.per_renderer_record_hook = false;
            self.updated = false;
            return true;
        }
        false
    }
    fn inner_record(
        resources: &T::Resources,
        packages: &mut RenderPackageStorage<T>,
        ginkgo: &Ginkgo,
        render_instruction_group: &mut RenderInstructionGroup,
        render_record_behavior: &RenderRecordBehavior<T>,
        per_renderer_record_hook: bool,
    ) {
        match render_record_behavior {
            RenderRecordBehavior::PerRenderer(behavior) => {
                if per_renderer_record_hook {
                    let recorder = RenderInstructionsRecorder::new(ginkgo);
                    let instructions =
                        behavior(resources, ginkgo.viewport.as_ref().unwrap(), recorder);
                    render_instruction_group.0 = vec![instructions];
                }
            }
            RenderRecordBehavior::PerPackage(behavior) => {
                for (_entity, package) in packages.0.iter_mut() {
                    let instructions = if package.should_record {
                        let recorder = RenderInstructionsRecorder::new(ginkgo);
                        let instructions = behavior(
                            resources,
                            ginkgo.viewport.as_ref().unwrap(),
                            package,
                            recorder,
                        );
                        package.instruction_handle.replace(instructions.clone());
                        package.should_record = false;
                        instructions
                    } else {
                        package.instruction_handle.as_ref().unwrap().clone()
                    };
                    render_instruction_group.0.push(instructions);
                }
            }
        }
    }
}

pub(crate) struct RendererStorage(pub(crate) AnyMap);

impl RendererStorage {
    pub(crate) fn obtain_mut<T: Render + 'static>(&mut self) -> &mut Renderer<T> {
        self.0.get_mut::<Renderer<T>>().unwrap()
    }
    pub(crate) fn obtain<T: Render + 'static>(&self) -> &Renderer<T> {
        self.0.get::<Renderer<T>>().unwrap()
    }
    pub(crate) fn establish<T: Render + 'static>(&mut self, ginkgo: &Ginkgo) {
        self.0.insert(Renderer::<T>::new(ginkgo));
    }
}

pub enum RenderRecordBehavior<T: Render> {
    PerRenderer(PerRendererRecordFn<T>),
    PerPackage(PerPackageRecordFn<T>),
}

pub(crate) struct RenderPackageStorage<T: Render>(pub(crate) Vec<(Entity, RenderPackage<T>)>);

impl<T: Render> RenderPackageStorage<T> {
    pub(crate) fn new() -> Self {
        Self(vec![])
    }
}

impl<T: Render> Default for RenderPackageStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) type PerRendererRecordFn<T> = Box<
    fn(&<T as Render>::Resources, &Viewport, RenderInstructionsRecorder) -> RenderInstructionHandle,
>;
pub(crate) type PerPackageRecordFn<T> = Box<
    fn(
        &<T as Render>::Resources,
        &Viewport,
        &mut RenderPackage<T>,
        RenderInstructionsRecorder,
    ) -> RenderInstructionHandle,
>;

pub struct RenderPackage<T: Render> {
    instruction_handle: Option<RenderInstructionHandle>,
    package_data: T::RenderPackage,
    should_record: bool,
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
