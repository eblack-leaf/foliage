//! Pictures drawn apart.
//!
//! An element drawn [`together`](crate::Place::together) and fading is drawn into a picture of its
//! own, at full strength, and the picture is laid onto the window once, at the element's opacity
//! and at its place in the stack. That is the whole difference from fading each element on its
//! own: where two of them overlap, the one behind does not show through the one in front, because
//! what is faded is what the two of them made together.
//!
//! A picture is a texture the size of the window, in the window's own pixels and format, with a
//! depth attachment of its own. Every renderer draws into one unchanged -- a pipeline is built
//! against a format, and this is the same format -- and blending onto clear leaves the picture
//! premultiplied, which is what the pipeline here lays down.

use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferDescriptor,
    BufferUsages, ColorTargetState, PipelineLayoutDescriptor, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, Texture,
    TextureSampleType, TextureView, TextureViewDescriptor, TextureViewDimension,
    VertexBufferLayout, VertexStepMode,
};

use crate::ginkgo::Ginkgo;
use crate::ginkgo::depth::Depth;

/// What one picture is laid down with.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct Laid {
    pub(crate) opacity: f32,
    pub(crate) depth: f32,
}

/// One picture: what is drawn into it, its depth, and how it is bound to be read.
pub(crate) struct Canvas {
    view: TextureView,
    depth: Depth,
    binding: BindGroup,
    /// Kept for its lifetime: the view and the binding read it.
    _texture: Texture,
}

impl Canvas {
    /// Where its pass draws: the picture, and its own depth.
    pub(crate) fn target(&self) -> (&TextureView, &Depth) {
        (&self.view, &self.depth)
    }
}

/// The pipeline every picture is laid down with, and the pictures themselves.
pub(crate) struct Layers {
    pipeline: RenderPipeline,
    layout: BindGroupLayout,
    /// As many as the most this window has needed at once, at its present size.
    canvases: Vec<Canvas>,
    /// The size they were made at, in device pixels. Another size makes them again.
    size: (u32, u32),
    laid: Buffer,
    capacity: u64,
}

impl Layers {
    pub(crate) fn new(ginkgo: &Ginkgo) -> Self {
        let device = ginkgo.device();
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("layer"),
            source: ShaderSource::Wgsl(include_str!("layer.wgsl").into()),
        });
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("layer"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                // Read texel for texel, never filtered.
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("layer"),
            bind_group_layouts: &[Some(ginkgo.viewport_layout()), Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("layer"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_entry"),
                compilation_options: Default::default(),
                buffers: &[Some(VertexBufferLayout {
                    array_stride: size_of::<Laid>() as u64,
                    step_mode: VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![0 => Float32, 1 => Float32],
                })],
            },
            primitive: Ginkgo::triangles(),
            depth_stencil: Ginkgo::depth_state(),
            multisample: Ginkgo::samples(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_entry"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: ginkgo.format(),
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: Default::default(),
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let capacity = 4;
        Self {
            pipeline,
            layout,
            canvases: Vec::new(),
            size: (0, 0),
            laid: laid(ginkgo, capacity),
            capacity,
        }
    }

    /// Room for `count` pictures at the window's present size, and what each is laid down with.
    ///
    /// A picture is made the first time this many are wanted at once and kept after, so a fade
    /// that runs for many frames makes its picture on the first and none after it.
    pub(crate) fn hold(&mut self, ginkgo: &Ginkgo, laid: &[Laid]) {
        let size = ginkgo.size();
        if size != self.size {
            self.canvases.clear();
            self.size = size;
        }
        while self.canvases.len() < laid.len() {
            self.canvases.push(self.canvas(ginkgo));
        }
        if laid.len() as u64 > self.capacity {
            self.capacity = (laid.len() as u64).next_power_of_two();
            self.laid = self::laid(ginkgo, self.capacity);
        }
        if !laid.is_empty() {
            ginkgo
                .queue()
                .write_buffer(&self.laid, 0, bytemuck::cast_slice(laid));
        }
    }

    /// The first `count` pictures, to draw into -- or none, where they are not at the window's
    /// size and a frame has not run since it changed to make them so.
    pub(crate) fn canvases(&self, ginkgo: &Ginkgo, count: usize) -> &[Canvas] {
        match self.size == ginkgo.size() && self.canvases.len() >= count {
            true => &self.canvases[..count],
            false => &[],
        }
    }

    /// Lays picture `index` down.
    pub(crate) fn draw(&self, pass: &mut RenderPass<'_>, index: usize) {
        let Some(canvas) = self.canvases.get(index) else {
            return;
        };
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(1, &canvas.binding, &[]);
        pass.set_vertex_buffer(0, self.laid.slice(..));
        let index = index as u32;
        pass.draw(0..3, index..index + 1);
    }

    fn canvas(&self, ginkgo: &Ginkgo) -> Canvas {
        let device = ginkgo.device();
        let (width, height) = self.size;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("layer"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ginkgo.format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&TextureViewDescriptor::default());
        let binding = device.create_bind_group(&BindGroupDescriptor {
            label: Some("layer"),
            layout: &self.layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view),
            }],
        });
        Canvas {
            view,
            depth: ginkgo.depth(),
            binding,
            _texture: texture,
        }
    }
}

fn laid(ginkgo: &Ginkgo, capacity: u64) -> Buffer {
    ginkgo.device().create_buffer(&BufferDescriptor {
        label: Some("layer"),
        size: capacity * size_of::<Laid>() as u64,
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
