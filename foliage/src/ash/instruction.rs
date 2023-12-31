use crate::ash::identification::RenderId;
use crate::ash::render::{Render, RenderPhase};
use crate::ash::renderer::{RenderPackage, RenderPackageStorage};
use crate::ginkgo::Ginkgo;
use std::rc::Rc;

pub struct RenderInstructionsRecorder<'a>(pub wgpu::RenderBundleEncoder<'a>);

impl<'a> RenderInstructionsRecorder<'a> {
    pub(crate) fn new(ginkgo: &'a Ginkgo) -> Self {
        tracing::trace!("acquiring-render-instruction-recorder");
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
        tracing::trace!("finishing-render-instruction-recorder");
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
        tracing::trace!("obtaining-render-instruction-group");
        let index = self.index_of(id);
        &mut self.instruction_groups.get_mut(index).unwrap().2
    }
    pub(crate) fn index_of(&mut self, id: &RenderId) -> usize {
        tracing::trace!("getting-index-of-render-instruction-group");
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
        tracing::trace!("getting-render-instructions");
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
        tracing::trace!("establishing-instruction-groups");
        self.instruction_groups
            .push((id, phase, RenderInstructionGroup::default()));
    }
}

pub enum RenderRecordBehavior<T: Render> {
    PerRenderer(PerRendererRecordFn<T>),
    PerPackage(PerPackageRecordFn<T>),
}

impl<T: Render> Default for RenderPackageStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) type PerRendererRecordFn<T> = Box<
    fn(&<T as Render>::Resources, RenderInstructionsRecorder) -> Option<RenderInstructionHandle>,
>;
pub(crate) type PerPackageRecordFn<T> = Box<
    fn(
        &<T as Render>::Resources,
        &mut RenderPackage<T>,
        RenderInstructionsRecorder,
    ) -> Option<RenderInstructionHandle>,
>;
