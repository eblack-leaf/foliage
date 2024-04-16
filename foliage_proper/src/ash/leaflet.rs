use std::collections::HashMap;

use crate::ash::Ash;
use crate::ash::identification::{RenderId, RenderIdentification};
use crate::ash::instruction::RenderInstructionGroup;
use crate::ash::render::Render;
use crate::ash::render_packet::RenderPacketPackage;
use crate::ash::renderer::RendererStorage;
use crate::ginkgo::Ginkgo;

pub(crate) struct RenderLeaflet {
    pub(crate) register_fn: Box<fn(&mut Ash, &Ginkgo)>,
    pub(crate) prepare_packages_fn:
        Box<fn(&mut RendererStorage, &Ginkgo, &mut RenderPacketPackage)>,
    pub(crate) prepare_resources_fn: Box<fn(&mut RendererStorage, &Ginkgo)>,
    pub(crate) record_fn: Box<fn(&mut RendererStorage, &Ginkgo) -> bool>,
    pub(crate) instruction_fetch_fn: Box<fn(&mut RendererStorage) -> &RenderInstructionGroup>,
}

impl RenderLeaflet {
    pub(crate) fn prepare_packages_wrapper<T: Render + 'static>(
        renderer_handler: &mut RendererStorage,
        ginkgo: &Ginkgo,
        queue_handler: &mut RenderPacketPackage,
    ) {
        if let Some(queue) = queue_handler.obtain::<T>() {
            renderer_handler
                .obtain_mut::<T>()
                .prepare_packages(ginkgo, queue, &queue_handler.1);
        }
    }
    pub(crate) fn prepare_resources_wrapper<T: Render + 'static>(
        renderer_handler: &mut RendererStorage,
        ginkgo: &Ginkgo,
    ) {
        renderer_handler
            .obtain_mut::<T>()
            .resource_preparation(ginkgo);
    }
    pub(crate) fn record_wrapper<T: Render + 'static>(
        renderer_handler: &mut RendererStorage,
        ginkgo: &Ginkgo,
    ) -> bool {
        renderer_handler.obtain_mut::<T>().record(ginkgo)
    }
    pub(crate) fn register<T: Render + 'static>(ash: &mut Ash, ginkgo: &Ginkgo) {
        ash.renderer_handler.establish::<T>(ginkgo);
        ash.instruction_groups
            .establish(T::render_id(), T::RENDER_PHASE);
    }
    pub(crate) fn instruction_fetch<T: Render + 'static>(
        renderer_handler: &mut RendererStorage,
    ) -> &RenderInstructionGroup {
        &renderer_handler.obtain::<T>().instructions
    }
    pub(crate) fn leaf_fn<T: Render + 'static>() -> Self {
        Self {
            register_fn: Box::new(Self::register::<T>),
            prepare_packages_fn: Box::new(Self::prepare_packages_wrapper::<T>),
            prepare_resources_fn: Box::new(Self::prepare_resources_wrapper::<T>),
            record_fn: Box::new(Self::record_wrapper::<T>),
            instruction_fetch_fn: Box::new(Self::instruction_fetch::<T>),
        }
    }
}

pub(crate) type RenderLeafletStorage = HashMap<RenderId, RenderLeaflet>;