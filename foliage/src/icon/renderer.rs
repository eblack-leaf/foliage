use crate::ash::instruction::{
    RenderInstructionHandle, RenderInstructionsRecorder, RenderRecordBehavior,
};
use crate::ash::render::{Render, RenderPhase};
use crate::ash::render_packet::{RenderPacket, RenderPacketStore};
use crate::ash::renderer::RenderPackage;
use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::CReprPosition;
use crate::coordinate::section::Section;
use crate::coordinate::NumericalContext;
use crate::ginkgo::Ginkgo;
use crate::icon::vertex::{Vertex, VERTICES};
use crate::icon::{Icon, IconId, IconScale, RequestData, WasRequest};
use crate::instance::{InstanceCoordinator, InstanceCoordinatorBuilder};
use crate::texture::coord::TexturePartition;
use bevy_ecs::entity::Entity;
use std::collections::HashMap;
use std::iter::StepBy;
use std::ops::RangeInclusive;
use wgpu::{BindGroup, BindGroupLayout};

pub struct IconRenderResources {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    #[allow(unused)]
    icon_bind_group_layout: wgpu::BindGroupLayout,
    icon_textures: HashMap<IconId, (InstanceCoordinator<Entity>, wgpu::BindGroup)>,
    entity_to_icon: HashMap<Entity, IconId>,
    scale_to_partition: HashMap<u32, TexturePartition>,
    icon_queue: HashMap<IconId, HashMap<Entity, RenderPacket>>,
}
impl IconRenderResources {
    pub(crate) fn queue_icon(
        &mut self,
        id: IconId,
        entity: Entity,
        mut render_packet: RenderPacket,
    ) {
        if self.icon_queue.get(&id).is_none() {
            self.icon_queue.insert(id, HashMap::new());
        }
        if let Some(packet) = self.icon_queue.get_mut(&id).unwrap().remove(&entity) {
            let mut store = RenderPacketStore::new();
            store.put(
                render_packet.get::<CReprPosition>().unwrap_or(
                    packet
                        .get::<CReprPosition>()
                        .unwrap_or(CReprPosition::default()),
                ),
            );
            store.put(
                render_packet
                    .get::<CReprArea>()
                    .unwrap_or(packet.get::<CReprArea>().unwrap_or(CReprArea::default())),
            );
            store.put(
                render_packet
                    .get::<Color>()
                    .unwrap_or(packet.get::<Color>().unwrap_or(Color::default())),
            );
            store.put(
                render_packet
                    .get::<Layer>()
                    .unwrap_or(packet.get::<Layer>().unwrap_or(Layer::default())),
            );
            render_packet = store.retrieve();
        }
        self.icon_queue
            .get_mut(&id)
            .unwrap()
            .insert(entity, render_packet);
    }
}
impl Icon {
    pub(crate) const TEXTURE_DIMENSIONS: u32 = 320;

