use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use crate::differential::Despawn;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Commands, Component, DetectChanges, RemovedComponents, Resource};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::{Query, ResMut};
use indexmap::IndexSet;
use std::collections::{HashMap, HashSet};

#[derive(Resource, Default)]
pub struct SceneCompositor {
    pub(crate) anchors: HashMap<Entity, SceneAnchor>,
    pub(crate) subscenes: HashMap<Entity, HashSet<Entity>>,
    pub(crate) roots: HashSet<Entity>,
}
impl SceneCompositor {
    fn subscene_resolve(&self, root: Entity) -> IndexSet<Entity> {
        let mut subscenes = IndexSet::new();
        if let Some(ss) = self.subscenes.get(&root) {
            for e in ss.iter() {
                subscenes.extend(self.subscene_resolve(*e));
            }
        }
        subscenes
    }
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
#[derive(Bundle, Copy, Clone)]
pub struct SceneAlignment {
    pos: PositionAlignment,
    layer: LayerAlignment,
}
#[derive(Component, Copy, Clone)]
pub struct PositionAlignment {
    pub horizontal: SceneAlignmentPoint,
    pub vertical: SceneAlignmentPoint,
}
#[derive(Component, Copy, Clone)]
pub struct LayerAlignment(pub CoordinateUnit);
impl LayerAlignment {
    pub fn calc_layer(&self, layer: Layer) -> Layer {
        layer + self.0.into()
    }
}
impl PositionAlignment {
    pub fn calc_pos(
        &self,
        anchor: SceneAnchor,
        node_area: Area<InterfaceContext>,
    ) -> Position<InterfaceContext> {
        todo!()
    }
}
pub(crate) fn resolve_anchor(
    mut compositor: ResMut<SceneCompositor>,
    mut query: Query<(
        &mut SceneAnchor,
        &SceneRoot,
        &mut Position<InterfaceContext>,
        &Area<InterfaceContext>,
        &PositionAlignment,
        &mut Layer,
        &LayerAlignment,
    )>,
    mut cmd: Commands,
) {
    if compositor.is_changed() {
        for root in compositor.roots.clone() {
            let dependents = compositor.subscene_resolve(*root);
            for dep in dependents {
                if let Ok((
                    mut anchor,
                    root,
                    mut pos,
                    mut area,
                    mut pos_align,
                    mut layer,
                    layer_align,
                )) = query.get_mut(dep)
                {
                    let root_anchor = *compositor.anchors.get(&root.current.unwrap()).unwrap();
                    *pos = pos_align.calc_pos(root_anchor, *area);
                    *layer = layer_align.calc_layer(root_anchor.0.layer);
                    if *pos != anchor.0.section.position || *layer != anchor.0.layer {
                        let new_anchor = SceneAnchor(Coordinate::new(
                            Section::new(*pos, *area),
                            Layer::new(layer.z),
                        ));
                        cmd.entity(dep).insert(new_anchor);
                        *anchor = new_anchor;
                        compositor.anchors.insert(dep, new_anchor);
                    }
                }
            }
        }
    }
}
pub(crate) fn register_root(
    mut compositor: ResMut<SceneCompositor>,
    mut query: Query<
        (Entity, &mut SceneRoot, &SceneAnchor, &Despawn),
        Or<(Changed<SceneAnchor>, Changed<SceneRoot>)>,
    >,
    mut removed: RemovedComponents<SceneRoot>,
) {
    for remove in removed.read() {
        compositor.roots.remove(&remove);
    }
    for (entity, mut root, anchor, despawn) in query.iter_mut() {
        compositor.anchors.insert(entity, *anchor);
        if compositor.subscenes.get(&entity).is_none() {
            compositor.subscenes.insert(entity, HashSet::new());
        }
        if let Some(old) = root.old.take() {
            // deregister
            if let Some(ss) = compositor.subscenes.get_mut(&old) {
                ss.remove(&entity);
            }
        }
        if let Some(current) = root.current {
            // add to subscenes
            if !despawn.should_despawn() {
                if compositor.subscenes.get(&current).is_none() {
                    compositor.subscenes.insert(current, HashSet::new());
                }
                compositor
                    .subscenes
                    .get_mut(&current)
                    .unwrap()
                    .insert(entity);
            } else {
                if let Some(ss) = compositor.subscenes.get_mut(&current) {
                    ss.remove(&entity);
                }
            }
        } else {
            if !despawn.should_despawn() {
                compositor.roots.insert(entity);
            } else {
                compositor.roots.remove(&entity);
            }
        }
    }
}
pub(crate) fn calc_alignments(
    mut pos_aligned: Query<
        (
            &SceneAnchor,
            &mut Position<InterfaceContext>,
            &Area<InterfaceContext>,
            &PositionAlignment,
        ),
        Or<(
            Changed<PositionAlignment>,
            Changed<SceneAnchor>,
            Changed<Position<InterfaceContext>>,
        )>,
    >,
    mut layer_aligned: Query<
        (&SceneAnchor, &mut Layer, &LayerAlignment),
        Or<(
            Changed<LayerAlignment>,
            Changed<Layer>,
            Changed<SceneAnchor>,
        )>,
    >,
) {
    for (anchor, mut pos, area, alignment) in pos_aligned.iter_mut() {
        *pos = alignment.calc_pos(*anchor, *area);
    }
    for (anchor, mut layer, alignment) in layer_aligned.iter_mut() {
        *layer = alignment.calc_layer(anchor.0.layer);
    }
}
impl<SAP: Into<SceneAlignmentPoint>> From<(SAP, SAP, i32)> for SceneAlignment {
    fn from(value: (SAP, SAP, i32)) -> Self {
        SceneAlignment {
            pos: PositionAlignment {
                horizontal: value.0.into(),
                vertical: value.1.into(),
            },
            layer: LayerAlignment(value.2 as CoordinateUnit),
        }
    }
}
impl<SAP: Into<SceneAlignmentPoint>> From<(SAP, SAP, f32)> for SceneAlignment {
    fn from(value: (SAP, SAP, f32)) -> Self {
        SceneAlignment {
            pos: PositionAlignment {
                horizontal: value.0.into(),
                vertical: value.1.into(),
            },
            layer: LayerAlignment(value.2),
        }
    }
}
impl<SAP: Into<SceneAlignmentPoint>> From<(SAP, SAP, u32)> for SceneAlignment {
    fn from(value: (SAP, SAP, u32)) -> Self {
        SceneAlignment {
            pos: PositionAlignment {
                horizontal: value.0.into(),
                vertical: value.1.into(),
            },
            layer: LayerAlignment(value.2 as CoordinateUnit),
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
    pub fn bind_scene<
        'a,
        S: Scene,
        SB: Into<SceneBinding>,
        SA: Into<SceneAlignment>,
        A: Into<SceneAnchor>,
    >(
        &mut self,
        binding: SB,
        alignment: SA,
        anchor: A,
        args: &S::Args<'a>,
        cmd: &mut Commands,
    ) {
        let anchor = anchor.into();
        let entity = cmd.spawn_scene::<S>(anchor, args, self.1);
        cmd.entity(entity).insert(alignment.into()).insert(anchor.0);
        self.2 .0.insert(binding.into(), entity);
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
