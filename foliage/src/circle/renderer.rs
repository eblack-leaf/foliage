use crate::ash::instruction::{
    RenderInstructionHandle, RenderInstructionsRecorder, RenderRecordBehavior,
};
use crate::ash::render::{Render, RenderPhase};
use crate::ash::render_packet::RenderPacket;
use crate::ash::renderer::RenderPackage;
use crate::circle::vertex::{Vertex, VERTICES};
use crate::circle::{Circle, CircleStyle};
use crate::color::Color;
use crate::coordinate::area::CReprArea;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::CReprPosition;
use crate::ginkgo::Ginkgo;
use crate::instance::{InstanceCoordinator, InstanceCoordinatorBuilder};
use crate::texture::factors::{MipsLevel, Progress};
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
impl Circle {
    const CIRCLE_TEXTURES: [&'static [u8]; Circle::MIPS as usize] = [
        include_bytes!("texture_resources/circle-texture-1536.cov"),
        include_bytes!("texture_resources/circle-texture-768.cov"),
        include_bytes!("texture_resources/circle-texture-384.cov"),
        include_bytes!("texture_resources/circle-texture-192.cov"),
        include_bytes!("texture_resources/circle-texture-96.cov"),
        include_bytes!("texture_resources/circle-texture-48.cov"),
        include_bytes!("texture_resources/circle-texture-24.cov"),
        include_bytes!("texture_resources/circle-texture-12.cov"),
    ];
    const CIRCLE_PROG_TEXTURES: [&'static [u8]; Circle::MIPS as usize] = [
        include_bytes!("texture_resources/circle-1536.prog"),
        include_bytes!("texture_resources/circle-768.prog"),
        include_bytes!("texture_resources/circle-384.prog"),
        include_bytes!("texture_resources/circle-192.prog"),
        include_bytes!("texture_resources/circle-96.prog"),
        include_bytes!("texture_resources/circle-48.prog"),
        include_bytes!("texture_resources/circle-24.prog"),
        include_bytes!("texture_resources/circle-12.prog"),
    ];
    const CIRCLE_RING_TEXTURES: [&'static [u8]; Circle::MIPS as usize] = [
        include_bytes!("texture_resources/circle-ring-texture-1536.cov"),
        include_bytes!("texture_resources/circle-ring-texture-768.cov"),
        include_bytes!("texture_resources/circle-ring-texture-384.cov"),
        include_bytes!("texture_resources/circle-ring-texture-192.cov"),
        include_bytes!("texture_resources/circle-ring-texture-96.cov"),
        include_bytes!("texture_resources/circle-ring-texture-48.cov"),
        include_bytes!("texture_resources/circle-ring-texture-24.cov"),
        include_bytes!("texture_resources/circle-ring-texture-12.cov"),
    ];
    const CIRCLE_RING_PROG_TEXTURES: [&'static [u8]; Circle::MIPS as usize] = [
        include_bytes!("texture_resources/circle-ring-1536.prog"),
        include_bytes!("texture_resources/circle-ring-768.prog"),
        include_bytes!("texture_resources/circle-ring-384.prog"),
        include_bytes!("texture_resources/circle-ring-192.prog"),
        include_bytes!("texture_resources/circle-ring-96.prog"),
        include_bytes!("texture_resources/circle-ring-48.prog"),
        include_bytes!("texture_resources/circle-ring-24.prog"),
        include_bytes!("texture_resources/circle-ring-12.prog"),
    ];
    fn texture_data(resources: [&[u8]; Circle::MIPS as usize]) -> Vec<u8> {
        let mut data = vec![];
        for n in resources {
            data.extend(rmp_serde::from_slice::<Vec<u8>>(n).unwrap());
        }
        data
    }
}
impl Render for Circle {
    type Resources = CircleRenderResources;
    type RenderPackage = ();
    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(4);

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo
            .device()
            .create_shader_module(wgpu::include_wgsl!("circle.wgsl"));
        let texture_data = Circle::texture_data(Circle::CIRCLE_TEXTURES);
        let (texture, view) = ginkgo.texture_r8unorm_d2(
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::MIPS,
            texture_data.as_slice(),
        );
        let ring_texture_data = Circle::texture_data(Circle::CIRCLE_RING_TEXTURES);
        let (ring_texture, ring_view) = ginkgo.texture_r8unorm_d2(
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::MIPS,
            ring_texture_data.as_slice(),
        );
        let progress_texture_data = Circle::texture_data(Circle::CIRCLE_PROG_TEXTURES);
        let (progress_texture, progress_view) = ginkgo.texture_r8unorm_d2(
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::MIPS,
            progress_texture_data.as_slice(),
        );
        let ring_progress_texture_data = Circle::texture_data(Circle::CIRCLE_RING_PROG_TEXTURES);
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