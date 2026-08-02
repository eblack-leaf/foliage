use std::collections::HashMap;

use crate::Attachment;
use crate::Trigger;
use crate::foliage::{Foliage, MainMarkers};
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::{Component, IntoScheduleConfigs};
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Query, Res, ResMut};
use futures_channel::oneshot::{Receiver, Sender};
use uuid::Uuid;

impl Attachment for Asset {
    fn attach(foliage: &mut Foliage) {
        foliage.world.insert_resource(AssetLoader::default());
        foliage.world.add_observer(handle_load_asset);
        foliage.main.add_systems(
            (await_assets, on_retrieve)
                .chain()
                .in_set(MainMarkers::External),
        );
    }
}

/// Where a runtime-loaded asset's bytes come from. `Url` is used exactly as given -- no
/// origin/base-url composition happens anywhere in this crate; the caller (native: a full
/// filesystem path or http(s) URL: wasm: a full, already-resolved URL) owns that entirely.
///
/// `Url` only exists as a variant at all on wasm (fetch is unconditional there) or when the
/// `remote-assets` feature is enabled on native. An app that disables it to shed
/// `reqwest`/`rustls` (see `Cargo.toml`) therefore fails to compile at the construction
/// site rather than panicking at load time.
pub enum AssetSource {
    Bytes(Vec<u8>),
    #[cfg(any(target_family = "wasm", feature = "remote-assets"))]
    Url(String),
}

/// The door into `AssetLoader.assets`. Internal: [`Foliage::load_asset`](crate::Foliage::load_asset)
/// generates the key and sends this, which is the only thing an app ever needed it for --
/// constructing one by hand just meant calling `generate_key` first and getting the pairing
/// right yourself.
#[derive(Event)]
pub(crate) struct LoadAsset {
    pub(crate) key: AssetKey,
    pub(crate) source: AssetSource,
}

