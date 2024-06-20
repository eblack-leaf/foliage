use bevy_ecs::system::{Commands, ResMut, Resource};
use futures_channel::oneshot::{Receiver, Sender};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Resource)]
pub struct AssetLoader {
    pub(crate) assets: HashMap<AssetKey, Asset>,
    awaiting: HashMap<AssetKey, AssetFetch>,
}
pub type AssetFn = fn(Vec<u8>, &mut Commands);
pub struct OnRetrieve {
    key: AssetKey,
    func: Box<AssetFn>,
}
pub(crate) fn on_retrieve() {}
pub(crate) fn await_assets(mut asset_loader: ResMut<AssetLoader>) {
    if !asset_loader.awaiting.is_empty() {
        let mut finished = HashSet::<(AssetKey, Asset)>::new();
        for (key, mut fetch) in asset_loader.awaiting.iter_mut() {
            if let Some(f) = fetch.recv.try_recv().ok() {
                if let Some(f) = f {
                    finished.insert((key.clone(), f));
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
    pub(crate) fn queue_fetch(&mut self, fetch: AssetFetch) {
        self.awaiting.push(fetch);
    }
    pub fn generate_key() -> AssetKey {
        Uuid::new_v4().as_u128()
    }
}
macro_rules! load_asset {
    (foliage:ident, path:lit) => {
        #[cfg(target_family = "wasm")]
        // foliage.load_remote_asset($path)
        #[cfg(not(target_family = "wasm"))]
        // foliage.load_native_asset(include_bytes!($path).to_vec())
    };
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
