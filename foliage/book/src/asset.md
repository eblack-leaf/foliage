# Assets

[Image](./image.md) and [Icon](./icon.md) (via `msdf_from_asset`) both need the same
thing: bytes that might already be in hand, or might need to be fetched first, without
the caller having to wait or branch on which case they're in. `AssetKey` is what makes
that possible -- a plain, immediately-usable handle, generated *before* the bytes exist:

```rust
// foliage_proper/src/asset.rs
pub type AssetKey = u128;
impl AssetLoader {
    pub fn generate_key() -> AssetKey { Uuid::new_v4().as_u128() }
}
pub enum AssetSource {
    Bytes(Vec<u8>),
    #[cfg(any(target_family = "wasm", feature = "remote-assets"))]
    Url(String),
}
```

`foliage.load_asset(AssetSource::Bytes(data))` or `::Url(url)` returns a key
immediately -- `Image::new(key)` can be spawned that same tick, before a `Url` fetch has
even started, because loading is decoupled from rendering: the entity exists and starts
rendering once the key resolves behind it, not before.

`Url` only *exists* as a variant on wasm (the browser's fetch is unconditional there) or
when the `remote-assets` Cargo feature is enabled on native (`default = ["remote-assets"]`
in `foliage_proper/Cargo.toml` -- an app that only ever loads bundled `Bytes` assets can
disable it to shed the `reqwest`/`rustls` dependency tree entirely). The cfg is on the
variant itself, not a runtime check: an app that disables the feature and still tries to
construct `AssetSource::Url(..)` on native gets a compile error at that call site,
not a panic discovered later at runtime.

## Loading: fire-and-arrives-later on every platform

```rust
// foliage_proper/src/asset.rs (handle_load_asset, abridged)
AssetSource::Bytes(bytes) => { asset_loader.assets.insert(key, Asset::new(bytes.clone())); }
#[cfg(target_family = "wasm")]
AssetSource::Url(url) => {
    // queue_fetch + wasm_bindgen_futures::spawn_local -- fetch() in the browser
}
#[cfg(all(not(target_family = "wasm"), feature = "remote-assets"))]
AssetSource::Url(url) => {
    // queue_fetch + a dedicated std::thread -- see below for why a thread, not async
}
```

`Bytes` resolves synchronously, same tick. `Url` never blocks the caller on either
platform: on native, the doc comment on this exact branch explains the thread instead of
an async task -- `reqwest`'s non-wasm backend needs an ambient Tokio runtime regardless
of how its future is polled, and `reqwest::blocking` is the one supported way to make
the call without the caller managing a runtime, so it runs on its own OS thread instead
of blocking anything else. Both platforms hand their result back through the same
`futures_channel::oneshot` pattern, drained once per tick by `await_assets`.

## Retrieval: poll a component, get a targeted event back

```rust
// foliage_proper/src/asset.rs
pub(crate) fn on_retrieve(retrievers: Query<(Entity, &AssetRetrieval)>, mut cmd: Commands, asset_loader: Res<AssetLoader>) {
    for (entity, on_retrieve) in retrievers.iter() {
        if asset_loader.assets.contains_key(&on_retrieve.key) {
            cmd.entity(entity).remove::<AssetRetrieval>();
            cmd.trigger(OnRetrieval { entity, key: on_retrieve.key });
        }
    }
}
```

An entity waiting on a key attaches `AssetRetrieval::new(key)` to itself and observes
`OnRetrieval`; `on_retrieve` runs every tick, checking whether the key has resolved yet
and firing the event (then removing the marker, so it only ever fires once) the moment
it has. `asset_retrieval(closure)` wraps this into a ready-made observer for the common
case of "give me the raw bytes once they land." Both [Image](./image.md) and
[Icon](./icon.md)'s `msdf_from_asset` path use exactly this -- no separate, per-primitive
loading logic.

## `bundled_asset!`: the common case, without repeating yourself

Most assets aren't dynamically-URL'd at all -- they're compiled into the native binary
and served as static files alongside the wasm build, at the same relative path both ways
(a build step mirrors the source `assets/` directory into the served dist output). The
macro exists so that common shape doesn't require restating the same path twice:

```rust
// application/src/main.rs
#[cfg(target_family = "wasm")]
fn asset_url(path: &str) -> String {
    format!("{}/foliage/{path}", Foliage::window_origin())
}
let music_player = bundled_asset!(foliage, "assets/music-player.png", asset_url);
```

`$path` is written once; on native it's embedded via `include_bytes!($path)`, and on wasm
it's fed through the caller's own hosting-convention function (`asset_url` here -- an
ordinary `fn(&str) -> String`, app-defined, since [`foliage_proper` makes no assumption
about where an app's assets are hosted](./web-ext.md)) as `asset_url($path)`. The literal
path exists in exactly one place in the caller's code, feeding both platform branches
from that one value.

For the rarer case where the native embed path and the wasm URL genuinely don't
correspond to the same relative path (a CDN, content-hashed filenames, any hosting layout
that doesn't mirror the source tree), a second form takes the URL directly instead of
deriving it from `$path`:

```rust
bundled_asset!(foliage, "assets/logo.png", url: "https://cdn.example.com/v2/branding/a1b2c3.png".to_string())
```

