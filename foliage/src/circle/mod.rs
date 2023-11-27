use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Component};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::ash::instruction::{
    RenderInstructionHandle, RenderInstructionsRecorder, RenderRecordBehavior,
};
use crate::ash::render::{Render, RenderPhase};
use crate::ash::render_package::RenderPackage;
use crate::ash::render_packet::RenderPacket;
use crate::color::Color;
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::{Elm, Leaf};
use crate::ginkgo::Ginkgo;
use crate::instance::{InstanceCoordinator, InstanceCoordinatorBuilder};
use crate::texture::{MipsLevel, TextureCoordinates};

#[repr(C)]
#[derive(Component, Copy, Clone, PartialEq, Default, Pod, Zeroable, Serialize, Deserialize)]
pub struct CircleStyle(pub(crate) f32);

impl CircleStyle {
    pub fn flat() -> Self {
        Self(0.0)
    }
    pub fn ring() -> Self {
        Self(1.0)
    }
}

const VERTICES: [Vertex; 6] = [
    Vertex::new(
        CReprPosition::new(1f32, 0f32),
        TextureCoordinates::new(1f32, 0f32),
    ),
    Vertex::new(
        CReprPosition::new(0f32, 0f32),
        TextureCoordinates::new(0f32, 0f32),
    ),
    Vertex::new(
        CReprPosition::new(0f32, 1f32),
        TextureCoordinates::new(0f32, 1f32),
    ),
    Vertex::new(
        CReprPosition::new(1f32, 0f32),
        TextureCoordinates::new(1f32, 0f32),
    ),
    Vertex::new(
        CReprPosition::new(0f32, 1f32),
        TextureCoordinates::new(0f32, 1f32),
    ),
    Vertex::new(
        CReprPosition::new(1f32, 1f32),
        TextureCoordinates::new(1f32, 1f32),
    ),
];

#[derive(Bundle)]
pub struct Circle {
    style: DifferentialBundle<CircleStyle>,
    position: DifferentialBundle<Position<InterfaceContext>>,
    area: DifferentialBundle<Area<InterfaceContext>>,
    color: DifferentialBundle<Color>,
    differentiable: Differentiable,
}

pub struct Diameter(pub CoordinateUnit);

impl Diameter {
    pub const MAX: CoordinateUnit = Circle::CIRCLE_TEXTURE_DIMENSIONS as CoordinateUnit;
    pub fn new(r: CoordinateUnit) -> Self {
        Self(r.min(Self::MAX).max(0f32))
    }
}

impl Circle {
    const CIRCLE_TEXTURE_DIMENSIONS: u32 = 1024;
    const MIPS: u32 = 3;
    pub fn new(
        style: CircleStyle,
        position: Position<InterfaceContext>,
        diameter: Diameter,
        layer: Layer,
        color: Color,
    ) -> Self {
        Self {
            style: DifferentialBundle::new(style),
            position: DifferentialBundle::new(position),
            area: DifferentialBundle::new(Area::new(diameter.0, diameter.0)),
            color: DifferentialBundle::new(color),
            differentiable: Differentiable::new::<Self>(layer),
        }
    }
}

impl Leaf for Circle {
    fn attach(elm: &mut Elm) {
        differential_enable!(
            elm,
            Position<InterfaceContext>,
            Area<InterfaceContext>,
            Color,
            CircleStyle
        );
    }
}

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub struct Vertex {
    position: CReprPosition,
    texture_coordinates: TextureCoordinates,
}

impl Vertex {
    const fn new(position: CReprPosition, texture_coordinates: TextureCoordinates) -> Self {
        Self {
            position,
            texture_coordinates,
        }
    }
}

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
}

impl Render for Circle {
    type Resources = CircleRenderResources;
    type RenderPackage = ();
    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(1);

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
                .unwrap()
                .iter(),
        );
        texture_data.extend(
            serde_json::from_str::<Vec<u8>>(include_str!(
                "texture_resources/circle-texture-256.cov"
            ))
                .unwrap()
                .iter(),
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
        let (ring_texture, ring_view) = ginkgo.texture_r8unorm_d2(
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::CIRCLE_TEXTURE_DIMENSIONS,
            Circle::MIPS,
            ring_texture_data.as_slice(),
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
                            attributes: &wgpu::vertex_attr_array![7 => Uint32],
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
        }
    }

    fn create_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        render_packet: RenderPacket,
    ) -> Self::RenderPackage {
        resources.instance_coordinator.queue_add(entity);
        Self::instance_coordinator_queue_write(ginkgo, resources, entity, render_packet);
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
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        _package: &mut RenderPackage<Self>,
        render_packet: RenderPacket,
    ) {
        Self::instance_coordinator_queue_write(ginkgo, resources, entity, render_packet);
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
            recorder.0.draw(
                0..VERTICES.len() as u32,
                0..resources.instance_coordinator.instances(),
            );
            return Some(recorder.finish());
        }
        None
    }
    fn instance_coordinator_queue_write(
        ginkgo: &Ginkgo,
        resources: &mut CircleRenderResources,
        entity: Entity,
        render_packet: RenderPacket,
    ) {
        if let Some(pos) = render_packet.get::<Position<InterfaceContext>>() {
            resources
                .instance_coordinator
                .queue_write(entity, pos.to_device(ginkgo.scale_factor()).to_c());
        }
        if let Some(area) = render_packet.get::<Area<InterfaceContext>>() {
            let area_scaled = area.to_device(ginkgo.scale_factor());
            let mips_level = MipsLevel::new(
                Area::new(
                    Circle::CIRCLE_TEXTURE_DIMENSIONS as CoordinateUnit,
                    Circle::CIRCLE_TEXTURE_DIMENSIONS as CoordinateUnit,
                ),
                Circle::MIPS,
                area_scaled,
            );
            resources
                .instance_coordinator
                .queue_write(entity, mips_level);
            resources
                .instance_coordinator
                .queue_write(entity, area_scaled.to_c());
        }
        if let Some(layer) = render_packet.get::<Layer>() {
            resources.instance_coordinator.queue_write(entity, layer);
            resources.instance_coordinator.queue_key_layer_change(entity, layer);
        }
        if let Some(color) = render_packet.get::<Color>() {
            resources.instance_coordinator.queue_write(entity, color);
        }
        if let Some(style) = render_packet.get::<CircleStyle>() {
            resources.instance_coordinator.queue_write(entity, style);
        }
    }
}

#[test]
fn png() {
    for mip in [1024, 512, 256] {
        Ginkgo::png_to_cov(
            format!("/home/salt/Desktop/dev/foliage/foliage/src/circle/texture_resources/circle-ring-{}.png", mip),
            format!("/home/salt/Desktop/dev/foliage/foliage/src/circle/texture_resources/circle-ring-texture-{}.cov", mip),
        );
        Ginkgo::png_to_cov(
            format!("/home/salt/Desktop/dev/foliage/foliage/src/circle/texture_resources/circle-{}.png", mip),
            format!("/home/salt/Desktop/dev/foliage/foliage/src/circle/texture_resources/circle-texture-{}.cov", mip),
        );
    }
}