    fn create_icon_resource(
        ginkgo: &Ginkgo,
        icon_bind_group_layout: &BindGroupLayout,
        icon_textures: &mut HashMap<IconId, (InstanceCoordinator<Entity>, BindGroup)>,
        index: usize,
        file: &[u8],
    ) {
        let texture_data = rmp_serde::from_slice::<Vec<u8>>(file).ok().unwrap();
        let (_texture, view) = ginkgo.texture_r8unorm_d2(
            Icon::TEXTURE_DIMENSIONS,
            Icon::TEXTURE_DIMENSIONS,
            1,
            texture_data.as_slice(),
        );
        icon_textures.insert(
            IconId::new(index as u32),
            (
                InstanceCoordinatorBuilder::new(4)
                    .with_attribute::<CReprPosition>()
                    .with_attribute::<CReprArea>()
                    .with_attribute::<Layer>()
                    .with_attribute::<Color>()
                    .with_attribute::<TexturePartition>()
                    .build(ginkgo),
                ginkgo
                    .device()
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("icon-bind-group"),
                        layout: icon_bind_group_layout,
                        entries: &[Ginkgo::texture_bind_group_entry(&view, 0)],
                    }),
            ),
        );
    }

    fn add_entity(
        resources: &mut IconRenderResources,
        entity: Entity,
        render_packet: RenderPacket,
        new: &IconId,
    ) {
        resources
            .icon_textures
            .get_mut(new)
            .unwrap()
            .0
            .queue_add(entity);
        let scale = render_packet.get::<CReprArea>().unwrap();
        let texture_partition = resources
            .scale_to_partition
            .get(&(IconScale::from_dim(scale.width).px() as u32))
            .cloned()
            .unwrap();
        resources
            .icon_textures
            .get_mut(new)
            .unwrap()
            .0
            .queue_write(entity, texture_partition);
        resources
            .icon_textures
            .get_mut(new)
            .unwrap()
            .0
            .queue_render_packet(entity, render_packet);
    }
}
pub(crate) fn placements() -> Vec<(u32, Section<NumericalContext>)> {
    let rects = (IconScale::LOWER_BOUND..=IconScale::UPPER_BOUND)
        .step_by(IconScale::INTERVAL as usize)
        .map(|x| binpack2d::Dimension::with_id(x as isize, x as i32, x as i32, 1))
        .collect::<Vec<binpack2d::Dimension>>();
    let mut bin = binpack2d::bin_new(
        binpack2d::BinType::MaxRects,
        Icon::TEXTURE_DIMENSIONS as i32,
        Icon::TEXTURE_DIMENSIONS as i32,
    );
    let (mut inserted, rejected) = bin.insert_list(&rects);
    if !rejected.is_empty() {
        panic!("could not fit all {:?}", rejected)
    }
    let mut r_val = inserted
        .drain(..)
        .map(|i| {
            (
                i.id() as u32,
                Section::new((i.x(), i.y()), (i.width(), i.height())),
            )
        })
        .collect::<Vec<(u32, Section<NumericalContext>)>>();
    r_val.sort_by(|lhs, rhs| lhs.0.partial_cmp(&rhs.0).unwrap());
    r_val
}
#[test]
fn smallest_size() {
    let placements = placements();
    for place in placements {
        println!("id: {:?}, rect: {:?}", place.0, place.1);
    }
}
#[allow(unused)]
pub(crate) fn icon_scale_range() -> StepBy<RangeInclusive<u32>> {
    (IconScale::LOWER_BOUND..=IconScale::UPPER_BOUND).step_by(IconScale::INTERVAL as usize)
}
impl Render for Icon {
    type Resources = IconRenderResources;
    type RenderPackage = bool;

    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(5);

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo
            .device()
            .create_shader_module(wgpu::include_wgsl!("icon.wgsl"));
        let icon_bind_group_layout =
            ginkgo
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("icon-render-pipeline-bind-group-layout"),
                    entries: &[Ginkgo::texture_d2_bind_group_entry(0)],
                });
        let icon_textures = HashMap::new();
        let placements = placements();
        let mut scale_to_partition = HashMap::new();
        for (scale, place) in placements {
            scale_to_partition.insert(
                scale,
                TexturePartition::new(
                    place,
                    Area::from((Icon::TEXTURE_DIMENSIONS, Icon::TEXTURE_DIMENSIONS)),
                ),
            );
        }
        // for (index, file) in ICON_RESOURCE_FILES.iter().enumerate() {
        //     Self::create_icon_resource(
        //         ginkgo,
        //         &icon_bind_group_layout,
        //         &mut icon_textures,
        //         index,
        //         file,
        //     );
        // }
        let sampler = ginkgo
            .device()
            .create_sampler(&wgpu::SamplerDescriptor::default());
        let bind_group_layout =
            ginkgo
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Icon-render-pipeline-bind-group-layout"),
                    entries: &[
                        Ginkgo::vertex_uniform_bind_group_layout_entry(0),
                        Ginkgo::sampler_bind_group_layout_entry(1),
                    ],
                });
        let bind_group = ginkgo
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("icon-render-pipeline-bind-group"),
                layout: &bind_group_layout,
                entries: &[
                    ginkgo.viewport_bind_group_entry(0),
                    Ginkgo::sampler_bind_group_entry(&sampler, 1),
                ],
            });
        let pipeline_layout =
            ginkgo
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("icon-render-pipeline-layout"),
                    bind_group_layouts: &[&bind_group_layout, &icon_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let pipeline = ginkgo
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("icon-render-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
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
                            array_stride: Ginkgo::buffer_address::<Color>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![5 => Float32x4],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<TexturePartition>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![6 => Float32x4],
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
        let vertex_buffer = ginkgo.vertex_buffer_with_data(&VERTICES, "icon-vertex-buffer");
        IconRenderResources {
            pipeline,
            vertex_buffer,
            bind_group,
            icon_bind_group_layout,
            icon_textures,
            entity_to_icon: HashMap::new(),
            scale_to_partition,
            icon_queue: HashMap::default(),
        }
    }

    fn create_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        render_packet: RenderPacket,
    ) -> Self::RenderPackage {
        let new = render_packet.get::<IconId>().unwrap();
        if render_packet.get::<WasRequest>().unwrap().0 {
            let data = render_packet
                .get::<RequestData>()
                .unwrap()
                .0
                .take()
                .unwrap();
            Self::create_icon_resource(
                ginkgo,
                &resources.icon_bind_group_layout,
                &mut resources.icon_textures,
                new.0 as usize,
                data.as_slice(),
            );
            return true;
        }
        resources.entity_to_icon.insert(entity, new);
        if resources.icon_textures.get(&new).is_none() {
            resources.queue_icon(new, entity, render_packet);
        } else {
            Self::add_entity(resources, entity, render_packet, &new);
        }
        false
    }

    fn on_package_removal(
        _ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        package: RenderPackage<Self>,
    ) {
        if !package.package_data {
            if let Some(icon_id) = resources.entity_to_icon.get(&entity) {
                if let Some(c) = resources.icon_textures.get_mut(icon_id) {
                    c.0.queue_remove(entity);
                }
            }
        }
    }
    #[allow(unused)]
    fn prepare_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        package: &mut RenderPackage<Self>,
        render_packet: RenderPacket,
    ) {
        if package.package_data {
            return;
        }
        let mut icon_id = *resources.entity_to_icon.get(&entity).unwrap();
        if let Some(id) = render_packet.get::<IconId>() {
            if resources.icon_textures.get(&id).is_none() {
                resources.queue_icon(id, entity, render_packet);
                return;
            }
            if icon_id != id {
                resources
                    .icon_textures
                    .get_mut(&icon_id)
                    .unwrap()
                    .0
                    .queue_remove(entity);
                icon_id = id;
                resources
                    .icon_textures
                    .get_mut(&icon_id)
                    .unwrap()
                    .0
                    .queue_add(entity);
                resources.entity_to_icon.insert(entity, id);
            }
        } else if resources.icon_textures.get(&icon_id).is_none() {
            resources.queue_icon(icon_id, entity, render_packet);
            return;
        }
        if let Some(scale) = render_packet.get::<CReprArea>() {
            let texture_partition = resources
                .scale_to_partition
                .get(&(IconScale::from_dim(scale.width).px() as u32))
                .cloned()
                .unwrap();
            resources
                .icon_textures
                .get_mut(&icon_id)
                .unwrap()
                .0
                .queue_write(entity, texture_partition);
        }
        resources
            .icon_textures
            .get_mut(&icon_id)
            .unwrap()
            .0
            .queue_render_packet(entity, render_packet);
    }

    fn prepare_resources(
        resources: &mut Self::Resources,
        ginkgo: &Ginkgo,
        per_renderer_record_hook: &mut bool,
    ) {
        let mut should_record = true;
        let mut confirmed = vec![];
        for (id, mapping) in resources.icon_queue.iter_mut() {
            if resources.icon_textures.get(id).is_some() {
                for (entity, packet) in mapping.drain() {
                    confirmed.push((*id, entity, packet));
                }
            }
        }
        for (id, entity, packet) in confirmed {
            resources.icon_queue.remove(&id);
            Icon::add_entity(resources, entity, packet, &id);
        }
        for (_id, (coordinator, _)) in resources.icon_textures.iter_mut() {
            if coordinator.prepare(ginkgo) {
                should_record = true;
            }
        }
        if should_record {
            *per_renderer_record_hook = true;
        }
    }

    fn record_behavior() -> RenderRecordBehavior<Self> {
        RenderRecordBehavior::PerRenderer(Box::new(Icon::record))
    }
}

