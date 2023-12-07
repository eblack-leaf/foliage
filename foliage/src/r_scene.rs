use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::location::Location;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use crate::differential::Despawn;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Commands, Component};
use std::collections::{HashMap, HashSet};

pub struct SceneCompositor {
    pub(crate) anchors: HashMap<Entity, SceneAnchor>,
    pub(crate) subscenes: HashMap<Entity, HashSet<Entity>>,
}
#[derive(Copy, Clone, Component)]
pub struct SceneAnchor(pub Coordinate<InterfaceContext>);
#[derive(Copy, Clone)]
pub enum SceneAlignmentBias {
    Near,
    Center,
    Far,
}
#[derive(Copy, Clone)]
pub struct SceneAlignmentPoint {
    pub point: SceneAlignmentBias,
    pub offset: CoordinateUnit,
}
#[derive(Component, Copy, Clone)]
pub struct SceneAlignment {
    pub horizontal: SceneAlignmentPoint,
    pub vertical: SceneAlignmentPoint,
    pub layer: CoordinateUnit,
}
impl SceneAlignment {
    pub fn calc(
        &self,
        anchor: SceneAnchor,
        node_area: Area<InterfaceContext>,
    ) -> Location<InterfaceContext> {
        todo!()
    }
}
impl<SAP: Into<SceneAlignmentPoint>> From<(SAP, SAP, i32)> for SceneAlignment {
    fn from(value: (SAP, SAP, i32)) -> Self {
        SceneAlignment {
            horizontal: value.0.into(),
            vertical: value.1.into(),
            layer: value.2 as CoordinateUnit,
        }
    }
}
impl<SAP: Into<SceneAlignmentPoint>> From<(SAP, SAP, f32)> for SceneAlignment {
    fn from(value: (SAP, SAP, f32)) -> Self {
        SceneAlignment {
            horizontal: value.0.into(),
            vertical: value.1.into(),
            layer: value.2,
        }
    }
}
impl<SAP: Into<SceneAlignmentPoint>> From<(SAP, SAP, u32)> for SceneAlignment {
    fn from(value: (SAP, SAP, u32)) -> Self {
        SceneAlignment {
            horizontal: value.0.into(),
            vertical: value.1.into(),
            layer: value.2 as CoordinateUnit,
        }
    }
}
pub trait SceneAligner {
    fn near(self) -> SceneAlignmentPoint;
    fn center(self) -> SceneAlignmentPoint;
    fn far(self) -> SceneAlignmentPoint;
}
impl SceneAligner for CoordinateUnit {
    fn near(self) -> SceneAlignmentPoint {
        SceneAlignmentPoint {
            point: SceneAlignmentBias::Near,
            offset: self,
        }
    }
    fn center(self) -> SceneAlignmentPoint {
        SceneAlignmentPoint {
            point: SceneAlignmentBias::Center,
            offset: self,
        }
    }
    fn far(self) -> SceneAlignmentPoint {
        SceneAlignmentPoint {
            point: SceneAlignmentBias::Far,
            offset: self,
        }
    }
}
impl SceneAligner for i32 {
    fn near(self) -> SceneAlignmentPoint {
        SceneAlignmentPoint {
            point: SceneAlignmentBias::Near,
            offset: self as CoordinateUnit,
        }
    }
    fn center(self) -> SceneAlignmentPoint {
        SceneAlignmentPoint {
            point: SceneAlignmentBias::Center,
            offset: self as CoordinateUnit,
        }
    }
    fn far(self) -> SceneAlignmentPoint {
        SceneAlignmentPoint {
            point: SceneAlignmentBias::Far,
            offset: self as CoordinateUnit,
        }
    }
}
#[derive(Component)]
pub struct SceneNodes(pub HashMap<SceneBinding, Entity>);
pub struct SceneBinder(SceneAnchor, SceneRoot, SceneNodes);
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
            .insert(SceneBind::new(alignment.into(), sb, self.0))
            .id();
        self.2 .0.insert(sb, entity);
    }
    pub fn bind_scene<'a, S: Scene, SB: Into<SceneBinding>, SA: Into<SceneAlignment>>(
        &mut self,
        binding: SB,
        alignment: SA,
        args: &S::Args<'a>,
        cmd: &mut Commands,
    ) {
        let entity = cmd.spawn_scene::<S>(self.0, args, self.1);
        cmd.entity(entity).insert(alignment.into());
        self.2 .0.insert(binding.into(), entity);
    }
}
#[derive(Default, Copy, Clone, Component)]
pub struct SceneRoot(pub Option<Entity>);
#[derive(Component)]
pub struct SceneBind {
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
pub struct SceneVisibility(pub bool);
impl Default for SceneVisibility {
    fn default() -> Self {
        Self(true)
    }
}
#[derive(Bundle)]
pub struct SceneBundle {
    anchor: SceneAnchor,
    nodes: SceneNodes,
    root: SceneRoot,
    visibility: SceneVisibility,
    despawn: Despawn,
}
impl SceneBundle {
    pub fn new(anchor: SceneAnchor, nodes: SceneNodes, root: SceneRoot) -> Self {
        Self {
            anchor,
            nodes,
            root,
            visibility: SceneVisibility::default(),
            despawn: Despawn::default(),
        }
    }
}
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct SceneBinding(pub u32);
impl From<u32> for SceneBinding {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
pub trait Scene
where
    Self: Bundle,
{
    type Args<'a>;
    fn bind_nodes<'a>(
        cmd: &mut Commands,
        anchor: SceneAnchor,
        args: &Self::Args<'a>,
        binder: &mut SceneBinder,
    ) -> Self;
}
pub trait SceneSpawn {
    fn spawn_scene<'a, S: Scene>(
        &mut self,
        anchor: SceneAnchor,
        args: &S::Args<'a>,
        root: SceneRoot,
    ) -> Entity;
}
impl<'a, 'b> SceneSpawn for Commands<'a, 'b> {
    fn spawn_scene<'c, S: Scene>(
        &mut self,
        anchor: SceneAnchor,
        args: &S::Args<'c>,
        root: SceneRoot,
    ) -> Entity {
        let this = self.spawn_empty().id();
        let mut binder = SceneBinder(anchor, root, SceneNodes(HashMap::new()));
        let bundle = S::bind_nodes(self, anchor, args, &mut binder);
        self.entity(this)
            .insert(bundle)
            .insert(SceneBundle::new(anchor, binder.2, root));
        this
    }
}
