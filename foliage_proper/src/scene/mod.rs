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
pub struct SceneDesc {
    root: Entity,
    bindings: Bindings,
}
impl SceneDesc {
    pub(crate) fn new(root: Entity, bindings: Bindings) -> Self {
        Self { root, bindings }
    }
    pub fn root(&self) -> Entity {
        self.root
    }
    pub fn bindings(&self) -> &Bindings {
        &self.bindings
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
    pub fn entity(&self) -> Entity {
        self.entity
    }
}
pub struct Binder<'a, 'w, 's> {
    nodes: HashMap<SceneBinding, SceneNode>,
    root: Entity,
    cmd: &'a mut Commands<'w, 's>,
}

impl<'a, 'w, 's> Binder<'a, 'w, 's> {
    pub fn finish<S: Scene>(self, comps: SceneComponents<S>) -> SceneDesc {
        let entity = self.root();
        let bindings = Bindings(self.nodes);
        self.cmd
            .entity(entity)
            .insert(comps)
            .insert(bindings.clone());
        SceneDesc::new(entity, bindings)
    }
    pub fn new(cmd: &'a mut Commands<'w, 's>, root: Option<Entity>) -> Self {
        Self {
            nodes: HashMap::new(),
            root: root.unwrap_or(cmd.spawn_empty().id()),
            cmd,
        }
    }
    pub fn root(&self) -> Entity {
        self.root
    }
    pub fn bind<SB: Into<SceneBinding>, SA: Into<Alignment>, B: Bundle>(
        &mut self,
        sb: SB,
        sa: SA,
        b: B,
    ) -> Entity {
        // add alignment stuff
        let entity = self
            .cmd
            .spawn(b)
            .insert(SceneBindingComponents::new(
                self.root,
                Anchor::default(),
                sa.into(),
            ))
            .id();
        self.nodes.insert(sb.into(), SceneNode::new(entity, false));
        entity
    }
    pub fn bind_scene<S: Scene, SB: Into<SceneBinding>, SA: Into<Alignment>>(
        &mut self,
        sb: SB,
        sa: SA,
        s: S,
    ) -> SceneDesc {
        // add alignment + scene stuff
        let scene_desc = s.create(Binder::new(self.cmd, None));
        self.cmd
            .entity(scene_desc.root())
            .insert(SceneBindingComponents::new(
                self.root,
                Anchor::default(),
                sa.into(),
            ));
        self.nodes
            .insert(sb.into(), SceneNode::new(scene_desc.root(), true));
        scene_desc
    }
    pub fn bind_conditional<C>() {
        todo!()
    }
    pub fn bind_conditional_scene<S>() {
        todo!()
    }
    pub fn extend<Ext>() {
        todo!()
    }
    pub fn extend_conditional<Ext>() {
        todo!()
    }
}
#[derive(Component, Default, Clone)]
pub struct Bindings(HashMap<SceneBinding, SceneNode>);
impl Bindings {
    pub fn get<SB: Into<SceneBinding>>(&self, sb: SB) -> Entity {
        self.0.get(&sb.into()).expect("no-scene-binding").entity
    }
    pub fn nodes(&self) -> &HashMap<SceneBinding, SceneNode> {
        &self.0
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
        if despawn.is_despawned() {
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
    fn create(self, binder: Binder) -> SceneDesc;
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
pub(crate) fn recursive_bindings(
    root: Entity,
    query: &Query<
        (Option<&Bindings>, &Despawn, &Disabled),
        Or<(
            With<Tag<IsScene>>,
            With<Tag<IsDep>>,
            Or<(Changed<Despawn>, Changed<Disabled>)>,
        )>,
    >,
) -> HashSet<Entity> {
    let mut dependents = HashSet::new();
    if let Ok(res) = query.get(root) {
        if let Some(binds) = res.0 {
            for b in binds.0.iter() {
                dependents.insert(b.1.entity);
                dependents.extend(recursive_bindings(b.1.entity, &query));
            }
        }
    }
    dependents
}
// TODO add disabled to this/ re-enabled
pub(crate) fn despawn_bindings(
    mut changed: ParamSet<(
        Query<
            (Option<&Bindings>, &Despawn, &Disabled),
            Or<(
                With<Tag<IsScene>>,
                With<Tag<IsDep>>,
                Or<(Changed<Despawn>, Changed<Disabled>)>,
            )>,
        >,
        Query<(&mut Despawn, &mut Disabled)>,
    )>,
) {
    let mut to_despawn = HashSet::new();
    let mut to_disable = HashSet::new();
    let mut to_enable = HashSet::new();
    for (bindings, despawn, disable) in changed.p0().iter() {
        if despawn.is_despawned() {
            if let Some(binds) = bindings {
                for b in binds.0.iter() {
                    to_despawn.insert(b.1.entity);
                }
            }
        }
        if disable.is_disabled() {
            if let Some(binds) = bindings {
                for b in binds.0.iter() {
                    to_disable.insert(b.1.entity);
                }
            }
        } else {
            if let Some(binds) = bindings {
                for b in binds.0.iter() {
                    to_enable.insert(b.1.entity);
                }
            }
        }
    }
    for e in to_despawn.clone() {
        let entities = recursive_bindings(e, &changed.p0());
        to_despawn.extend(entities);
    }
    for e in to_disable.clone() {
        let entities = recursive_bindings(e, &changed.p0());
        to_disable.extend(entities);
    }
    for e in to_enable.clone() {
        let entities = recursive_bindings(e, &changed.p0());
        to_enable.extend(entities);
    }
    for e in to_despawn {
        if let Ok(mut d) = changed.p1().get_mut(e) {
            d.0.despawn();
        }
    }
    for e in to_disable {
        if let Ok(mut d) = changed.p1().get_mut(e) {
            d.1.disable();
        }
    }
    for e in to_enable {
        if let Ok(mut d) = changed.p1().get_mut(e) {
            d.1.enable();
        }
    }
}

pub enum ExtendTarget {
    This,
    Binding(SceneBinding),
}
