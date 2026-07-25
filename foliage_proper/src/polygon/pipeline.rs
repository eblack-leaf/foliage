use crate::ash::clip::ClipContext;
use crate::ash::differential::RenderQueueHandle;
use crate::ash::instance::{Instance, InstanceBuffer, InstanceId};
use crate::ash::node::{Nodes, RemoveNode};
use crate::ash::render::{Parameters, PipelineId, Render, RenderGroup, Renderer};
use crate::ginkgo::Ginkgo;
use crate::opacity::BlendedOpacity;
use crate::polygon::Polygon;
use crate::{
    CReprColor, CReprSection, Color, Coordinates, Logical, ResolvedElevation, Section, Stem,
};
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use wgpu::{
    BindGroupDescriptor, BindGroupLayoutDescriptor, PipelineLayoutDescriptor, RenderPass,
    RenderPipelineDescriptor, ShaderStages, VertexState, VertexStepMode, include_wgsl,
};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default)]
pub(crate) struct Vertex {
    position: Coordinates,
}
impl Vertex {
    pub(crate) const fn new(position: Coordinates) -> Self {
        Self { position }
    }
}
pub(crate) const VERTICES: [Vertex; 6] = [
    Vertex::new(Coordinates::new(1f32, 0f32)),
    Vertex::new(Coordinates::new(0f32, 0f32)),
    Vertex::new(Coordinates::new(0f32, 1f32)),
    Vertex::new(Coordinates::new(1f32, 0f32)),
    Vertex::new(Coordinates::new(0f32, 1f32)),
    Vertex::new(Coordinates::new(1f32, 1f32)),
];
pub(crate) struct Resources {}
pub(crate) struct Group {
    sections: InstanceBuffer<CReprSection>,
    elevations: InstanceBuffer<ResolvedElevation>,
    colors: InstanceBuffer<CReprColor>,
    opacities: InstanceBuffer<BlendedOpacity>,
    params: InstanceBuffer<Polygon>,
}
impl Render for Polygon {
    type Group = Group;
    type Resources = Resources;

