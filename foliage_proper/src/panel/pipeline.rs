use crate::ash::clip::ClipContext;
use crate::ash::differential::RenderQueueHandle;
use crate::ash::instance::{Instance, InstanceBuffer};
use crate::ash::node::{Nodes, RemoveNode};
use crate::ash::render::{Parameters, PipelineId, Render, RenderGroup, Renderer};
use crate::ginkgo::Ginkgo;
use crate::opacity::BlendedOpacity;
use crate::panel::vertex;
use crate::rounding::CornerRadii;
use crate::{
    CReprColor, CReprSection, Color, Coordinates, Logical, Outline, Panel, Parent,
    ResolvedElevation, Section,
};
use bevy_ecs::entity::Entity;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use wgpu::{
    BindGroupDescriptor, BindGroupLayoutDescriptor, PipelineLayoutDescriptor, RenderPass,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, VertexState,
    VertexStepMode,
};

pub(crate) struct Resources {
    layer_and_weights: HashMap<Entity, LayerAndWeight>,
    opacity: HashMap<Entity, BlendedOpacity>,
    color: HashMap<Entity, Color>,
}
pub(crate) struct Group {
    sections: InstanceBuffer<CReprSection>,
    lws: InstanceBuffer<LayerAndWeight>,
    colors: InstanceBuffer<CReprColor>,
    radii: InstanceBuffer<CornerRadii>,
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug, Default)]
pub(crate) struct LayerAndWeight {
    elevation: ResolvedElevation,
    weight: f32,
}
impl LayerAndWeight {
    pub(crate) fn new(elevation: ResolvedElevation, weight: f32) -> Self {
        Self { elevation, weight }
    }
}
impl Render for Panel {
    type Group = Group;
    type Resources = Resources;

