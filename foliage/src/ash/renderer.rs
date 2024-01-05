use crate::ash::identification::RenderIdentification;
use anymap::AnyMap;
use bevy_ecs::entity::Entity;
use std::cmp::Ordering;

use crate::ash::instruction::{
    RenderInstructionGroup, RenderInstructionHandle, RenderInstructionsRecorder,
    RenderRecordBehavior,
};
use crate::ash::render::Render;
use crate::ash::render_packet::RenderPacketQueue;
use crate::coordinate::layer::Layer;
use crate::ginkgo::Ginkgo;

pub(crate) struct Renderer<T: Render + 'static> {
    resources: T::Resources,
    packages: RenderPackageStorage<T>,
    pub(crate) instructions: RenderInstructionGroup,
    per_renderer_record_hook: bool,
    record_behavior: RenderRecordBehavior<T>,
    updated: bool,
}

impl<T: Render + 'static> Renderer<T> {
    pub(crate) fn new(ginkgo: &Ginkgo) -> Self {
        Self {
            resources: T::create_resources(ginkgo),
            packages: RenderPackageStorage::default(),
            instructions: RenderInstructionGroup::default(),
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
            &mut self.updated,
        );
    }
    fn inner_prepare_packages(
        resources: &mut T::Resources,
        packages: &mut RenderPackageStorage<T>,
        ginkgo: &Ginkgo,
        mut queue: RenderPacketQueue,
        updated_hook: &mut bool,
    ) {
        let id = <T as RenderIdentification>::render_id();
        tracing::trace!("preparing-renderer:{:?}", id);
        for entity in queue.retrieve_removals() {
            if let Some(index) = packages.index(entity) {
                let old = packages.0.remove(index);
                T::on_package_removal(ginkgo, resources, old.0, old.2);
                *updated_hook = true;
            }
        }
        let mut should_sort = false;
        for (entity, layer, package) in packages.0.iter_mut() {
            if let Some(render_packet) = queue.retrieve_packet(*entity) {
                if let Some(l) = render_packet.get::<Layer>() {
                    *layer = l;
                    should_sort = true;
                    *updated_hook = true;
                }
                tracing::trace!("preparing-package-for:{:?}", *entity);
                T::prepare_package(ginkgo, resources, *entity, package, render_packet);
                *updated_hook = true;
            }
        }
        if !queue.0.is_empty() {
            for (entity, render_packet) in queue.0.drain() {
                packages.0.push((
                    entity,
                    render_packet.get::<Layer>().unwrap(),
                    RenderPackage::new(T::create_package(ginkgo, resources, entity, render_packet)),
                ));
                should_sort = true;
            }
            *updated_hook = true;
        }
        if should_sort {
            packages.order_by_layer();
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
                    if let Some(instructions) = behavior(resources, recorder) {
                        render_instruction_group.0 = vec![instructions];
                    }
                }
            }
            RenderRecordBehavior::PerPackage(behavior) => {
                render_instruction_group.0.clear();
                for (_entity, _layer, package) in packages.0.iter_mut() {
                    if package.should_record {
                        let recorder = RenderInstructionsRecorder::new(ginkgo);
                        if let Some(instructions) = behavior(resources, package, recorder) {
                            package.instruction_handle.replace(instructions.clone());
                            package.should_record = false;
                        }
                    }
                    if let Some(instructions) = package.instruction_handle.as_ref() {
                        render_instruction_group.0.push(instructions.clone());
                    }
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
    instruction_handle: Option<RenderInstructionHandle>,
    pub package_data: T::RenderPackage,
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