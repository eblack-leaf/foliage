use crate::coordinate::area::Area;
use crate::coordinate::{Coordinate, InterfaceContext};
use crate::differential::Despawn;
use crate::elm::leaf::Tag;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Commands, Component, Entity, Query};
use bevy_ecs::query::{Changed, With, Without};
use bevy_ecs::system::SystemParam;
#[derive(Component, Copy, Clone, Default)]
struct Anchor(Coordinate<InterfaceContext>);
struct Alignment {
    // placement markers (grid or custom)
}
struct SceneBinding(i32);
#[derive(Copy, Clone)]
struct SceneNode {
    entity: Entity,
    is_scene: bool,
}
struct ScenePtr(Entity);
#[derive(Default)]
struct Bindings(Vec<SceneNode>);
impl Bindings {
    fn get<SB: Into<SceneBinding>>(&self, sb: SB) -> SceneNode {
        *self.0.get(sb.into().0 as usize).expect("no-scene-binding")
    }
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
        // add alignment + scene stuff + ScenePtr for root
        todo!()
    }
}
#[derive(Bundle)]
struct SceneComponents<T>(T, Bindings, Anchor, Despawn, Tag<T>);
impl<T> SceneComponents<T> {
    fn new(bindings: Bindings, t: T) -> Self {
        Self {
            0: t,
            1: bindings,
            2: Default::default(),
            3: Default::default(),
            4: Tag::new(),
        }
    }
}
// will need to add this for every scene added
fn config<S: Scene>(
    mut query: Query<
        (&mut Area<InterfaceContext>, &mut Anchor, &Despawn),
        (With<Tag<S>>, Changed<Area<InterfaceContext>>),
    >,
    mut areas: Query<&mut Area<InterfaceContext>, Without<Tag<S>>>,
    mut ext: S::ConfigParams,
) {
    for (mut area, mut anchor, despawn) in query.iter_mut() {
        if despawn.should_despawn() {
            continue;
        }
        // disabled?
        // do rest
        S::config(*anchor, area.as_mut(), &mut areas, &mut ext);
        anchor.0.section.area = *area;
    }
}
trait Scene {
    type ConfigParams: SystemParam;
    type Components: Bundle;
    // or i structure below query and call Scene::config(params) inside it after despawn.should_despawn() { continue }
    fn config(
        anchor: Anchor,
        area: &mut Area<InterfaceContext>,
        area_query: &mut Query<&mut Area<InterfaceContext>, Without<Tag<Self>>>,
        ext: &mut Self::ConfigParams,
    );
    // self is the Args to the scene
    // only create bindings; will be configured above
    fn bind(self, bindings: Bindings, cmd: &mut Commands) -> SceneComponents<Self::Components>;
}
fn resolve_anchor() {
    // queue resolution requests
}
fn finalize_anchor() {
    // take resolution requests and finalize
}
fn despawn_bindings() {
    // loop entity-pool (bindings) +
    //      if is_scene => loop that ones entity-pool
    //          despawn.signal_despawn()
}
