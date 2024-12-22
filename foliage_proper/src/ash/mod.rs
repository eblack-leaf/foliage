use crate::ash::clip::{prepare_clip_section, ClipSection};
use crate::ginkgo::Ginkgo;
use crate::{Attachment, Color, Component, DiffMarkers, Foliage, Layer, Resource};
use bevy_ecs::prelude::IntoSystemConfigs;
use std::collections::HashMap;
use std::ops::Range;
use wgpu::{CommandEncoderDescriptor, RenderPassDescriptor, TextureViewDescriptor};

pub(crate) mod clip;
pub(crate) mod differential;

impl Attachment for Ash {
    fn attach(foliage: &mut Foliage) {
        foliage
            .diff
            .add_systems(prepare_clip_section.in_set(DiffMarkers::Prepare));
    }
}
pub(crate) struct Ash {
    pub(crate) drawn: bool,
    pub(crate) nodes: Vec<Node>,
    pub(crate) contiguous: Vec<ContiguousSpan>,
}
impl Default for Ash {
    fn default() -> Self {
        Self::new()
    }
}
impl Ash {
    pub(crate) fn new() -> Self {
        Self { drawn: false, nodes: vec![], contiguous: vec![] }
    }
    pub(crate) fn initialize(&mut self, ginkgo: &Ginkgo) {
        // TODO create renderers
    }
    pub(crate) fn render(&mut self, ginkgo: &Ginkgo) {
        let surface_texture = ginkgo.surface_texture();
        let view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder =
            ginkgo
                .context()
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("present-encoder"),
                });
        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("render-pass"),
            color_attachments: &ginkgo.color_attachment(&view, Color::gray(50)),
            depth_stencil_attachment: ginkgo.depth_stencil_attachment(),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        // draw
        drop(rpass);
        ginkgo
            .context()
            .queue
            .submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}
pub(crate) trait Render {
    fn render(&mut self, ginkgo: &Ginkgo, call: Call);
}
pub(crate) type Order = i32;
pub(crate) type InstanceId = i32;
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub(crate) enum PipelineId {
    Text,
    Icon,
    Shape,
    Panel,
    Image
}
pub(crate) type GroupId = i32;
#[derive(Clone)]
pub(crate) struct ContiguousSpan {
    pub(crate) pipeline: PipelineId,
    pub(crate) group: GroupId,
    pub(crate) range: Range<Order>,
    pub(crate) clip_section: ClipSection,
}
impl ContiguousSpan {
    pub(crate) fn parameters(&self) -> Call {
        Call {
            group: self.group,
            range: self.range.clone(),
            clip_section: self.clip_section,
        }
    }
}
#[derive(Clone)]
pub(crate) struct Call {
    pub(crate) group: GroupId,
    pub(crate) range: Range<Order>,
    pub(crate) clip_section: ClipSection,
}
#[derive(Copy, Clone)]
pub(crate) struct Node {
    pub(crate) layer: Layer,
    pub(crate) pipeline: PipelineId,
    pub(crate) group: GroupId,
    pub(crate) order: Order,
    pub(crate) clip_section: ClipSection,
}
#[derive(Copy, Clone)]
pub(crate) struct Instance {
    pub(crate) layer: Layer,
    pub(crate) clip_section: ClipSection,
    pub(crate) id: InstanceId,
}
#[derive(Copy, Clone)]
pub(crate) struct Swap {
    pub(crate) old: Order,
    pub(crate) new: Order,
}
pub(crate) struct InstanceCoordinator {
    pub(crate) instances: Vec<Instance>,
    pub(crate) cache: Vec<Instance>,
    pub(crate) swaps: Vec<Swap>,
}
pub(crate) struct InstanceBuffer<I: bytemuck::Pod + bytemuck::Zeroable> {
    pub(crate) cpu: Vec<I>,
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) writes: HashMap<InstanceId, I>,
    _phantom: std::marker::PhantomData<I>,
}
impl<I: bytemuck::Pod + bytemuck::Zeroable> InstanceBuffer<I> {
    pub(crate) fn new() -> Self {
        todo!()
    }
}