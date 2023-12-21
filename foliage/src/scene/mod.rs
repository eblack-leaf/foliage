pub mod align;
pub mod bind;

use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, InterfaceContext};
use crate::differential::Despawn;
use crate::elm::leaf::Tag;
use align::{LayerAlignment, PositionAlignment, SceneAnchor};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Commands, DetectChanges, ParamSet, RemovedComponents, Resource};
use bevy_ecs::query::{Changed, Or, With};
use bevy_ecs::system::{Query, ResMut, SystemParam, SystemParamItem};
use bind::{SceneBinder, SceneNodes, SceneRoot, SceneVisibility};
use indexmap::IndexSet;
use std::collections::{HashMap, HashSet};

#[derive(Resource, Default)]
pub struct SceneCoordinator {
    pub(crate) anchors: HashMap<Entity, SceneAnchor>,
    pub(crate) subscenes: HashMap<Entity, HashSet<Entity>>,
    pub(crate) roots: HashSet<Entity>,
    pub(crate) removes: HashSet<Entity>,
}
impl SceneCoordinator {
    fn subscene_resolve(&self, root: Entity) -> IndexSet<Entity> {
        let mut subscenes = IndexSet::new();
        if let Some(ss) = self.subscenes.get(&root) {
            for e in ss.iter() {
                subscenes.insert(*e);
                subscenes.extend(self.subscene_resolve(*e));
            }
        }
        subscenes
    }
}
pub(crate) fn resolve_anchor(
    mut coordinator: ResMut<SceneCoordinator>,
    mut param_set: ParamSet<(
        Query<(
            &mut SceneAnchor,
            &SceneRoot,
            &SceneNodes,
            &mut Position<InterfaceContext>,
            &Area<InterfaceContext>,
            &PositionAlignment,
            &mut Layer,
            &LayerAlignment,
        )>,
        Query<&SceneNodes>,
    )>,
    mut cmd: Commands,
) {
    if coordinator.is_changed() {
        for root in coordinator.roots.clone() {
            if coordinator.removes.contains(&root) {
                if let Ok(nodes) = param_set.p1().get(root) {
                    nodes.despawn_non_scene(&mut cmd);
                }
            }
            let dependents = coordinator.subscene_resolve(root);
            for dep in dependents {
                if let Ok((
                    mut anchor,
                    dep_root,
                    nodes,
                    mut pos,
                    area,
                    pos_align,
                    mut layer,
                    layer_align,
                )) = param_set.p0().get_mut(dep)
                {
                    let root_anchor = *coordinator.anchors.get(&dep_root.current.unwrap()).unwrap();
                    let despawned = coordinator
                        .removes
                        .get(dep_root.current.as_ref().unwrap())
                        .is_some();
                    if despawned || coordinator.removes.get(&dep).is_some() {
                        coordinator.removes.insert(dep);
                        if let Some(ss) =
                            coordinator.subscenes.get_mut(&dep_root.current().unwrap())
                        {
                            ss.remove(&dep);
                        }
                        nodes.despawn_non_scene(&mut cmd);
                    } else {
                        let new_position = pos_align.calc_pos(root_anchor, *area);
                        let new_layer = layer_align.calc_layer(root_anchor.0.layer);
                        if new_position != anchor.0.section.position || new_layer != anchor.0.layer
                        {
                            *pos = new_position;
                            *layer = new_layer;
                            let new_anchor = SceneAnchor(Coordinate::new(
                                Section::new(*pos, *area),
                                Layer::new(layer.z),
                            ));
                            *anchor = new_anchor;
                            coordinator.anchors.insert(dep, new_anchor);
                            nodes.set_anchor_non_scene(new_anchor, &mut cmd);
                        }
                    }
                }
            }
        }
        let _ = coordinator.removes.drain().map(|r| {
            cmd.entity(r).insert(Despawn::signal_despawn());
        });
    }
}
pub(crate) fn scene_register(
    mut coordinator: ResMut<SceneCoordinator>,
    mut query: Query<
        (Entity, &mut SceneRoot, &SceneAnchor, &Despawn),
        Or<(Changed<SceneAnchor>, Changed<SceneRoot>, Changed<Despawn>)>,
    >,
    mut removed: RemovedComponents<SceneRoot>,
) {
    for remove in removed.read() {
        coordinator.removes.insert(remove);
    }
    for (entity, mut root, anchor, despawn) in query.iter_mut() {
        let need_insert = if coordinator.anchors.get(&entity).is_none() {
            true
        } else {
            coordinator.anchors.get(&entity).unwrap().0 != anchor.0
        };
        if need_insert {
            coordinator.anchors.insert(entity, *anchor);
        }
        if coordinator.subscenes.get(&entity).is_none() {
            coordinator.subscenes.insert(entity, HashSet::new());
        }
        if let Some(old) = root.old.take() {
            // deregister
            if let Some(ss) = coordinator.subscenes.get_mut(&old) {
                ss.remove(&entity);
            }
        }
        if let Some(current) = root.current {
            if coordinator.subscenes.get(&current).is_none() {
                coordinator.subscenes.insert(current, HashSet::new());
            }
            coordinator
                .subscenes
                .get_mut(&current)
                .unwrap()
                .insert(entity);
        } else {
            coordinator.roots.insert(entity);
        }
        if despawn.should_despawn() {
            coordinator.removes.insert(entity);
        }
    }
}
pub(crate) fn hook_to_anchor(
    mut changed: Query<
        (
            &mut SceneAnchor,
            &Position<InterfaceContext>,
            &Area<InterfaceContext>,
            &Layer,
        ),
        (
            Or<(
                Changed<Position<InterfaceContext>>,
                Changed<Area<InterfaceContext>>,
                Changed<Layer>,
            )>,
            With<Tag<IsScene>>,
        ),
    >,
) {
}
#[derive(Bundle)]
pub(crate) struct SceneBundle {
    anchor: SceneAnchor,
    nodes: SceneNodes,
    root: SceneRoot,
    visibility: SceneVisibility,
    despawn: Despawn,
    tag: Tag<IsScene>,
}
pub struct IsScene();
impl SceneBundle {
    pub(crate) fn new(anchor: SceneAnchor, nodes: SceneNodes, root: SceneRoot) -> Self {
        Self {
            anchor,
            nodes,
            root,
            visibility: SceneVisibility::default(),
            despawn: Despawn::default(),
            tag: Tag::default(),
        }
    }
}
pub trait Scene
where
    Self: Bundle,
{
    type Args<'a>: Send + Sync;
    type ExternalResources: SystemParam;
    fn bind_nodes(
        cmd: &mut Commands,
        anchor: SceneAnchor,
        args: &Self::Args<'_>,
        external_args: &SystemParamItem<Self::ExternalResources>,
        binder: &mut SceneBinder,
    ) -> Self;
}
pub trait SceneSpawn {
    fn spawn_scene<S: Scene>(
        &mut self,
        anchor: SceneAnchor,
        args: &S::Args<'_>,
        external_args: &SystemParamItem<S::ExternalResources>,
        root: SceneRoot,
    ) -> Entity;
}
impl<'a, 'b> SceneSpawn for Commands<'a, 'b> {
    fn spawn_scene<S: Scene>(
        &mut self,
        anchor: SceneAnchor,
        args: &S::Args<'_>,
        external_args: &SystemParamItem<S::ExternalResources>,
        root: SceneRoot,
    ) -> Entity {
        let this = self.spawn_empty().id();
        let mut binder = SceneBinder::new(anchor, this);
        let bundle = S::bind_nodes(self, anchor, args, external_args, &mut binder);
        self.entity(this)
            .insert(bundle)
            .insert(SceneBundle::new(anchor, binder.nodes, root))
            .insert(anchor.0);
        this
    }
}