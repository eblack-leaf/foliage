use crate::ginkgo::Ginkgo;
use crate::Render;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, BufferUsages, ColorTargetState, FragmentState, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, RenderPipeline, RenderPipelineDescriptor,
    ShaderModule, ShaderModuleDescriptor, TextureView,
};

pub struct Aspen;

impl Aspen {
    pub fn fragment_state<'a>(
        module: &'a ShaderModule,
        entry_point: &'a str,
        targets: &'a [Option<ColorTargetState>],
    ) -> Option<FragmentState<'a>> {
        Some(FragmentState {
            module,
            entry_point,
            compilation_options: Default::default(),
            targets,
        })
    }
    pub fn texture_bind_group_entry(view: &TextureView, binding: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(view),
        }
    }
    pub fn sampler_bind_group_entry(sampler: &wgpu::Sampler, binding: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Sampler(sampler),
        }
    }
    pub fn triangle_list_primitive() -> PrimitiveState {
        PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        }
    }
    pub fn sampler_bind_group_layout_entry(binding: u32) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
            count: None,
        }
    }
    pub fn texture_d2_bind_group_entry(binding: u32) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }
    }
    pub fn vertex_uniform_bind_group_layout_entry(binding: u32) -> BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }
    pub fn bind_group_layout(ginkgo: &Ginkgo, desc: &BindGroupLayoutDescriptor) -> BindGroupLayout {
        let bind_group_layout = ginkgo.context().device.create_bind_group_layout(desc);
        bind_group_layout
    }
    pub fn bind_group(ginkgo: &Ginkgo, desc: &BindGroupDescriptor) -> BindGroup {
        let bind_group = ginkgo.context().device.create_bind_group(desc);
        bind_group
    }
    pub fn create_pipeline_layout(
        ginkgo: &Ginkgo,
        desc: &PipelineLayoutDescriptor,
    ) -> PipelineLayout {
        let layout = ginkgo.context().device.create_pipeline_layout(desc);
        layout
    }
    pub fn create_shader(ginkgo: &Ginkgo, shader_source: ShaderModuleDescriptor) -> ShaderModule {
        let shader = ginkgo.context().device.create_shader_module(shader_source);
        shader
    }
    pub fn create_vertex_buffer<R: Render, VB: AsRef<[R::Vertex]>>(
        ginkgo: &Ginkgo,
        vb_data: VB,
    ) -> Buffer {
        let vertex_buffer =
            ginkgo
                .context()
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("vertex-buffer"),
                    contents: bytemuck::cast_slice(vb_data.as_ref()),
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                });
        vertex_buffer
    }
    pub fn create_pipeline(ginkgo: &Ginkgo, desc: &RenderPipelineDescriptor) -> RenderPipeline {
        let pipeline = ginkgo.context().device.create_render_pipeline(desc);
        pipeline
    }
    pub fn triangle_primitive() -> () {
        todo!()
    }
}
