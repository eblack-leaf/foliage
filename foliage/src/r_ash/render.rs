use crate::elm::compact_string_type_id;
use crate::ginkgo::viewport::Viewport;
use crate::ginkgo::Ginkgo;
use crate::r_ash::render_packet::RenderPacket;
use crate::r_ash::InstructionGroups;
use anymap::AnyMap;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;
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

pub(crate) struct RenderResourceHandler(pub(crate) AnyMap);
impl RenderResourceHandler {
    pub(crate) fn obtain<T: Render>(&mut self) -> &mut T::Resources {
        &mut self.0.get_mut::<RenderResources<T>>().unwrap().0
    }
    pub(crate) fn establish<T: Render>(&mut self, resources: T::Resources) {
        self.0.insert(RenderResources::new(resources));
    }
}
pub(crate) struct RenderPackageListHandler(pub(crate) AnyMap);
impl RenderPackageListHandler {
    pub(crate) fn obtain<T: Render>(&mut self) -> &mut RenderPackageList<T> {
        self.0.get_mut::<RenderPackageList<T>>().unwrap()
    }
    pub(crate) fn establish<T: Render>(&mut self) {
        self.0.insert(RenderPackageList::default());
    }
}
pub(crate) struct EntityToRenderPackageHandler(pub(crate) AnyMap);
impl EntityToRenderPackageHandler {
    pub(crate) fn obtain<T: Render>(&mut self) -> &mut EntityToRenderPackage<T> {
        self.0.get_mut::<EntityToRenderPackage<T>>().unwrap()
    }
    pub(crate) fn establish<T: Render>(&mut self) {
        self.0.insert(EntityToRenderPackage::new());
    }
}
pub(crate) struct RenderPacketQueueHandler(pub(crate) HashMap<RenderId, RenderPacketQueue>);
impl RenderPacketQueueHandler {
    pub(crate) fn obtain<T: Render>(&mut self) -> &mut RenderPacketQueue {
        self.0.get_mut(&T::id()).unwrap()
    }
    pub(crate) fn establish<T: Render>(&mut self, render_id: RenderId) {
        self.0.insert(render_id, RenderPacketQueue::new());
    }
}
pub(crate) struct RecordFns(pub(crate) AnyMap);
impl RecordFns {
    pub(crate) fn obtain<T: Render>(&self) -> &RenderRecordBehavior<T> {
        self.0.get::<RenderRecordBehavior<T>>().unwrap()
    }
    pub(crate) fn set<T: Render>(&mut self, behavior: RenderRecordBehavior<T>) {
        self.0.insert(behavior);
    }
}
pub(crate) fn package_preparation<T: Render>(
    resource_handler: &mut RenderResourceHandler,
    package_handler: &mut RenderPackageListHandler,
    e_to_r_handler: &mut EntityToRenderPackageHandler,
    render_packet_queue_handler: &mut RenderPacketQueueHandler,
    ginkgo: &Ginkgo,
) {
    let res = resource_handler.obtain::<T>();
    let rpl = package_handler.obtain::<T>();
    let e_to_r = e_to_r_handler.obtain::<T>();
    let rpq = render_packet_queue_handler.obtain::<T>();
    for (entity, packet) in rpq.0.drain() {
        // if no index, create package and push in list, then reorder by z and get new indices
        let index = e_to_r.0.get(&entity).unwrap().clone();
        let package = rpl.0.get_mut(index).unwrap();
        T::prepare_package(ginkgo, res, package, packet);
    }
}
pub(crate) fn renderer_preparation<T: Render>(
    resource_handler: &mut RenderResourceHandler,
    ginkgo: &Ginkgo,
) {
    // T::prepare_resources
}
pub(crate) fn package_record<T: Render>(
    resource_handler: &mut RenderResourceHandler,
    package_handler: &mut RenderPackageListHandler,
    instruction_groups: &mut InstructionGroups,
    record_fns: &RecordFns, // use this to skip remake with T::record_behavior
    ginkgo: &Ginkgo,
) {
    // record in order of sorted list without entity indexing, just need render_id
    if let RenderRecordBehavior::PerPackage(r_fn) = T::record_behavior() {
        // iter packages and store recordings in correct group
        // if dirty
    } else {
        panic!("per_package call on per_renderer behavior")
    }
}
pub(crate) fn renderer_record<T: Render>(
    resource_handler: &mut RenderResourceHandler,
    record_fns: &RecordFns,
    ginkgo: &Ginkgo,
) {
    // if Behavior::PerRenderer(...) -> r_fn(...) with correct input
    // r_fn(res_handler, ginkgo) -> which calls inner to get specifics
}
pub struct RenderResources<T: Render>(pub T::Resources);
impl<T: Render> RenderResources<T> {
    pub(crate) fn new(resources: T::Resources) -> Self {
        Self(resources)
    }
}
#[derive(Default)]
pub struct RenderPackageList<T: Render>(pub Vec<RenderPackage<T::RenderPackage>>);
pub struct EntityToRenderPackage<T: Render>(pub HashMap<Entity, usize>, PhantomData<T>);
impl<T: Render> EntityToRenderPackage<T> {
    pub(crate) fn new() -> Self {
        Self(HashMap::new(), PhantomData)
    }
}
pub struct RenderPacketQueue(pub HashMap<Entity, RenderPacket>);
impl RenderPacketQueue {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }
    pub(crate) fn insert(&mut self, entity: Entity, render_packet: RenderPacket) {
        self.0.insert(entity, render_packet);
    }
}
pub(crate) type PerRendererRecordFn<T: Render> =
    Box<fn(&T::Resources, &Viewport, RenderInstructionsRecorder) -> RenderInstructionHandle>;
pub(crate) type PerPackageRecordFn<T: Render> = Box<
    fn(
        &T::Resources,
        &Viewport,
        &mut RenderPackage<T::RenderPackage>,
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
#[derive(Clone)]
pub(crate) struct RenderInstructionHandle(pub(crate) Rc<wgpu::RenderBundle>);
pub(crate) struct RenderInstructionGroup {
    instructions: Vec<RenderInstructionHandle>,
}
pub enum RenderPhase {
    Opaque,
    Alpha(i32),
}
pub trait Render {
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
        package: &mut RenderPackage<Self::RenderPackage>,
        render_packet: RenderPacket,
    );
    fn prepare_resources(resources: &mut Self::Resources, ginkgo: &Ginkgo);
    fn record_behavior() -> RenderRecordBehavior<Self>;
}
