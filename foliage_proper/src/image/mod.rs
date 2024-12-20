use std::collections::HashMap;

use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, IntoSystemConfigs};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::{Query, ResMut};
use bytemuck::{Pod, Zeroable};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, PipelineLayoutDescriptor, RenderPass,
    RenderPipeline, RenderPipelineDescriptor, ShaderStages, Texture, TextureAspect, TextureFormat,
    TextureSampleType, TextureView, TextureViewDimension, VertexState, VertexStepMode,
};

use crate::ash::{ClippingContext, DrawRange, Render, Renderer};
use crate::color::Color;
use crate::coordinate::elevation::RenderLayer;
use crate::coordinate::section::{GpuSection, Section};
use crate::coordinate::{Coordinates, DeviceContext, LogicalContext, NumericalContext};
use crate::differential::{Differential, RenderAddQueue, RenderLink};
use crate::elm::{Elm, InternalStage, RenderQueueHandle};
use crate::ginkgo::Ginkgo;
use crate::instances::Instances;
use crate::leaf::{Remove, Visibility};
use crate::texture::TextureCoordinates;
use crate::Root;

#[derive(Copy, Clone, Component, PartialEq)]
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
    pub fn reciprocal(&self) -> f32 {
        1f32 / self.0
    }
    pub fn constrain(&self, target: Coordinates) -> Coordinates {
        let mut attempted_width = target.horizontal();
        let mut attempted_height = target.horizontal() * self.reciprocal();
        while attempted_height > target.vertical() {
            attempted_width -= 1f32;
            attempted_height = attempted_width * self.reciprocal();
        }
        Coordinates::new(attempted_width, attempted_height)
    }
    pub fn grow_to_cover(&self, target: Coordinates) -> Coordinates {
        let mut attempted_width = target.horizontal();
        let mut attempted_height = attempted_width * self.reciprocal();
        while attempted_height < target.vertical() {
            attempted_width += 1f32;
            attempted_height = attempted_width * self.reciprocal();
        }
        Coordinates::new(attempted_width, attempted_height)
    }
}
impl From<f32> for AspectRatio {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}
#[derive(Bundle, Clone)]
pub struct Image {
    link: RenderLink,
    fill: Differential<ImageFill>,
    f: ImageFill,
    view: Differential<ImageView>,
    iv: ImageView,
    color: Differential<Color>,
    c: Color,
    gpu_section: Differential<GpuSection>,
    layer: Differential<RenderLayer>,
}
type ImageBitsRepr = u8;
impl Image {
    const PRECISION: usize = 1;
    const FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
    pub fn memory<I: Into<ImageSlotId>, C: Into<Coordinates>>(id: I, c: C) -> ImageSlot {
        ImageSlot {
            link: RenderLink::new::<Image>(),
            extent: Differential::new(),
            isd: ImageSlotDescriptor(id.into(), c.into()),
            remove: Default::default(),
            visibility: Default::default(),
        }
    }
    pub fn new<I: Into<ImageSlotId>>(id: I, data: Vec<u8>) -> Self {
        let image = image::load_from_memory(data.as_slice())
            .unwrap()
            .into_rgba8();
        let dimensions = Coordinates::new(image.width() as f32, image.height() as f32);
        let image_bytes = image.as_raw().clone();
        let id = id.into();
        Self {
            link: RenderLink::new::<Image>(),
            fill: Differential::new(),
            f: ImageFill(id, image_bytes, dimensions),
            view: Differential::new(),
            iv: ImageView::Stretch,
            color: Differential::new(),
            c: Color::WHITE,
            gpu_section: Differential::new(),
            layer: Differential::new(),
        }
    }
    pub fn with_aspect_ratio<A: Into<AspectRatio>>(mut self, a: A) -> Self {
        self.iv = ImageView::Aspect(a.into());
        self
    }
    pub fn inherit_aspect_ratio(mut self) -> Self {
        self.iv = ImageView::Aspect(AspectRatio::from_coordinates(self.f.2));
        self
    }
    pub fn crop(mut self) -> Self {
        self.iv = ImageView::Crop(Section::default());
        self
    }
}
#[derive(Copy, Clone, Component, PartialEq)]
pub enum ImageView {
    Aspect(AspectRatio),
    Stretch,
    Crop(Section<NumericalContext>),
}
fn constrain(
    mut images: Query<
        (&ImageFill, &mut ImageView, &mut Section<LogicalContext>),
        Or<(Changed<ImageView>, Changed<Section<LogicalContext>>)>,
    >,
) {
    for (fill, mut view, mut section) in images.iter_mut() {
        let old = *section;
        match *view.as_ref() {
            ImageView::Aspect(a) => {
                // keep the biggest aspect ratio can fit inside bounds
                let new_area = a.constrain(old.area.coordinates);
                let diff = (old.area.coordinates - new_area) / 2f32;
                let new = Section::logical(old.position.coordinates + diff, new_area);
                section.area.coordinates = new.area.coordinates;
                section.position.coordinates = new.position.coordinates;
            }
            ImageView::Crop(_) => {
                let aspect = AspectRatio::from_coordinates(fill.2);
                let adjusted_extent = aspect.grow_to_cover(old.area.coordinates);
                let fill_section = Section::logical(old.position, adjusted_extent);
                let center_diff = old.center() - fill_section.center();
                let adjusted_fill_section =
                    Section::logical(fill_section.position + center_diff, fill_section.area);
                let tex_coords_adjustments = Section::numerical(
                    (
                        (old.left() - adjusted_fill_section.left()) / adjusted_fill_section.width(),
                        (old.top() - adjusted_fill_section.top()) / adjusted_fill_section.height(),
                    ),
                    (
                        (adjusted_fill_section.right() - old.right())
                            / adjusted_fill_section.width(),
                        (adjusted_fill_section.bottom() - old.bottom())
                            / adjusted_fill_section.height(),
                    ),
                );
                *view = ImageView::Crop(tex_coords_adjustments);
            }
            _ => {}
        };
    }
}
#[derive(Component, Clone, PartialEq)]
pub struct ImageFill(pub ImageSlotId, pub Vec<ImageBitsRepr>, pub Coordinates);
#[derive(Component, Clone, PartialEq)]
pub struct ImageSlotDescriptor(pub ImageSlotId, pub Coordinates);
#[derive(Bundle)]
pub struct ImageSlot {
    link: RenderLink,
    extent: Differential<ImageSlotDescriptor>,
    isd: ImageSlotDescriptor,
    remove: Remove,
    visibility: Visibility,
}
fn image_fill_differential(
    mut fills: Query<(Entity, &mut ImageFill), Changed<ImageFill>>,
    mut render_queue: ResMut<RenderAddQueue<ImageFill>>,
) {
    for (entity, mut fill) in fills.iter_mut() {
        if fill.1.is_empty() {
            continue;
        }
        render_queue
            .queue
            .get_mut(&RenderLink::new::<Image>())
            .unwrap()
            .insert(
                entity,
                ImageFill {
                    0: fill.0,
                    1: fill.1.drain(..).collect(),
                    2: fill.2,
                },
            );
    }
}
impl Root for Image {
    fn attach(elm: &mut Elm) {
        elm.enable_differential::<Self, GpuSection>();
        elm.enable_differential::<Self, RenderLayer>();
        elm.enable_differential::<Self, ImageSlotDescriptor>();
        elm.enable_differential::<Self, ImageView>();
        elm.enable_differential::<Self, Color>();
        let mut queue = RenderAddQueue::<ImageFill>::default();
        queue
            .queue
            .insert(RenderLink::new::<Image>(), HashMap::new());
        elm.ecs.insert_resource(queue);
        elm.scheduler.main.add_systems((
            constrain.in_set(InternalStage::Resolve),
            image_fill_differential.in_set(InternalStage::Differential),
        ));
    }
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
#[derive(Copy, Clone, Component, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct ImageSlotId(pub i32);
impl From<i32> for ImageSlotId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
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
            bind_group_layouts: &[&group_layout, &bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = ginkgo.create_pipeline(&RenderPipelineDescriptor {
            label: Some("image-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Option::from("vertex_entry"),
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
                    Ginkgo::vertex_buffer_layout::<RenderLayer>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![3 => Float32],
                    ),
                    Ginkgo::vertex_buffer_layout::<TextureCoordinates>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![4 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<Color>(
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
            cache: None,
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
    ) {
        for entity in queue_handle.read_removes::<Self>() {
            let id = renderer.resource_handle.entity_to_image.remove(&entity);
            if let Some(id) = id {
                renderer
                    .resource_handle
                    .groups
                    .get_mut(&id)
                    .unwrap()
                    .instances
                    .queue_remove(entity);
            }
        }
        for packet in queue_handle.read_adds::<Self, ImageSlotDescriptor>() {
            renderer.associate_directive_group(packet.value.0 .0, packet.value.0);
            let (tex, view) = ginkgo.create_texture(
                Self::FORMAT,
                packet.value.1,
                1,
                bytemuck::cast_slice(&vec![
                    0f32;
                    Self::PRECISION
                        * packet.value.1.horizontal() as usize
                        * packet.value.1.vertical() as usize
                ]),
            );
            renderer.resource_handle.groups.insert(
                packet.value.0,
                ImageGroup {
                    bind_group: ginkgo.create_bind_group(&BindGroupDescriptor {
                        label: Some("image-group-bind-group"),
                        layout: &renderer.resource_handle.group_layout,
                        entries: &[Ginkgo::texture_bind_group_entry(&view, 0)],
                    }),
                    instances: Instances::new(1)
                        .with_attribute::<GpuSection>(ginkgo)
                        .with_attribute::<RenderLayer>(ginkgo)
                        .with_attribute::<TextureCoordinates>(ginkgo)
                        .with_attribute::<Color>(ginkgo),
                    slot_extent: packet.value.1,
                    image_extent: Default::default(),
                    texture_coordinates: Default::default(),
                    should_record: false,
                    tex,
                    view,
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
                        packet.value.2.horizontal() as u32
                            * size_of::<f32>() as u32
                            * Self::PRECISION as u32,
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
                .clear(Some(packet.entity));
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
        for packet in queue_handle.read_adds::<Self, ClippingContext>() {
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
                .set_clipping_context(packet.entity, packet.value);
        }
        for packet in queue_handle.read_adds::<Self, ImageView>() {
            let id = *renderer
                .resource_handle
                .entity_to_image
                .get(&packet.entity)
                .unwrap();
            let tex_coords = renderer
                .resource_handle
                .groups
                .get(&id)
                .unwrap()
                .texture_coordinates;
            let new = match packet.value {
                ImageView::Aspect(_) => tex_coords,
                ImageView::Stretch => tex_coords,
                ImageView::Crop(adjustments) => TextureCoordinates::new(
                    (
                        tex_coords.top_left.horizontal()
                            + tex_coords.bottom_right.horizontal() * adjustments.left(),
                        tex_coords.top_left.vertical()
                            + tex_coords.bottom_right.vertical() * adjustments.top(),
                    ),
                    (
                        tex_coords.bottom_right.horizontal()
                            - tex_coords.bottom_right.horizontal() * adjustments.width(),
                        tex_coords.bottom_right.vertical()
                            - tex_coords.bottom_right.vertical() * adjustments.height(),
                    ),
                ),
            };
            renderer
                .resource_handle
                .groups
                .get_mut(&id)
                .unwrap()
                .instances
                .checked_write(packet.entity, new);
        }
        for packet in queue_handle.read_adds::<Self, RenderLayer>() {
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
                .set_layer(packet.entity, packet.value);
            renderer
                .resource_handle
                .groups
                .get_mut(&id)
                .unwrap()
                .instances
                .checked_write(packet.entity, packet.value);
        }
        for packet in queue_handle.read_adds::<Self, Color>() {
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
        for (slot_id, group) in renderer.resource_handle.groups.iter_mut() {
            if group.instances.resolve_changes(ginkgo) {
                renderer
                    .node_manager
                    .set_nodes(slot_id.0, group.instances.render_nodes());
            }
        }
    }

    fn draw<'a>(
        renderer: &'a Renderer<Self>,
        group_key: Self::DirectiveGroupKey,
        alpha_range: DrawRange,
        clipping_section: Section<DeviceContext>,
        render_pass: &mut RenderPass<'a>,
    ) {
        let group = renderer.resource_handle.groups.get(&group_key).unwrap();
        render_pass.set_scissor_rect(
            clipping_section.left() as u32,
            clipping_section.top() as u32,
            clipping_section.width() as u32,
            clipping_section.height() as u32,
        );
        render_pass.set_pipeline(&renderer.resource_handle.pipeline);
        render_pass.set_bind_group(0, &group.bind_group, &[]);
        render_pass.set_bind_group(1, &renderer.resource_handle.bind_group, &[]);
        render_pass.set_vertex_buffer(0, renderer.resource_handle.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, group.instances.buffer::<GpuSection>().slice(..));
        render_pass.set_vertex_buffer(2, group.instances.buffer::<RenderLayer>().slice(..));
        render_pass.set_vertex_buffer(3, group.instances.buffer::<TextureCoordinates>().slice(..));
        render_pass.set_vertex_buffer(4, group.instances.buffer::<Color>().slice(..));
        render_pass.draw(0..VERTICES.len() as u32, alpha_range.start..alpha_range.end);
    }
}