    fn renderer(ginkgo: &Ginkgo) -> Renderer<Self> {
        let shader = ginkgo.create_shader(include_wgsl!("polygon.wgsl"));
        let vertex_buffer = ginkgo.create_vertex_buffer(VERTICES);
        let bind_group_layout = ginkgo.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("polygon-bind-group-layout"),
            entries: &[Ginkgo::bind_group_layout_entry(0)
                .at_stages(ShaderStages::VERTEX)
                .uniform_entry()],
        });
        let bind_group = ginkgo.create_bind_group(&BindGroupDescriptor {
            label: Some("polygon-bind-group"),
            layout: &bind_group_layout,
            entries: &[ginkgo.viewport_bind_group_entry(0)],
        });
        let pipeline_layout = ginkgo.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("polygon-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = ginkgo.create_pipeline(&RenderPipelineDescriptor {
            label: Some("polygon-render-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Option::from("vertex_entry"),
                compilation_options: Default::default(),
                buffers: &[
                    Ginkgo::vertex_buffer_layout::<Vertex>(
                        VertexStepMode::Vertex,
                        &wgpu::vertex_attr_array![0 => Float32x2],
                    ),
                    Ginkgo::vertex_buffer_layout::<CReprSection>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![1 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<ResolvedElevation>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![2 => Float32],
                    ),
                    Ginkgo::vertex_buffer_layout::<CReprColor>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![3 => Float32x4],
                    ),
                    Ginkgo::vertex_buffer_layout::<BlendedOpacity>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![4 => Float32],
                    ),
                    Ginkgo::vertex_buffer_layout::<Polygon>(
                        VertexStepMode::Instance,
                        &wgpu::vertex_attr_array![5 => Float32x3],
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
        let mut groups = HashMap::new();
        groups.insert(
            0,
            RenderGroup::new(Group {
                sections: InstanceBuffer::new(ginkgo, 1),
                elevations: InstanceBuffer::new(ginkgo, 1),
                colors: InstanceBuffer::new(ginkgo, 1),
                opacities: InstanceBuffer::new(ginkgo, 1),
                params: InstanceBuffer::new(ginkgo, 1),
            }),
        );
        Renderer {
            pipeline,
            vertex_buffer,
            bind_group,
            groups,
            resources: Resources {},
        }
    }

    fn prepare(
        renderer: &mut Renderer<Self>,
        queues: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) -> Nodes {
        tracing::trace!("pipeline: polygon prepare");
        let mut nodes = Nodes::new();
        let group = renderer.groups.get_mut(&0).unwrap();
        for entity in queues.removes::<Polygon>() {
            let id = entity.index().index() as InstanceId;
            if group.coordinator.has_instance(id) {
                let order = group.coordinator.order(id);
                group.coordinator.remove(order);
                nodes.remove(RemoveNode::new(PipelineId::Polygon, 0, id));
                queues.remove_attr::<Polygon, Section<Logical>>(entity);
                queues.remove_attr::<Polygon, Polygon>(entity);
                queues.remove_attr::<Polygon, ResolvedElevation>(entity);
                queues.remove_attr::<Polygon, ClipContext>(entity);
                queues.remove_attr::<Polygon, Color>(entity);
                queues.remove_attr::<Polygon, BlendedOpacity>(entity);
            }
        }
        for (entity, elevation) in queues.attribute::<Polygon, ResolvedElevation>() {
            let id = entity.index().index() as InstanceId;
            if !group.coordinator.has_instance(id) {
                group
                    .coordinator
                    .add(Instance::new(elevation, Stem::default(), id));
            } else {
                group.coordinator.update_elevation(id, elevation);
            }
            group.group.elevations.queue(id, elevation);
        }
        for (entity, section) in queues.attribute::<Polygon, Section<Logical>>() {
            let id = entity.index().index() as InstanceId;
            group.group.sections.queue(
                id,
                section
                    .to_physical(ginkgo.configuration().scale_factor.value())
                    .rounded()
                    .c_repr(),
            );
        }
        for (entity, clip) in queues.attribute::<Polygon, ClipContext>() {
            let id = entity.index().index() as InstanceId;
            group.coordinator.update_clip_context(id, clip.0);
        }
        for (entity, color) in queues.attribute::<Polygon, Color>() {
            let id = entity.index().index() as InstanceId;
            group.group.colors.queue(id, color.c_repr());
        }
        for (entity, opacity) in queues.attribute::<Polygon, BlendedOpacity>() {
            let id = entity.index().index() as InstanceId;
            group.group.opacities.queue(id, opacity);
        }
        for (entity, polygon) in queues.attribute::<Polygon, Polygon>() {
            let id = entity.index().index() as InstanceId;
            group.group.params.queue(id, polygon);
        }
        if let Some(n) = group.coordinator.grown() {
            group.group.sections.grow(ginkgo, n);
            group.group.elevations.grow(ginkgo, n);
            group.group.colors.grow(ginkgo, n);
            group.group.opacities.grow(ginkgo, n);
            group.group.params.grow(ginkgo, n);
        }
        for swap in group.coordinator.sort() {
            group.group.sections.swap(swap);
            group.group.elevations.swap(swap);
            group.group.colors.swap(swap);
            group.group.opacities.swap(swap);
            group.group.params.swap(swap);
        }
        for (id, data) in group.group.sections.queued() {
            let order = group.coordinator.order(id);
            group.group.sections.write_cpu(order, data);
        }
        for (id, data) in group.group.elevations.queued() {
            let order = group.coordinator.order(id);
            group.group.elevations.write_cpu(order, data);
        }
        for (id, data) in group.group.colors.queued() {
            let order = group.coordinator.order(id);
            group.group.colors.write_cpu(order, data);
        }
        for (id, data) in group.group.opacities.queued() {
            let order = group.coordinator.order(id);
            group.group.opacities.write_cpu(order, data);
        }
        for (id, data) in group.group.params.queued() {
            let order = group.coordinator.order(id);
            group.group.params.write_cpu(order, data);
        }
        group.group.sections.write_gpu(ginkgo);
        group.group.elevations.write_gpu(ginkgo);
        group.group.colors.write_gpu(ginkgo);
        group.group.opacities.write_gpu(ginkgo);
        group.group.params.write_gpu(ginkgo);
        for node in group.coordinator.updated_nodes(PipelineId::Polygon, 0) {
            nodes.update(node);
        }
        nodes
    }

    fn render(renderer: &mut Renderer<Self>, render_pass: &mut RenderPass, parameters: Parameters) {
        let group = renderer.groups.get(&0).unwrap();
        render_pass.set_pipeline(&renderer.pipeline);
        render_pass.set_bind_group(0, &renderer.bind_group, &[]);
        render_pass.set_vertex_buffer(0, renderer.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, group.group.sections.buffer.slice(..));
        render_pass.set_vertex_buffer(2, group.group.elevations.buffer.slice(..));
        render_pass.set_vertex_buffer(3, group.group.colors.buffer.slice(..));
        render_pass.set_vertex_buffer(4, group.group.opacities.buffer.slice(..));
        render_pass.set_vertex_buffer(5, group.group.params.buffer.slice(..));
        render_pass.draw(0..VERTICES.len() as u32, parameters.range);
    }
}
