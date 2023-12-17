use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, InterfaceContext};
use crate::differential::Despawn;
use crate::scene::align::{SceneAlignment, SceneAnchor};
use crate::scene::{Scene, SceneSpawn};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Commands;
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Component)]
pub struct SceneBinding(pub u32);

impl From<u32> for SceneBinding {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
pub struct SceneNodeEntry {
    entity: Entity,
    is_scene: bool,
}
impl SceneNodeEntry {
    pub(crate) fn new(entity: Entity, is_scene: bool) -> Self {
        Self { entity, is_scene }
    }
    pub fn entity(&self) -> Entity {
        self.entity
    }
    pub fn is_scene(&self) -> bool {
        self.is_scene
    }
}
#[derive(Component, Default)]
pub struct SceneNodes(HashMap<SceneBinding, SceneNodeEntry>);

impl SceneNodes {
    pub(crate) fn set_anchor_non_scene(&self, new_anchor: SceneAnchor, cmd: &mut Commands) {
        for (_, entry) in self.0.iter() {
            if !entry.is_scene {
                cmd.entity(entry.entity).insert(new_anchor);
            }
        }
    }
    pub fn nodes(&self) -> &HashMap<SceneBinding, SceneNodeEntry> {
        &self.0
    }
    pub fn get<SB: Into<SceneBinding>>(&self, binding: SB) -> &SceneNodeEntry {
        self.0.get(&binding.into()).unwrap()
    }
    pub(crate) fn despawn_non_scene(&self, cmd: &mut Commands) {
        for (_, entry) in self.0.iter() {
            if !entry.is_scene {
                cmd.entity(entry.entity).insert(Despawn::signal_despawn());
            }
        }
    }
}

pub struct SceneBinder {
    anchor: SceneAnchor,
    this: Entity,
    pub(crate) nodes: SceneNodes,
}

impl SceneBinder {
    pub(crate) fn new(anchor: SceneAnchor, this: Entity) -> Self {
        Self {
            anchor,
            this,
            nodes: SceneNodes::default(),
        }
    }
}

impl SceneBinder {
    pub fn bind<B: Bundle, SB: Into<SceneBinding>, SA: Into<SceneAlignment>>(
        &mut self,
        binding: SB,
        alignment: SA,
        b: B,
        cmd: &mut Commands,
    ) {
        let sb = binding.into();
        let entity = cmd
            .spawn(b)
            .insert(SceneBind::new(alignment.into(), sb, self.anchor))
            .id();
        self.nodes.0.insert(sb, SceneNodeEntry::new(entity, false));
    }
    pub fn bind_scene<S: Scene>(
        &mut self,
        binding: SceneBinding,
        alignment: SceneAlignment,
        area: Area<InterfaceContext>,
        args: &S::Args<'_>,
        external_args: &S::ExternalResources<'_>,
        cmd: &mut Commands,
    ) {
        let anchor = SceneAnchor(Coordinate::new(
            Section::new(Position::default(), area),
            Layer::default(),
        ));
        let entity = cmd.spawn_scene::<S>(anchor, args, external_args, SceneRoot::new(self.this));
        cmd.entity(entity).insert(alignment).insert(anchor.0);
        self.nodes
            .0
            .insert(binding, SceneNodeEntry::new(entity, true));
    }
}

#[derive(Default, Copy, Clone, Component)]
pub struct SceneRoot {
    pub(crate) current: Option<Entity>,
    pub(crate) old: Option<Entity>,
}

impl SceneRoot {
    pub fn new(current: Entity) -> Self {
        Self {
            current: Some(current),
            old: None,
        }
    }
    pub fn current(&self) -> Option<Entity> {
        self.current
    }
    pub fn change(&mut self, new: Entity) {
        if let Some(current) = self.current {
            self.old.replace(current);
        }
        self.current.replace(new);
    }
}

#[derive(Bundle)]
pub(crate) struct SceneBind {
    alignment: SceneAlignment,
    binding: SceneBinding,
    anchor: SceneAnchor,
    visibility: SceneVisibility,
}

impl SceneBind {
    pub(crate) fn new(
        alignment: SceneAlignment,
        binding: SceneBinding,
        anchor: SceneAnchor,
    ) -> Self {
        Self {
            alignment,
            binding,
            anchor,
            visibility: SceneVisibility::default(),
        }
    }
}

#[derive(Component, Copy, Clone)]
pub(crate) struct SceneVisibility(pub bool);

impl Default for SceneVisibility {
    fn default() -> Self {
        Self(true)
    }
}
