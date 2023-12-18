use crate::ash::instruction::{
    RenderInstructionHandle, RenderInstructionsRecorder, RenderRecordBehavior,
};
use crate::ash::render::{Render, RenderPhase};
use crate::ash::render_packet::RenderPacket;
use crate::ash::renderer::RenderPackage;
use crate::coordinate::area::CReprArea;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::CReprPosition;
use crate::ginkgo::Ginkgo;
use crate::image::vertex::{Vertex, VERTICES};
use crate::image::{Image, ImageData, ImageId};
use crate::instance::{InstanceCoordinator, InstanceCoordinatorBuilder};
use bevy_ecs::entity::Entity;
use std::collections::HashMap;
use wgpu::{BindGroup, BindGroupDescriptor, VertexState};

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
    vertex_buffer: wgpu::Buffer,
}
pub struct ImageRenderPackage {
    last: ImageId,
    was_request: bool,
}
impl Render for Image {
    type Resources = ImageRenderResources;
    type RenderPackage = ImageRenderPackage;
    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(6);

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let resource_layout =
            ginkgo
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("image-layout"),
                    entries: &[
                        Ginkgo::vertex_uniform_bind_group_layout_entry(0),
                        Ginkgo::sampler_bind_group_layout_entry(1),
                    ],
                });
        let package_layout =
            ginkgo
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("image-package-layout"),
                    entries: &[Ginkgo::texture_d2_bind_group_entry(0)],
                });
        let shader = ginkgo
            .device()
            .create_shader_module(wgpu::include_wgsl!("image.wgsl"));
        let pipeline_layout =
            ginkgo
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("image-pipeline-layout"),
                    bind_group_layouts: &[&resource_layout, &package_layout],
                    push_constant_ranges: &[],
                });
        let sampler = ginkgo
            .device()
            .create_sampler(&wgpu::SamplerDescriptor::default());
        let bind_group = ginkgo.device().create_bind_group(&BindGroupDescriptor {
            label: Some("image-bind-group"),
            layout: &resource_layout,
            entries: &[
                ginkgo.viewport_bind_group_entry(0),
                Ginkgo::sampler_bind_group_entry(&sampler, 1),
            ],
        });
        let pipeline = ginkgo
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("image-render-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vertex_entry",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<Vertex>(1),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<CReprPosition>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![2 => Float32x2],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<CReprArea>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![3 => Float32x2],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<Layer>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![4 => Float32],
                        },
                    ],
                },
                primitive: Ginkgo::triangle_list_primitive(),
                depth_stencil: ginkgo.depth_stencil_state(),
                multisample: ginkgo.msaa_multisample_state(),
                fragment: ginkgo.fragment_state(
                    &shader,
                    "fragment_entry",
                    &ginkgo.alpha_color_target_state(),
                ),
                multiview: None,
            });
        let vertex_buffer = ginkgo.vertex_buffer_with_data(&VERTICES, "image-vertices");
        ImageRenderResources {
            pipeline,
            bind_group,
            package_layout,
            vertex_buffer,
            groups: HashMap::new(),
        }
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
                package.signal_record();
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
        resources: &mut Self::Resources,
        ginkgo: &Ginkgo,
        _per_renderer_record_hook: &mut bool,
    ) {
        // iter groups and prepare coordinators
        for (_id, group) in resources.groups.iter_mut() {
            if group.coordinator.prepare(ginkgo) {
                *_per_renderer_record_hook = true;
            }
        }
    }

    fn record_behavior() -> RenderRecordBehavior<Self> {
        RenderRecordBehavior::PerRenderer(Box::new(record))
    }
}

fn record<'a>(
    resources: &'a ImageRenderResources,
    mut recorder: RenderInstructionsRecorder<'a>,
) -> Option<RenderInstructionHandle> {
    recorder.0.set_pipeline(&resources.pipeline);
    recorder.0.set_bind_group(0, &resources.bind_group, &[]);
    recorder
        .0
        .set_vertex_buffer(0, resources.vertex_buffer.slice(..));
    let mut recorded = false;
    for (_id, group) in resources.groups.iter() {
        if group.coordinator.has_instances() {
            recorded = true;
            recorder
                .0
                .set_bind_group(1, group.bind_group.as_ref().unwrap(), &[]);
            recorder
                .0
                .set_vertex_buffer(1, group.coordinator.buffer::<CReprPosition>().slice(..));
            recorder
                .0
                .set_vertex_buffer(2, group.coordinator.buffer::<CReprArea>().slice(..));
            recorder
                .0
                .set_vertex_buffer(3, group.coordinator.buffer::<Layer>().slice(..));
            recorder
                .0
                .draw(0..VERTICES.len() as u32, 0..group.coordinator.instances());
        }
    }
    if recorded {
        Some(recorder.finish())
    } else {
        None
    }
}