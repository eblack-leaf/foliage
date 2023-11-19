use crate::ash::render_packet::RenderPacket;
use crate::ash::renderer::{RenderPackage, RenderRecordBehavior};
use crate::ginkgo::Ginkgo;
use std::cmp::Ordering;

pub enum RenderPhase {
    Opaque,
    Alpha(i32),
}

impl PartialEq<Self> for RenderPhase {
    fn eq(&self, other: &Self) -> bool {
        return match self {
            RenderPhase::Opaque => match other {
                RenderPhase::Opaque => true,
                RenderPhase::Alpha(_) => false,
            },
            RenderPhase::Alpha(priority_one) => match other {
                RenderPhase::Opaque => false,
                RenderPhase::Alpha(priority_two) => priority_one == priority_two,
            },
        };
    }
}

impl PartialOrd for RenderPhase {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return match self {
            RenderPhase::Opaque => match other {
                RenderPhase::Opaque => Some(Ordering::Equal),
                RenderPhase::Alpha(_) => Some(Ordering::Less),
            },
            RenderPhase::Alpha(priority_one) => match other {
                RenderPhase::Opaque => Some(Ordering::Greater),
                RenderPhase::Alpha(priority_two) => {
                    if priority_one < priority_two {
                        Some(Ordering::Less)
                    } else if priority_two < priority_one {
                        Some(Ordering::Greater)
                    } else {
                        Some(Ordering::Equal)
                    }
                }
            },
        };
    }
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
