use std::collections::{HashMap, HashSet};

use bevy_ecs::entity::Entity;
use wgpu::{BindGroup, BindGroupDescriptor, VertexState};

use crate::ash::instruction::{
    RenderInstructionHandle, RenderInstructionsRecorder, RenderRecordBehavior,
};
use crate::ash::render::{Render, RenderPhase};
use crate::ash::render_packet::RenderPacket;
use crate::ash::renderer::RenderPackage;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::CReprPosition;
use crate::coordinate::section::Section;
use crate::coordinate::NumericalContext;
use crate::ginkgo::Ginkgo;
use crate::image::vertex::{Vertex, VERTICES};
use crate::image::{Image, ImageData, ImageId, ImageStorage};
use crate::instance::{InstanceCoordinator, InstanceCoordinatorBuilder};
use crate::texture::coord::TexturePartition;

struct ImageGroup {
    coordinator: InstanceCoordinator<Entity>,
    tex: Option<(wgpu::Texture, wgpu::TextureView)>,
    bind_group: Option<BindGroup>,
    dimensions: Area<NumericalContext>,
    storage: Area<NumericalContext>,
    data: Vec<u8>,
    data_queued: bool,
}
impl ImageGroup {
    fn new(ginkgo: &Ginkgo) -> Self {
        Self {
            coordinator: InstanceCoordinatorBuilder::new(1)
                .with_attribute::<CReprPosition>()
                .with_attribute::<CReprArea>()
                .with_attribute::<Layer>()
                .with_attribute::<TexturePartition>()
                .build(ginkgo),
            tex: None,
            bind_group: None,
            dimensions: Default::default(),
            storage: Area::default(),
            data: vec![],
            data_queued: false,
        }
    }
    fn queue_data(&mut self, data: Vec<u8>) {
        self.data = data;
        self.data_queued = true;
    }
    fn write_data(&mut self, ginkgo: &Ginkgo) -> TexturePartition {
        self.data_queued = false;
        if self.data.is_empty() {
            return TexturePartition::default();
        }
        let slice = self.data.as_slice();
        let image = image::load_from_memory(slice).unwrap().to_rgba32f();
        self.dimensions = (image.width(), image.height()).into();
        let image_bytes = image
            .pixels()
            .flat_map(|p| p.0.to_vec())
            .collect::<Vec<f32>>();
        ginkgo.queue().write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.tex.as_ref().unwrap().0,
                mip_level: 0,
                origin: wgpu::Origin3d::default(),
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(image_bytes.as_slice()),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Option::from(
                    self.dimensions.width as u32 * std::mem::size_of::<f32>() as u32 * 4,
                ),
                rows_per_image: Option::from(self.dimensions.height as u32),
            },
            wgpu::Extent3d {
                width: self.dimensions.width as u32,
                height: self.dimensions.height as u32,
                depth_or_array_layers: 1,
            },
        );
        TexturePartition::new(Section::default().with_area(self.dimensions), self.storage)
    }
    fn fill(
        &mut self,
        ginkgo: &Ginkgo,
        layout: &wgpu::BindGroupLayout,
        storage: Area<NumericalContext>,
    ) {
        self.storage = storage;
        let tex = ginkgo.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("image-tex"),
            size: wgpu::Extent3d {
                width: self.storage.width as u32,
                height: self.storage.height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba32Float],
        });
        let tex_view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        self.tex.replace((tex, tex_view));
        self.bind_group
            .replace(ginkgo.device().create_bind_group(&BindGroupDescriptor {
                label: Some("image-group-bind-group"),
                layout,
                entries: &[Ginkgo::texture_bind_group_entry(
                    &self.tex.as_ref().unwrap().1,
                    0,
                )],
            }));
        // self.write_data(ginkgo)
    }
}
pub struct ImageRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: BindGroup,
    package_layout: wgpu::BindGroupLayout,
    groups: HashMap<ImageId, ImageGroup>,
    vertex_buffer: wgpu::Buffer,
    full_coords: HashMap<ImageId, TexturePartition>,
    view_queue: HashSet<(ImageId, Entity)>,
    write_needed: HashSet<ImageId>,
}
pub struct ImageRenderPackage {
    last: ImageId,
    was_request: bool,
}
impl Render for Image {
    type Resources = ImageRenderResources;
    type RenderPackage = ImageRenderPackage;
    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(2);

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let resource_layout =
            ginkgo
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("image-layout"),
                    entries: &[
                        Ginkgo::vertex_uniform_bind_group_layout_entry(0),
                        Ginkgo::sampler_bind_group_layout_entry(1),
                    ],
                });
        let package_layout =
            ginkgo
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("image-package-layout"),
                    entries: &[Ginkgo::texture_d2_bind_group_entry(0)],
                });
        let shader = ginkgo
            .device()
            .create_shader_module(wgpu::include_wgsl!("image.wgsl"));
        let pipeline_layout =
            ginkgo
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("image-pipeline-layout"),
                    bind_group_layouts: &[&resource_layout, &package_layout],
                    push_constant_ranges: &[],
                });
        let sampler = ginkgo
            .device()
            .create_sampler(&wgpu::SamplerDescriptor::default());
        let bind_group = ginkgo.device().create_bind_group(&BindGroupDescriptor {
            label: Some("image-bind-group"),
            layout: &resource_layout,
            entries: &[
                ginkgo.viewport_bind_group_entry(0),
                Ginkgo::sampler_bind_group_entry(&sampler, 1),
            ],
        });
        let pipeline = ginkgo
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("image-render-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vertex_entry",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<Vertex>(1),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Uint32x2],
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
                            array_stride: Ginkgo::buffer_address::<TexturePartition>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![5 => Float32x4],
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
        let vertex_buffer = ginkgo.vertex_buffer_with_data(&VERTICES, "image-vertices");
        ImageRenderResources {
            pipeline,
            bind_group,
            package_layout,
            vertex_buffer,
            groups: HashMap::new(),
            full_coords: HashMap::new(),
            view_queue: HashSet::new(),
            write_needed: HashSet::new(),
        }
    }

    fn create_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        render_packet: RenderPacket,
    ) -> Self::RenderPackage {
        let image_id = render_packet.get::<ImageId>().unwrap();
        tracing::trace!("creating package for image:{:?}", image_id);
        if resources.groups.get(&image_id).is_none() {
            resources.groups.insert(image_id, ImageGroup::new(ginkgo));
        }
        let image_data = render_packet.get::<ImageData>().unwrap();
        let mut wr = false;
        if let Some(data) = image_data.0 {
            // queue fill
            if !data.is_empty() {
                tracing::trace!("image-id queued:{:?}", image_id);
                resources
                    .groups
                    .get_mut(&image_id)
                    .unwrap()
                    .queue_data(data);
                wr = true;
                resources.write_needed.insert(image_id);
            }
        }
        if let Some(storage) = render_packet.get::<ImageStorage>().unwrap().0 {
            tracing::trace!("fill image-id:{:?}", image_id);
            resources.groups.get_mut(&image_id).unwrap().fill(
                ginkgo,
                &resources.package_layout,
                storage,
            );
            wr = true;
        }
        if wr {
            tracing::trace!("view-queue image-id:{:?}", image_id);
            for instance in resources.groups.get(&image_id).unwrap().coordinator.keys() {
                resources.view_queue.insert((image_id, instance));
            }
            return ImageRenderPackage {
                last: image_id,
                was_request: true,
            };
        } else {
            tracing::trace!("create image instance for:{:?}", image_id);
            resources
                .groups
                .get_mut(&image_id)
                .unwrap()
                .coordinator
                .queue_add(entity);
            if let Some(view) = resources.full_coords.get(&image_id) {
                tracing::trace!("image-id coords available on-create:{:?}", image_id);
                resources
                    .groups
                    .get_mut(&image_id)
                    .unwrap()
                    .coordinator
                    .queue_write(entity, *view);
            } else {
                tracing::trace!("defer fill image-id:{:?}", image_id);
                resources.view_queue.insert((image_id, entity));
            }
            resources
                .groups
                .get_mut(&image_id)
                .unwrap()
                .coordinator
                .queue_render_packet(entity, render_packet);
        }
        ImageRenderPackage {
            last: image_id,
            was_request: false,
        }
    }

    fn on_package_removal(
        _ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        package: RenderPackage<Self>,
    ) {
        if !package.package_data.was_request {
            let id = &package.package_data.last;
            tracing::trace!("fill image-id:{:?}", id);
            resources
                .groups
                .get_mut(id)
                .unwrap()
                .coordinator
                .queue_remove(entity);
        }
    }

    fn prepare_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        package: &mut RenderPackage<Self>,
        render_packet: RenderPacket,
    ) {
        if !package.package_data.was_request {
            tracing::trace!("preparing image:{:?}", entity);
            if let Some(id) = render_packet.get::<ImageId>() {
                if id != package.package_data.last {
                    tracing::trace!("changed image-id:{:?}", id);
                    resources
                        .groups
                        .get_mut(&package.package_data.last)
                        .unwrap()
                        .coordinator
                        .queue_remove(entity);
                    if resources.groups.get(&id).is_none() {
                        tracing::trace!("on-fly creation of image-id:{:?}", id);
                        resources.groups.insert(id, ImageGroup::new(ginkgo));
                    }
                    resources
                        .groups
                        .get_mut(&id)
                        .unwrap()
                        .coordinator
                        .queue_add(entity);
                    tracing::trace!("deferring image-id:{:?}", id);
                    resources.view_queue.insert((id, entity));
                    package.package_data.last = id;
                    package.signal_record();
                }
            }
            resources
                .groups
                .get_mut(&package.package_data.last)
                .unwrap()
                .coordinator
                .queue_render_packet(entity, render_packet);
        }
    }

    fn prepare_resources(
        resources: &mut Self::Resources,
        ginkgo: &Ginkgo,
        _per_renderer_record_hook: &mut bool,
    ) {
        // iter groups and prepare coordinators
        for id in resources.write_needed.drain() {
            let partition = resources.groups.get_mut(&id).unwrap().write_data(ginkgo);
            tracing::trace!("write image-id:{:?} w/ partition:{:?}", id, partition);
            resources.full_coords.insert(id, partition);
        }
        for (id, queued) in resources.view_queue.drain() {
            if let Some(coords) = resources.full_coords.get_mut(&id).cloned() {
                tracing::trace!("coords adjusted for image-id:{:?}", id);
                resources
                    .groups
                    .get_mut(&id)
                    .unwrap()
                    .coordinator
                    .queue_write(queued, coords);
            }
        }
        for (_id, group) in resources.groups.iter_mut() {
            if group.coordinator.prepare(ginkgo) {
                tracing::trace!("setting render record hook");
                *_per_renderer_record_hook = true;
            }
        }
    }

    fn record_behavior() -> RenderRecordBehavior<Self> {
        RenderRecordBehavior::PerRenderer(Box::new(record))
    }
}

