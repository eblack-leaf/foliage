use bevy_ecs::component::Component;
use std::collections::HashMap;
use bevy_ecs::entity::Entity;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Commands, Query};
use bevy_ecs::system::{ResMut, Resource};
use serde::{Deserialize, Serialize};
use crate::differential::Despawn;
use crate::scene::align::{AlignmentAnchor, AlignmentCoordinate};
use crate::scene::chain::{BundleChain, ChainedBundle};
use crate::scene::Sceneable;

#[derive(Component, Copy, Clone)]
pub struct SceneVisibility(pub bool);

// TODO incorporate into visibility check
impl Default for SceneVisibility {
    fn default() -> Self {
        SceneVisibility(true)
    }
}

#[derive(Bundle)]
pub struct SceneBind {
    alignment: AlignmentCoordinate,
    anchor: AlignmentAnchor,
    binding: SceneBinding,
    visibility: SceneVisibility,
}

impl SceneBind {
    pub fn new(ac: AlignmentCoordinate, anchor: AlignmentAnchor, binding: SceneBinding) -> Self {
        Self {
            alignment: ac,
            anchor,
            binding,
            visibility: SceneVisibility::default(),
        }
    }
}

#[derive(Component, Default)]
pub struct SceneNodes(pub HashMap<SceneBinding, Entity>, Option<Entity>);

impl SceneNodes {
    pub(crate) fn new(parent: Option<Entity>) -> Self {
        Self(HashMap::new(), parent)
    }
    pub fn release(&mut self, cmd: &mut Commands) {
        self.0.drain().for_each(|n| {
            cmd.entity(n.1).insert(Despawn::new(true));
        });
    }
    pub fn bind<T: Bundle>(&mut self) {
        // spawn bundle and bind to nodes
    }
    /// If not called when using a scene, this will not spawn scene elements,
    /// but instead the given bundle
    pub fn bind_scene<S: Sceneable>(&mut self) {
        // if scene use .spawn_scene to get entity and bind that to nodes
        // insert parent from self.1
    }
}
#[derive(Resource)]
pub struct SceneCompositor {
    // when added -> scenes add parent to
}

#[derive(
    Component, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Default,
)]
pub struct SceneBinding(pub u32);

impl From<u32> for SceneBinding {
    fn from(value: u32) -> Self {
        SceneBinding(value)
    }
}