use crate::ash::render_packet::RenderPacket;
use crate::ash::renderer::{RenderPackage, RenderRecordBehavior};
use crate::ginkgo::Ginkgo;

pub enum RenderPhase {
    Opaque,
    Alpha(i32),
}
pub trait Render
where
    Self: Sized,
{
    type Resources;
    type RenderPackage;
    const RENDER_PHASE: RenderPhase;
    fn resources(ginkgo: &Ginkgo) -> Self::Resources;
    fn package(
        ginkgo: &Ginkgo,
        resources: &Self::Resources,
        render_packet: RenderPacket,
    ) -> Self::RenderPackage;
    fn prepare_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        package: &mut RenderPackage<Self>,
        render_packet: RenderPacket,
    );
    fn prepare_resources(
        resources: &mut Self::Resources,
        ginkgo: &Ginkgo,
        per_renderer_record_hook: &mut bool,
    );
    fn record_behavior() -> RenderRecordBehavior<Self>;
}