fn record<'a>(
    resources: &'a ImageRenderResources,
    mut recorder: RenderInstructionsRecorder<'a>,
) -> Option<RenderInstructionHandle> {
    recorder.0.set_pipeline(&resources.pipeline);
    recorder.0.set_bind_group(0, &resources.bind_group, &[]);
    recorder
        .0
        .set_vertex_buffer(0, resources.vertex_buffer.slice(..));
    for (id, group) in resources.groups.iter() {
        if group.coordinator.has_instances() {
            tracing::trace!("recording images for:{:?}", id);
            recorder
                .0
                .set_bind_group(1, group.bind_group.as_ref().unwrap(), &[]);
            recorder
                .0
                .set_vertex_buffer(1, group.coordinator.buffer::<CReprPosition>().slice(..));
            recorder
                .0
                .set_vertex_buffer(2, group.coordinator.buffer::<CReprArea>().slice(..));
            recorder
                .0
                .set_vertex_buffer(3, group.coordinator.buffer::<Layer>().slice(..));
            recorder
                .0
                .set_vertex_buffer(4, group.coordinator.buffer::<TexturePartition>().slice(..));
            recorder
                .0
                .draw(0..VERTICES.len() as u32, 0..group.coordinator.instances());
        }
    }
    tracing::trace!("finish-recording");
    Some(recorder.finish())
}
