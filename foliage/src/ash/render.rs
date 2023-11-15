use crate::ash::render_package::{
    RenderPackage, RenderPackageManager,
};
use crate::ash::render_packet::RenderPackets;
use crate::ginkgo::viewport::Viewport;
use crate::ginkgo::Ginkgo;
use std::hash::Hash;
use crate::ash::render_instructions::{RenderInstructions, RenderInstructionsRecorder};
pub trait Render
where
    Self: Sized,
{
    type Key: Hash + Eq + PartialEq;
    type RenderPackageResources;
    fn create(ginkgo: &Ginkgo) -> Self;
    fn prepare(
        pm: &mut RenderPackageManager<Self>,
        render_packets: Option<RenderPackets>,
        ginkgo: &Ginkgo,
    );
    fn record_package(
        &self,
        package: &RenderPackage<Self::RenderPackageResources>,
        recorder: RenderInstructionsRecorder,
        viewport: &Viewport,
    ) -> RenderInstructions;
}
