use crate::ash::instruction::{
    RenderInstructionHandle, RenderInstructionsRecorder, RenderRecordBehavior,
};
use crate::ash::render::{Render, RenderPhase};
use crate::ash::render_package::RenderPackage;
use crate::ash::render_packet::RenderPacket;
use crate::color::Color;
use crate::coordinate::area::CReprArea;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::CReprPosition;
use crate::ginkgo::Ginkgo;
use crate::icon::bundled_cov::FILENAMES;
use crate::icon::vertex::{Vertex, VERTICES};
use crate::icon::{Icon, IconId};
use crate::instance::{InstanceCoordinator, InstanceCoordinatorBuilder};
use bevy_ecs::entity::Entity;
use std::collections::HashMap;

pub struct IconRenderResources {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    #[allow(unused)]
    icon_bind_group_layout: wgpu::BindGroupLayout,
    icon_textures: HashMap<IconId, (InstanceCoordinator<Entity>, wgpu::BindGroup)>,
    entity_to_icon: HashMap<Entity, IconId>,
}
impl Icon {
    pub(crate) const TEXTURE_DIMENSIONS: u32 = 20;
}
impl Render for Icon {
    type Resources = IconRenderResources;
    type RenderPackage = ();

    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(0);

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo
            .device()
            .create_shader_module(wgpu::include_wgsl!("icon.wgsl"));
        let icon_bind_group_layout =
            ginkgo
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("icon-render-pipeline-bind-group-layout"),
                    entries: &[Ginkgo::texture_d2_bind_group_entry(0)],
                });
        let mut icon_textures = HashMap::new();
        // choose icon size by scale factor
        // which set of filenames to iter and use and the dims
        for (index, file) in FILENAMES.iter().enumerate() {
            let texture_data = serde_json::from_str::<Vec<u8>>(*file).ok().unwrap();
            let (_texture, view) = ginkgo.texture_r8unorm_d2(
                Icon::TEXTURE_DIMENSIONS,
                Icon::TEXTURE_DIMENSIONS,
                1,
                texture_data.as_slice(),
            );
            icon_textures.insert(
                IconId(index as u32),
                (
                    InstanceCoordinatorBuilder::new(4)
                        .with_attribute::<CReprPosition>()
                        .with_attribute::<CReprArea>()
                        .with_attribute::<Layer>()
                        .with_attribute::<Color>()
                        .build(ginkgo),
                    ginkgo
                        .device()
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("icon-bind-group"),
                            layout: &icon_bind_group_layout,
                            entries: &[Ginkgo::texture_bind_group_entry(&view, 0)],
                        }),
                ),
            );
        }
        let sampler = ginkgo
            .device()
            .create_sampler(&wgpu::SamplerDescriptor::default());
        let bind_group_layout =
            ginkgo
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Icon-render-pipeline-bind-group-layout"),
                    entries: &[
                        Ginkgo::vertex_uniform_bind_group_layout_entry(0),
                        Ginkgo::sampler_bind_group_layout_entry(1),
                    ],
                });
        let bind_group = ginkgo
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("icon-render-pipeline-bind-group"),
                layout: &bind_group_layout,
                entries: &[
                    ginkgo.viewport_bind_group_entry(0),
                    Ginkgo::sampler_bind_group_entry(&sampler, 1),
                ],
            });
        let pipeline_layout =
            ginkgo
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("icon-render-pipeline-layout"),
                    bind_group_layouts: &[&bind_group_layout, &icon_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let pipeline = ginkgo
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("icon-render-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
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
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<Color>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![5 => Float32x4],
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
        let vertex_buffer = ginkgo.vertex_buffer_with_data(&VERTICES, "icon-vertex-buffer");
        IconRenderResources {
            pipeline,
            vertex_buffer,
            bind_group,
            icon_bind_group_layout,
            icon_textures,
            entity_to_icon: HashMap::new(),
        }
    }

    fn create_package(
        _ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        render_packet: RenderPacket,
    ) -> Self::RenderPackage {
        if let Some(icon_id) = resources.entity_to_icon.get(&entity) {
            resources
                .icon_textures
                .get_mut(icon_id)
                .unwrap()
                .0
                .queue_remove(entity);
        }
        let new = render_packet.get::<IconId>().unwrap();
        resources.entity_to_icon.insert(entity, new);
        resources
            .icon_textures
            .get_mut(&new)
            .unwrap()
            .0
            .queue_add(entity);
        resources
            .icon_textures
            .get_mut(&new)
            .unwrap()
            .0
            .queue_render_packet(entity, render_packet);
    }

    fn on_package_removal(
        _ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        _package: RenderPackage<Self>,
    ) {
        if let Some(icon_id) = resources.entity_to_icon.get(&entity) {
            resources
                .icon_textures
                .get_mut(icon_id)
                .unwrap()
                .0
                .queue_remove(entity);
        }
    }

    fn prepare_package(
        _ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        _package: &mut RenderPackage<Self>,
        render_packet: RenderPacket,
    ) {
        let icon_id = resources.entity_to_icon.get(&entity).unwrap();
        resources
            .icon_textures
            .get_mut(icon_id)
            .unwrap()
            .0
            .queue_render_packet(entity, render_packet);
    }

    fn prepare_resources(
        resources: &mut Self::Resources,
        ginkgo: &Ginkgo,
        per_renderer_record_hook: &mut bool,
    ) {
        let mut should_record = false;
        for (_id, (coordinator, _)) in resources.icon_textures.iter_mut() {
            if coordinator.prepare(ginkgo) {
                should_record = true;
            }
        }
        if should_record {
            *per_renderer_record_hook = true;
        }
    }

    fn record_behavior() -> RenderRecordBehavior<Self> {
        RenderRecordBehavior::PerRenderer(Box::new(Icon::record))
    }
}

impl Icon {
    fn record<'a>(
        resources: &'a IconRenderResources,
        mut recorder: RenderInstructionsRecorder<'a>,
    ) -> Option<RenderInstructionHandle> {
        let mut something_recorded = false;
        for (_, (instance_coordinator, bind_group)) in resources.icon_textures.iter() {
            if instance_coordinator.has_instances() {
                recorder.0.set_pipeline(&resources.pipeline);
                recorder.0.set_bind_group(0, &resources.bind_group, &[]);
                recorder.0.set_bind_group(1, bind_group, &[]);
                recorder
                    .0
                    .set_vertex_buffer(0, resources.vertex_buffer.slice(..));
                recorder
                    .0
                    .set_vertex_buffer(1, instance_coordinator.buffer::<CReprPosition>().slice(..));
                recorder
                    .0
                    .set_vertex_buffer(2, instance_coordinator.buffer::<CReprArea>().slice(..));
                recorder
                    .0
                    .set_vertex_buffer(3, instance_coordinator.buffer::<Layer>().slice(..));
                recorder
                    .0
                    .set_vertex_buffer(4, instance_coordinator.buffer::<Color>().slice(..));
                recorder.0.draw(
                    0..VERTICES.len() as u32,
                    0..instance_coordinator.instances(),
                );
                something_recorded = true;
            }
        }
        return if something_recorded {
            Some(recorder.finish())
        } else {
            None
        };
    }
}
