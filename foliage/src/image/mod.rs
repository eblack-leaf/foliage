use std::collections::HashMap;

use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, IntoSystemConfigs};
use bevy_ecs::query::{Changed, Or, With};
use bevy_ecs::system::Query;
use bytemuck::{Pod, Zeroable};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, PipelineLayoutDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderStages, Texture, TextureAspect, TextureFormat,
    TextureSampleType, TextureView, TextureViewDimension, VertexState, VertexStepMode,
};

use crate::ash::{Render, RenderPhase, Renderer};
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::section::{GpuSection, Section};
use crate::coordinate::{Coordinates, LogicalContext};
use crate::differential::{Differential, RenderLink};
use crate::elm::{Elm, RenderQueueHandle, ScheduleMarkers};
use crate::ginkgo::texture::TextureCoordinates;
use crate::ginkgo::Ginkgo;
use crate::instances::Instances;
use crate::Leaf;

#[derive(Copy, Clone, Component)]
pub struct AspectRatio(pub f32);
impl AspectRatio {
    pub fn from_dimensions(w: f32, h: f32) -> Self {
        Self(w / h)
    }
    pub fn from_coordinates(c: Coordinates) -> Self {
        Self(c.horizontal() / c.vertical())
    }
    pub fn new(r: f32) -> Self {
        Self(r)
    }
    pub fn constrain(&self, o: Coordinates) -> Coordinates {
        todo!()
    }
}
#[derive(Bundle)]
pub struct Image {
    link: RenderLink,
    fill: Differential<ImageFill>,
    gpu_section: Differential<GpuSection>,
    section: Section<LogicalContext>,
    layer: Differential<Layer>,
    aspect_ratio: AspectRatio,
}
impl Image {
    pub fn slot<I: Into<ImageSlotId>, C: Into<Coordinates>>(id: I, c: C) -> ImageSlot {
        ImageSlot {
            link: RenderLink::new::<Image>(),
            extent: Differential::new(ImageSlotDescriptor(id.into(), c.into())),
        }
    }
    pub fn fill<
        I: Into<ImageSlotId>,
        S: Into<Section<LogicalContext>>,
        L: Into<Layer>,
        C: Into<Coordinates>,
    >(
        id: I,
        s: S,
        l: L,
        data: Vec<u8>,
    ) -> Self {
        let s = s.into();
        let slice = data.as_slice();
        let image = image::load_from_memory(slice).unwrap().to_rgba32f();
        let dimensions = Coordinates::new(image.width() as f32, image.height() as f32);
        let image_bytes = image
            .pixels()
            .flat_map(|p| p.0.to_vec())
            .collect::<Vec<f32>>();
        Self {
            link: RenderLink::new::<Image>(),
            fill: Differential::new(ImageFill(id.into(), image_bytes, dimensions)),
            gpu_section: Differential::new(GpuSection::default()),
            section: s,
            layer: Differential::new(l.into()),
            aspect_ratio: AspectRatio::from_coordinates(dimensions),
        }
    }
    pub fn with_aspect_ratio(mut self, a: AspectRatio) -> Self {
        self.aspect_ratio = a;
        self
    }
}
fn constrain(
    mut images: Query<
        (&AspectRatio, &mut Area<LogicalContext>),
        (
            With<ImageFill>,
            Or<(Changed<AspectRatio>, Changed<Area<LogicalContext>>)>,
        ),
    >,
) {
    for (ratio, mut area) in images.iter_mut() {
        area.coordinates = ratio.constrain(area.coordinates);
    }
}
#[derive(Component, Clone, PartialEq)]
pub struct ImageFill(pub ImageSlotId, pub Vec<f32>, pub Coordinates);
#[derive(Component, Clone, PartialEq)]
pub struct ImageSlotDescriptor(pub ImageSlotId, pub Coordinates);
#[derive(Bundle)]
pub struct ImageSlot {
    link: RenderLink,
    extent: Differential<ImageSlotDescriptor>,
}
impl Leaf for Image {
    fn attach(elm: &mut Elm) {
        elm.enable_differential::<Self, GpuSection>();
        elm.enable_differential::<Self, Layer>();
        elm.enable_differential::<Self, ImageFill>();
        elm.enable_differential::<Self, ImageSlotDescriptor>();
        elm.scheduler
            .main
            .add_systems(constrain.in_set(ScheduleMarkers::Config));
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub struct Vertex {
    position: Coordinates,
    tx_index: Coordinates,
}

impl Vertex {
    pub(crate) const fn new(position: Coordinates, tx_index: Coordinates) -> Self {
        Self { position, tx_index }
    }
}

pub(crate) const VERTICES: [Vertex; 6] = [
    Vertex::new(Coordinates::new(1f32, 0f32), Coordinates::new(0f32, 2f32)),
    Vertex::new(Coordinates::new(0f32, 0f32), Coordinates::new(0f32, 1f32)),
    Vertex::new(Coordinates::new(0f32, 1f32), Coordinates::new(0f32, 3f32)),
    Vertex::new(Coordinates::new(1f32, 0f32), Coordinates::new(0f32, 2f32)),
    Vertex::new(Coordinates::new(0f32, 1f32), Coordinates::new(0f32, 3f32)),
    Vertex::new(Coordinates::new(1f32, 1f32), Coordinates::new(2f32, 3f32)),
];
#[derive(Copy, Clone, Component, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ImageSlotId(pub i32);
pub struct ImageResources {
    pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group: BindGroup,
    group_layout: BindGroupLayout,
    groups: HashMap<ImageSlotId, ImageGroup>,
    entity_to_image: HashMap<Entity, ImageSlotId>,
}
pub(crate) struct ImageGroup {
    tex: Texture,
    view: TextureView,
    bind_group: BindGroup,
    instances: Instances<Entity>,
    slot_extent: Coordinates,
    image_extent: Coordinates,
    texture_coordinates: TextureCoordinates,
    should_record: bool,
}
impl Render for Image {
    type DirectiveGroupKey = ImageSlotId;
    const RENDER_PHASE: RenderPhase = RenderPhase::Opaque;
    type Resources = ImageResources;

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo.create_shader(include_wgsl!("image.wgsl"));
        let group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("image-group-bind-group-layout"),
            entries: &[Ginkgo::bind_group_layout_entry(0)
                .at_stages(ShaderStages::FRAGMENT)
                .texture_entry(
                    TextureViewDimension::D2,
                    TextureSampleType::Float { filterable: false },
                )],
        });
        let bind_group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("image-bind-group-layout"),
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
            label: Some("image-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                ginkgo.viewport_bind_group_entry(0),
                Ginkgo::sampler_bind_group_entry(&sampler, 1),
            ],
        });
        let pipeline_layout = ginkgo.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("image-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout, &group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = ginkgo.create_pipeline(&RenderPipelineDescriptor {
            label: Some("image-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vertex_entry",
                compilation_options: Default::default(),
                buffers: &[
                    Ginkgo::vertex_buffer_layout::<Vertex>(
                        VertexStepMode::Vertex,
                        &wgpu::vertex_attr_array![0 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<GpuSection>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![1 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<Layer>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![2 => Float32],
                    ),
                    Ginkgo::vertex_buffer_layout::<TextureCoordinates>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![3 => Float32x4],
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
        ImageResources {
            pipeline,
            vertex_buffer: ginkgo.create_vertex_buffer(VERTICES),
            bind_group,
            group_layout,
            groups: HashMap::new(),
            entity_to_image: Default::default(),
        }
    }

    fn prepare(
        renderer: &mut Renderer<Self>,
        queue_handle: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) -> bool {
        for entity in queue_handle.read_removes::<Self>() {
            let id = renderer.resource_handle.entity_to_image.remove(&entity);
            if let Some(id) = id {
                renderer
                    .resource_handle
                    .groups
                    .get_mut(&id)
                    .unwrap()
                    .instances
                    .remove(entity);
            }
        }
        for packet in queue_handle.read_adds::<Self, ImageSlotDescriptor>() {
            let (tex, view) =
                ginkgo.create_texture(TextureFormat::Rgba32Float, packet.value.1, 1, &[]);
            renderer.resource_handle.groups.insert(
                packet.value.0,
                ImageGroup {
                    tex,
                    view,
                    bind_group: ginkgo.create_bind_group(&BindGroupDescriptor {
                        label: Some("image-group-bind-group"),
                        layout: &renderer.resource_handle.group_layout,
                        entries: &[Ginkgo::texture_bind_group_entry(&view, 0)],
                    }),
                    instances: Instances::new(1)
                        .with_attribute::<GpuSection>(ginkgo)
                        .with_attribute::<Layer>(ginkgo)
                        .with_attribute::<TextureCoordinates>(ginkgo),
                    slot_extent: packet.value.1,
                    image_extent: Default::default(),
                    texture_coordinates: Default::default(),
                    should_record: false,
                },
            );
        }
        for packet in queue_handle.read_adds::<Self, ImageFill>() {
            ginkgo.context().queue.write_texture(
                ImageCopyTexture {
                    texture: &renderer
                        .resource_handle
                        .groups
                        .get(&packet.value.0)
                        .unwrap()
                        .tex,
                    mip_level: 0,
                    origin: Origin3d::default(),
                    aspect: TextureAspect::All,
                },
                bytemuck::cast_slice(&packet.value.1),
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        packet.value.2.horizontal() as u32 * std::mem::size_of::<f32>() as u32 * 4,
                    ),
                    rows_per_image: Some(packet.value.2.vertical() as u32),
                },
                Extent3d {
                    width: packet.value.2.horizontal() as u32,
                    height: packet.value.2.vertical() as u32,
                    depth_or_array_layers: 1,
                },
            );
            renderer
                .resource_handle
                .groups
                .get_mut(&packet.value.0)
                .unwrap()
                .image_extent = packet.value.2;
            let whole = renderer
                .resource_handle
                .groups
                .get(&packet.value.0)
                .unwrap()
                .slot_extent;
            let texture_coordinates =
                TextureCoordinates::from_section(Section::new((0, 0), packet.value.2), whole);
            renderer
                .resource_handle
                .groups
                .get_mut(&packet.value.0)
                .unwrap()
                .texture_coordinates = texture_coordinates;
            let old_keys = renderer
                .resource_handle
                .groups
                .get_mut(&packet.value.0)
                .unwrap()
                .instances
                .clear();
            for old in old_keys {
                renderer.resource_handle.entity_to_image.remove(&old);
            }
            renderer
                .resource_handle
                .groups
                .get_mut(&packet.value.0)
                .unwrap()
                .instances
                .add(packet.entity);
            renderer
                .resource_handle
                .groups
                .get_mut(&packet.value.0)
                .unwrap()
                .instances
                .checked_write(packet.entity, texture_coordinates);
            renderer
                .resource_handle
                .entity_to_image
                .insert(packet.entity, packet.value.0);
        }
        for packet in queue_handle.read_adds::<Self, GpuSection>() {
            let id = *renderer
                .resource_handle
                .entity_to_image
                .get(&packet.entity)
                .unwrap();
            renderer
                .resource_handle
                .groups
                .get_mut(&id)
                .unwrap()
                .instances
                .checked_write(packet.entity, packet.value);
        }
        for packet in queue_handle.read_adds::<Self, Layer>() {
            let id = *renderer
                .resource_handle
                .entity_to_image
                .get(&packet.entity)
                .unwrap();
            renderer
                .resource_handle
                .groups
                .get_mut(&id)
                .unwrap()
                .instances
                .checked_write(packet.entity, packet.value);
        }
        let mut should_record = false;
        for (_, group) in renderer.resource_handle.groups.iter_mut() {
            if group.instances.resolve_changes(ginkgo) {
                should_record = true;
                group.should_record = true;
            }
        }
        should_record
    }

    fn record(renderer: &mut Renderer<Self>, ginkgo: &Ginkgo) {
        todo!()
    }
}
