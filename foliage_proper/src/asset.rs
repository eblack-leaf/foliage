use std::collections::HashMap;

use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use bevy_ecs::system::{Commands, Query, Res, ResMut, Resource};
use futures_channel::oneshot::{Receiver, Sender};
use uuid::Uuid;

#[derive(Resource, Default)]
pub struct AssetLoader {
    pub(crate) assets: HashMap<AssetKey, Asset>,
    awaiting: HashMap<AssetKey, AssetFetch>,
}
pub type AssetFn<B> = fn(Vec<u8>) -> B;
#[derive(Component, Clone)]
pub struct OnRetrieve<B> {
    key: AssetKey,
    bundle_using: Box<AssetFn<B>>,
}
impl<B: Bundle + 'static + Send + Sync> OnRetrieve<B> {
    pub fn new(key: AssetKey, func: AssetFn<B>) -> Self {
        Self {
            key,
            bundle_using: Box::new(func),
        }
    }
}
pub(crate) fn on_retrieve<B: Bundle + Send + Sync + 'static>(
    retrievers: Query<(Entity, &OnRetrieve<B>)>,
    mut cmd: Commands,
    asset_loader: Res<AssetLoader>,
) {
    for (entity, on_retrieve) in retrievers.iter() {
        if let Some(asset) = asset_loader.retrieve(on_retrieve.key) {
            cmd.entity(entity).remove::<OnRetrieve<B>>();
            cmd.entity(entity)
                .insert((on_retrieve.bundle_using)(asset.data));
        }
    }
}
pub(crate) fn await_assets(mut asset_loader: ResMut<AssetLoader>) {
    if !asset_loader.awaiting.is_empty() {
        let mut finished = Vec::<(AssetKey, Asset)>::new();
        for (key, fetch) in asset_loader.awaiting.iter_mut() {
            if let Some(f) = fetch.recv.try_recv().ok() {
                if let Some(f) = f {
                    finished.push((key.clone(), f));
                }
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