fn handle_load_asset(trigger: Trigger<LoadAsset>, mut asset_loader: ResMut<AssetLoader>) {
    let event = trigger.event();
    let key = event.key;
    match &event.source {
        AssetSource::Bytes(bytes) => {
            asset_loader.assets.insert(key, Asset::new(bytes.clone()));
        }
        #[cfg(target_family = "wasm")]
        AssetSource::Url(url) => {
            let (fetch, sender) = AssetFetch::new(key);
            asset_loader.queue_fetch(fetch);
            let url = url.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let asset = reqwest::Client::new()
                    .get(url)
                    .header("Accept", "application/octet-stream")
                    .send()
                    .await
                    .expect("asset-request")
                    .bytes()
                    .await
                    .expect("asset-bytes")
                    .to_vec();
                sender.send(Asset::new(asset)).ok();
            });
        }
        // mirrors the wasm branch exactly: queue the fetch, do the actual work off the
        // calling thread, hand the result to `await_assets` through the same channel --
        // fire-and-arrives-later on both platforms, never a blocking call on the caller's
        // thread. A background OS thread (not the async `reqwest::Client`) because reqwest's
        // non-wasm backend needs an ambient Tokio runtime regardless of how its future is
        // polled -- `reqwest::blocking` is the one Rust-supported way to make the call
        // without the caller managing a runtime; running it on its own thread is what keeps
        // it from blocking anything else.
        #[cfg(all(not(target_family = "wasm"), feature = "remote-assets"))]
        AssetSource::Url(url) => {
            let (fetch, sender) = AssetFetch::new(key);
            asset_loader.queue_fetch(fetch);
            let url = url.clone();
            std::thread::spawn(move || {
                let bytes = reqwest::blocking::Client::new()
                    .get(url)
                    .header("Accept", "application/octet-stream")
                    .send()
                    .expect("asset-request")
                    .bytes()
                    .expect("asset-bytes")
                    .to_vec();
                sender.send(Asset::new(bytes)).ok();
            });
        }
    }
}
#[derive(Resource, Default)]
/// Holds loaded assets by [`AssetKey`], and drives the fetches still in flight.
pub struct AssetLoader {
    pub(crate) assets: HashMap<AssetKey, Asset>,
    awaiting: HashMap<AssetKey, AssetFetch>,
}
#[derive(Component, Clone)]
/// Marks an entity as waiting on `key`, so [`OnRetrieval`] can find it once the bytes
/// land. Internal: inserted by [`Tree::on_asset`](crate::Tree::on_asset) together with
/// the observer that reads the result, since the two only ever made sense as a pair --
/// a component naming one key and an observer expecting another is a silent no-op.
pub(crate) struct AssetRetrieval {
    key: AssetKey,
}
impl AssetRetrieval {
    /// Waits on `key`.
    pub(crate) fn new(key: AssetKey) -> Self {
        Self { key }
    }
}
#[foliage_macros::targeted_event]
#[derive(Copy)]
/// Fired at an entity once the asset its `AssetRetrieval` marker (`pub(crate)`) names has
/// arrived. On native
/// that is effectively immediate; on wasm it is whenever the fetch resolves, so anything
/// depending on the bytes belongs here rather than after the spawn.
pub struct OnRetrieval {
    pub key: AssetKey,
}
pub(crate) fn on_retrieve(
    retrievers: Query<(Entity, &AssetRetrieval)>,
    mut tree: crate::Tree,
    asset_loader: Res<AssetLoader>,
) {
    for (entity, on_retrieve) in retrievers.iter() {
        if asset_loader.assets.contains_key(&on_retrieve.key) {
            tree.strip::<AssetRetrieval>(entity);
            tree.send(OnRetrieval {
                entity,
                key: on_retrieve.key,
            });
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
    /// The asset for `key`, or `None` while it is still loading.
    pub fn retrieve(&self, key: AssetKey) -> Option<Asset> {
        self.assets.get(&key).cloned()
    }
    #[allow(unused)]
    pub(crate) fn queue_fetch(&mut self, fetch: AssetFetch) {
        self.awaiting.insert(fetch.key, fetch);
    }
    /// A fresh key, usable immediately -- before the asset it names has loaded. Internal:
    /// [`Foliage::load_asset`](crate::Foliage::load_asset) calls this and hands the key
    /// back, so an app never has to mint one itself.
    pub(crate) fn generate_key() -> AssetKey {
        Uuid::new_v4().as_u128()
    }
}
/// Handle to an asset, valid from the moment it is issued and independent of whether the
/// bytes have arrived.
pub type AssetKey = u128;
#[derive(Clone)]
/// Loaded asset bytes, undecoded -- whatever consumes them decides what they mean.
pub struct Asset {
    pub data: Vec<u8>,
}
impl Asset {
    /// Wraps already-loaded bytes.
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}
pub(crate) struct AssetFetch {
    pub(crate) key: AssetKey,
    pub(crate) recv: Receiver<Asset>,
}
impl AssetFetch {
    #[allow(unused)]
    pub(crate) fn new(key: AssetKey) -> (Self, Sender<Asset>) {
        let (sender, recv) = futures_channel::oneshot::channel();
        (Self { key, recv }, sender)
    }
}

/// A bundled asset -- embedded via `include_bytes!` on native, fetched on wasm from a URL.
/// Two forms:
///
/// - `bundled_asset!(foliage, $path)` -- the common case, where the web build mirrors the
///   same relative path (a build step copies `assets/` into the served dist output, say).
///   `$path` is written once and both platform branches use it: native embeds it, wasm
///   resolves it through `Foliage::asset_url` (wasm-only), so the two
///   can't drift apart by a typo or a rename that only touches one. Declare the hosting
///   convention once with [`Foliage::asset_base`](crate::Foliage::asset_base).
/// - `bundled_asset!(foliage, $path, url: $url)` -- the escape hatch, for when the native
///   embed path and the wasm URL genuinely don't correspond to the same relative path (a
///   CDN, content-hashed filenames, any hosting layout that doesn't mirror the source tree).
///   `$url` is used exactly as given, independent of `$path` and of `asset_base`.
///
/// The *mechanics* of the platform split (which `AssetSource` variant) are the only thing
/// this provides -- this crate still makes no assumption about where an app's assets are
/// actually hosted, only about what the app told it.
#[macro_export]
macro_rules! bundled_asset {
    ($foliage:expr, $path:literal) => {{
        #[cfg(not(target_family = "wasm"))]
        let source = $crate::AssetSource::Bytes(include_bytes!($path).to_vec());
        #[cfg(target_family = "wasm")]
        let source = $crate::AssetSource::Url($foliage.asset_url($path));
        $foliage.load_asset(source)
    }};
    ($foliage:expr, $path:literal, url: $url:expr) => {{
        #[cfg(not(target_family = "wasm"))]
        let source = $crate::AssetSource::Bytes(include_bytes!($path).to_vec());
        #[cfg(target_family = "wasm")]
        let source = $crate::AssetSource::Url($url);
        $foliage.load_asset(source)
    }};
}
