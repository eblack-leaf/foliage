use crate::r_ash::render::{RenderId, RenderInstructionGroup, RenderPhase};
use crate::r_ash::render_packet::RenderPacketPackage;
use anymap::AnyMap;
use std::collections::HashMap;

pub mod render;
pub mod render_packet;
pub(crate) struct Renderers(pub(crate) AnyMap);
pub(crate) struct Ash {
    pub(crate) package: Option<RenderPacketPackage>,
    pub(crate) opaque_instruction_groups: Vec<RenderInstructionGroup>,
    pub(crate) alpha_instruction_groups: Vec<(RenderPhase, RenderInstructionGroup)>,
    pub(crate) render_id_to_instruction_group: HashMap<RenderId, usize>,
    pub(crate) renderers: Renderers,
}
pub(crate) struct AshFns {
    // map render_id to Renderer::prepare call wrapped
    // map render_id to Renderer::record call wrapped
    //
}
