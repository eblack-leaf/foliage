use crate::ash::instruction::{
    RenderInstructionHandle, RenderInstructionsRecorder, RenderRecordBehavior,
};
use crate::ash::render::{Render, RenderPhase};
use crate::ash::render_package::RenderPackage;
use crate::ash::render_packet::RenderPacket;
use crate::circle::vertex::{Vertex, VERTICES};
use crate::circle::{Circle, CircleStyle};
use crate::color::Color;
use crate::coordinate::area::CReprArea;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::CReprPosition;
use crate::ginkgo::Ginkgo;
use crate::instance::{InstanceCoordinator, InstanceCoordinatorBuilder};
use crate::panel::Panel;
use crate::texture::{MipsLevel, Progress};
use bevy_ecs::entity::Entity;

pub struct CircleRenderResources {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    #[allow(unused)]
    texture: wgpu::Texture,
    #[allow(unused)]
    view: wgpu::TextureView,
    #[allow(unused)]
    sampler: wgpu::Sampler,
    instance_coordinator: InstanceCoordinator<Entity>,
    #[allow(unused)]
    ring_texture: wgpu::Texture,
    #[allow(unused)]
    ring_view: wgpu::TextureView,
    #[allow(unused)]
    ring_progress_texture: wgpu::Texture,
    #[allow(unused)]
    ring_progress_view: wgpu::TextureView,
    #[allow(unused)]
    progress_texture: wgpu::Texture,
    #[allow(unused)]
    progress_view: wgpu::TextureView,
}

impl Render for Circle {
    type Resources = CircleRenderResources;
    type RenderPackage = ();

    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(Panel::RENDER_PHASE.value() + 1);

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo
            .device()
            .create_shader_module(wgpu::include_wgsl!("circle.wgsl"));
        let mut texture_data = serde_json::from_str::<Vec<u8>>(include_str!(
            "texture_resources/circle-texture-1024.cov"
        ))
        .ok()
        .unwrap();
        texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!(
                "texture_resources/circle-texture-512.cov"
            ))
            .unwrap(),
        );
        texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!(
                "texture_resources/circle-texture-256.cov"
            ))
            .unwrap(),
        );
        texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!(
                "texture_resources/circle-texture-128.cov"
            ))
            .unwrap(),
        );
        texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!(
                "texture_resources/circle-texture-64.cov"
            ))
            .unwrap(),
        );
        texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!(
                "texture_resources/circle-texture-32.cov"
            ))
            .unwrap(),
        );
        let (texture, view) = ginkgo.texture_r8unorm_d2(
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::MIPS,
            texture_data.as_slice(),
        );
        let mut ring_texture_data = serde_json::from_str::<Vec<u8>>(include_str!(
            "texture_resources/circle-ring-texture-1024.cov"
        ))
        .ok()
        .unwrap();
        ring_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!(
                "texture_resources/circle-ring-texture-512.cov"
            ))
            .unwrap(),
        );
        ring_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!(
                "texture_resources/circle-ring-texture-256.cov"
            ))
            .unwrap(),
        );
        ring_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!(
                "texture_resources/circle-ring-texture-128.cov"
            ))
            .unwrap(),
        );
        ring_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!(
                "texture_resources/circle-ring-texture-64.cov"
            ))
            .unwrap(),
        );
        ring_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!(
                "texture_resources/circle-ring-texture-32.cov"
            ))
            .unwrap(),
        );
        let (ring_texture, ring_view) = ginkgo.texture_r8unorm_d2(
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::MIPS,
            ring_texture_data.as_slice(),
        );
        let mut progress_texture_data = vec![];
        progress_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!("texture_resources/circle-1024.prog"))
                .unwrap(),
        );
        progress_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!("texture_resources/circle-512.prog"))
                .unwrap(),
        );
        progress_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!("texture_resources/circle-256.prog"))
                .unwrap(),
        );
        progress_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!("texture_resources/circle-128.prog"))
                .unwrap(),
        );
        progress_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!("texture_resources/circle-64.prog"))
                .unwrap(),
        );
        progress_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!("texture_resources/circle-32.prog"))
                .unwrap(),
        );
        let (progress_texture, progress_view) = ginkgo.texture_r8unorm_d2(
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::MIPS,
            progress_texture_data.as_slice(),
        );
        let mut ring_progress_texture_data = vec![];
        ring_progress_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!(
                "texture_resources/circle-ring-1024.prog"
            ))
            .unwrap(),
        );
        ring_progress_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!("texture_resources/circle-ring-512.prog"))
                .unwrap(),
        );
        ring_progress_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!("texture_resources/circle-ring-256.prog"))
                .unwrap(),
        );
        ring_progress_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!("texture_resources/circle-ring-128.prog"))
                .unwrap(),
        );
        ring_progress_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!("texture_resources/circle-ring-64.prog"))
                .unwrap(),
        );
        ring_progress_texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!("texture_resources/circle-ring-32.prog"))
                .unwrap(),
        );
        let (ring_progress_texture, ring_progress_view) = ginkgo.texture_r8unorm_d2(
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::MIPS,
            ring_progress_texture_data.as_slice(),
        );
        let sampler = ginkgo
            .device()
            .create_sampler(&wgpu::SamplerDescriptor::default());
        let bind_group_layout =
            ginkgo
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("circle-render-pipeline-bind-group-layout"),
                    entries: &[
                        Ginkgo::vertex_uniform_bind_group_layout_entry(0),
                        Ginkgo::texture_d2_bind_group_entry(1),
                        Ginkgo::texture_d2_bind_group_entry(2),
                        Ginkgo::sampler_bind_group_layout_entry(3),
                        Ginkgo::texture_d2_bind_group_entry(4),
                        Ginkgo::texture_d2_bind_group_entry(5),
                    ],
                });
        let bind_group = ginkgo
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("circle-render-pipeline-bind-group"),
                layout: &bind_group_layout,
                entries: &[
                    ginkgo.viewport_bind_group_entry(0),
                    Ginkgo::texture_bind_group_entry(&view, 1),
                    Ginkgo::texture_bind_group_entry(&ring_view, 2),
                    Ginkgo::sampler_bind_group_entry(&sampler, 3),
                    Ginkgo::texture_bind_group_entry(&progress_view, 4),
                    Ginkgo::texture_bind_group_entry(&ring_progress_view, 5),
                ],
            });
        let pipeline_layout =
            ginkgo
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("circle-render-pipeline-layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });
        let pipeline = ginkgo
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("circle-render-pipeline"),
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
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<CircleStyle>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![6 => Float32],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<MipsLevel>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![7 => Float32],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<Progress>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![8 => Float32x2],
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
        let vertex_buffer = ginkgo.vertex_buffer_with_data(&VERTICES, "circle-vertex-buffer");
        let instance_coordinator = InstanceCoordinatorBuilder::new(4)
            .with_attribute::<CReprPosition>()
            .with_attribute::<CReprArea>()
            .with_attribute::<Layer>()
            .with_attribute::<Color>()
            .with_attribute::<CircleStyle>()
            .with_attribute::<MipsLevel>()
            .with_attribute::<Progress>()
            .build(ginkgo);
        CircleRenderResources {
            pipeline,
            vertex_buffer,
            bind_group,
            texture,
            view,
            sampler,
            instance_coordinator,
            ring_texture,
            ring_view,
            ring_progress_texture,
            ring_progress_view,
            progress_texture,
            progress_view,
        }
    }

    fn create_package(
        _ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        render_packet: RenderPacket,
    ) -> Self::RenderPackage {
        resources.instance_coordinator.queue_add(entity);
        resources
            .instance_coordinator
            .queue_render_packet(entity, render_packet);
    }

    fn on_package_removal(
        _ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        _package: RenderPackage<Self>,
    ) {
        resources.instance_coordinator.queue_remove(entity);
    }

    fn prepare_package(
        _ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        _package: &mut RenderPackage<Self>,
        render_packet: RenderPacket,
    ) {
        resources
            .instance_coordinator
            .queue_render_packet(entity, render_packet);
    }

    fn prepare_resources(
        resources: &mut Self::Resources,
        ginkgo: &Ginkgo,
        per_renderer_record_hook: &mut bool,
    ) {
        let should_record = resources.instance_coordinator.prepare(ginkgo);
        if should_record {
            *per_renderer_record_hook = true;
        }
    }

    fn record_behavior() -> RenderRecordBehavior<Self> {
        RenderRecordBehavior::PerRenderer(Box::new(Circle::record))
    }
}

