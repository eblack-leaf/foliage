use crate::elm::compact_string_type_id;
use crate::ginkgo::viewport::Viewport;
use crate::ginkgo::Ginkgo;
use crate::r_ash::render_packet::RenderPacket;
use anymap::AnyMap;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Component)]
pub struct RenderId(pub CompactString);
pub trait RenderIdentification {
    fn id() -> RenderId;
}
impl<T: Render + 'static> RenderIdentification for T {
    fn id() -> RenderId {
        RenderId(compact_string_type_id::<T>())
    }
}
pub struct RenderInstructionsRecorder<'a>(pub wgpu::RenderBundleEncoder<'a>);

impl<'a> RenderInstructionsRecorder<'a> {
    pub(crate) fn new(ginkgo: &'a Ginkgo) -> Self {
        Self(
            ginkgo
                .device
                .as_ref()
                .unwrap()
                .create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                    label: Some("render-bundle"),
                    color_formats: &ginkgo.color_attachment_format(),
                    depth_stencil: ginkgo.render_bundle_depth_stencil(),
                    sample_count: ginkgo.msaa_samples(),
                    multiview: None,
                }),
        )
    }
    pub fn finish(self) -> RenderInstructionHandle {
        RenderInstructionHandle(Rc::new(self.0.finish(&wgpu::RenderBundleDescriptor {
            label: Some("render-bundle-desc"),
        })))
    }
}
pub enum RenderRecordBehavior<T: Render> {
    PerRenderer(PerRendererRecordFn<T>),
    PerPackage(PerPackageRecordFn<T>),
}
pub(crate) struct RenderPackageHandler<T: Render>(pub(crate) Vec<(Entity, RenderPackage<T>)>);
impl<T: Render> RenderPackageHandler<T> {
    pub(crate) fn new() -> Self {
        Self(vec![])
    }
}
impl<T: Render> Default for RenderPackageHandler<T> {
    fn default() -> Self {
        Self::new()
    }
}
pub(crate) struct Renderer<T: Render> {
    resources: T::Resources,
    packages: RenderPackageHandler<T>,
    instructions: RenderInstructionGroup,
    entity_to_package: HashMap<Entity, usize>,
    per_renderer_record_hook: bool,
    record_behavior: RenderRecordBehavior<T>,
    updated: bool,
}
pub(crate) struct RenderLeaflet {
    prepare_package: Box<fn(&mut RendererHandler, &Ginkgo, &mut RenderPacketQueueHandler)>,
    prepare_resources: Box<fn(&mut RendererHandler, &Ginkgo)>,
    record: Box<fn(&mut RendererHandler, &Ginkgo)>,
}
impl RenderLeaflet {
    pub(crate) fn prepare_package_wrapper<T: Render + 'static>(
        renderer_handler: &mut RendererHandler,
        ginkgo: &Ginkgo,
        queue_handler: &mut RenderPacketQueueHandler,
    ) {
        if let Some(queue) = queue_handler.obtain::<T>() {
            renderer_handler
                .obtain::<T>()
                .prepare_packages(ginkgo, queue);
        }
    }
    pub(crate) fn prepare_resources_wrapper<T: Render + 'static>(
        renderer_handler: &mut RendererHandler,
        ginkgo: &Ginkgo,
    ) {
        renderer_handler.obtain::<T>().resource_preparation(ginkgo);
    }
    pub(crate) fn record_wrapper<T: Render + 'static>(
        renderer_handler: &mut RendererHandler,
        ginkgo: &Ginkgo,
    ) {
        renderer_handler.obtain::<T>().record(ginkgo);
    }
    pub(crate) fn leaf_fn<T: Render + 'static>() -> Self {
        Self {
            prepare_package: Box::new(Self::prepare_package_wrapper::<T>),
            prepare_resources: Box::new(Self::prepare_resources_wrapper::<T>),
            record: Box::new(Self::record_wrapper::<T>),
        }
    }
}

