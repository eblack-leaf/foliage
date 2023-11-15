use std::collections::HashMap;
use anymap::AnyMap;
use crate::r_ash::render::{RenderId, RenderInstructionGroup};
use crate::r_ash::render_packet::RenderPacketPackage;

pub mod render;
pub mod render_packet;
pub(crate) struct Renderers(pub(crate) AnyMap);
pub(crate) struct Ash {
    pub(crate) package: Option<RenderPacketPackage>,
    pub(crate) instruction_groups: Vec<RenderInstructionGroup>,
    pub(crate) render_id_to_instruction_group: HashMap<RenderId, usize>,
    pub(crate) renderers: Renderers,
}