impl Icon {
    fn record<'a>(
        resources: &'a IconRenderResources,
        mut recorder: RenderInstructionsRecorder<'a>,
    ) -> Option<RenderInstructionHandle> {
        for (_, (instance_coordinator, bind_group)) in resources.icon_textures.iter() {
            if instance_coordinator.has_instances() {
                recorder.0.set_pipeline(&resources.pipeline);
                recorder.0.set_bind_group(0, &resources.bind_group, &[]);
                recorder.0.set_bind_group(1, bind_group, &[]);
                recorder
                    .0
                    .set_vertex_buffer(0, resources.vertex_buffer.slice(..));
                recorder
                    .0
                    .set_vertex_buffer(1, instance_coordinator.buffer::<CReprPosition>().slice(..));
                recorder
                    .0
                    .set_vertex_buffer(2, instance_coordinator.buffer::<CReprArea>().slice(..));
                recorder
                    .0
                    .set_vertex_buffer(3, instance_coordinator.buffer::<Layer>().slice(..));
                recorder
                    .0
                    .set_vertex_buffer(4, instance_coordinator.buffer::<Color>().slice(..));
                recorder.0.set_vertex_buffer(
                    5,
                    instance_coordinator.buffer::<TexturePartition>().slice(..),
                );
                recorder.0.draw(
                    0..VERTICES.len() as u32,
                    0..instance_coordinator.instances(),
                );
            }
        }
        Some(recorder.finish())
    }
}
