use crate::elm::compact_string_type_id;
use crate::ginkgo::Ginkgo;
use crate::r_ash::render_packet::RenderPacket;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;
use crate::ginkgo::viewport::Viewport;

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
pub(crate) type PerRendererFn<T: Render> = Box<fn(&T::Resources, &Viewport) -> RenderInstructionHandle>;
pub(crate) type PerPackagePrepareFn<T: Render> = Box<fn()>;
pub(crate) type PerPackageRecordFn<T: Render> = Box<
    fn(
        &T::Resources,
        &Viewport,
        &mut RenderPackage<T::RenderPackage>,
        recorder: RenderInstructionsRecorder,
    ) -> RenderInstructionHandle,
>;
pub enum RenderRecordBehavior<T: Render> {
    PerRenderer(PerRendererFn<T>),
    PerRenderPackage(PerPackageRecordFn<T>),
}
pub struct Renderer<T: Render> {
    resources: T::Resources,
    packages: Vec<RenderPackage<T::RenderPackage>>,
    entity_to_package: HashMap<Entity, usize>,
    packet_queue: Vec<(Entity, RenderPacket)>,
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
pub(crate) struct RenderInstructionHandle(pub(crate) Rc<wgpu::RenderBundle>);
pub struct RenderInstructionGroup {
    instructions: Vec<RenderInstructionHandle>,
}
pub enum RenderPhase {
    Opaque,
    Alpha(i32),
}
pub trait Render {
    type Resources;
    type RenderPackage;
}
