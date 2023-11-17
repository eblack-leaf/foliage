use crate::ginkgo::Ginkgo;
use crate::r_ash::render::{
    Render, RenderId, RenderIdentification, RenderInstructionGroup, RenderInstructionHandle,
    RenderLeaflet, RenderPacketQueueHandler, RenderPhase, RendererHandler,
};
use crate::r_ash::render_packet::RenderPacketPackage;
use anymap::AnyMap;
use std::collections::HashMap;

pub mod render;
pub mod render_packet;
pub(crate) type RenderLeafletStorage = HashMap<RenderId, RenderLeaflet>;
pub(crate) struct Ash {
    pub(crate) render_packet_package: Option<RenderPacketPackage>,
    pub(crate) renderer_handler: RendererHandler,
    pub(crate) render_packet_queue_handler: RenderPacketQueueHandler,
    pub(crate) instruction_groups: InstructionGroups,
    pub(crate) render_leaflets: RenderLeafletStorage,
}
impl Ash {
    pub(crate) fn new() -> Self {
        Self {
            render_packet_package: None,
            renderer_handler: RendererHandler(AnyMap::new()),
            render_packet_queue_handler: RenderPacketQueueHandler(HashMap::new()),
            instruction_groups: InstructionGroups::default(),
            render_leaflets: RenderLeafletStorage::new(),
        }
    }
    pub(crate) fn establish(&mut self, ginkgo: &Ginkgo, render_leaflet: RenderLeafletStorage) {
        // renderer_handler::establish
        // instruction_group::establish
        // sort i_group by render_phase
    }
    pub(crate) fn extract(&mut self, package: RenderPacketPackage) {
        self.render_packet_package.replace(package);
    }
    pub(crate) fn prepare(&mut self, ginkgo: &Ginkgo) {
        // call prepare fns in leaflets for package
        // call prepare fns in leaflets for resources
    }
    pub(crate) fn record(&mut self, ginkgo: &Ginkgo) {
        // call record_fns in leaflets
    }
    pub(crate) fn render(&self, ginkgo: &Ginkgo) {
        // execute self.instruction_group.instructions()
    }
}
#[derive(Default)]
pub(crate) struct InstructionGroups {
    pub(crate) instruction_groups: Vec<(RenderId, RenderPhase, RenderInstructionGroup)>,
    pub(crate) render_id_to_instruction_group: HashMap<RenderId, usize>,
}
impl InstructionGroups {
    pub(crate) fn obtain<T: Render + 'static>(
        &mut self,
        id1: RenderId,
    ) -> &mut RenderInstructionGroup {
        let index = *self.render_id_to_instruction_group.get(&T::id()).unwrap();
        &mut self.instruction_groups.get_mut(index).unwrap().2
    }
    pub(crate) fn instructions(&self) -> Vec<RenderInstructionHandle> {
        let mut instructions = vec![];
        for group in self.instruction_groups.iter() {
            instructions.extend(group.2 .0.clone());
        }
        instructions
    }
    // establish group per render_id on ash startup, read to slot in record
}
