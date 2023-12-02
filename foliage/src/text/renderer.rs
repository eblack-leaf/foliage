use crate::ash::instruction::{
    RenderInstructionHandle, RenderInstructionsRecorder, RenderRecordBehavior,
};
use crate::ash::render::{Render, RenderPhase};
use crate::ash::render_packet::RenderPacket;
use crate::ash::renderer::RenderPackage;
use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::CReprPosition;
use crate::coordinate::{CoordinateUnit, NumericalContext};
use crate::ginkgo::uniform::AlignedUniform;
use crate::ginkgo::Ginkgo;
use crate::instance::{InstanceCoordinator, InstanceCoordinatorBuilder};
use crate::text::font::MonospacedFont;
use crate::text::glyph::{GlyphChangeQueue, GlyphKey, GlyphRemoveQueue};
use crate::text::vertex::{Vertex, VERTICES};
use crate::text::{FontSize, Text, TextValueUniqueCharacters};
use crate::texture::{AtlasBlock, TextureAtlas, TexturePartition};
use bevy_ecs::entity::Entity;
use std::collections::{HashMap, HashSet};

pub struct TextRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    font: MonospacedFont,
    package_layout: wgpu::BindGroupLayout,
}
pub(crate) type TextKey = usize;
pub struct TextRenderPackage {
    instance_coordinator: InstanceCoordinator<TextKey>,
    bind_group: wgpu::BindGroup,
    uniform: AlignedUniform<CoordinateUnit>, // pos + layer
    atlas: TextureAtlas<GlyphKey, u8>,
    rasterizations: HashMap<GlyphKey, (char, Area<NumericalContext>, Vec<u8>)>,
    rasterization_entries: HashMap<GlyphKey, HashSet<TextKey>>,
    font_size: FontSize,
    block: AtlasBlock,
}
impl Render for Text {
    type Resources = TextRenderResources;
    type RenderPackage = TextRenderPackage;
    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(3);

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        let shader = ginkgo
            .device()
            .create_shader_module(wgpu::include_wgsl!("text.wgsl"));
        let package_layout =
            ginkgo
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("text-package-layout"),
                    entries: &[
                        Ginkgo::vertex_uniform_bind_group_layout_entry(0),
                        Ginkgo::texture_d2_bind_group_entry(1),
                    ],
                });
        let resource_layout =
            ginkgo
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("text-resource-layout"),
                    entries: &[
                        Ginkgo::vertex_uniform_bind_group_layout_entry(0),
                        Ginkgo::sampler_bind_group_layout_entry(1),
                    ],
                });
        let sampler = ginkgo
            .device()
            .create_sampler(&wgpu::SamplerDescriptor::default());
        let bind_group = ginkgo
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("text-bind-group"),
                layout: &resource_layout,
                entries: &[
                    ginkgo.viewport_bind_group_entry(0),
                    Ginkgo::sampler_bind_group_entry(&sampler, 1),
                ],
            });
        let pipeline_layout =
            ginkgo
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("rectangle-render-pipeline-layout"),
                    bind_group_layouts: &[&resource_layout, &package_layout],
                    push_constant_ranges: &[],
                });
        let pipeline = ginkgo
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("text-pipeline"),
                layout: Option::from(&pipeline_layout),
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
                            array_stride: Ginkgo::buffer_address::<Color>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![4 => Float32x4],
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
        let vertex_buffer = ginkgo.vertex_buffer_with_data(&VERTICES, "text-vertex-buffer");
        TextRenderResources {
            pipeline,
            bind_group,
            vertex_buffer,
            font: MonospacedFont::new(Text::DEFAULT_OPT_SCALE),
            package_layout,
        }
    }

    fn create_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        _entity: Entity,
        render_packet: RenderPacket,
    ) -> Self::RenderPackage {
        let font_size = render_packet.get::<FontSize>().unwrap();
        let unique_characters = render_packet.get::<TextValueUniqueCharacters>().unwrap();
        let pos = render_packet.get::<CReprPosition>().unwrap();
        let layer = render_packet.get::<Layer>().unwrap();
        let uniform = AlignedUniform::new(ginkgo.device(), Some([pos.x, pos.y, layer.z, 0.0]));
        let mut atlas = TextureAtlas::new(
            ginkgo,
            AtlasBlock(
                resources
                    .font
                    .character_dimensions(font_size.px(ginkgo.scale_factor()))
                    .to_numerical(),
            ),
            unique_characters.0,
            wgpu::TextureFormat::R8Unorm,
        );
        let bind_group = ginkgo
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("text-package-bind-group"),
                layout: &resources.package_layout,
                entries: &[
                    uniform.bind_group_entry(0),
                    Ginkgo::texture_bind_group_entry(&atlas.view, 1),
                ],
            });
        let mut instance_coordinator = InstanceCoordinatorBuilder::new(unique_characters.0)
            .with_attribute::<CReprPosition>()
            .with_attribute::<CReprArea>()
            .with_attribute::<Color>()
            .with_attribute::<TexturePartition>()
            .build(ginkgo);
        let mut rasterizations = HashMap::new();
        let mut rasterization_entries = HashMap::new();
        for (key, glyph) in render_packet.get::<GlyphChangeQueue>().unwrap().0.drain(..) {
            let (glyph_key, _) = glyph.key.unwrap();
            if !atlas.has_key(&glyph_key) {
                rasterization_entries.insert(glyph_key, HashSet::new());
                let location = atlas.next_location().unwrap();
                let rasterization = resources.font.0.rasterize(
                    glyph.character.unwrap(),
                    font_size.px(ginkgo.scale_factor()),
                );
                rasterizations.insert(
                    glyph_key,
                    (
                        glyph.character.unwrap(),
                        (rasterization.0.width, rasterization.0.height).into(),
                        rasterization.1,
                    ),
                );
                let _partition = atlas.write_location(
                    glyph_key,
                    ginkgo,
                    location,
                    rasterizations.get(&glyph_key).unwrap().1,
                    &rasterizations.get(&glyph_key).unwrap().2,
                );
            }
            rasterization_entries
                .get_mut(&glyph_key)
                .unwrap()
                .insert(key);
            instance_coordinator.queue_add(key);
            instance_coordinator.queue_write(key, glyph.section.unwrap().position.to_c());
            instance_coordinator.queue_write(key, glyph.section.unwrap().area.to_c());
            instance_coordinator.queue_write(key, glyph.color.unwrap());
            instance_coordinator.queue_write(key, atlas.get(&glyph_key).unwrap());
        }
        instance_coordinator.prepare(ginkgo);
        let package = TextRenderPackage {
            instance_coordinator,
            bind_group,
            uniform,
            font_size,
            block: atlas.block,
            atlas,
            rasterizations,
            rasterization_entries,
        };
        package
    }

    fn on_package_removal(
        _ginkgo: &Ginkgo,
        _resources: &mut Self::Resources,
        _entity: Entity,
        _package: RenderPackage<Self>,
    ) {
        // do nothing?
    }

    fn prepare_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        _entity: Entity,
        package: &mut RenderPackage<Self>,
        render_packet: RenderPacket,
    ) {
        if let Some(pos) = render_packet.get::<CReprPosition>() {
            package.package_data.uniform.set_aspect(0, pos.x);
            package.package_data.uniform.set_aspect(1, pos.y);
        }
        if let Some(layer) = render_packet.get::<Layer>() {
            package.package_data.uniform.set_aspect(2, layer.z);
        }
        if package.package_data.uniform.needs_update() {
            package.package_data.uniform.update(ginkgo.queue());
        }
        if let Some(removes) = render_packet.get::<GlyphRemoveQueue>() {
            for change in removes.0 {
                // if references == 0 -> package.package_data.rasterizations.remove(&change.1);
                package
                    .package_data
                    .rasterization_entries
                    .get_mut(&change.1)
                    .unwrap()
                    .remove(&change.0);
                package
                    .package_data
                    .instance_coordinator
                    .queue_remove(change.0);
            }
        }
        let glyph_changes = render_packet.get::<GlyphChangeQueue>();
        let font_size_change = render_packet.get::<FontSize>();
        let new_glyph_key_count = {
            let mut count = 0;
            if let Some(changes) = glyph_changes.as_ref() {
                for (_, change) in changes.0.iter() {
                    if let Some(key) = change.key {
                        if !package.package_data.atlas.has_key(&key.0) {
                            count += 1;
                        }
                    }
                }
            }
            count
        };
        let needed_capacity =
            package.package_data.atlas.num_filled_locations() + new_glyph_key_count;
        let mut font_size_changed = false;
        if font_size_change.is_some() {
            let font_size = font_size_change.unwrap();
            if font_size != package.package_data.font_size {
                package.package_data.font_size = font_size;
                let block = AtlasBlock(
                    resources
                        .font
                        .character_dimensions(font_size.px(ginkgo.scale_factor()))
                        .to_numerical(),
                );
                package.package_data.block = block;
                font_size_changed = true;
            }
        }
        if package.package_data.atlas.capacity < needed_capacity || font_size_changed {
            package
                .package_data
                .atlas
                .grow(ginkgo, package.package_data.block, needed_capacity);
            package.package_data.bind_group =
                ginkgo
                    .device()
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("text-package-bind-group"),
                        layout: &resources.package_layout,
                        entries: &[
                            package.package_data.uniform.bind_group_entry(0),
                            Ginkgo::texture_bind_group_entry(&package.package_data.atlas.view, 1),
                        ],
                    });
            for (key, (ch, extent, data)) in package.package_data.rasterizations.iter_mut() {
                if font_size_changed {
                    let rasterization = resources.font.0.rasterize(
                        *ch,
                        package.package_data.font_size.px(ginkgo.scale_factor()),
                    );
                    *extent = (rasterization.0.width, rasterization.0.height).into();
                    *data = rasterization.1;
                }
                if let Some(location) = package.package_data.atlas.next_location() {
                    let partition = package
                        .package_data
                        .atlas
                        .write_location(*key, ginkgo, location, *extent, &data);
                    for key in package
                        .package_data
                        .rasterization_entries
                        .get(key)
                        .unwrap()
                        .iter()
                    {
                        package
                            .package_data
                            .instance_coordinator
                            .queue_write(*key, partition);
                    }
                }
            }
        }

        if let Some(changes) = glyph_changes {
            for (key, glyph) in changes.0 {
                if !package.package_data.instance_coordinator.has_key(&key) {
                    package.package_data.instance_coordinator.queue_add(key);
                }
                if let Some((new, old)) = glyph.key {
                    if let Some(old) = old {
                        if package.package_data.rasterizations.remove(&old).is_some() {
                            package
                                .package_data
                                .rasterization_entries
                                .get_mut(&old)
                                .unwrap()
                                .remove(&key);
                        }
                    }
                    if !package.package_data.atlas.has_key(&new) {
                        package
                            .package_data
                            .rasterization_entries
                            .insert(new, HashSet::new());
                        let location = package.package_data.atlas.next_location().unwrap();
                        let rasterization = resources.font.0.rasterize(
                            glyph.character.expect("need glyph-character"),
                            package.package_data.font_size.px(ginkgo.scale_factor()),
                        );
                        package.package_data.rasterizations.insert(
                            new,
                            (
                                glyph.character.unwrap(),
                                (rasterization.0.width, rasterization.0.height).into(),
                                rasterization.1,
                            ),
                        );
                        let _partition = package.package_data.atlas.write_location(
                            new,
                            ginkgo,
                            location,
                            package.package_data.rasterizations.get(&new).unwrap().1,
                            &package.package_data.rasterizations.get(&new).unwrap().2,
                        );
                    }
                    package
                        .package_data
                        .instance_coordinator
                        .queue_write(key, package.package_data.atlas.get(&new).unwrap());
                    package
                        .package_data
                        .rasterization_entries
                        .get_mut(&new)
                        .unwrap()
                        .insert(key);
                }
                // color
                if let Some(color) = glyph.color {
                    package
                        .package_data
                        .instance_coordinator
                        .queue_write(key, color);
                }
                // section changes
                if let Some(section) = glyph.section {
                    package
                        .package_data
                        .instance_coordinator
                        .queue_write(key, section.position.to_c());
                    package
                        .package_data
                        .instance_coordinator
                        .queue_write(key, section.area.to_c());
                }
            }
        }
        if package.package_data.instance_coordinator.prepare(ginkgo) {
            package.signal_record();
        }
    }

    fn prepare_resources(
        _resources: &mut Self::Resources,
        _ginkgo: &Ginkgo,
        _per_renderer_record_hook: &mut bool,
    ) {
        // do nothing?
    }

    fn record_behavior() -> RenderRecordBehavior<Self> {
        RenderRecordBehavior::PerPackage(Box::new(record))
    }
}

