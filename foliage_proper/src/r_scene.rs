use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::{Coordinate, InterfaceContext};
use crate::differential::Despawn;
use crate::elm::leaf::Tag;
use crate::elm::Disabled;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Commands, Component, Entity, Query};
use bevy_ecs::query::{Changed, With, Without};
use bevy_ecs::system::{ParamSet, SystemParam};

#[derive(Component, Copy, Clone, Default)]
struct Anchor(Coordinate<InterfaceContext>);

impl Anchor {
    pub(crate) fn aligned(&self, alignment: Alignment) -> Self {
        todo!()
    }
}

#[derive(Component, Copy, Clone)]
struct Alignment {
    // placement markers (grid or custom)
}
#[derive(Copy, Clone)]
struct SceneBinding(i32);
#[derive(Copy, Clone)]
struct SceneNode {
    entity: Entity,
    is_scene: bool,
}
#[derive(Component, Copy, Clone)]
struct ScenePtr(Entity);
struct Binder(Vec<SceneNode>);
impl Binder {
    fn bind<SB: Into<SceneBinding>, B: Bundle>(
        &mut self,
        sb: SB,
        b: B,
        cmd: &mut Commands,
    ) -> Entity {
        // add alignment stuff
        todo!()
    }
    fn bind_scene<S: Scene, SB: Into<SceneBinding>>(
        &mut self,
        sb: SB,
        s: S,
        cmd: &mut Commands,
    ) -> Entity {
        // add alignment + scene stuff
        todo!()
    }
    fn bindings(self) -> Bindings {
        Bindings(self.0)
    }
}
#[derive(Default, Component)]
struct Bindings(Vec<SceneNode>);
impl Bindings {
    fn get<SB: Into<SceneBinding>>(&self, sb: SB) -> SceneNode {
        *self.0.get(sb.into().0 as usize).expect("no-scene-binding")
    }
}
#[derive(Component, Copy, Clone)]
struct IsScene;
#[derive(Component, Copy, Clone)]
struct IsDep;
#[derive(Bundle)]
struct SceneComponents<T: Bundle + Send + Sync + 'static> {
    t: T,
    bindings: Bindings,
    coordinate: Coordinate<InterfaceContext>,
    despawn: Despawn,
    disabled: Disabled,
    tag: Tag<T>,
    scene_tag: Tag<IsScene>,
}
impl<T: Bundle + Send + Sync + 'static> SceneComponents<T> {
    fn new(coordinate: Coordinate<InterfaceContext>, bindings: Bindings, t: T) -> Self {
        Self {
            t,
            bindings,
            coordinate,
            despawn: Default::default(),
            disabled: Default::default(),
            tag: Tag::new(),
            scene_tag: Tag::new(),
        }
    }
}
#[derive(Bundle)]
struct SceneBindingComponents<T: Bundle + Send + Sync + 'static> {
    t: T,
    tag: Tag<IsDep>,
    anchor: Anchor,
    alignment: Alignment,
}
impl<T: Bundle + Send + Sync + 'static> SceneBindingComponents<T> {
    fn new(t: T, anchor: Anchor, alignment: Alignment) -> Self {
        Self {
            t,
            tag: Tag::new(),
            anchor,
            alignment,
        }
    }
}
// will need to add this for every scene added
fn config<S: Scene + Send + Sync + 'static>(
    query: Query<
        (&Area<InterfaceContext>, &Despawn, &Bindings),
        (With<Tag<S>>, Changed<Area<InterfaceContext>>),
    >,
    mut areas: Query<&mut Area<InterfaceContext>, Without<Tag<S>>>,
    mut ext: S::ConfigParams,
) {
    for (area, despawn, bindings) in query.iter() {
        if despawn.should_despawn() {
            continue;
        }
        // disabled?
        // do rest
        S::config(*area, &mut areas, &mut ext, bindings);
    }
}
trait Scene
where
    Self: Sized + Send + Sync + 'static,
{
    type ConfigParams: SystemParam;
    type Components: Bundle;
    // or i structure below query and call Scene::config(params) inside it after despawn.should_despawn() { continue }
    fn config(
        area: Area<InterfaceContext>,
        area_query: &mut Query<&mut Area<InterfaceContext>, Without<Tag<Self>>>,
        ext: &mut Self::ConfigParams,
        bindings: &Bindings,
    );
    // self is the Args to the scene
    // only create bindings; will be configured above
    fn bind(self, binder: Binder, cmd: &mut Commands) -> SceneComponents<Self::Components>;
}
fn recursive_fetch(
    root_coordinate: Coordinate<InterfaceContext>,
    target_entity: Entity,
    query: &Query<
        (
            &Anchor,
            &Area<InterfaceContext>,
            &Alignment,
            Option<&Bindings>,
        ),
        With<Tag<IsDep>>,
    >,
) -> Vec<(Entity, Anchor)> {
    let mut fetch = vec![];
    if let Ok(res) = query.get(target_entity) {
        if let Some(bindings) = res.3 {
            for bind in bindings.0.iter() {
                if let Ok(dep) = query.get(bind.entity) {
                    let alignment = *dep.2;
                    let anchor = Anchor(root_coordinate.with_area(*dep.1)).aligned(alignment);
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
fn resolve_anchor(
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
        Query<
            (
                &Anchor,
                &Area<InterfaceContext>,
                &Alignment,
                Option<&Bindings>,
            ),
            With<Tag<IsDep>>,
        >,
        Query<(&mut Anchor, &Area<InterfaceContext>, &Alignment), With<Tag<IsDep>>>,
    )>,
) {
    for (pos, area, layer, bindings) in roots.iter() {
        let coordinate = Coordinate::new((*pos, *area), *layer);
        for bind in bindings.0.iter() {
            let alignment = *deps.p1().get_mut(bind.entity).unwrap().2;
            let dep_area = *deps.p1().get_mut(bind.entity).unwrap().1;
            *deps.p1().get_mut(bind.entity).unwrap().0 =
                Anchor(coordinate.with_area(dep_area)).aligned(alignment);
            if bind.is_scene {
                let rf = recursive_fetch(coordinate, bind.entity, &deps.p0());
                for (e, a) in rf {
                    *deps.p1().get_mut(e).unwrap().0 = a;
                }
            }
        }
    }
}
fn update_from_anchor() {
    // if Changed<Anchor> then update from alignment (only pos + layer)
}
fn despawn_bindings() {
    // same root + loop deps as resolve_anchor
    // if one in chain is despawn => all subscenes will return should_despawn in recursive fetch
    // loop entity-pool (bindings) +
    //      if is_scene => loop that ones entity-pool
    //          despawn.signal_despawn()
}