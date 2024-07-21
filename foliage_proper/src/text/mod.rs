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

use crate::action::HasRenderLink;
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
        elm.enable_filtering::<Text>();
    }
}
impl HasRenderLink for Text {
    fn has_link() -> bool {
        true
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
    font_size: FontSize,
}
impl Text {
    pub fn new<S: AsRef<str>, C: Into<Color>>(tv: S, font_size: FontSize, color: C) -> Self {
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
            font_size,
        }
    }
}
#[derive(Copy, Clone, Component)]
pub struct FontSize(pub(crate) f32);
impl From<u32> for FontSize {
    fn from(value: u32) -> Self {
        Self(value as f32)
    }
}
impl From<i32> for FontSize {
    fn from(value: i32) -> Self {
        Self(value as f32)
    }
}
impl FontSize {
    pub fn new(v: u32) -> Self {
        Self(v as f32)
    }
    pub fn value(&self) -> f32 {
        self.0
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
impl<S: AsRef<str>> From<S> for TextValue {
    fn from(value: S) -> Self {
        Self::new(value)
    }
}
impl TextValue {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self(s.as_ref().to_string())
    }
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
            &FontSize,
        ),
        Or<(
            Changed<TextValue>,
            Changed<Position<LogicalContext>>,
            Changed<Area<LogicalContext>>,
            Changed<FontSize>,
        )>,
    >,
    font: Res<MonospacedFont>,
    scale_factor: Res<ScaleFactor>,
) {
    for (value, mut glyphs, colors, base, mut area, mut pos, mut metrics, font_size) in
        texts.iter_mut()
    {
        let mut placer = fontdue::layout::Layout::new(CoordinateSystem::PositiveYDown);
        let scaled_area = area.to_device(scale_factor.value());
        placer.reset(&fontdue::layout::LayoutSettings {
            max_width: Some(scaled_area.width()),
            max_height: Some(scaled_area.height()),
            ..fontdue::layout::LayoutSettings::default()
        });
        let projected = font_size.value() * scale_factor.value();
        let character_dims = {
            let character_metrics = font.0.metrics('a', projected);
            let horizontal_metrics = font.0.horizontal_line_metrics(projected).unwrap();
            Coordinates::new(
                (character_metrics.advance_width).ceil(),
                (horizontal_metrics.ascent - horizontal_metrics.descent).ceil(),
            )
        };
        glyphs.font_size = projected;
        placer.append(
            &[&font.0],
            &fontdue::layout::TextStyle::new(value.0.as_str(), glyphs.font_size, 0),
        );
        if metrics.block != character_dims {
            metrics.block = character_dims;
            metrics.unique_characters = value.num_unique_characters();
        }
        let old_glyphs = glyphs.glyphs.drain().collect::<Vec<(GlyphOffset, Glyph)>>();
        glyphs.removed.clear();
        let mut new_extent = Coordinates::default();
        for glyph in placer.glyphs().iter() {
            // TODO filter?
            let section = Section::new((glyph.x, glyph.y), (glyph.width, glyph.height));
            if section.right() > new_extent.horizontal() {
                new_extent.0[0] = section.right();
            }
            if section.bottom() > new_extent.vertical() {
                new_extent.0[1] = section.bottom();
            }
            glyphs.glyphs.insert(
                glyph.byte_offset,
                Glyph {
                    key: GlyphKey::new(glyph.key),
                    section,
                    parent: glyph.parent,
                    color: colors.obtain(*base, glyph.byte_offset),
                },
            );
        }
        let num_lines = (new_extent.vertical() / character_dims.vertical()).ceil();
        let old = Section::logical(*pos, *area);
        let adjusted = Section::device(
            pos.to_device(scale_factor.value()),
            (
                new_extent.horizontal(),
                character_dims.vertical() * num_lines * 1.1,
            ),
        )
        .to_logical(scale_factor.value());
        let diff = old.center() - adjusted.center();
        // TODO changing pos + area here gets smaller each time
        // need to keep pos + area as guidelines for where to adjust from
        pos.coordinates = (old.position + diff).coordinates;
        area.coordinates = adjusted.area.coordinates;
        println!(
            "old:{:?} adjusted:{:?}",
            old,
            (old.position + diff).coordinates
        );
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
            let (changed, grown) = renderer
                .resource_handle
                .groups
                .get_mut(&packet.entity)
                .unwrap()
                .texture_atlas
                .resolve(ginkgo);
            for key in changed {
                let (metrics, rasterization) = renderer
                    .resource_handle
                    .font
                    .0
                    .rasterize_indexed(key.glyph_index, packet.value.font_size);
                let entry = AtlasEntry::new(rasterization, (metrics.width, metrics.height));
                let updated = renderer
                    .resource_handle
                    .groups
                    .get_mut(&packet.entity)
                    .unwrap()
                    .texture_atlas
                    .write_entry(ginkgo, key, entry);
                for change in updated.iter() {
                    renderer
                        .resource_handle
                        .groups
                        .get_mut(&packet.entity)
                        .unwrap()
                        .instances
                        .checked_write(change.key, change.tex_coords);
                }
            }
            if grown {
                // remake bind group
                let new_bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
                    label: Some("text-group-bind-group"),
                    layout: &renderer.resource_handle.group_layout,
                    entries: &[
                        Ginkgo::texture_bind_group_entry(
                            renderer
                                .resource_handle
                                .groups
                                .get(&packet.entity)
                                .unwrap()
                                .texture_atlas
                                .view(),
                            0,
                        ),
                        Ginkgo::uniform_bind_group_entry(
                            &renderer
                                .resource_handle
                                .groups
                                .get(&packet.entity)
                                .unwrap()
                                .pos_and_layer
                                .uniform,
                            1,
                        ),
                    ],
                });
                renderer
                    .resource_handle
                    .groups
                    .get_mut(&packet.entity)
                    .unwrap()
                    .bind_group = new_bind_group;
                renderer
                    .resource_handle
                    .groups
                    .get_mut(&packet.entity)
                    .unwrap()
                    .should_record = true;
                should_record = true;
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
                    renderer.directive_manager.fill(*entity, recorder.finish());
                } else {
                    renderer.directive_manager.remove(*entity);
                }
                group.should_record = false;
            }
        }
    }
}
