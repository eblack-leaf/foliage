use crate::ash::clip::ClipSection;
use crate::ash::differential::Elm;
use crate::ash::instance::{InstanceCoordinator, Order};
use crate::ash::node::Nodes;
use crate::ginkgo::{Ginkgo, ScaleFactor};
use crate::{Physical, Section};
use std::collections::HashMap;
use std::ops::Range;
use wgpu::RenderPass;

pub(crate) trait Render
where
    Self: Sized,
{
    type Group;
    type Resources;
    fn renderer(ginkgo: &Ginkgo) -> Renderer<Self>;
    fn prepare(renderer: &mut Renderer<Self>, elm: &mut Elm, ginkgo: &Ginkgo) -> Nodes;
    fn render(renderer: &mut Renderer<Self>, render_pass: &mut RenderPass, parameters: Parameters);
}
pub(crate) type GroupId = i32;
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub(crate) enum PipelineId {
    Text,
    Icon,
    Shape,
    Panel,
    Image,
}
#[derive(Clone)]
pub(crate) struct ContiguousSpan {
    pub(crate) pipeline: PipelineId,
    pub(crate) group: GroupId,
    pub(crate) range: Range<Order>,
    pub(crate) clip_section: ClipSection,
}
impl ContiguousSpan {
    pub(crate) fn parameters(
        &self,
        view_section: Section<Physical>,
        scale_factor: ScaleFactor,
    ) -> Parameters {
        let clip_section = if let Some(present) = self.clip_section.0 {
            present
                .max(Section::new((0, 0), (0, 0)))
                .to_physical(scale_factor.value())
                .intersection(view_section)
        } else {
            None
        };
        Parameters {
            group: self.group,
            range: self.range.clone(),
            clip_section,
        }
    }
}
#[derive(Clone)]
pub(crate) struct Parameters {
    pub(crate) group: GroupId,
    pub(crate) range: Range<Order>,
    pub(crate) clip_section: Option<Section<Physical>>,
}
pub(crate) struct RenderGroup<R: Render> {
    pub(crate) coordinator: InstanceCoordinator,
    pub(crate) group: R::Group,
}
impl<R: Render> RenderGroup<R> {
    pub(crate) fn new(group: R::Group) -> Self {
        Self {
            coordinator: InstanceCoordinator::new(1),
            group,
        }
    }
}
pub(crate) struct Renderer<R: Render> {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) groups: HashMap<GroupId, RenderGroup<R>>,
    pub(crate) resources: R::Resources,
}