impl<T: Render> Renderer<T> {
    pub(crate) fn new(ginkgo: &Ginkgo) -> Self {
        Self {
            resources: T::resources(ginkgo),
            packages: RenderPackageHandler::default(),
            instructions: RenderInstructionGroup::default(),
            entity_to_package: HashMap::new(),
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
        packages: &mut RenderPackageHandler<T>,
        ginkgo: &Ginkgo,
        mut queue: RenderPacketQueue,
        updated_hook: &mut bool,
    ) {
        for (entity, package) in packages.0.iter_mut() {
            if let Some(packet) = queue.retrieve(*entity) {
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
    pub(crate) fn record(&mut self, ginkgo: &Ginkgo) {
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
        }
    }
    fn inner_record(
        resources: &T::Resources,
        packages: &mut RenderPackageHandler<T>,
        ginkgo: &Ginkgo,
        render_instruction_group: &mut RenderInstructionGroup,
        render_record_behavior: &RenderRecordBehavior<T>,
        per_renderer_record_hook: bool,
    ) {
        match render_record_behavior {
            RenderRecordBehavior::PerRenderer(behavior) => {
                let recorder = RenderInstructionsRecorder::new(ginkgo);
                if per_renderer_record_hook {
                    let instructions =
                        behavior(resources, ginkgo.viewport.as_ref().unwrap(), recorder);
                    render_instruction_group.0 = vec![instructions];
                }
            }
            RenderRecordBehavior::PerPackage(behavior) => {
                for (entity, package) in packages.0.iter_mut() {
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
pub(crate) struct RendererHandler(pub(crate) AnyMap);
impl RendererHandler {
    pub(crate) fn obtain<T: Render + 'static>(&mut self) -> &mut Renderer<T> {
        self.0.get_mut::<Renderer<T>>().unwrap()
    }
    pub(crate) fn establish<T: Render + 'static>(&mut self, ginkgo: &Ginkgo) {
        self.0.insert(Renderer::<T>::new(ginkgo));
    }
}
pub(crate) struct RenderPacketQueueHandler(pub(crate) HashMap<RenderId, RenderPacketQueue>);
impl RenderPacketQueueHandler {
    pub(crate) fn obtain<T: Render + 'static>(&mut self) -> Option<RenderPacketQueue> {
        self.establish::<T>()
    }
    pub(crate) fn establish<T: Render + 'static>(&mut self) -> Option<RenderPacketQueue> {
        self.0.insert(T::id(), RenderPacketQueue::new())
    }
}
pub(crate) struct RenderPacketQueue(pub HashMap<Entity, RenderPacket>);
impl RenderPacketQueue {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }
    pub(crate) fn insert(&mut self, entity: Entity, render_packet: RenderPacket) {
        self.0.insert(entity, render_packet);
    }
    pub(crate) fn retrieve(&mut self, entity: Entity) -> Option<RenderPacket> {
        self.0.remove(&entity)
    }
}
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
#[derive(Clone)]
pub struct RenderInstructionHandle(pub(crate) Rc<wgpu::RenderBundle>);
#[derive(Default)]
pub(crate) struct RenderInstructionGroup(pub(crate) Vec<RenderInstructionHandle>);
pub enum RenderPhase {
    Opaque,
    Alpha(i32),
}
pub trait Render
where
    Self: Sized,
{
    type Resources;
    type RenderPackage;
    fn resources(ginkgo: &Ginkgo) -> Self::Resources;
    fn package(
        ginkgo: &Ginkgo,
        resources: &Self::Resources,
        render_packet: RenderPacket,
    ) -> Self::RenderPackage;
    fn prepare_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        package: &mut RenderPackage<Self>,
        render_packet: RenderPacket,
    );
    fn prepare_resources(
        resources: &mut Self::Resources,
        ginkgo: &Ginkgo,
        per_renderer_record_hook: &mut bool,
    );
    fn record_behavior() -> RenderRecordBehavior<Self>;
}
