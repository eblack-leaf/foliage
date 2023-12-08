pub mod align;
pub mod bind;

use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, InterfaceContext};
use crate::differential::Despawn;
use align::{LayerAlignment, PositionAlignment, SceneAnchor};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Commands, DetectChanges, ParamSet, RemovedComponents, Resource};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::{Query, ResMut};
use bind::{SceneBinder, SceneNodes, SceneRoot, SceneVisibility};
use indexmap::IndexSet;
use std::collections::{HashMap, HashSet};

#[derive(Resource, Default)]
pub struct SceneCompositor {
    pub(crate) anchors: HashMap<Entity, SceneAnchor>,
    pub(crate) subscenes: HashMap<Entity, HashSet<Entity>>,
    pub(crate) roots: HashSet<Entity>,
    pub(crate) removes: HashSet<Entity>,
}
impl SceneCompositor {
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
    mut compositor: ResMut<SceneCompositor>,
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
        Query<(&SceneNodes)>,
    )>,
    mut cmd: Commands,
) {
    if compositor.is_changed() {
        for root in compositor.roots.clone() {
            if compositor.removes.contains(&root) {
                if let Ok(nodes) = param_set.p1().get(root) {
                    nodes.despawn_non_scene(&mut cmd);
                }
            }
            let dependents = compositor.subscene_resolve(root);
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
                    let root_anchor = *compositor.anchors.get(&dep_root.current.unwrap()).unwrap();
                    let despawned = compositor
                        .removes
                        .get(dep_root.current.as_ref().unwrap())
                        .is_some();
                    if despawned || compositor.removes.get(&dep).is_some() {
                        compositor.removes.insert(dep);
                        if let Some(ss) = compositor.subscenes.get_mut(&dep_root.current().unwrap())
                        {
                            ss.remove(&dep);
                        }
                        nodes.despawn_non_scene(&mut cmd);
                    } else {
                        *pos = pos_align.calc_pos(root_anchor, *area);
                        *layer = layer_align.calc_layer(root_anchor.0.layer);
                        if *pos != anchor.0.section.position || *layer != anchor.0.layer {
                            let new_anchor = SceneAnchor(Coordinate::new(
                                Section::new(*pos, *area),
                                Layer::new(layer.z),
                            ));
                            *anchor = new_anchor;
                            compositor.anchors.insert(dep, new_anchor);
                            for (_, entry) in nodes.0.iter() {
                                if !entry.is_scene {
                                    cmd.entity(entry.entity).insert(new_anchor);
                                }
                            }
                        }
                    }
                }
            }
        }
        let _ = compositor.removes.drain().map(|r| {
            cmd.entity(r).insert(Despawn::signal_despawn());
        });
    }
}
pub(crate) fn register_root(
    mut compositor: ResMut<SceneCompositor>,
    mut query: Query<
        (Entity, &mut SceneRoot, &SceneAnchor, &Despawn),
        Or<(Changed<SceneAnchor>, Changed<SceneRoot>, Changed<Despawn>)>,
    >,
    mut removed: RemovedComponents<SceneRoot>,
) {
    for remove in removed.read() {
        compositor.removes.insert(remove);
    }
    for (entity, mut root, anchor, despawn) in query.iter_mut() {
        if compositor.anchors.get(&entity).is_none() {
            compositor.anchors.insert(entity, *anchor);
        } else if compositor.anchors.get(&entity).unwrap().0 != anchor.0 {
            compositor.anchors.insert(entity, *anchor);
        }
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
            if compositor.subscenes.get(&current).is_none() {
                compositor.subscenes.insert(current, HashSet::new());
            }
            compositor
                .subscenes
                .get_mut(&current)
                .unwrap()
                .insert(entity);
        } else {
            compositor.roots.insert(entity);
        }
        if despawn.should_despawn() {
            compositor.removes.insert(entity);
        }
    }
}

#[derive(Bundle)]
pub(crate) struct SceneBundle {
    anchor: SceneAnchor,
    nodes: SceneNodes,
    root: SceneRoot,
    visibility: SceneVisibility,
    despawn: Despawn,
}
impl SceneBundle {
    pub(crate) fn new(anchor: SceneAnchor, nodes: SceneNodes, root: SceneRoot) -> Self {
        Self {
            anchor,
            nodes,
            root,
            visibility: SceneVisibility::default(),
            despawn: Despawn::default(),
        }
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
        let mut binder = SceneBinder::new(anchor, this);
        let bundle = S::bind_nodes(self, anchor, args, &mut binder);
        self.entity(this)
            .insert(bundle)
            .insert(SceneBundle::new(anchor, binder.nodes, root));
        this
    }
}
