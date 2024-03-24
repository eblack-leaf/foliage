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
use crate::text::front_end::{FontSize, Text, TextValueUniqueCharacters};
use crate::text::front_end::{Glyph, GlyphKey, TextColorChanges, TextGlyphChanges};
use crate::text::vertex::{Vertex, VERTICES};
use crate::texture::coord::TexturePartition;
use crate::texture::{AtlasBlock, TextureAtlas};
use bevy_ecs::entity::Entity;

pub struct TextRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    font: MonospacedFont,
    package_layout: wgpu::BindGroupLayout,
}
pub type TextKey = usize;
pub struct TextRenderPackage {
    instance_coordinator: InstanceCoordinator<TextKey>,
    bind_group: wgpu::BindGroup,
    uniform: AlignedUniform<CoordinateUnit>, // pos + layer
    atlas: TextureAtlas<TextKey, GlyphKey, u8>,
    font_size: FontSize,
    block: AtlasBlock,
}
impl Render for Text {
    type Resources = TextRenderResources;
    type RenderPackage = TextRenderPackage;
    const RENDER_PHASE: RenderPhase = RenderPhase::Alpha(5);

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
        tracing::trace!("creating text package: {:?}", _entity);
        let font_size = render_packet.get::<FontSize>().unwrap();
        let unique_characters = render_packet.get::<TextValueUniqueCharacters>().unwrap();
        let pos = render_packet.get::<CReprPosition>().unwrap();
        let layer = render_packet.get::<Layer>().unwrap();
        let base_color = render_packet.get::<Color>().unwrap();
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
        let mut instance_coordinator = InstanceCoordinatorBuilder::new(unique_characters.0)
            .with_attribute::<CReprPosition>()
            .with_attribute::<CReprArea>()
            .with_attribute::<Color>()
            .with_attribute::<TexturePartition>()
            .build(ginkgo);
        for (key, glyph) in render_packet
            .get::<TextGlyphChanges>()
            .unwrap()
            .added
            .drain()
        {
            match glyph {
                Glyph::Control => {}
                Glyph::Char(ch) => {
                    if !atlas.has_key(&ch.key) {
                        let rasterization = resources.font.0.rasterize_indexed(
                            ch.key.glyph_index,
                            font_size.px(ginkgo.scale_factor()),
                        );
                        let extent: Area<NumericalContext> =
                            (rasterization.0.width, rasterization.0.height).into();
                        atlas.write_next(ch.key, ginkgo, extent, rasterization.1);
                    }
                    atlas.add_reference(key, ch.key);
                    instance_coordinator.queue_add(key);
                    instance_coordinator.queue_write(key, ch.section.unwrap().position.to_c());
                    instance_coordinator.queue_write(key, ch.section.unwrap().area.to_c());
                    instance_coordinator.queue_write(key, base_color);
                    instance_coordinator.queue_write(key, atlas.get(&ch.key).unwrap());
                }
            }
        }
        for (key, color) in render_packet.get::<TextColorChanges>().unwrap().0.drain() {
            instance_coordinator.queue_write(key, color);
        }
        instance_coordinator.prepare(ginkgo);
        let bind_group = ginkgo
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("text-package-bind-group"),
                layout: &resources.package_layout,
                entries: &[
                    uniform.bind_group_entry(0),
                    Ginkgo::texture_bind_group_entry(atlas.view(), 1),
                ],
            });

        TextRenderPackage {
            instance_coordinator,
            bind_group,
            uniform,
            font_size,
            block: atlas.block(),
            atlas,
        }
    }

    fn on_package_removal(
        _ginkgo: &Ginkgo,
        _resources: &mut Self::Resources,
        _entity: Entity,
        _package: RenderPackage<Self>,
    ) {
        tracing::trace!("removing text-package:{:?}", _entity);
        // do nothing?
    }

    fn prepare_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        _entity: Entity,
        package: &mut RenderPackage<Self>,
        render_packet: RenderPacket,
    ) {
        tracing::trace!("preparing text-package: {:?}", _entity);
        if let Some(pos) = render_packet.get::<CReprPosition>() {
            tracing::trace!("updating-text-pos: {:?}", pos);
            package.package_data.uniform.set_aspect(0, pos.x);
            package.package_data.uniform.set_aspect(1, pos.y);
        }
        if let Some(layer) = render_packet.get::<Layer>() {
            package.package_data.uniform.set_aspect(2, layer.z);
        }
        if package.package_data.uniform.needs_update() {
            tracing::trace!("updating-text-uniform");
            package.package_data.uniform.update(ginkgo.queue());
        }
        let mut glyph_changes = render_packet.get::<TextGlyphChanges>();
        if let Some(mut changes) = glyph_changes.as_mut() {
            for (key, glyph) in changes.removed.drain() {
                tracing::trace!("removing-text-glyph: {:?}:{:?}", key, glyph);
                match glyph {
                    Glyph::Control => {}
                    Glyph::Char(ch) => {
                        package.package_data.atlas.remove_reference(key, ch.key);
                        package.package_data.instance_coordinator.queue_remove(key);
                    }
                }
            }
            for (key, glyph) in changes.added.drain() {
                match glyph {
                    Glyph::Control => {}
                    Glyph::Char(ch) => {
                        package.package_data.atlas.add_reference(key, ch.key);
                    }
                }
            }
        }
        // end new iteration

        let glyph_changes = render_packet.get::<GlyphChangeQueue>();
        let font_size_change = render_packet.get::<FontSize>();
        let new_glyph_key_count = {
            let mut count = 0;
            if let Some(changes) = glyph_changes.as_ref() {
                for (_, change) in changes.0.iter() {
                    if let Some(key) = change.key {
                        if !package.package_data.atlas.has_key(&key.0) {
                            tracing::trace!("incrementing-new-glyph-count");
                            count += 1;
                        }
                    }
                }
            }
            count
        };
        let mut font_size_changed = false;
        if font_size_change.is_some() {
            let font_size = font_size_change.unwrap();
            if font_size != package.package_data.font_size {
                tracing::trace!("font-size-changed: {:?}", font_size);
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
        if package.package_data.atlas.would_grow(new_glyph_key_count) || font_size_changed {
            tracing::trace!(
                "growing atlas by {:?} | font_size_changed:{:?}",
                new_glyph_key_count,
                font_size_changed
            );
            for (key, entry) in package.package_data.atlas.entries_mut() {
                tracing::trace!("rewriting entry: {:?}", key);
                if font_size_changed {
                    let rasterization = resources.font.0.rasterize_indexed(
                        key.glyph_index,
                        package.package_data.font_size.px(ginkgo.scale_factor()),
                    );
                    entry.set(
                        (rasterization.0.width, rasterization.0.height).into(),
                        rasterization.1,
                    );
                }
            }
            for (key, partition) in package.package_data.atlas.grow_by(
                new_glyph_key_count,
                ginkgo,
                package.package_data.block,
            ) {
                tracing::trace!("queuing rewrite for {:?}:{:?}", key, partition);
                package
                    .package_data
                    .instance_coordinator
                    .queue_write(key, partition);
            }
            tracing::trace!("rebinding-text-bind-group");
            package.package_data.bind_group =
                ginkgo
                    .device()
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("text-package-bind-group"),
                        layout: &resources.package_layout,
                        entries: &[
                            package.package_data.uniform.bind_group_entry(0),
                            Ginkgo::texture_bind_group_entry(package.package_data.atlas.view(), 1),
                        ],
                    });
            package.signal_record();
        }
        if let Some(changes) = glyph_changes {
            for (key, glyph) in changes.0 {
                if !package.package_data.instance_coordinator.has_key(&key) {
                    tracing::trace!("adding key: {:?}", key);
                    package.package_data.instance_coordinator.queue_add(key);
                }
                if let Some((new, old)) = glyph.key {
                    if let Some(old) = old {
                        if old != new {
                            tracing::trace!(
                                "removing reference to old: {:?} from new {:?}",
                                old.glyph_index,
                                new.glyph_index
                            );
                            package.package_data.atlas.remove_reference(key, old);
                        }
                    }
                    if !package.package_data.atlas.has_key(&new) {
                        tracing::trace!("rasterizing new: {:?}", new.glyph_index);
                        let rasterization = resources.font.0.rasterize_indexed(
                            new.glyph_index,
                            package.package_data.font_size.px(ginkgo.scale_factor()),
                        );
                        package.package_data.atlas.write_next(
                            new,
                            ginkgo,
                            (rasterization.0.width, rasterization.0.height).into(),
                            rasterization.1,
                        );
                    }
                    tracing::trace!("adding reference to new: {:?}", new.glyph_index);
                    package.package_data.atlas.add_reference(key, new);
                    package
                        .package_data
                        .instance_coordinator
                        .queue_write(key, package.package_data.atlas.get(&new).unwrap());
                }
                // color
                if let Some(color) = glyph.color {
                    tracing::trace!("writing text-glyph-color: {:?}", color);
                    package
                        .package_data
                        .instance_coordinator
                        .queue_write(key, color);
                }
                // section changes
                if let Some(section) = glyph.section {
                    tracing::trace!("updating text-glyph-section: {:?}", section);
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
            tracing::trace!("text signal record");
            package.signal_record();
        }
    }

    fn prepare_resources(
        _resources: &mut Self::Resources,
        _ginkgo: &Ginkgo,
        _per_renderer_record_hook: &mut bool,
    ) {
        tracing::trace!("prepare text-resources: {:?}", ());
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
        tracing::trace!(
            "record-text w/ instances: {:?}",
            package.package_data.instance_coordinator.instances()
        );
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
    }
    Some(recorder.finish())
}