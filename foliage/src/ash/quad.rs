//! One quad per element: the pipeline five of the six renderers share.
//!
//! Every renderer here draws the same two triangles and differs only in what it puts on them --
//! a rounded rectangle's distance field, a regular polygon's, a stroke's four corners, a mark's
//! distance field, a picture's pixels. So the pipeline is built once, generically, and a renderer
//! supplies the three things that are actually its own: the shader, the shape of its instance, and
//! whether it samples a texture.
//!
//! [`Texts`](crate::ash::text::Texts) is the one that is not here, and the difference is structural
//! rather than a matter of shader: a run is **one** entry in the stack holding many quads, so it
//! keeps two numberings where these keep one.

use core::ops::Range;

use bytemuck::Pod;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferUsages, Device, FilterMode, PipelineLayoutDescriptor, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, TextureSampleType, TextureView,
    TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode,
};

use crate::ash::CORNERS;
use crate::ash::instances::Instances;
use crate::ginkgo::Ginkgo;

/// What one renderer contributes to the shared pipeline.
pub(crate) struct Recipe {
    pub(crate) label: &'static str,
    /// The whole shader, already assembled -- a renderer that rounds a box prepends `sdf.wgsl`
    /// itself, because WGSL has no include and one file is what stops two renderers rounding
    /// differently.
    pub(crate) shader: String,
    /// The instance's own vertex attributes, from shader location 1 upward.
    pub(crate) attributes: &'static [VertexAttribute],
    /// Which shader location the depth arrives at, which is one past the instance's own.
    pub(crate) depth: u32,
    /// Whether the pipeline samples a texture at group 1.
    pub(crate) sampled: bool,
}

/// The pipeline, the quad it is drawn over, and what the GPU is holding.
pub(crate) struct Quads<I: Pod> {
    pipeline: RenderPipeline,
    corners: Buffer,
    pub(crate) instances: Instances<I>,
}

impl<I: Pod> Quads<I> {
    pub(crate) fn new(
        ginkgo: &Ginkgo,
        recipe: Recipe,
        sampling: Option<&BindGroupLayout>,
        capacity: u32,
    ) -> Self {
        let device = ginkgo.device();
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(recipe.label),
            source: ShaderSource::Wgsl(recipe.shader.into()),
        });
        let mut layouts = vec![Some(ginkgo.viewport_layout())];
        if recipe.sampled {
            layouts.push(Some(
                sampling.expect("a sampled renderer declares a layout"),
            ));
        }
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(recipe.label),
            bind_group_layouts: &layouts,
            immediate_size: 0,
        });
        // The depth is its own buffer rather than a field of the instance, because the two change
        // for different reasons: a depth comes from an instance's *position* in the whole stack, so
        // a value written to an instance already in the stack does not disturb anyone else's.
        let depth = [VertexAttribute {
            format: VertexFormat::Float32,
            offset: 0,
            shader_location: recipe.depth,
        }];
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(recipe.label),
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vertex_entry"),
                compilation_options: Default::default(),
                buffers: &[
                    Some(VertexBufferLayout {
                        array_stride: size_of::<[f32; 2]>() as u64,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    }),
                    Some(VertexBufferLayout {
                        array_stride: size_of::<I>() as u64,
                        step_mode: VertexStepMode::Instance,
                        attributes: recipe.attributes,
                    }),
                    Some(VertexBufferLayout {
                        array_stride: size_of::<f32>() as u64,
                        step_mode: VertexStepMode::Instance,
                        attributes: &depth,
                    }),
                ],
            },
            primitive: Ginkgo::triangles(),
            depth_stencil: Ginkgo::depth_state(),
            multisample: Ginkgo::samples(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_entry"),
                compilation_options: Default::default(),
                targets: &ginkgo.blending(),
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            corners: device.create_buffer_init(&BufferInitDescriptor {
                label: Some(recipe.label),
                contents: bytemuck::cast_slice(&CORNERS),
                usage: BufferUsages::VERTEX,
            }),
            instances: Instances::new(device, recipe.label, capacity),
        }
    }

    /// Draws one run of what is held.
    ///
    /// The range is a span of the stack, and which spans there are and what order they go in is
    /// [`Ash`](crate::ash::Ash)'s: a renderer draws the slots it is asked for and decides nothing
    /// about when it is asked. `sampled` is what the span's own binding turned out to be, which for
    /// a picture is a different texture from one span to the next.
    pub(crate) fn draw(
        &self,
        pass: &mut RenderPass<'_>,
        span: Range<u32>,
        sampled: Option<&BindGroup>,
    ) {
        pass.set_pipeline(&self.pipeline);
        if let Some(binding) = sampled {
            pass.set_bind_group(1, binding, &[]);
        }
        pass.set_vertex_buffer(0, self.corners.slice(..));
        pass.set_vertex_buffer(1, self.instances.data());
        pass.set_vertex_buffer(2, self.instances.depths());
        pass.draw(0..CORNERS.len() as u32, span);
    }
}

/// The layout every pipeline that samples declares at group 1: a texture, then a filtering sampler.
///
/// One shape across the three that sample -- glyph coverage, a mark's field, a picture's pixels --
/// because they differ in what is *on* the texture and in nothing about how it is bound.
pub(crate) fn sampling(device: &Device, label: &'static str) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

/// A filtering sampler, clamped at the edges.
///
/// Clamped rather than repeating because everything sampled here is a rect inside a larger sheet:
/// a wrapped sample would reach the far side of the sheet rather than the far side of the glyph.
pub(crate) fn sampler(device: &Device, label: &'static str) -> Sampler {
    device.create_sampler(&SamplerDescriptor {
        label: Some(label),
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..SamplerDescriptor::default()
    })
}

/// One texture bound against [`sampling`]'s layout.
pub(crate) fn bind(
    device: &Device,
    layout: &BindGroupLayout,
    view: &TextureView,
    sampler: &Sampler,
    label: &'static str,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(sampler),
            },
        ],
    })
}
