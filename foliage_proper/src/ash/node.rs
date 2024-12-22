use crate::ash::clip::ClipSection;
use crate::Layer;
use std::collections::HashMap;
use std::ops::Range;

pub(crate) type Order = i32;
pub(crate) type InstanceId = i32;
pub(crate) type PipelineId = i32;
pub(crate) type GroupId = i32;
#[derive(Clone)]
pub(crate) struct Call {
    pub(crate) pipeline: PipelineId,
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