impl Circle {
    fn record<'a>(
        resources: &'a CircleRenderResources,
        mut recorder: RenderInstructionsRecorder<'a>,
    ) -> Option<RenderInstructionHandle> {
        if resources.instance_coordinator.has_instances() {
            recorder.0.set_pipeline(&resources.pipeline);
            recorder.0.set_bind_group(0, &resources.bind_group, &[]);
            recorder
                .0
                .set_vertex_buffer(0, resources.vertex_buffer.slice(..));
            recorder.0.set_vertex_buffer(
                1,
                resources
                    .instance_coordinator
                    .buffer::<CReprPosition>()
                    .slice(..),
            );
            recorder.0.set_vertex_buffer(
                2,
                resources
                    .instance_coordinator
                    .buffer::<CReprArea>()
                    .slice(..),
            );
            recorder.0.set_vertex_buffer(
                3,
                resources.instance_coordinator.buffer::<Layer>().slice(..),
            );
            recorder.0.set_vertex_buffer(
                4,
                resources.instance_coordinator.buffer::<Color>().slice(..),
            );
            recorder.0.set_vertex_buffer(
                5,
                resources
                    .instance_coordinator
                    .buffer::<CircleStyle>()
                    .slice(..),
            );
            recorder.0.set_vertex_buffer(
                6,
                resources
                    .instance_coordinator
                    .buffer::<MipsLevel>()
                    .slice(..),
            );
            recorder.0.set_vertex_buffer(
                7,
                resources
                    .instance_coordinator
                    .buffer::<Progress>()
                    .slice(..),
            );
            recorder.0.draw(
                0..VERTICES.len() as u32,
                0..resources.instance_coordinator.instances(),
            );
            return Some(recorder.finish());
        }
        None
    }
}
