use crate::coordinate::{Coordinate, InterfaceContext};
use crate::differential::Despawn;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::system::Commands;
use serde::{Deserialize, Serialize};
use align::AlignmentAnchor;
use bind::{SceneNodes, SceneVisibility};
use chain::BundleChain;

pub mod chain;
pub mod align;
pub mod bind;

pub trait Sceneable where Self: Bundle {
    type Args;
    fn new(coordinate: Coordinate<InterfaceContext>, args: &Self::Args, cmd: &mut Commands, nodes: &mut SceneNodes) -> Self;
}
#[derive(Bundle)]
pub struct Scene<T: Sceneable> {
    scene: SceneBundle,
    t: T,
}
impl<T: Sceneable> Scene<T> {
    pub fn new(anchor: Coordinate<InterfaceContext>, args: &T::Args, cmd: &mut Commands) -> Self {
        let (t, nodes) = T::new(anchor, args, cmd);
        Self{
            t,
            scene: SceneBundle::new(anchor, nodes),
        }
    }
}
pub trait SetTheScene {
    fn spawn_scene<T: Sceneable>(&mut self, coordinate: Coordinate<InterfaceContext>, args: &T::Args) -> Scene<T>;
}
impl<'a, 'b> SetTheScene for Commands<'a, 'b> {
    fn spawn_scene<T: Sceneable>(&mut self, anchor: Coordinate<InterfaceContext>, args: &T::Args) -> Entity {
        let this = Scene::<T>::new(anchor, args, self);
        self.spawn(this).id()
    }
}
#[derive(Bundle)]
pub struct SceneBundle {
    pub anchor: AlignmentAnchor,
    pub nodes: SceneNodes,
    pub visibility: SceneVisibility,
    pub despawn: Despawn,
}
impl SceneBundle {
    pub fn new(anchor: Coordinate<InterfaceContext>, nodes: SceneNodes) -> Self {
        Self {
            anchor: AlignmentAnchor(anchor),
            nodes,
            visibility: SceneVisibility::default(),
            despawn: Despawn::default(),
        }
    }
}