use std::collections::HashMap;

use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, Resource};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayoutDescriptor,
    PipelineLayoutDescriptor, RenderPipelineDescriptor, ShaderStages, TextureSampleType,
    TextureViewDimension, VertexState, VertexStepMode,
};
use wgpu::{BindGroupLayout, RenderPipeline};

use crate::ash::{Render, RenderPhase, Renderer};
use crate::color::Color;
use crate::coordinate::layer::Layer;
use crate::coordinate::section::{GpuSection, Section};
use crate::coordinate::{Coordinates, DeviceContext, LogicalContext};
use crate::differential::{Differential, RenderLink};
use crate::elm::{Elm, RenderQueueHandle};
use crate::texture::TextureCoordinates;
use crate::ginkgo::{Ginkgo, VectorUniform};
use crate::instances::Instances;
use crate::Leaf;
impl Leaf for Text {
    fn attach(elm: &mut Elm) {
        elm.enable_differential::<Self, GpuSection>();
        elm.enable_differential::<Self, Layer>();
        elm.enable_differential::<Self, Glyphs>();
        elm.ecs.world.insert_resource(MonospacedFont::new(40));
    }
}
#[derive(Bundle, Clone)]
pub struct Text {
    link: RenderLink,
    section: Section<LogicalContext>,
    layer: Differential<Layer>,
    gpu_section: Differential<GpuSection>,
    glyphs: Differential<Glyphs>,
}
#[derive(Serialize, Deserialize, Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub(crate) struct GlyphKey {
    pub(crate) glyph_index: u16,
    pub(crate) px: u32,
    pub(crate) font_hash: usize,
}
impl GlyphKey {
    pub(crate) fn new(raster_config: fontdue::layout::GlyphRasterConfig) -> Self {
        Self {
            glyph_index: raster_config.glyph_index,
            px: raster_config.px as u32,
            font_hash: raster_config.font_hash,
        }
    }
}
#[derive(Resource)]
pub struct MonospacedFont(pub(crate) fontdue::Font);
impl MonospacedFont {
    pub(crate) fn new(opt_scale: u32) -> Self {
        Self(
            fontdue::Font::from_bytes(
                include_bytes!("JetBrainsMono-Medium.ttf").as_slice(),
                fontdue::FontSettings {
                    scale: opt_scale as f32,
                    ..fontdue::FontSettings::default()
                },
            )
            .expect("font"),
        )
    }
}
#[derive(PartialEq, Clone)]
pub(crate) struct Glyph {
    pub(crate) key: GlyphKey,
    pub(crate) section: Section<DeviceContext>,
    pub(crate) parent: char,
    pub(crate) color: Color,
}
pub type GlyphOffset = usize;
#[derive(PartialEq, Clone, Component)]
pub(crate) struct Glyphs {
    glyphs: HashMap<GlyphOffset, Glyph>,
}
#[derive(PartialEq, Clone, Component)]
pub struct GlyphColors {
    position_to_color: HashMap<GlyphOffset, Color>,
}
pub(crate) struct TextGroup {
    bind_group: BindGroup,
    instances: Instances<GlyphOffset>,
    pos_and_layer: VectorUniform<f32>,
    should_record: bool,
}
pub struct TextResources {
    pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group: BindGroup,
    group_layout: BindGroupLayout,
    groups: HashMap<Entity, TextGroup>,
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub struct Vertex {
    position: Coordinates,
    tx_index: [u32; 2],
}
impl Vertex {
    pub(crate) const fn new(position: Coordinates, tx_index: [u32; 2]) -> Self {
        Self { position, tx_index }
    }
}
pub(crate) const VERTICES: [Vertex; 6] = [
    Vertex::new(Coordinates::new(1f32, 0f32), [2, 1]),
    Vertex::new(Coordinates::new(0f32, 0f32), [0, 1]),
    Vertex::new(Coordinates::new(0f32, 1f32), [0, 3]),
    Vertex::new(Coordinates::new(1f32, 0f32), [2, 1]),
    Vertex::new(Coordinates::new(0f32, 1f32), [0, 3]),
    Vertex::new(Coordinates::new(1f32, 1f32), [2, 3]),
];
impl Render for Text {
    type DirectiveGroupKey = Entity;
    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(3);
    type Resources = TextResources;

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo.create_shader(include_wgsl!("text.wgsl"));
        let vertex_buffer = ginkgo.create_vertex_buffer(VERTICES);
        let bind_group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("text-bind-group-layout"),
            entries: &[
                Ginkgo::bind_group_layout_entry(0)
                    .at_stages(ShaderStages::VERTEX)
                    .uniform_entry(),
                Ginkgo::bind_group_layout_entry(1)
                    .at_stages(ShaderStages::FRAGMENT)
                    .sampler_entry(),
            ],
        });
        let sampler = ginkgo.create_sampler();
        let bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
            label: Some("text-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                ginkgo.viewport_bind_group_entry(0),
                Ginkgo::sampler_bind_group_entry(&sampler, 1),
            ],
        });
        let group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("text-group-bind-group-layout"),
            entries: &[
                Ginkgo::bind_group_layout_entry(0)
                    .at_stages(ShaderStages::FRAGMENT)
                    .texture_entry(
                        TextureViewDimension::D2,
                        TextureSampleType::Float { filterable: false },
                    ),
                Ginkgo::bind_group_layout_entry(1)
                    .at_stages(ShaderStages::VERTEX)
                    .uniform_entry(),
            ],
        });
        let pipeline_layout = ginkgo.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("text-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout, &group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = ginkgo.create_pipeline(&RenderPipelineDescriptor {
            label: Some("text-render-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vertex_entry",
                compilation_options: Default::default(),
                buffers: &[
                    Ginkgo::vertex_buffer_layout::<Vertex>(
                        VertexStepMode::Vertex,
                        &wgpu::vertex_attr_array![0 => Float32x2, 1 => Uint32x2],
                    ),
                    Ginkgo::vertex_buffer_layout::<GpuSection>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![2 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<Layer>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![3 => Float32],
                    ),
                    Ginkgo::vertex_buffer_layout::<Color>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![4 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<TextureCoordinates>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![5 => Float32x4],
                    ),
                ],
            },
            primitive: Ginkgo::triangle_list_primitive(),
            depth_stencil: ginkgo.depth_stencil_state(),
            multisample: ginkgo.msaa_state(),
            fragment: Ginkgo::fragment_state(
                &shader,
                "fragment_entry",
                &ginkgo.alpha_color_target_state(),
            ),
            multiview: None,
        });
        TextResources {
            pipeline,
            vertex_buffer,
            bind_group,
            group_layout,
            groups: Default::default(),
        }
    }

    fn prepare(
        renderer: &mut Renderer<Self>,
        queue_handle: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) -> bool {
        todo!()
    }

    fn record(renderer: &mut Renderer<Self>, ginkgo: &Ginkgo) {
        todo!()
    }
}
