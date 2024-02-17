pub mod micro_grid;

use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::{Coordinate, InterfaceContext};
use crate::differential::Despawn;
use crate::elm::leaf::Tag;
use crate::elm::Disabled;
use crate::scene::micro_grid::MicroGrid;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Commands, Component, Entity, Query};
use bevy_ecs::query::{Changed, Or, ReadOnlyWorldQuery, With, Without};
use bevy_ecs::system::{ParamSet, StaticSystemParam, SystemParam, SystemParamItem};
use micro_grid::Alignment;
use std::collections::{HashMap, HashSet};

#[derive(Component, Copy, Clone, Default)]
pub struct Anchor(Coordinate<InterfaceContext>);

impl Anchor {
    pub(crate) fn aligned(&self, grid: &MicroGrid, alignment: &Alignment) -> Self {
        Anchor(grid.determine(self.0, alignment))
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct SceneBinding(pub i32);
impl From<i32> for SceneBinding {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
#[derive(Copy, Clone)]
pub struct SceneNode {
    entity: Entity,
    is_scene: bool,
}
impl SceneNode {
    fn new(entity: Entity, is_scene: bool) -> Self {
        Self { entity, is_scene }
    }
}
pub struct Binder(HashMap<SceneBinding, SceneNode>, Entity);

impl Binder {
    pub fn finish<S: Scene>(self, comps: SceneComponents<S>, cmd: &mut Commands) -> Entity {
        let entity = self.root();
        cmd.entity(entity).insert(comps).insert(Bindings(self.0));
        entity
    }
    pub fn new(cmd: &mut Commands) -> Self {
        Self(HashMap::new(), cmd.spawn_empty().id())
    }
    pub fn root(&self) -> Entity {
        self.1
    }
    pub fn bind<SB: Into<SceneBinding>, SA: Into<Alignment>, B: Bundle>(
        &mut self,
        sb: SB,
        sa: SA,
        b: B,
        cmd: &mut Commands,
    ) -> Entity {
        // add alignment stuff
        let entity = cmd
            .spawn(b)
            .insert(SceneBindingComponents::new(
                self.1,
                Anchor::default(),
                sa.into(),
            ))
            .id();
        self.0.insert(sb.into(), SceneNode::new(entity, false));
        entity
    }
    pub fn bind_scene<S: Scene, SB: Into<SceneBinding>, SA: Into<Alignment>>(
        &mut self,
        sb: SB,
        sa: SA,
        s: S,
        cmd: &mut Commands,
    ) -> Entity {
        // add alignment + scene stuff
        let entity = s.create(cmd);
        cmd.entity(entity).insert(SceneBindingComponents::new(
            self.1,
            Anchor::default(),
            sa.into(),
        ));
        self.0.insert(sb.into(), SceneNode::new(entity, true));
        entity
    }
}
#[derive(Component, Default)]
pub struct Bindings(HashMap<SceneBinding, SceneNode>);
impl Bindings {
    pub fn get<SB: Into<SceneBinding>>(&self, sb: SB) -> Entity {
        self.0.get(&sb.into()).expect("no-scene-binding").entity
    }
}
#[derive(Component, Copy, Clone)]
pub struct IsScene;
#[derive(Component, Copy, Clone)]
pub struct IsDep;
#[derive(Bundle)]
pub struct SceneComponents<S: Scene> {
    t: S::Components,
    bindings: Bindings,
    coordinate: Coordinate<InterfaceContext>,
    despawn: Despawn,
    disabled: Disabled,
    tag: Tag<S>,
    scene_tag: Tag<IsScene>,
    grid: MicroGrid,
}
impl<S: Scene> SceneComponents<S> {
    pub fn new(grid: MicroGrid, t: S::Components) -> Self {
        Self {
            t,
            bindings: Bindings::default(),
            coordinate: Coordinate::default(),
            despawn: Default::default(),
            disabled: Default::default(),
            tag: Tag::new(),
            scene_tag: Tag::new(),
            grid,
        }
    }
}
#[derive(Bundle)]
struct SceneBindingComponents {
    tag: Tag<IsDep>,
    anchor: Anchor,
    alignment: Alignment,
    ptr: ScenePtr,
    despawn: Despawn,
}
impl SceneBindingComponents {
    fn new(ptr: Entity, anchor: Anchor, alignment: Alignment) -> Self {
        Self {
            tag: Tag::new(),
            anchor,
            alignment,
            ptr: ScenePtr(ptr),
            despawn: Despawn::default(),
        }
    }
}
// will need to add this for every scene added
pub fn config<S: Scene + Send + Sync + 'static>(
    query: Query<
        (
            Entity,
            &Position<InterfaceContext>,
            &Area<InterfaceContext>,
            &Layer,
            &Despawn,
            &Bindings,
        ),
        (
            With<Tag<S>>,
            Or<(
                Changed<Area<InterfaceContext>>,
                Changed<Position<InterfaceContext>>,
                Changed<Layer>,
                S::Filter,
            )>,
        ),
    >,
    mut ext: StaticSystemParam<S::Params>,
) {
    for (entity, pos, area, layer, despawn, bindings) in query.iter() {
        if despawn.should_despawn() {
            continue;
        }
        // disabled?
        S::config(
            entity,
            Coordinate::new((*pos, *area), *layer),
            &mut ext,
            bindings,
        );
    }
}
pub trait Scene
where
    Self: Sized + Send + Sync + 'static,
{
    type Params: SystemParam + 'static;
    type Filter: ReadOnlyWorldQuery;
    type Components: Bundle;
    fn config(
        entity: Entity,
        coordinate: Coordinate<InterfaceContext>,
        ext: &mut SystemParamItem<Self::Params>,
        bindings: &Bindings,
    );
    fn create(self, cmd: &mut Commands) -> Entity;
}
#[derive(Component, Copy, Clone)]
pub struct ScenePtr(Entity);
impl ScenePtr {
    pub fn value(self) -> Entity {
        self.0
    }
}
#[derive(Bundle, Copy, Clone, Default)]
pub struct BlankNode {
    coordinate: Coordinate<InterfaceContext>,
}
fn recursive_fetch(
    root_coordinate: Coordinate<InterfaceContext>,
    target_entity: Entity,
    query: &Query<(&Anchor, &Alignment, Option<&Bindings>, &ScenePtr), With<Tag<IsDep>>>,
    grids: &Query<&MicroGrid>,
) -> Vec<(Entity, Anchor)> {
    let mut fetch = vec![];
    if let Ok(res) = query.get(target_entity) {
        if let Some(bindings) = res.2 {
            for (_, bind) in bindings.0.iter() {
                if let Ok(dep) = query.get(bind.entity) {
                    let alignment = dep.1;
                    let ptr = *dep.3;
                    let grid = grids.get(ptr.0).expect("scene-grid");
                    let anchor = Anchor(root_coordinate).aligned(grid, alignment);
                    fetch.push((bind.entity, anchor));
                    if bind.is_scene {
                        let others = recursive_fetch(anchor.0, bind.entity, &query, &grids);
                        fetch.extend(others);
                    }
                }
            }
        }
    }
    fetch
}
pub(crate) fn resolve_anchor(
    roots: Query<
        (
            &Position<InterfaceContext>,
            &Area<InterfaceContext>,
            &Layer,
            &Bindings,
        ),
        (With<Tag<IsScene>>, Without<Tag<IsDep>>),
    >,
    mut deps: ParamSet<(
        Query<(&Anchor, &Alignment, Option<&Bindings>, &ScenePtr), With<Tag<IsDep>>>,
        Query<&mut Anchor, With<Tag<IsDep>>>,
    )>,
    grids: Query<&MicroGrid>,
) {
    for (pos, area, layer, bindings) in roots.iter() {
        let coordinate = Coordinate::new((*pos, *area), *layer);
        for (_, bind) in bindings.0.iter() {
            let ptr = *deps.p0().get(bind.entity).unwrap().3;
            let grid = grids.get(ptr.0).expect("scene-grid");
            let anchor = Anchor(coordinate).aligned(grid, deps.p0().get(bind.entity).unwrap().1);
            *deps.p1().get_mut(bind.entity).unwrap() = anchor;
            if bind.is_scene {
                let rf = recursive_fetch(anchor.0, bind.entity, &deps.p0(), &grids);
                for (e, a) in rf {
                    *deps.p1().get_mut(e).unwrap() = a;
                }
            }
        }
    }
}
pub(crate) fn update_from_anchor(
    mut anchors: Query<
        (
            &Anchor,
            &mut Position<InterfaceContext>,
            &mut Area<InterfaceContext>,
            &mut Layer,
        ),
        Changed<Anchor>,
    >,
) {
    for (anchor, mut pos, mut area, mut layer) in anchors.iter_mut() {
        *pos = anchor.0.section.position;
        *area = anchor.0.section.area;
        *layer = anchor.0.layer;
    }
}
pub(crate) fn recursive_despawn(
    root: Entity,
    query: &Query<(Option<&Bindings>, &Despawn), Or<(With<Tag<IsScene>>, With<Tag<IsDep>>)>>,
) -> HashSet<Entity> {
    let mut to_despawn = HashSet::new();
    if let Ok(res) = query.get(root) {
        if let Some(binds) = res.0 {
            for b in binds.0.iter() {
                to_despawn.insert(b.1.entity);
                to_despawn.extend(recursive_despawn(b.1.entity, &query));
            }
        }
    }
    to_despawn
}
pub(crate) fn despawn_bindings(
    mut despawned: ParamSet<(
        Query<(Option<&Bindings>, &Despawn), Or<(With<Tag<IsScene>>, With<Tag<IsDep>>)>>,
        Query<&mut Despawn>,
    )>,
) {
    let mut to_despawn = HashSet::new();
    for (bindings, despawn) in despawned.p0().iter() {
        if despawn.should_despawn() {
            if let Some(binds) = bindings {
                for b in binds.0.iter() {
                    to_despawn.insert(b.1.entity);
                }
            }
        }
    }
    for e in to_despawn.clone() {
        let entities = recursive_despawn(e, &despawned.p0());
        to_despawn.extend(entities);
    }
    for e in to_despawn {
        if let Ok(mut d) = despawned.p1().get_mut(e) {
            d.despawn();
        }
    }
}