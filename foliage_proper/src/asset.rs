use std::collections::HashMap;

use crate::tree::Tree;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::{Component, Trigger};
use bevy_ecs::system::{Commands, Query, Res, ResMut, Resource};
use futures_channel::oneshot::{Receiver, Sender};
use uuid::Uuid;

#[derive(Resource, Default)]
pub struct AssetLoader {
    pub(crate) assets: HashMap<AssetKey, Asset>,
    awaiting: HashMap<AssetKey, AssetFetch>,
}
pub type AssetFn = fn(&mut Tree, Entity, Vec<u8>);
#[derive(Component, Clone)]
pub struct AssetRetrieval {
    key: AssetKey,
}
impl AssetRetrieval {
    pub fn new(key: AssetKey) -> Self {
        Self { key }
    }
}
#[derive(Event, Copy, Clone)]
pub struct OnRetrieval {
    pub key: AssetKey,
}
pub fn asset_retrieval<'w, AFN: FnMut(&mut Tree, Entity, Vec<u8>) + 'static>(
    mut afn: AFN,
) -> impl FnMut(Trigger<OnRetrieval>, Tree, Res<AssetLoader>) {
    let obs =
        move |trigger: Trigger<OnRetrieval>, mut tree: Tree, asset_loader: Res<AssetLoader>| {
            let asset = asset_loader.retrieve(trigger.event().key).unwrap();
            afn(&mut tree, trigger.entity(), asset.data);
        };
    obs
}
pub(crate) fn on_retrieve(
    retrievers: Query<(Entity, &AssetRetrieval)>,
    mut cmd: Commands,
    asset_loader: Res<AssetLoader>,
) {
    for (entity, on_retrieve) in retrievers.iter() {
        if let Some(asset) = asset_loader.retrieve(on_retrieve.key) {
            cmd.entity(entity).remove::<AssetRetrieval>();
            cmd.trigger_targets(
                OnRetrieval {
                    key: on_retrieve.key,
                },
                entity,
            );
        }
    }
}
pub(crate) fn await_assets(mut asset_loader: ResMut<AssetLoader>) {
    if !asset_loader.awaiting.is_empty() {
        let mut finished = Vec::<(AssetKey, Asset)>::new();
        for (key, fetch) in asset_loader.awaiting.iter_mut() {
            if let Ok(Some(f)) = fetch.recv.try_recv() {
                finished.push((*key, f));
            }
        }
        for (key, asset) in finished {
            asset_loader.awaiting.remove(&key);
            asset_loader.assets.insert(key, asset);
        }
    }
}
impl AssetLoader {
    pub fn retrieve(&self, key: AssetKey) -> Option<Asset> {
        self.assets.get(&key).cloned()
    }
    #[allow(unused)]
    pub(crate) fn queue_fetch(&mut self, fetch: AssetFetch) {
        self.awaiting.insert(fetch.key, fetch);
    }
    pub fn generate_key() -> AssetKey {
        Uuid::new_v4().as_u128()
    }
}
#[macro_export]
macro_rules! load_asset {
    ($foliage:ident, $path:literal) => {{
        #[cfg(target_family = "wasm")]
        let id = $foliage.load_remote_asset($path);
        #[cfg(not(target_family = "wasm"))]
        let id = $foliage.load_native_asset(include_bytes!($path).to_vec());
        id
    }};
}
pub type AssetKey = u128;
#[derive(Clone)]
pub struct Asset {
    pub data: Vec<u8>,
}
impl Asset {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}
pub(crate) struct AssetFetch {
    pub(crate) key: AssetKey,
    pub(crate) recv: Receiver<Asset>,
}
impl AssetFetch {
    pub(crate) fn new(key: AssetKey) -> (Self, Sender<Asset>) {
        let (sender, recv) = futures_channel::oneshot::channel();
        (Self { key, recv }, sender)
    }
}
