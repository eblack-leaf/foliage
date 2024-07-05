use std::collections::{HashMap, HashSet};

use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, IntoSystemConfigs, Resource};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::{Query, Res};
use bytemuck::{Pod, Zeroable};
use fontdue::layout::CoordinateSystem;
use serde::{Deserialize, Serialize};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayoutDescriptor,
    PipelineLayoutDescriptor, RenderPipelineDescriptor, ShaderStages, TextureFormat,
    TextureSampleType, TextureViewDimension, VertexState, VertexStepMode,
};
use wgpu::{BindGroupLayout, RenderPipeline};

use crate::ash::{Render, RenderDirectiveRecorder, RenderPhase, Renderer};
use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::{GpuSection, Section};
use crate::coordinate::{Coordinates, DeviceContext, LogicalContext};
use crate::differential::{Differential, RenderLink};
use crate::elm::{Elm, RenderQueueHandle, ScheduleMarkers};
use crate::ginkgo::{Ginkgo, ScaleFactor, VectorUniform};
use crate::instances::Instances;
use crate::texture::{AtlasEntry, TextureAtlas, TextureCoordinates};
use crate::Leaf;
impl Leaf for Text {
    fn attach(elm: &mut Elm) {
        elm.enable_differential::<Self, GpuSection>();
        elm.enable_differential::<Self, Layer>();
        elm.enable_differential::<Self, Glyphs>();
        elm.enable_differential::<Self, GlyphMetrics>();
        elm.ecs.world.insert_resource(MonospacedFont::new(40));
        elm.scheduler.main.add_systems((
            (distill, color_changes).in_set(ScheduleMarkers::Config),
            clear_removed.after(ScheduleMarkers::Differential),
        ));
    }
}
#[derive(Bundle, Clone)]
pub struct Text {
    link: RenderLink,
    value: TextValue,
    section: Section<LogicalContext>,
    layer: Differential<Layer>,
    gpu_section: Differential<GpuSection>,
    glyphs: Differential<Glyphs>,
    glyph_colors: GlyphColors,
    color: Color,
    metrics: Differential<GlyphMetrics>,
}
impl Text {
    pub fn new<S: AsRef<str>, C: Into<Color>>(tv: S, color: C) -> Self {
        let color = color.into();
        Self {
            link: RenderLink::new::<Self>(),
            value: TextValue(tv.as_ref().to_string()),
            section: Default::default(),
            layer: Differential::new(Layer::default()),
            gpu_section: Differential::new(GpuSection::default()),
            glyphs: Differential::new(Glyphs::default()),
            glyph_colors: GlyphColors {
                position_to_color: Default::default(),
            },
            color,
            metrics: Differential::new(GlyphMetrics::default()),
        }
    }
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
#[derive(Component, Copy, Clone, PartialEq, Default)]
pub(crate) struct GlyphMetrics {
    unique_characters: u32,
    block: Coordinates,
}
#[derive(PartialEq, Clone, Component, Default)]
pub(crate) struct Glyphs {
    glyphs: HashMap<GlyphOffset, Glyph>,
    removed: HashSet<GlyphOffset>,
    font_size: f32,
}
#[derive(PartialEq, Clone, Component)]
pub struct GlyphColors {
    position_to_color: HashMap<GlyphOffset, Color>,
}
impl GlyphColors {
    pub fn obtain(&self, base: Color, offset: GlyphOffset) -> Color {
        if let Some(alternate) = self.position_to_color.get(&offset) {
            *alternate
        } else {
            base
        }
    }
}
#[derive(Clone, Component)]
pub struct TextValue(pub String);
impl TextValue {
    pub fn num_unique_characters(&self) -> u32 {
        let mut uc = HashSet::new();
        for c in self.0.chars() {
            uc.insert(c);
        }
        uc.len() as u32
    }
}
pub(crate) fn color_changes(
    mut texts: Query<
        (&mut Glyphs, &GlyphColors, &Color),
        Or<(Changed<GlyphColors>, Changed<Color>)>,
    >,
) {
    for (mut glyphs, colors, base) in texts.iter_mut() {
        for (offset, glyph) in glyphs.glyphs.iter_mut() {
            glyph.color = colors.obtain(*base, *offset);
        }
    }
}
pub(crate) fn distill(
    mut texts: Query<
        (
            &TextValue,
            &mut Glyphs,
            &GlyphColors,
            &Color,
            &mut Area<LogicalContext>,
            &mut Position<LogicalContext>,
            &mut GlyphMetrics,
        ),
        Or<(
            Changed<TextValue>,
            Changed<Position<LogicalContext>>,
            Changed<Area<LogicalContext>>,
        )>,
    >,
    font: Res<MonospacedFont>,
    scale_factor: Res<ScaleFactor>,
) {
    for (value, mut glyphs, colors, base, mut area, mut pos, mut metrics) in texts.iter_mut() {
        let mut placer = fontdue::layout::Layout::new(CoordinateSystem::PositiveYDown);
        let scaled_area = area.to_device(scale_factor.value());
        placer.reset(&fontdue::layout::LayoutSettings {
            max_width: Some(scaled_area.width()),
            max_height: Some(scaled_area.height()),
            ..fontdue::layout::LayoutSettings::default()
        });
        let mut line_counts = Vec::new();
        for line in value.0.lines() {
            line_counts.push(line.len());
        }
        let num_lines = line_counts.len();
        let max_in_a_line = line_counts.iter().max().copied().unwrap_or_default();
        let (projected_font_size, adjusted_bounds, character_size) = {
            let mut attempted_size = 0f32;
            let mut attempted_dims = Coordinates::new(0.0, 0.0);
            let mut character_dims = Coordinates::default();
            while attempted_dims.horizontal() < scaled_area.coordinates.horizontal()
                && attempted_dims.vertical() < scaled_area.coordinates.vertical()
            {
                attempted_size += 1f32 * scale_factor.value();
                let metrics = font.0.metrics('a', attempted_size);
                let horizontal_metrics = font.0.horizontal_line_metrics(attempted_size).unwrap();
                character_dims = Coordinates::new(
                    (metrics.advance_width * 1.1).ceil(),
                    ((horizontal_metrics.ascent - horizontal_metrics.descent) * 1.1).ceil(),
                );
                attempted_dims = Coordinates::new(
                    character_dims.horizontal() * max_in_a_line as f32,
                    character_dims.vertical() * num_lines as f32,
                );
            }
            let final_font_size = (attempted_size - 1.0 * scale_factor.value()).max(1.0);
            let metrics = font.0.metrics('a', final_font_size);
            let horizontal_metrics = font.0.horizontal_line_metrics(final_font_size).unwrap();
            let final_dims = Coordinates::new(
                metrics.advance_width.ceil(),
                (horizontal_metrics.ascent - horizontal_metrics.descent).ceil(),
            );
            let final_area = Coordinates::new(
                character_dims.horizontal() * max_in_a_line as f32,
                character_dims.vertical() * num_lines as f32,
            );
            let adjusted_section = Section::device(pos.to_device(scale_factor.value()), final_area)
                .to_logical(scale_factor.value());
            let old = Section::logical(*pos, *area);
            let diff = old.center() - adjusted_section.center();
            let final_section =
                Section::logical(adjusted_section.position + diff, adjusted_section.area);
            (final_font_size, final_section, final_dims)
        };
        pos.coordinates = adjusted_bounds.position.coordinates;
        area.coordinates = adjusted_bounds.area.coordinates;
        glyphs.font_size = projected_font_size;
        placer.append(
            &[&font.0],
            &fontdue::layout::TextStyle::new(value.0.as_str(), glyphs.font_size, 0),
        );
        if metrics.block != character_size {
            metrics.block = character_size;
            metrics.unique_characters = value.num_unique_characters();
        }
        let old_glyphs = glyphs.glyphs.drain().collect::<Vec<(GlyphOffset, Glyph)>>();
        glyphs.removed.clear();
        for glyph in placer.glyphs().iter() {
            // TODO filter?
            glyphs.glyphs.insert(
                glyph.byte_offset,
                Glyph {
                    key: GlyphKey::new(glyph.key),
                    section: Section::new((glyph.x, glyph.y), (glyph.width, glyph.height)),
                    parent: glyph.parent,
                    color: colors.obtain(*base, glyph.byte_offset),
                },
            );
        }
        for (offset, glyph) in old_glyphs {
            if !glyphs.glyphs.contains_key(&offset) {
                glyphs.removed.insert(offset);
            }
        }
    }
}
pub(crate) fn clear_removed(mut texts: Query<&mut Glyphs>) {
    for mut glyphs in texts.iter_mut() {
        glyphs.removed.clear();
    }
}
pub(crate) struct TextGroup {
    texture_atlas: TextureAtlas<GlyphKey, GlyphOffset, u8>,
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
    font: MonospacedFont,
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
            bind_group_layouts: &[&group_layout, &bind_group_layout],
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
                    Ginkgo::vertex_buffer_layout::<Color>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![3 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<TextureCoordinates>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![4 => Float32x4],
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
            font: MonospacedFont::new(40),
        }
    }

