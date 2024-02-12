use crate::compositor::segment::Grid;
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::{Coordinate, InterfaceContext};
use crate::differential::Despawn;
use crate::elm::leaf::Tag;
use crate::elm::Disabled;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Commands, Component, Entity, Query};
use bevy_ecs::query::{Changed, Or, ReadOnlyWorldQuery, With, Without};
use bevy_ecs::system::{ParamSet, SystemParam};
use std::collections::HashMap;

#[derive(Component, Copy, Clone, Default)]
pub struct Anchor(Coordinate<InterfaceContext>);

impl Anchor {
    pub(crate) fn aligned(&self, alignment: Alignment) -> Self {
        // calc grid and give back coordinate
        // using self as aligner
        todo!()
    }
}

#[derive(Component, Copy, Clone)]
pub struct Alignment {
    // placement markers (grid or custom)
}
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct SceneBinding(i32);
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
#[derive(Default)]
struct Binder(HashMap<SceneBinding, SceneNode>);
impl Binder {
    fn bind<SB: Into<SceneBinding>, SA: Into<Alignment>, B: Bundle>(
        &mut self,
        sb: SB,
        sa: SA,
        b: B,
        cmd: &mut Commands,
    ) -> Entity {
        // add alignment stuff
        let entity = cmd
            .spawn(b)
            .insert(SceneBindingComponents::new(Anchor::default(), sa.into()))
            .id();
        self.0.insert(sb.into(), SceneNode::new(entity, false));
        entity
    }
    fn bind_scene<S: Scene, SB: Into<SceneBinding>>(
        &mut self,
        sb: SB,
        s: S,
        cmd: &mut Commands,
    ) -> Entity {
        // add alignment + scene stuff
        let components = s.create(cmd);
        let entity = cmd.spawn(components).id();
        self.0.insert(sb.into(), SceneNode::new(entity, true));
        entity
    }
    fn bindings(self) -> Bindings {
        Bindings(self.0)
    }
}
#[derive(Default, Component)]
pub struct Bindings(HashMap<SceneBinding, SceneNode>);
impl Bindings {
    fn get<SB: Into<SceneBinding>>(&self, sb: SB) -> SceneNode {
        *self.0.get(&sb.into()).expect("no-scene-binding")
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
}
impl<T: Bundle + Send + Sync + 'static> SceneComponents<T> {
    pub fn new(bindings: Bindings, t: T) -> Self {
        Self {
            t,
            bindings,
            coordinate: Coordinate::default(),
            despawn: Default::default(),
            disabled: Default::default(),
            tag: Tag::new(),
            scene_tag: Tag::new(),
        }
    }
}
#[derive(Bundle)]
struct SceneBindingComponents {
    tag: Tag<IsDep>,
    anchor: Anchor,
    alignment: Alignment,
}
impl SceneBindingComponents {
    fn new(anchor: Anchor, alignment: Alignment) -> Self {
        Self {
            tag: Tag::new(),
            anchor,
            alignment,
        }
    }
}
// will need to add this for every scene added
fn config<S: Scene + Send + Sync + 'static>(
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
    mut ext: S::ConfigParams,
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
    const GRID: Grid;
    type ConfigParams: SystemParam;
    type Filter: ReadOnlyWorldQuery;
    type Components: Bundle;
    // or i structure below query and call Scene::config(params) inside it after despawn.should_despawn() { continue }
    fn config(
        entity: Entity,
        coordinate: Coordinate<InterfaceContext>,
        ext: &mut Self::ConfigParams,
        bindings: &Bindings,
    );
    // self is the Args to the scene
    // only create bindings; will be configured above
    fn create(self, cmd: &mut Commands) -> SceneComponents<Self::Components>;
}
fn recursive_fetch(
    root_coordinate: Coordinate<InterfaceContext>,
    target_entity: Entity,
    query: &Query<(&Anchor, &Alignment, Option<&Bindings>), With<Tag<IsDep>>>,
) -> Vec<(Entity, Anchor)> {
    let mut fetch = vec![];
    if let Ok(res) = query.get(target_entity) {
        if let Some(bindings) = res.2 {
            for (_, bind) in bindings.0.iter() {
                if let Ok(dep) = query.get(bind.entity) {
                    let alignment = *dep.1;
                    let anchor = Anchor(root_coordinate).aligned(alignment);
                    fetch.push((bind.entity, anchor));
                    if bind.is_scene {
                        let others = recursive_fetch(anchor.0, bind.entity, &query);
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
        Query<(&Anchor, &Alignment, Option<&Bindings>), With<Tag<IsDep>>>,
        Query<&mut Anchor, With<Tag<IsDep>>>,
    )>,
) {
    for (pos, area, layer, bindings) in roots.iter() {
        let coordinate = Coordinate::new((*pos, *area), *layer);
        for (_, bind) in bindings.0.iter() {
            let alignment = *deps.p0().get(bind.entity).unwrap().1;
            *deps.p1().get_mut(bind.entity).unwrap() = Anchor(coordinate).aligned(alignment);
            if bind.is_scene {
                let rf = recursive_fetch(coordinate, bind.entity, &deps.p0());
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