fn record<'a>(
    resources: &'a TextRenderResources,
    package: &'a mut RenderPackage<Text>,
    mut recorder: RenderInstructionsRecorder<'a>,
) -> Option<RenderInstructionHandle> {
    if package.package_data.instance_coordinator.has_instances() {
        recorder.0.set_pipeline(&resources.pipeline);
        recorder.0.set_bind_group(0, &resources.bind_group, &[]);
        recorder
            .0
            .set_bind_group(1, &package.package_data.bind_group, &[]);
        recorder
            .0
            .set_vertex_buffer(0, resources.vertex_buffer.slice(..));
        recorder.0.set_vertex_buffer(
            1,
            package
                .package_data
                .instance_coordinator
                .buffer::<CReprPosition>()
                .slice(..),
        );
        recorder.0.set_vertex_buffer(
            2,
            package
                .package_data
                .instance_coordinator
                .buffer::<CReprArea>()
                .slice(..),
        );
        recorder.0.set_vertex_buffer(
            3,
            package
                .package_data
                .instance_coordinator
                .buffer::<Color>()
                .slice(..),
        );
        recorder.0.set_vertex_buffer(
            4,
            package
                .package_data
                .instance_coordinator
                .buffer::<TexturePartition>()
                .slice(..),
        );
        recorder.0.draw(
            0..VERTICES.len() as u32,
            0..package.package_data.instance_coordinator.instances(),
        );
        return Some(recorder.finish());
    }
    None
}