    fn prepare(
        renderer: &mut Renderer<Self>,
        queue_handle: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) -> bool {
        let mut should_record = false;
        for packet in queue_handle.read_removes::<Self>() {
            renderer
                .resource_handle
                .groups
                .get_mut(&packet)
                .unwrap()
                .should_record = true;
            should_record = true;
            renderer
                .resource_handle
                .groups
                .get_mut(&packet)
                .unwrap()
                .instances
                .clear();
            renderer.directive_manager.remove(packet);
        }
        for packet in queue_handle.read_adds::<Self, GlyphMetrics>() {
            let atlas = TextureAtlas::new(
                ginkgo,
                packet.value.block,
                packet.value.unique_characters,
                TextureFormat::R8Unorm,
            );
            let uniform = VectorUniform::new(ginkgo.context(), [0.0, 0.0, 0.0, 0.0]);
            let bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
                label: Some("text-group-bind-group"),
                layout: &renderer.resource_handle.group_layout,
                entries: &[
                    Ginkgo::texture_bind_group_entry(atlas.view(), 0),
                    Ginkgo::uniform_bind_group_entry(&uniform.uniform, 1),
                ],
            });
            renderer.resource_handle.groups.insert(
                packet.entity,
                TextGroup {
                    texture_atlas: atlas,
                    bind_group,
                    instances: Instances::new(packet.value.unique_characters)
                        .with_attribute::<GpuSection>(ginkgo)
                        .with_attribute::<Color>(ginkgo)
                        .with_attribute::<TextureCoordinates>(ginkgo),
                    pos_and_layer: uniform,
                    should_record: false,
                },
            );
        }
        for packet in queue_handle.read_adds::<Self, GpuSection>() {
            renderer
                .resource_handle
                .groups
                .get_mut(&packet.entity)
                .unwrap()
                .pos_and_layer
                .set(0, packet.value.pos.0.horizontal());
            renderer
                .resource_handle
                .groups
                .get_mut(&packet.entity)
                .unwrap()
                .pos_and_layer
                .set(1, packet.value.pos.0.vertical());
            renderer
                .resource_handle
                .groups
                .get_mut(&packet.entity)
                .unwrap()
                .pos_and_layer
                .write(ginkgo.context());
        }
        for packet in queue_handle.read_adds::<Self, Layer>() {
            renderer
                .resource_handle
                .groups
                .get_mut(&packet.entity)
                .unwrap()
                .pos_and_layer
                .set(2, packet.value.0);
            renderer
                .resource_handle
                .groups
                .get_mut(&packet.entity)
                .unwrap()
                .pos_and_layer
                .write(ginkgo.context());
        }
        for packet in queue_handle.read_adds::<Self, Glyphs>() {
            for offset in packet.value.removed.iter() {
                renderer
                    .resource_handle
                    .groups
                    .get_mut(&packet.entity)
                    .unwrap()
                    .instances
                    .queue_remove(*offset);
                // TODO remove reference to glyph.key
            }
            let mut queued_tex_reads = HashSet::new();
            for (offset, glyph) in packet.value.glyphs.iter() {
                renderer
                    .resource_handle
                    .groups
                    .get_mut(&packet.entity)
                    .unwrap()
                    .instances
                    .add(*offset);
                renderer
                    .resource_handle
                    .groups
                    .get_mut(&packet.entity)
                    .unwrap()
                    .instances
                    .checked_write(*offset, glyph.section.to_gpu());
                renderer
                    .resource_handle
                    .groups
                    .get_mut(&packet.entity)
                    .unwrap()
                    .instances
                    .checked_write(*offset, glyph.color);
                if !renderer
                    .resource_handle
                    .groups
                    .get_mut(&packet.entity)
                    .unwrap()
                    .texture_atlas
                    .has_key(glyph.key)
                {
                    let (metrics, rasterization) = renderer
                        .resource_handle
                        .font
                        .0
                        .rasterize_indexed(glyph.key.glyph_index, packet.value.font_size);
                    let entry = AtlasEntry::new(rasterization, (metrics.width, metrics.height));
                    renderer
                        .resource_handle
                        .groups
                        .get_mut(&packet.entity)
                        .unwrap()
                        .texture_atlas
                        .add_entry(glyph.key, entry);
                    queued_tex_reads.insert((glyph.key, *offset));
                } else {
                    let tex_coords = renderer
                        .resource_handle
                        .groups
                        .get_mut(&packet.entity)
                        .unwrap()
                        .texture_atlas
                        .tex_coordinates(glyph.key);
                    renderer
                        .resource_handle
                        .groups
                        .get_mut(&packet.entity)
                        .unwrap()
                        .instances
                        .checked_write(*offset, tex_coords);
                }
                renderer
                    .resource_handle
                    .groups
                    .get_mut(&packet.entity)
                    .unwrap()
                    .texture_atlas
                    .add_reference(glyph.key, *offset);
            }
            let changed = renderer
                .resource_handle
                .groups
                .get_mut(&packet.entity)
                .unwrap()
                .texture_atlas
                .resolve(ginkgo);
            for info in changed {
                renderer
                    .resource_handle
                    .groups
                    .get_mut(&packet.entity)
                    .unwrap()
                    .instances
                    .checked_write(info.key, info.tex_coords);
            }
            for (key, referrer) in queued_tex_reads {
                let tex_coords = renderer
                    .resource_handle
                    .groups
                    .get_mut(&packet.entity)
                    .unwrap()
                    .texture_atlas
                    .tex_coordinates(key);
                renderer
                    .resource_handle
                    .groups
                    .get_mut(&packet.entity)
                    .unwrap()
                    .instances
                    .checked_write(referrer, tex_coords);
            }
        }
        for (_, group) in renderer.resource_handle.groups.iter_mut() {
            if group.instances.resolve_changes(ginkgo) {
                group.should_record = true;
                should_record = true;
            }
        }
        should_record
    }

    fn record(renderer: &mut Renderer<Self>, ginkgo: &Ginkgo) {
        for (entity, group) in renderer.resource_handle.groups.iter_mut() {
            if group.should_record {
                let mut recorder = RenderDirectiveRecorder::new(ginkgo);
                if group.instances.num_instances() > 0 {
                    recorder.0.set_pipeline(&renderer.resource_handle.pipeline);
                    recorder
                        .0
                        .set_bind_group(1, &renderer.resource_handle.bind_group, &[]);
                    recorder.0.set_bind_group(0, &group.bind_group, &[]);
                    recorder
                        .0
                        .set_vertex_buffer(0, renderer.resource_handle.vertex_buffer.slice(..));
                    recorder
                        .0
                        .set_vertex_buffer(1, group.instances.buffer::<GpuSection>().slice(..));
                    recorder
                        .0
                        .set_vertex_buffer(2, group.instances.buffer::<Color>().slice(..));
                    recorder.0.set_vertex_buffer(
                        3,
                        group.instances.buffer::<TextureCoordinates>().slice(..),
                    );
                    recorder
                        .0
                        .draw(0..VERTICES.len() as u32, 0..group.instances.num_instances());
                }
                renderer.directive_manager.fill(*entity, recorder.finish());
                group.should_record = false;
            }
        }
    }
}
