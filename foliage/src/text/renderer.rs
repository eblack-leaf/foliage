use crate::ash::instruction::{
    RenderInstructionHandle, RenderInstructionsRecorder, RenderRecordBehavior,
};
use crate::ash::render::{Render, RenderPhase};
use crate::ash::render_package::RenderPackage;
use crate::ash::render_packet::RenderPacket;
use crate::color::Color;
use crate::coordinate::area::CReprArea;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::CReprPosition;
use crate::coordinate::CoordinateUnit;
use crate::ginkgo::uniform::AlignedUniform;
use crate::ginkgo::Ginkgo;
use crate::instance::{InstanceCoordinator, InstanceCoordinatorBuilder};
use crate::text::font::MonospacedFont;
use crate::text::vertex::{Vertex, VERTICES};
use crate::text::{FontSize, GlyphAdds, GlyphRemoves, Text, TextValueUniqueCharacters};
use crate::texture::{AtlasBlock, TextureAtlas, TexturePartition};
use bevy_ecs::entity::Entity;

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
    atlas: TextureAtlas<TextKey, u8>,
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
                            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
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
                            attributes: &wgpu::vertex_attr_array![4 => Float32x2],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: Ginkgo::buffer_address::<TexturePartition>(1),
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![5 => Float32x2],
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
            font: MonospacedFont::new(40),
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
        let text_char_len = render_packet.get::<TextValueUniqueCharacters>().unwrap();
        let pos = render_packet.get::<CReprPosition>().unwrap();
        let layer = render_packet.get::<Layer>().unwrap();
        let uniform = AlignedUniform::new(ginkgo.device(), Some([pos.x, pos.y, layer.z, 0.0]));
        let atlas = TextureAtlas::new(
            ginkgo,
            AtlasBlock(
                resources
                    .font
                    .character_dimensions(font_size.px(ginkgo.scale_factor())),
            ),
            text_char_len.0,
            wgpu::TextureFormat::R8Unorm,
        );
        let bind_group = ginkgo
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("text-package-bind-group"),
                layout: &resources.package_layout,
                entries: &[
                    uniform.uniform.bind_group_entry(0),
                    Ginkgo::texture_bind_group_entry(&atlas.view, 1),
                ],
            });
        let instance_coordinator = InstanceCoordinatorBuilder::new(text_char_len.0)
            .with_attribute::<CReprPosition>()
            .with_attribute::<CReprArea>()
            .with_attribute::<Color>()
            .with_attribute::<TexturePartition>()
            .build(ginkgo);
        // iter char_changes and fill atlas + coordinator
        for change in render_packet.get::<GlyphAdds>().unwrap().0.drain(..) {
            // only have adds here as no removes could be useful
        }
        let package = TextRenderPackage {
            instance_coordinator,
            bind_group,
            uniform,
            atlas,
        };
        package
    }

    fn on_package_removal(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        package: RenderPackage<Self>,
    ) {
        // do nothing?
        todo!()
    }

    fn prepare_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        package: &mut RenderPackage<Self>,
        render_packet: RenderPacket,
    ) {
        // write to atlas + coordinator as above
        if let Some(removes) = render_packet.get::<GlyphRemoves>() {
            for change in removes.0 {}
        }
        if let Some(adds) = render_packet.get::<GlyphAdds>() {
            for change in adds.0 {}
        }
        // prepare all packages coordinators
        todo!()
    }

    fn prepare_resources(
        resources: &mut Self::Resources,
        ginkgo: &Ginkgo,
        per_renderer_record_hook: &mut bool,
    ) {
        // do nothing?
        todo!()
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