use crate::ash::instance::{InstanceId, Order};
use crate::ash::render::{GroupId, PipelineId};
use crate::{ResolvedElevation, Stem};

#[derive(Copy, Clone, Debug)]
pub(crate) struct Node {
    pub(crate) elevation: ResolvedElevation,
    pub(crate) pipeline: PipelineId,
    pub(crate) group: GroupId,
    pub(crate) order: Order,
    pub(crate) clip_context: Stem,
    pub(crate) instance_id: InstanceId,
}

impl Node {
    pub(crate) fn new(
        elevation: ResolvedElevation,
        pipeline_id: PipelineId,
        group_id: GroupId,
        order: Order,
        clip_context: Stem,
        instance_id: InstanceId,
    ) -> Self {
        Self {
            elevation,
            pipeline: pipeline_id,
            group: group_id,
            order,
            clip_context,
            instance_id,
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct RemoveNode {
    pub(crate) pipeline_id: PipelineId,
    pub(crate) group_id: GroupId,
    pub(crate) instance_id: InstanceId,
}

impl RemoveNode {
    pub fn new(pipeline_id: PipelineId, group_id: GroupId, instance_id: InstanceId) -> Self {
        Self {
            pipeline_id,
            group_id,
            instance_id,
        }
    }
}

pub(crate) struct Nodes {
    pub(crate) updated: Vec<Node>,
    pub(crate) removed: Vec<RemoveNode>,
}

impl Nodes {
    pub(crate) fn new() -> Self {
        Self {
            updated: vec![],
            removed: vec![],
        }
    }
    pub(crate) fn update(&mut self, node: Node) {
        self.updated.push(node);
    }
    pub(crate) fn remove(&mut self, remove_node: RemoveNode) {
        self.removed.push(remove_node);
    }
}
