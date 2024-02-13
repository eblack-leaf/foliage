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
use std::collections::HashMap;

#[derive(Component, Copy, Clone, Default)]
pub struct Anchor(Coordinate<InterfaceContext>);

impl Anchor {
    pub(crate) fn aligned(&self, grid: MicroGrid, alignment: Alignment) -> Self {
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
    pub fn finish<S: Scene>(
        self,
        comps: SceneComponents<S::Components>,
        cmd: &mut Commands,
    ) -> Entity {
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
pub struct SceneComponents<T: Bundle + Send + Sync + 'static> {
    t: T,
    bindings: Bindings,
    coordinate: Coordinate<InterfaceContext>,
    despawn: Despawn,
    disabled: Disabled,
    tag: Tag<T>,
    scene_tag: Tag<IsScene>,
    grid: MicroGrid,
}
impl<T: Bundle + Send + Sync + 'static> SceneComponents<T> {
    pub fn new(grid: MicroGrid, t: T) -> Self {
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
}
impl SceneBindingComponents {
    fn new(ptr: Entity, anchor: Anchor, alignment: Alignment) -> Self {
        Self {
            tag: Tag::new(),
            anchor,
            alignment,
            ptr: ScenePtr(ptr),
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
        // do rest
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
    // or i structure below query and call Scene::config(params) inside it after despawn.should_despawn() { continue }
    fn config(
        entity: Entity,
        coordinate: Coordinate<InterfaceContext>,
        ext: &mut SystemParamItem<Self::Params>,
        bindings: &Bindings,
    );
    // self is the Args to the scene
    // only create bindings; will be configured above
    fn create(self, cmd: &mut Commands) -> Entity;
}
#[derive(Component, Copy, Clone)]
pub struct ScenePtr(Entity);
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
                    let alignment = *dep.1;
                    let ptr = *dep.3;
                    let grid = *grids.get(ptr.0).expect("scene-grid");
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
            let alignment = *deps.p0().get(bind.entity).unwrap().1;
            let ptr = *deps.p0().get(bind.entity).unwrap().3;
            let grid = *grids.get(ptr.0).expect("scene-grid");
            *deps.p1().get_mut(bind.entity).unwrap() = Anchor(coordinate).aligned(grid, alignment);
            if bind.is_scene {
                let rf = recursive_fetch(coordinate, bind.entity, &deps.p0(), &grids);
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
pub(crate) fn despawn_bindings() {
    // same root + loop deps as resolve_anchor
    // if one in chain is despawn => all subscenes will return should_despawn in recursive fetch
    // loop entity-pool (bindings) +
    //      if is_scene => loop that ones entity-pool
    //          despawn.signal_despawn()
}