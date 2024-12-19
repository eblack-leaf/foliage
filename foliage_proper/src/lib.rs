mod ash;
mod asset;
mod attachment;
mod color;
mod coordinate;
mod disable;
mod enable;
mod ginkgo;
mod interaction;
mod leaf;
mod opacity;
mod ops;
mod photosynthesis;
mod platform;
mod remove;
mod text;
mod texture;
mod time;
mod tree;
mod virtual_keyboard;
mod visibility;
mod web_ext;
mod willow;

use self::ash::differential::cached_differential;
pub use self::ash::queue::RenderQueue;
pub use self::ash::queue::RenderRemoveQueue;
pub use self::ash::queue::RenderToken;
pub use crate::ash::clip::{ClipContext, ClipSection};
pub use crate::ash::differential::Differential;
use crate::ash::Ash;
use crate::asset::Asset;
pub use crate::coordinate::{
    area::{Area, CReprArea},
    position::{CReprPosition, Position},
    section::{CReprSection, Section},
    CoordinateContext, CoordinateUnit, Coordinates, DeviceContext, LogicalContext,
    NumericalContext,
};
use crate::photosynthesis::Photosynthesis;
use crate::remove::Remove;
use crate::time::Time;
use crate::willow::Willow;
pub use attachment::Attachment;
pub use bevy_ecs;
use bevy_ecs::event::{event_update_system, EventRegistry};
use bevy_ecs::observer::TriggerTargets;
pub use bevy_ecs::prelude::*;
use bevy_ecs::system::IntoObserverSystem;
pub use color::Color;
pub use coordinate::elevation::{Elevation, Layer};
pub use leaf::{Branch, Leaf, Stem};
pub use opacity::Opacity;
pub use ops::{Update, Write};
#[cfg(target_os = "android")]
pub use platform::AndroidApp;
pub use platform::AndroidConnection;
pub use text::{FontSize, Text};
pub use tree::{EcsExtension, Tree};
pub use visibility::{InheritedVisibility, ResolvedVisibility, Visibility};
pub struct Foliage {
    pub world: World,
    pub main: Schedule,
    pub user: Schedule,
    pub diff: Schedule,
    pub(crate) willow: Willow,
    pub base_url: String,
}
impl Foliage {
    pub fn new() -> Foliage {
        let mut foliage = Foliage {
            world: Default::default(),
            main: Default::default(),
            user: Default::default(),
            diff: Default::default(),
            willow: Default::default(),
            base_url: "".to_string(),
        };
        foliage
            .diff
            .configure_sets((DiffMarkers::Prepare, DiffMarkers::Extract).chain());
        foliage.main.add_systems(event_update_system);
        Ash::attach(&mut foliage);
        Text::attach(&mut foliage);
        Asset::attach(&mut foliage);
        Time::attach(&mut foliage);
        Remove::attach(&mut foliage);
        foliage
    }
    pub fn photosynthesize(self) {
        Photosynthesis::new().run(self);
    }
    pub fn desktop_size<V: Into<Area<DeviceContext>>>(&mut self, v: V) {
        self.willow.requested_size.replace(v.into());
    }
    pub fn url<S: AsRef<str>>(&mut self, path: S) {
        self.base_url = path.as_ref().to_string();
    }
    pub fn define<E: Event + 'static, B: Bundle, M, D: IntoObserverSystem<E, B, M>>(
        &mut self,
        obs: D,
    ) {
        self.world.add_observer(obs);
    }
    pub fn leaf<B: Bundle>(&mut self, b: B) -> Entity {
        self.world.leaf(b)
    }
    pub fn send_to<E: Event>(
        &mut self,
        e: E,
        targets: impl TriggerTargets + Send + Sync + 'static,
    ) {
        self.world.send_to(e, targets);
    }
    pub fn send<E: Event>(&mut self, e: E) {
        self.world.send(e);
    }
    pub fn queue<E: Event>(&mut self, e: E) {
        self.world.queue(e);
    }
    pub fn enable_queued_event<E: Event + Clone + Send + Sync + 'static>(&mut self) {
        if self.world.get_resource::<Events<E>>().is_none() {
            self.world.insert_resource(Events::<E>::default());
            EventRegistry::register_event::<E>(&mut self.world);
        }
    }
    pub fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.world.write_to(entity, b);
    }
    pub fn remove(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.world.remove(targets);
    }
    pub fn enable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.world.enable(targets);
    }
    pub fn disable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.world.disable(targets);
    }
    pub fn remove_queue<R: Clone + Send + Sync + 'static>(&mut self) {
        debug_assert_eq!(
            self.world.get_resource::<RenderRemoveQueue<R>>().is_none(),
            true
        );
        self.world.insert_resource(RenderRemoveQueue::<R>::new());
    }
    pub fn differential<
        R: Clone + Send + Sync + 'static,
        RT: Clone + Send + Sync + 'static + Component + PartialEq,
    >(
        &mut self,
    ) {
        debug_assert_eq!(
            self.world.get_resource::<RenderQueue<R, RT>>().is_none(),
            true
        );
        self.world.insert_resource(RenderQueue::<R, RT>::new());
        self.diff
            .add_systems(cached_differential::<R, RT>.in_set(DiffMarkers::Extract));
    }
}
#[derive(SystemSet, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub(crate) enum DiffMarkers {
    Prepare,
    Extract,
}
