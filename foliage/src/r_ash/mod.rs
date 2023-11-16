use crate::r_ash::render::{
    EntityToRenderPackageHandler, RecordFns, RenderId, RenderInstructionGroup,
    RenderPackageListHandler, RenderPacketQueueHandler, RenderPhase,
};
use crate::r_ash::render_packet::RenderPacketPackage;
use render::RenderResourceHandler;
use std::collections::HashMap;

pub mod render;
pub mod render_packet;

pub(crate) struct Ash {
    pub(crate) render_packet_package: Option<RenderPacketPackage>,
    pub(crate) resource_handler: RenderResourceHandler,
    pub(crate) render_package_handler: RenderPackageListHandler,
    pub(crate) entity_to_render_package_handler: EntityToRenderPackageHandler,
    pub(crate) render_packet_queue_handler: RenderPacketQueueHandler,
    pub(crate) instruction_groups: InstructionGroups,
}
impl Ash {
    pub(crate) fn new() -> Self {
        todo!()
    }
}
#[derive(Default)]
pub(crate) struct InstructionGroups {
    pub(crate) opaque_instruction_groups: Vec<RenderInstructionGroup>,
    pub(crate) alpha_instruction_groups: Vec<(RenderPhase, RenderInstructionGroup)>,
    pub(crate) render_id_to_instruction_group: HashMap<RenderId, usize>,
}
impl InstructionGroups {
    // establish group per render_id on ash startup, read to slot in record
}
pub(crate) struct AshFns {
    // map render_id to Renderer::prepare call wrapped
    // map render_id to Renderer::record call wrapped
    record_fns: RecordFns,
}
