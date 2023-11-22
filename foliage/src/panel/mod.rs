use crate::ash::identification::{RenderId, RenderIdentification};
use crate::ash::instruction::{RenderInstructionHandle, RenderInstructionsRecorder};
use crate::ash::render::{Render, RenderPhase};
use crate::ash::render_packet::{RenderPacket, RenderPacketStore};
use crate::ash::renderer::{RenderPackage, RenderRecordBehavior};
use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::InterfaceContext;
use crate::differential::{Despawn, DifferentialBundle, DifferentialDisable};
use crate::differential_enable;
use crate::elm::{Elm, Leaf};
use crate::ginkgo::Ginkgo;
use crate::instance::{InstanceCoordinator, InstanceCoordinatorBuilder};
use crate::texture::TextureCoordinates;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::Entity;
use bytemuck::{Pod, Zeroable};

#[derive(Bundle)]
pub struct Panel {
    render_id: RenderId,
    position: DifferentialBundle<Position<InterfaceContext>>,
    area: DifferentialBundle<Area<InterfaceContext>>,
    layer: DifferentialBundle<Layer>,
    color: DifferentialBundle<Color>,
    differential_disable: DifferentialDisable,
    despawn: Despawn,
    render_packet_store: RenderPacketStore,
}
impl Panel {
    pub fn new(
        pos: Position<InterfaceContext>,
        area: Area<InterfaceContext>,
        layer: Layer,
        color: Color,
    ) -> Self {
        Self {
            render_id: <Self as RenderIdentification>::id(),
            position: DifferentialBundle::new(pos),
            area: DifferentialBundle::new(area),
            layer: DifferentialBundle::new(layer),
            color: DifferentialBundle::new(color),
            differential_disable: DifferentialDisable::default(),
            despawn: Despawn::default(),
            render_packet_store: RenderPacketStore::default(),
        }
    }
}
impl Leaf for Panel {
    fn attach(elm: &mut Elm) {
        differential_enable!(
            elm,
            Position<InterfaceContext>,
            Area<InterfaceContext>,
            Layer,
            Color
        );
        elm.job.container.spawn(Panel::new(
            (100, 100).into(),
            (200, 100).into(),
            2.into(),
            Color::OFF_WHITE.into(),
        ));
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
struct Vertex {
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
const VERTICES: [Vertex; 6] = [
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
pub struct PanelRenderResources {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    instance_coordinator: InstanceCoordinator<Entity>,
}
impl PanelRenderResources {
    const TEXTURE_DIMENSION: u32 = 10;
}
impl Render for Panel {
    type Resources = PanelRenderResources;
    type RenderPackage = ();
    const RENDER_PHASE: RenderPhase = RenderPhase::Opaque;

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo
            .device()
            .create_shader_module(wgpu::include_wgsl!("panel.wgsl"));
        let (texture, view) = ginkgo.texture_r8unorm_d2(
            PanelRenderResources::TEXTURE_DIMENSION,
            PanelRenderResources::TEXTURE_DIMENSION,
            include_bytes!("panel-texture.cov"),
        );
        let sampler = ginkgo
            .device()
            .create_sampler(&wgpu::SamplerDescriptor::default());
        let bind_group_layout =
            ginkgo
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("panel-render-pipeline-bind-group-layout"),
                    entries: &[
                        Ginkgo::vertex_uniform_bind_group_layout_entry(0),
                        Ginkgo::texture_d2_bind_group_entry(1),
                        Ginkgo::sampler_bind_group_layout_entry(2),
                    ],
                });
        let bind_group = ginkgo
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("panel-render-pipeline-bind-group"),
                layout: &bind_group_layout,
                entries: &[
                    ginkgo.viewport_bind_group_entry(0),
                    Ginkgo::texture_bind_group_entry(&view, 1),
                    Ginkgo::sampler_bind_group_entry(&sampler, 2),
                ],
            });
        let pipeline_layout =
            ginkgo
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("panel-render-pipeline-layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });
        let pipeline = ginkgo
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("panel-render-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vertex_entry",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<Vertex>(1),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![0 => Float32x4],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<CReprPosition>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![1 => Float32x2],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<CReprArea>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![2 => Float32x2],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<Layer>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![3 => Float32x2],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<Color>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![4 => Float32x4],
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
        let vertex_buffer = ginkgo.vertex_buffer_with_data(&VERTICES, "panel-vertex-buffer");
        let instance_coordinator = InstanceCoordinatorBuilder::new(4)
            .with_attribute::<CReprPosition>()
            .with_attribute::<CReprArea>()
            .with_attribute::<Layer>()
            .with_attribute::<Color>()
            .build(ginkgo);
        PanelRenderResources {
            pipeline,
            vertex_buffer,
            bind_group,
            texture,
            view,
            sampler,
            instance_coordinator,
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
        ()
    }

    fn on_package_removal(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        package: RenderPackage<Self>,
    ) {
        resources.instance_coordinator.queue_remove(entity);
    }

    fn prepare_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        package: &mut RenderPackage<Self>,
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
        RenderRecordBehavior::PerRenderer(Box::new(Panel::record))
    }
}

impl Panel {
    fn record<'a>(
        resources: &'a PanelRenderResources,
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
        resources: &mut PanelRenderResources,
        entity: Entity,
        render_packet: RenderPacket,
    ) {
        if let Some(pos) = render_packet.get::<Position<InterfaceContext>>() {
            resources
                .instance_coordinator
                .queue_write(entity, pos.to_device(ginkgo.scale_factor()).to_c());
        }
        if let Some(area) = render_packet.get::<Area<InterfaceContext>>() {
            resources
                .instance_coordinator
                .queue_write(entity, area.to_device(ginkgo.scale_factor()).to_c());
        }
        if let Some(layer) = render_packet.get::<Layer>() {
            resources.instance_coordinator.queue_write(entity, layer);
        }
        if let Some(color) = render_packet.get::<Color>() {
            resources.instance_coordinator.queue_write(entity, color);
        }
    }
}
