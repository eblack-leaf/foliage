use std::collections::HashMap;

use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use bevy_ecs::query::{Changed, Or, With};
use bevy_ecs::system::Query;
use bytemuck::{Pod, Zeroable};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages,
    TextureSampleType, TextureView, TextureViewDimension, VertexState, VertexStepMode,
};

use crate::ash::{Render, RenderPhase, Renderer};
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::section::{GpuSection, Section};
use crate::coordinate::{Coordinates, LogicalContext};
use crate::differential::{Differential, RenderLink};
use crate::elm::{Elm, RenderQueueHandle};
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
    id: Differential<ImageId>,
    gpu_section: Differential<GpuSection>,
    section: Section<LogicalContext>,
    layer: Differential<Layer>,
    aspect_ratio: AspectRatio,
}
#[derive(Bundle)]
pub struct ImageFill {
    link: RenderLink,
    data: Differential<ImageFillInfo>,
}
impl Image {
    pub fn slot<I: Into<ImageId>, C: Into<Coordinates>>(id: I, c: C) -> ImageSlot {
        ImageSlot {
            link: RenderLink::new::<Image>(),
            extent: Differential::new(ImageSlotInfo(id.into(), c.into())),
        }
    }
    pub fn fill<I: Into<ImageId>, C: Into<Coordinates>>(id: I, data: Vec<u8>, extent: C) -> ImageFill {
        ImageFill {
            link: RenderLink::new::<Image>(),
            data: Differential::new(ImageFillInfo(id.into(), data, extent.into())),
        }
    }
    pub fn instance<
        I: Into<ImageId>,
        S: Into<Section<LogicalContext>>,
        L: Into<Layer>,
    >(
        id: I,
        s: S,
        l: L,
    ) -> Self {
        let s = s.into();
        Self {
            link: RenderLink::new::<Image>(),
            id: Differential::new(id.into()),
            gpu_section: Differential::new(GpuSection::default()),
            section: s,
            layer: Differential::new(l.into()),
            aspect_ratio: AspectRatio::from_coordinates(s.area.coordinates),
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
        (With<ImageId>, Or<(Changed<AspectRatio>, Changed<Area<LogicalContext>>)>),
    >,
) {
    for (ratio, mut area) in images.iter_mut() {
        area.coordinates = ratio.constrain(area.coordinates);
    }
}
#[derive(Component, Clone, PartialEq)]
pub struct ImageFillInfo(pub ImageId, pub Vec<u8>, pub Coordinates);
#[derive(Component, Clone, PartialEq)]
pub struct ImageSlotInfo(pub ImageId, pub Coordinates);
#[derive(Bundle)]
pub struct ImageSlot {
    link: RenderLink,
    extent: Differential<ImageSlotInfo>,
}
impl Leaf for Image {
    fn attach(elm: &mut Elm) {
        elm.enable_differential::<Self, ImageId>();
        elm.enable_differential::<Self, GpuSection>();
        elm.enable_differential::<Self, Layer>();
        elm.enable_differential::<Self, ImageFillInfo>();
        elm.enable_differential::<Self, ImageSlotInfo>();
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
pub struct ImageId(pub i32);
pub struct ImageResources {
    pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group: BindGroup,
    group_layout: BindGroupLayout,
    groups: HashMap<ImageId, ImageGroup>,
    entity_to_image: HashMap<Entity, ImageId>,
}
pub(crate) struct ImageGroup {
    view: TextureView,
    bind_group: BindGroup,
    instances: Instances<Entity>,
    extent: Coordinates,
    texture_coordinates: TextureCoordinates,
}
impl Render for Image {
    type DirectiveGroupKey = ImageId;
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
        for packet in queue_handle.read_removes::<Self>() {
            let id = renderer.resource_handle.entity_to_image.remove(&packet);
            if let Some(id) = id {
                renderer.resource_handle.groups.get_mut(&id).unwrap().instances.remove(packet);
            }
        }
        for packet in queue_handle.read_adds::<Self, ImageSlotInfo>() {
            // create group with packet.id
            // save texture extent with packet.id
        }
        for packet in queue_handle.read_adds::<Self, ImageFillInfo>() {
            // fill group @ packet.id
            // set tx-coordinates for packet.id
        }
        for packet in queue_handle.read_adds::<Self, ImageId>() {
            // add instance of packet.entity to group @ packet.value
        }
        // ... other attributes
        for packet in queue_handle.read_adds::<Self, GpuSection>() {
            // checked write
        }
        for packet in queue_handle.read_adds::<Self, Layer>() {
            // checked write
        }
        true
    }

    fn record(renderer: &mut Renderer<Self>, ginkgo: &Ginkgo) {
        todo!()
    }
}
