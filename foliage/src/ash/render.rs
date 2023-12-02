use std::cmp::Ordering;

use bevy_ecs::prelude::Entity;

use crate::ash::instruction::RenderRecordBehavior;
use crate::ash::render_packet::RenderPacket;
use crate::ash::renderer::RenderPackage;
use crate::ginkgo::Ginkgo;

#[derive(Copy, Clone)]
pub enum RenderPhase {
    Opaque,
    Alpha(i32),
}

impl RenderPhase {
    pub const fn value(&self) -> i32 {
        match self {
            RenderPhase::Opaque => 0,
            RenderPhase::Alpha(priority) => *priority,
        }
    }
}

impl PartialEq<Self> for RenderPhase {
    fn eq(&self, other: &Self) -> bool {
        match self {
            RenderPhase::Opaque => match other {
                RenderPhase::Opaque => true,
                RenderPhase::Alpha(_) => false,
            },
            RenderPhase::Alpha(priority_one) => match other {
                RenderPhase::Opaque => false,
                RenderPhase::Alpha(priority_two) => priority_one == priority_two,
            },
        }
    }
}

impl PartialOrd for RenderPhase {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            RenderPhase::Opaque => match other {
                RenderPhase::Opaque => Some(Ordering::Equal),
                RenderPhase::Alpha(_) => Some(Ordering::Less),
            },
            RenderPhase::Alpha(priority_one) => match other {
                RenderPhase::Opaque => Some(Ordering::Greater),
                RenderPhase::Alpha(priority_two) => Some(priority_one.cmp(priority_two)),
            },
        }
    }
}

pub trait Render
where
    Self: Sized,
{
    type Resources;
    type RenderPackage;
    const RENDER_PHASE: RenderPhase;
    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources;
    fn create_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        render_packet: RenderPacket,
    ) -> Self::RenderPackage;
    fn on_package_removal(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
        package: RenderPackage<Self>,
    );
    fn prepare_package(
        ginkgo: &Ginkgo,
        resources: &mut Self::Resources,
        entity: Entity,
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