    fn renderer(ginkgo: &Ginkgo) -> Renderer<Self> {
        let shader = ginkgo.create_shader(ShaderModuleDescriptor {
            label: Some("panel-shader"),
            source: ShaderSource::Wgsl(
                format!(
                    "{}{}",
                    include_str!("../sdf.wgsl"),
                    include_str!("panel.wgsl")
                )
                .into(),
            ),
        });
        let vertex_buffer = ginkgo.create_vertex_buffer(vertex::VERTICES);
        let bind_group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("panel-bind-group-layout"),
            entries: &[Ginkgo::bind_group_layout_entry(0)
                .at_stages(ShaderStages::VERTEX)
                .uniform_entry()],
        });
        let bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
            label: Some("panel-bind-group"),
            layout: &bind_group_layout,
            entries: &[ginkgo.viewport_bind_group_entry(0)],
        });
        let pipeline_layout = ginkgo.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("panel-pipeline-layout-descriptor"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = ginkgo.create_pipeline(&RenderPipelineDescriptor {
            label: Some("panel-render-pipeline"),
            layout: Option::from(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Option::from("vertex_entry"),
                compilation_options: Default::default(),
                buffers: &[
                    Ginkgo::vertex_buffer_layout::<Coordinates>(
                        VertexStepMode::Vertex,
                        &wgpu::vertex_attr_array![0 => Float32x2],
                    ),
                    Ginkgo::vertex_buffer_layout::<CReprSection>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![1 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<LayerAndWeight>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![2 => Float32x2],
                    ),
                    Ginkgo::vertex_buffer_layout::<CReprColor>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![3 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<CornerRadii>(
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
            multiview_mask: None,
            cache: None,
        });
        Renderer {
            pipeline,
            vertex_buffer,
            bind_group,
            groups: {
                let mut groups = HashMap::new();
                groups.insert(
                    0,
                    RenderGroup::new(Group {
                        sections: InstanceBuffer::new(ginkgo, 10),
                        lws: InstanceBuffer::new(ginkgo, 10),
                        colors: InstanceBuffer::new(ginkgo, 10),
                        radii: InstanceBuffer::new(ginkgo, 10),
                    }),
                );
                groups
            },
            resources: Resources {
                layer_and_weights: HashMap::new(),
                opacity: Default::default(),
                color: Default::default(),
            },
        }
    }

    fn prepare(
        renderer: &mut Renderer<Self>,
        queues: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) -> Nodes {
        tracing::trace!("pipeline: panel prepare");
        let mut nodes = Nodes::new();
        let render_group = renderer.groups.get_mut(&0).unwrap();
        for r in queues.removes::<Panel>() {
            if render_group.coordinator.has_instance(r.to_bits()) {
                renderer.resources.layer_and_weights.remove(&r);
                nodes.remove(RemoveNode::new(PipelineId::Panel, 0, r.to_bits()));
                let order = render_group.coordinator.order(r.to_bits());
                render_group.coordinator.remove(order);
                queues.remove_attr::<Panel, ResolvedElevation>(r);
                queues.remove_attr::<Panel, Section<Logical>>(r);
                queues.remove_attr::<Panel, ClipContext>(r);
                queues.remove_attr::<Panel, Outline>(r);
                queues.remove_attr::<Panel, BlendedOpacity>(r);
                queues.remove_attr::<Panel, Color>(r);
                queues.remove_attr::<Panel, Panel>(r);
            }
        }
        for (entity, elevation) in queues.attribute::<Panel, ResolvedElevation>() {
            let id = entity.to_bits();
            if !render_group.coordinator.has_instance(id) {
                render_group
                    .coordinator
                    .add(Instance::new(elevation, Parent::default(), id));
            } else {
                render_group.coordinator.update_elevation(id, elevation);
            }
            let lw = if let Some(lw) = renderer.resources.layer_and_weights.get_mut(&entity) {
                lw.elevation = elevation;
                *lw
            } else {
                let val = LayerAndWeight::new(elevation, 0.0);
                renderer.resources.layer_and_weights.insert(entity, val);
                val
            };
            render_group.group.lws.queue(id, lw);
        }
        for (entity, section) in queues.attribute::<Self, Section<Logical>>() {
            render_group.group.sections.queue(
                entity.to_bits(),
                section
                    .to_physical(ginkgo.configuration().scale_factor.value())
                    .rounded()
                    .c_repr(),
            );
        }
        for (entity, clip_context) in queues.attribute::<Panel, ClipContext>() {
            render_group
                .coordinator
                .update_clip_context(entity.to_bits(), clip_context.0);
        }
        for (entity, outline) in queues.attribute::<Panel, Outline>() {
            renderer
                .resources
                .layer_and_weights
                .get_mut(&entity)
                .unwrap()
                .weight = outline.value as f32 * ginkgo.configuration().scale_factor.value();
            let lw = *renderer.resources.layer_and_weights.get(&entity).unwrap();
            render_group.group.lws.queue(entity.to_bits(), lw);
        }
        for (entity, opacity) in queues.attribute::<Self, BlendedOpacity>() {
            renderer.resources.opacity.insert(entity, opacity);
            if let Some(color) = renderer.resources.color.get(&entity) {
                render_group
                    .group
                    .colors
                    .queue(entity.to_bits(), color.with_opacity(opacity.value).c_repr())
            }
        }
        for (entity, color) in queues.attribute::<Self, Color>() {
            renderer.resources.color.insert(entity, color);
            let opacity = renderer.resources.opacity.get(&entity).unwrap();
            render_group
                .group
                .colors
                .queue(entity.to_bits(), color.with_opacity(opacity.value).c_repr());
        }
        for (entity, panel) in queues.attribute::<Self, Self>() {
            render_group
                .group
                .radii
                .queue(entity.to_bits(), panel.radii);
        }
        if let Some(n) = render_group.coordinator.grown() {
            render_group.group.sections.grow(ginkgo, n);
            render_group.group.lws.grow(ginkgo, n);
            render_group.group.colors.grow(ginkgo, n);
            render_group.group.radii.grow(ginkgo, n);
        }
        for swap in render_group.coordinator.sort() {
            render_group.group.sections.swap(swap);
            render_group.group.lws.swap(swap);
            render_group.group.colors.swap(swap);
            render_group.group.radii.swap(swap);
        }
        for (id, data) in render_group.group.sections.queued() {
            let order = render_group.coordinator.order(id);
            render_group.group.sections.write_cpu(order, data);
        }
        for (id, data) in render_group.group.lws.queued() {
            let order = render_group.coordinator.order(id);
            render_group.group.lws.write_cpu(order, data);
        }
        for (id, data) in render_group.group.colors.queued() {
            let order = render_group.coordinator.order(id);
            render_group.group.colors.write_cpu(order, data);
        }
        for (id, data) in render_group.group.radii.queued() {
            let order = render_group.coordinator.order(id);
            render_group.group.radii.write_cpu(order, data);
        }
        render_group.group.sections.write_gpu(ginkgo);
        render_group.group.lws.write_gpu(ginkgo);
        render_group.group.colors.write_gpu(ginkgo);
        render_group.group.radii.write_gpu(ginkgo);
        for node in render_group.coordinator.updated_nodes(PipelineId::Panel, 0) {
            nodes.update(node);
        }
        nodes
    }

    fn render(renderer: &mut Renderer<Self>, render_pass: &mut RenderPass, parameters: Parameters) {
        render_pass.set_pipeline(&renderer.pipeline);
        render_pass.set_bind_group(0, &renderer.bind_group, &[]);
        render_pass.set_vertex_buffer(0, renderer.vertex_buffer.slice(..));
        let group = &renderer.groups.get(&0).unwrap().group;
        render_pass.set_vertex_buffer(1, group.sections.buffer.slice(..));
        render_pass.set_vertex_buffer(2, group.lws.buffer.slice(..));
        render_pass.set_vertex_buffer(3, group.colors.buffer.slice(..));
        render_pass.set_vertex_buffer(4, group.radii.buffer.slice(..));
        render_pass.draw(0..vertex::VERTICES.len() as u32, parameters.range);
    }
}
