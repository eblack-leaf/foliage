use crate::ash::identification::RenderId;
use crate::ash::render::RenderPhase;
use crate::ginkgo::Ginkgo;
use std::collections::HashMap;
use std::rc::Rc;

pub struct RenderInstructionsRecorder<'a>(pub wgpu::RenderBundleEncoder<'a>);

impl<'a> RenderInstructionsRecorder<'a> {
    pub(crate) fn new(ginkgo: &'a Ginkgo) -> Self {
        Self(
            ginkgo
                .device
                .as_ref()
                .unwrap()
                .create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                    label: Some("render-bundle"),
                    color_formats: &ginkgo.color_attachment_format(),
                    depth_stencil: ginkgo.render_bundle_depth_stencil(),
                    sample_count: ginkgo.msaa_samples(),
                    multiview: None,
                }),
        )
    }
    pub fn finish(self) -> RenderInstructionHandle {
        RenderInstructionHandle(Rc::new(self.0.finish(&wgpu::RenderBundleDescriptor {
            label: Some("render-bundle-desc"),
        })))
    }
}

#[derive(Clone)]
pub struct RenderInstructionHandle(pub(crate) Rc<wgpu::RenderBundle>);

#[derive(Default)]
pub(crate) struct RenderInstructionGroup(pub(crate) Vec<RenderInstructionHandle>);

#[derive(Default)]
pub(crate) struct InstructionGroups {
    pub(crate) instruction_groups: Vec<(RenderId, RenderPhase, RenderInstructionGroup)>,
    pub(crate) instructions: Vec<RenderInstructionHandle>,
    pub(crate) updated: bool,
}

impl InstructionGroups {
    pub(crate) fn obtain(&mut self, id: &RenderId) -> &mut RenderInstructionGroup {
        let index = self.index_of(id);
        &mut self.instruction_groups.get_mut(index).unwrap().2
    }
    pub(crate) fn index_of(&mut self, id: &RenderId) -> usize {
        let mut index = 0;
        for (r_id, _, _) in self.instruction_groups.iter() {
            if r_id == id {
                return index;
            }
            index += 1;
        }
        index
    }
    pub(crate) fn instructions(&mut self) -> &Vec<RenderInstructionHandle> {
        if self.updated {
            self.instructions.clear();
            for group in self.instruction_groups.iter() {
                self.instructions.extend(group.2 .0.clone());
            }
            self.updated = false;
        }
        &self.instructions
    }

    pub(crate) fn establish(&mut self, id: RenderId, phase: RenderPhase) {
        self.instruction_groups
            .push((id, phase, RenderInstructionGroup::default()));
    }
}
