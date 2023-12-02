use crate::ash::instruction::RenderRecordBehavior;
use crate::ash::render::{Render, RenderPhase};
use crate::ash::render_package::RenderPackage;
use crate::ash::render_packet::RenderPacket;
use crate::ginkgo::uniform::Uniform;
use crate::ginkgo::Ginkgo;
use crate::text::Text;
use bevy_ecs::entity::Entity;

pub(crate) struct TextRenderResources {}
pub(crate) struct TextRenderPackage {}
impl Render for Text {
    type Resources = TextRenderResources;
    type RenderPackage = TextRenderPackage;
    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(3);

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo
            .device()
            .create_shader_module(wgpu::include_wgsl!("text.wgsl"));
        let package_layout =
            ginkgo
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("text-package-layout"),
                    entries: &[Ginkgo::vertex_uniform_bind_group_layout_entry(0)],
                });
        let resource_layout =
            ginkgo
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("text-resource-layout"),
                    entries: &[
                        Ginkgo::vertex_uniform_bind_group_layout_entry(0),
                        Ginkgo::sampler_bind_group_layout_entry(1),
                    ],
                });
    }

    fn create_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        render_packet: RenderPacket,
    ) -> Self::RenderPackage {
        todo!()
    }

    fn on_package_removal(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        package: RenderPackage<Self>,
    ) {
        todo!()
    }

    fn prepare_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        package: &mut RenderPackage<Self>,
        render_packet: RenderPacket,
    ) {
        todo!()
    }

    fn prepare_resources(
        resources: &mut Self::Resources,
        ginkgo: &Ginkgo,
        per_renderer_record_hook: &mut bool,
    ) {
        todo!()
    }

    fn record_behavior() -> RenderRecordBehavior<Self> {
        todo!()
    }
}
