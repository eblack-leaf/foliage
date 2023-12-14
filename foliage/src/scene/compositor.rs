use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::InterfaceContext;
use crate::differential::Despawn;
use crate::ginkgo::viewport::ViewportHandle;
use crate::scene::align::SceneAnchor;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Commands, Component, DetectChanges, Resource};
use bevy_ecs::system::{Query, Res, ResMut};
use std::collections::{HashMap, HashSet};

#[derive(Resource)]
pub struct SceneCompositor {
    pub(crate) slots: SceneCompositorSlotStorage,
    pub(crate) layout: SceneCompositorLayout,
}
fn remake_slots(
    mut compositor: ResMut<SceneCompositor>,
    viewport_handle: Res<ViewportHandle>,
    mut cmd: Commands,
    mut scenes: Query<(
        &mut SceneAnchor,
        &mut Position<InterfaceContext>,
        &mut Area<InterfaceContext>,
    )>,
) {
    if viewport_handle.is_changed() {
        let mut slots = compositor.layout.slots(viewport_handle.section());
        for (key, slot) in compositor.slots.iter_mut() {
            if slots.get(key).is_none() {
                if let Some(entity) = slot.scene.take() {
                    cmd.entity(entity).insert(Despawn::signal_despawn());
                }
            } else {
                if let Some(entity) = slot.scene {
                    slots.get_mut(key).unwrap().scene.replace(entity);
                }
            }
        }
    }
}
fn assign_to_slot() {}
fn compositor_changed(
    mut compositor: ResMut<SceneCompositor>,
    mut query: Query<(
        &SceneCompositorSlotHandle,
        &mut Position<InterfaceContext>,
        &mut Area<InterfaceContext>,
        &mut SceneAnchor,
    )>,
) {
    if compositor.is_changed() {
        for (handle, mut pos, mut area, mut anchor) in query.iter_mut() {
            if let Some(slot) = compositor.slots.get_mut(handle) {
                *pos = slot.section.position;
            }
        }
    }
}
#[derive(Hash, Eq, PartialEq, Copy, Clone, Component)]
pub struct SceneCompositorSlotHandle(pub i32);
impl From<i32> for SceneCompositorSlotHandle {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
pub(crate) struct SceneCompositorSlot {
    pub(crate) section: Section<InterfaceContext>,
    pub(crate) scene: Option<Entity>,
}
pub struct SceneCompositorLayoutRange {
    pub horizontal: (u32, u32),
    pub vertical: (u32, u32),
}
pub const MOBILE_PORTRAIT: SceneCompositorLayoutRange = SceneCompositorLayoutRange {
    horizontal: (0, 440),
    vertical: (0, 800),
};
pub const MOBILE_LANDSCAPE: SceneCompositorLayoutRange = SceneCompositorLayoutRange {
    horizontal: (0, 800),
    vertical: (0, 440),
};
pub const TABLET_PORTRAIT: SceneCompositorLayoutRange = SceneCompositorLayoutRange {
    horizontal: (441, 1000),
    vertical: (0, 800),
};
pub struct SceneCompositorLayout {}
pub(crate) type SceneCompositorSlotStorage =
    HashMap<SceneCompositorSlotHandle, SceneCompositorSlot>;
impl SceneCompositorLayout {
    pub(crate) fn slots(&self, section: Section<InterfaceContext>) -> SceneCompositorSlotStorage {
        todo!()
    }
}
