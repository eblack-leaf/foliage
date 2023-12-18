use crate::ash::instruction::RenderRecordBehavior;
use crate::ash::render::{Render, RenderPhase};
use crate::ash::render_packet::RenderPacket;
use crate::ash::renderer::RenderPackage;
use crate::coordinate::area::CReprArea;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::CReprPosition;
use crate::ginkgo::Ginkgo;
use crate::image::{Image, ImageData, ImageId};
use crate::instance::{InstanceCoordinator, InstanceCoordinatorBuilder};
use bevy_ecs::entity::Entity;
use std::collections::HashMap;
use wgpu::{BindGroup, BindGroupDescriptor};

struct ImageGroup {
    coordinator: InstanceCoordinator<Entity>,
    tex: Option<(wgpu::Texture, wgpu::TextureView)>,
    bind_group: Option<BindGroup>,
}
impl ImageGroup {
    fn new(ginkgo: &Ginkgo) -> Self {
        Self {
            coordinator: InstanceCoordinatorBuilder::new(1)
                .with_attribute::<CReprPosition>()
                .with_attribute::<CReprArea>()
                .with_attribute::<Layer>()
                .build(ginkgo),
            tex: None,
            bind_group: None,
        }
    }
    fn fill(
        &mut self,
        ginkgo: &Ginkgo,
        layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
        data: &[u8],
    ) {
        self.tex
            .replace(ginkgo.texture_rgba8unorm_d2(width, height, 1, data));
        self.bind_group
            .replace(ginkgo.device().create_bind_group(&BindGroupDescriptor {
                label: Some("image-group-bind-group"),
                layout,
                entries: &[Ginkgo::texture_bind_group_entry(
                    &self.tex.as_ref().unwrap().1,
                    0,
                )],
            }));
    }
}
pub struct ImageRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: BindGroup,
    package_layout: wgpu::BindGroupLayout,
    groups: HashMap<ImageId, ImageGroup>,
}
pub struct ImageRenderPackage {
    last: ImageId,
    was_request: bool,
}
impl Render for Image {
    type Resources = ImageRenderResources;
    type RenderPackage = ImageRenderPackage;
    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(6);

    fn create_resources(_ginkgo: &Ginkgo) -> Self::Resources {
        todo!()
    }

    fn create_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        render_packet: RenderPacket,
    ) -> Self::RenderPackage {
        let image_id = render_packet.get::<ImageId>().unwrap();
        if resources.groups.get(&image_id).is_none() {
            resources.groups.insert(image_id, ImageGroup::new(ginkgo));
        }
        let image_data = render_packet.get::<ImageData>().unwrap();
        if let Some(data) = image_data.0 {
            resources.groups.get_mut(&image_id).unwrap().fill(
                ginkgo,
                &resources.package_layout,
                image_data.1,
                image_data.2,
                data.as_slice(),
            );
            return ImageRenderPackage {
                last: image_id,
                was_request: true,
            };
        } else {
            resources
                .groups
                .get_mut(&image_id)
                .unwrap()
                .coordinator
                .queue_add(entity);
            resources
                .groups
                .get_mut(&image_id)
                .unwrap()
                .coordinator
                .queue_render_packet(entity, render_packet);
        }
        ImageRenderPackage {
            last: image_id,
            was_request: false,
        }
    }

    fn on_package_removal(
        _ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        package: RenderPackage<Self>,
    ) {
        if !package.package_data.was_request {
            resources
                .groups
                .get_mut(&package.package_data.last)
                .unwrap()
                .coordinator
                .queue_remove(entity);
        }
    }

    fn prepare_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        package: &mut RenderPackage<Self>,
        render_packet: RenderPacket,
    ) {
        if !package.package_data.was_request {
            if let Some(id) = render_packet.get::<ImageId>() {
                resources
                    .groups
                    .get_mut(&package.package_data.last)
                    .unwrap()
                    .coordinator
                    .queue_remove(entity);
                if resources.groups.get(&id).is_none() {
                    resources.groups.insert(id, ImageGroup::new(ginkgo));
                }
                resources
                    .groups
                    .get_mut(&id)
                    .unwrap()
                    .coordinator
                    .queue_add(entity);
                package.package_data.last = id;
            }
            resources
                .groups
                .get_mut(&package.package_data.last)
                .unwrap()
                .coordinator
                .queue_render_packet(entity, render_packet);
        }
    }

    fn prepare_resources(
        _resources: &mut Self::Resources,
        _ginkgo: &Ginkgo,
        _per_renderer_record_hook: &mut bool,
    ) {
        // iter groups and prepare coordinators
        todo!()
    }

    fn record_behavior() -> RenderRecordBehavior<Self> {
        todo!()
    }
}
